//! Walk along the notation tree, skipping the boring parts.

use super::pretty_doc::{PrettyDoc, Style};
use crate::geometry::{str_width, Width};
use crate::notation::{normalize_child_index, CheckPos, Notation};
use std::fmt;
use std::rc::Rc;

/// A `Notation` says how to print a _single_ node in a document. The _notation tree_ is what you
/// get from gluing together the `Notation`s for every node in the document. A
/// `ConsolidatedNotation` is a node in a _simplified_ version of the notation tree, in which many
/// of the kinds of `Notation`s have been automatically resolved away. The remaining kinds are
/// listed in `ConsolidatedNotation`.
///
/// (Note that this is not a "visitor". It does not invoke a callback on every node. Instead, it
///  allows you to walk the (simplified) tree yourself.)
#[derive(Debug)]
pub enum ConsolidatedNotation<'d, D: PrettyDoc<'d>> {
    Empty,
    EndOfLine,
    Newline(Option<Rc<IndentNode<'d, D>>>),
    Textual(Textual<'d, D>),
    Concat(
        DelayedConsolidatedNotation<'d, D>,
        DelayedConsolidatedNotation<'d, D>,
    ),
    Choice(
        DelayedConsolidatedNotation<'d, D>,
        DelayedConsolidatedNotation<'d, D>,
    ),
    Child(usize, DelayedConsolidatedNotation<'d, D>),
}

/// A fully resolved piece of text.
#[derive(Debug)]
pub struct Segment<'d, D: PrettyDoc<'d>> {
    pub str: &'d str,
    /// The unicode width of `str`, stored for performance reasons
    pub width: Width,
    pub style: D::Style,
}

/// A styled piece of text from `Notation::Literal` or `Notation::Text` or `Notation::Indent`.
#[derive(Debug)]
pub struct Textual<'d, D: PrettyDoc<'d>> {
    pub str: &'d str,
    /// The unicode width of `str`, stored for performance reasons
    pub width: Width,
    pub style: D::Style,
}

// Performance Note: We've tested three implementations of indentation so far:
// - Indentation as a number of spaces (less expressive)
// - Identation as an `Rc<IndentNode>` (the current impl) -- 25% slower
// - Indentation as a `Vec<Segment>` -- much slower
// If there's a way to implement it with fewer heap alloations, we should try it.

/// One level of indentation, plus a reference to the level of indentation to its left. These
/// references form trees, with each child referencing its parent.
#[derive(Debug)]
pub struct IndentNode<'d, D: PrettyDoc<'d>> {
    /// The rightmost/current level of indentation.
    pub segment: Segment<'d, D>,
    /// The level of indentation to the left of this one.
    pub parent: Option<Rc<IndentNode<'d, D>>>,
}

/// A `ConsolidatedNotation` that has not yet been evaluated, to prevent the entire notation tree
/// from being in memory at once. Call `.eval()` to get a `ConsolidatedNotation`.
#[derive(Debug)]
pub struct DelayedConsolidatedNotation<'d, D: PrettyDoc<'d>> {
    notation: &'d Notation<D::StyleLabel, D::Condition>,
    /// The document node that this notation came from.
    doc: D,
    /// Whether we are inside a `Notation::Flat`.
    flat: bool,
    /// The indentation that will be applied to any newlines inside of this notation.
    indent: Option<Rc<IndentNode<'d, D>>>,
    /// The style that will be applied to any text, literals, or indentation inside of this notation
    style: D::Style,
    /// If we are inside a `Notation::Fold`'s `join` case, this stores context about the join.
    join_pos: Option<JoinPos<'d, D>>,
}

#[derive(Debug)]
struct JoinPos<'d, D: PrettyDoc<'d>> {
    /// The document node containing the `Notation::Fold`.
    parent: D,
    /// The next child to process.
    child: D,
    /// The index of `child`.
    index: usize,
    /// The notation to use for displaying the first child.
    first: &'d Notation<D::StyleLabel, D::Condition>,
    /// The notation to use for joining multiple children.
    join: &'d Notation<D::StyleLabel, D::Condition>,
}

impl<'d, D: PrettyDoc<'d>> Clone for Textual<'d, D> {
    fn clone(&self) -> Self {
        Textual {
            str: self.str,
            width: self.width,
            style: self.style.clone(),
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Clone for Segment<'d, D> {
    fn clone(&self) -> Self {
        Segment {
            str: self.str,
            width: self.width,
            style: self.style.clone(),
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Clone for ConsolidatedNotation<'d, D> {
    fn clone(&self) -> Self {
        use ConsolidatedNotation::*;

        match self {
            Empty => Empty,
            EndOfLine => EndOfLine,
            Newline(ind) => Newline(ind.clone()),
            Textual(textual) => Textual(textual.clone()),
            Concat(note1, note2) => Concat(note1.clone(), note2.clone()),
            Choice(note1, note2) => Choice(note1.clone(), note2.clone()),
            Child(i, child) => Child(*i, child.clone()),
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Clone for DelayedConsolidatedNotation<'d, D> {
    fn clone(&self) -> Self {
        DelayedConsolidatedNotation {
            doc: self.doc,
            notation: self.notation,
            flat: self.flat,
            indent: self.indent.clone(),
            join_pos: self.join_pos,
            style: self.style.clone(),
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Clone for JoinPos<'d, D> {
    fn clone(&self) -> Self {
        JoinPos {
            parent: self.parent,
            child: self.child,
            index: self.index,
            first: self.first,
            join: self.join,
        }
    }
}
impl<'d, D: PrettyDoc<'d>> Copy for JoinPos<'d, D> {}

/// An error that can occur while printing (laying out) the document.
#[derive(thiserror::Error, Debug, Clone)]
pub enum PrintingError {
    #[error("Pretty printing path invalid at child index {0}.")]
    InvalidPath(usize),
    #[error("Notation/doc mismatch: Notation was Text but doc node did not contain text.")]
    TextNotationOnTextlessDoc,
    #[error(
        "Notation/doc mismatch: Notation was Child({index}) but doc node only had {len} children."
    )]
    ChildIndexOutOfBounds { index: isize, len: usize },
    #[error("Notation/doc mismatch: Notation was Child but doc node contained text instead.")]
    ChildNotationOnChildlessDoc,
    #[error(
        "Notation/doc mismatch: Notation contained CheckPos::Child({index}) but doc node only had {len} children."
    )]
    CheckPosChildIndexOutOfBounds { index: isize, len: usize },
    #[error("Notation/doc mismatch: Notation contained CheckPos::Child(_) but doc node contained text instead.")]
    CheckPosChildOnChildlessDoc,
    #[error("Notation/doc mismatch: Notation was Count but doc node contained text instead of children.")]
    CountNotationOnChildlessDoc,
    #[error("Doc node's num_children() changed between invocations!")]
    NumChildrenChanged,
    #[error("Pretty printing encountered a Text or Literal after an EndOfLine.")]
    TextAfterEndOfLine,
}

impl<'d, D: PrettyDoc<'d>> DelayedConsolidatedNotation<'d, D> {
    pub fn new(doc: D) -> Self {
        DelayedConsolidatedNotation {
            doc,
            notation: &doc.notation().0,
            flat: false,
            indent: None,
            join_pos: None,
            style: doc.node_style(),
        }
    }

    pub fn doc(&self) -> &D {
        &self.doc
    }

    /// Expand this node to get a usable `ConsolidatedNotation`.
    pub fn eval(mut self) -> Result<ConsolidatedNotation<'d, D>, PrintingError> {
        use Notation::*;

        match self.notation {
            Empty => Ok(ConsolidatedNotation::Empty),
            EndOfLine => Ok(ConsolidatedNotation::EndOfLine),
            Newline => Ok(ConsolidatedNotation::Newline(self.indent)),
            Literal(lit) => Ok(ConsolidatedNotation::Textual(Textual {
                str: lit.str(),
                width: lit.width(),
                style: self.style,
            })),
            Text => {
                if self.doc.num_children().is_some() {
                    Err(PrintingError::TextNotationOnTextlessDoc)
                } else {
                    let text = self.doc.unwrap_text();
                    Ok(ConsolidatedNotation::Textual(Textual {
                        str: text,
                        width: str_width(text),
                        style: self.style,
                    }))
                }
            }
            Flat(note) => {
                self.flat = true;
                self.notation = note;
                self.eval()
            }
            Indent(prefix, style_label, note) => {
                let style = if let Some(label) = style_label {
                    D::Style::combine(&self.style, &self.doc.lookup_style(label.clone()))
                } else {
                    self.style.clone()
                };
                let new_indent = Rc::new(IndentNode {
                    segment: Segment {
                        str: prefix.str(),
                        width: prefix.width(),
                        style,
                    },
                    parent: self.indent,
                });
                self.indent = Some(new_indent);
                self.notation = note;
                self.eval()
            }
            Concat(note1, note2) => {
                let mut cnote1 = self.clone();
                cnote1.notation = note1;
                let mut cnote2 = self;
                cnote2.notation = note2;
                Ok(ConsolidatedNotation::Concat(cnote1, cnote2))
            }
            Choice(note1, _note2) if self.flat => {
                self.notation = note1;
                self.eval()
            }
            Choice(note1, note2) => {
                let mut cnote1 = self.clone();
                cnote1.notation = note1;
                let mut cnote2 = self;
                cnote2.notation = note2;
                Ok(ConsolidatedNotation::Choice(cnote1, cnote2))
            }
            Check(cond, pos, note1, note2) => {
                let doc_to_inspect = match pos {
                    CheckPos::Here => self.doc,
                    CheckPos::Child(i) => match self.doc.num_children() {
                        None => return Err(PrintingError::CheckPosChildOnChildlessDoc),
                        Some(n) => match normalize_child_index(*i, n) {
                            None => {
                                return Err(PrintingError::CheckPosChildIndexOutOfBounds {
                                    index: *i,
                                    len: n,
                                })
                            }
                            Some(index) => self.doc.unwrap_child(index),
                        },
                    },
                    // ValidNotation::validate() ensures these unwraps are safe
                    CheckPos::RightChild => self.join_pos.unwrap().child,
                    CheckPos::LeftChild => {
                        let join_pos = self.join_pos.unwrap();
                        join_pos
                            .child
                            .unwrap_prev_sibling(join_pos.parent, join_pos.index - 1)
                    }
                };
                if doc_to_inspect.condition(cond) {
                    self.notation = note1;
                    self.eval()
                } else {
                    self.notation = note2;
                    self.eval()
                }
            }
            Child(i) => match self.doc.num_children() {
                None => Err(PrintingError::ChildNotationOnChildlessDoc),
                Some(n) => match normalize_child_index(*i, n) {
                    None => Err(PrintingError::ChildIndexOutOfBounds { index: *i, len: n }),
                    Some(index) => {
                        self.doc = self.doc.unwrap_child(index);
                        self.notation = &self.doc.notation().0;
                        self.style = D::Style::combine(&self.style, &self.doc.node_style());
                        Ok(ConsolidatedNotation::Child(index, self))
                    }
                },
            },
            Style(style_label, note) => {
                self.style =
                    D::Style::combine(&self.style, &self.doc.lookup_style(style_label.clone()));
                self.notation = note;
                self.eval()
            }
            Count { zero, one, many } => match self.doc.num_children() {
                None => Err(PrintingError::CountNotationOnChildlessDoc),
                Some(0) => {
                    self.notation = zero;
                    self.eval()
                }
                Some(1) => {
                    self.notation = one;
                    self.eval()
                }
                Some(_) => {
                    self.notation = many;
                    self.eval()
                }
            },
            Fold { first, join } => match self.doc.num_children() {
                None => Err(PrintingError::NumChildrenChanged),
                Some(0) => Ok(ConsolidatedNotation::Empty),
                Some(1) => {
                    self.notation = first;
                    self.eval()
                }
                Some(n) => {
                    self.join_pos = Some(JoinPos {
                        parent: self.doc,
                        child: self.doc.unwrap_last_child(),
                        index: n - 1,
                        first,
                        join,
                    });
                    self.notation = join;
                    self.eval()
                }
            },
            Left => match &mut self.join_pos {
                None => {
                    panic!("Bug: Left used outside of fold; should have been caught by validation")
                }
                Some(JoinPos {
                    parent,
                    child,
                    index,
                    first,
                    join,
                }) => {
                    if *index == 1 {
                        self.notation = *first;
                        self.join_pos = None;
                        self.eval()
                    } else {
                        *child = child.unwrap_prev_sibling(*parent, *index - 1);
                        *index -= 1;
                        self.notation = *join;
                        self.eval()
                    }
                }
            },
            Right => match &mut self.join_pos {
                None => {
                    panic!("Bug: Right used outside of fold; should have been caught by validation")
                }
                Some(JoinPos { child, index, .. }) => {
                    let index = *index;
                    self.doc = *child;
                    self.notation = &child.notation().0;
                    self.style = D::Style::combine(&self.style, &self.doc.node_style());
                    self.join_pos = None;
                    Ok(ConsolidatedNotation::Child(index, self))
                }
            },
        }
    }
}

// For debugging. Should match impl fmt::Display for Notation.
impl<'d, D: PrettyDoc<'d>> fmt::Display for ConsolidatedNotation<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ConsolidatedNotation::*;

        match self {
            Empty => write!(f, "ε"),
            EndOfLine => write!(f, "EOL"),
            Newline(_) => write!(f, "↵"),
            Textual(textual) => write!(f, "'{}'", textual.str),
            Concat(left, right) => write!(f, "{} + {}", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            Child(i, _) => write!(f, "${}", i),
        }
    }
}

impl<'d, D: PrettyDoc<'d>> fmt::Display for DelayedConsolidatedNotation<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.notation)
    }
}
