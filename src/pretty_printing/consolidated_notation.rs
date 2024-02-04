//! Walk along the notation tree, skipping the boring parts.

use super::pretty_doc::PrettyDoc;
use crate::geometry::{str_width, Width};
use crate::notation::Notation;
use std::fmt;

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
    Newline(Width),
    Textual(&'d str, Width, &'d D::Style),
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

/// A `ConsolidatedNotation` that has not yet been evaluated, to prevent the entire notation tree
/// from being in memory at once. Call `.eval()` to get a `ConsolidatedNotation`.
#[derive(Debug)]
pub struct DelayedConsolidatedNotation<'d, D: PrettyDoc<'d>> {
    doc: D,
    notation: &'d Notation<D::Style>,
    flat: bool,
    indent: Width,
    join_pos: Option<JoinPos<'d, D>>,
    mark: Option<&'d D::Mark>,
}

#[derive(Debug)]
struct JoinPos<'d, D: PrettyDoc<'d>> {
    parent: D,
    child: D,
    index: usize,
    first: &'d Notation<D::Style>,
    join: &'d Notation<D::Style>,
}

impl<'d, D: PrettyDoc<'d>> Clone for ConsolidatedNotation<'d, D> {
    fn clone(&self) -> Self {
        use ConsolidatedNotation::*;

        match self {
            Empty => Empty,
            Newline(ind) => Newline(*ind),
            Textual(str, width, style) => Textual(*str, *width, *style),
            Concat(note1, note2) => Concat(*note1, *note2),
            Choice(note1, note2) => Choice(*note1, *note2),
            Child(i, child) => Child(*i, *child),
        }
    }
}
impl<'d, D: PrettyDoc<'d>> Copy for ConsolidatedNotation<'d, D> {}

impl<'d, D: PrettyDoc<'d>> Clone for DelayedConsolidatedNotation<'d, D> {
    fn clone(&self) -> Self {
        DelayedConsolidatedNotation {
            doc: self.doc,
            notation: self.notation,
            flat: self.flat,
            indent: self.indent,
            join_pos: self.join_pos,
            mark: self.mark,
        }
    }
}
impl<'d, D: PrettyDoc<'d>> Copy for DelayedConsolidatedNotation<'d, D> {}

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

#[derive(thiserror::Error, Debug, Clone)]
pub enum PrintingError {
    #[error("Pretty printing path invalid at child index {0}.")]
    InvalidPath(usize),
    #[error("Notation/doc mismatch: Notation was Text but doc node did not contain text.")]
    TextNotationOnTextlessDoc,
    #[error("Notation/doc mismatch: Notation was IfEmptyText but doc node did not contain text.")]
    IfEmptyTextNotationOnTextlessDoc,
    #[error(
        "Notation/doc mismatch: Notation was Child({index}) but doc node only had {len} children."
    )]
    ChildIndexOutOfBounds { index: usize, len: usize },
    #[error("Notation/doc mismatch: Notation was Child but doc node contained text instead.")]
    ChildNotationOnChildlessDoc,
    // Count used on texty node
    #[error("Notation/doc mismatch: Notation was Count but doc node contained text instead of children.")]
    CountNotationOnChildlessDoc,
    #[error("Doc node's num_children() changed between invocations!")]
    NumChildrenChanged,
    #[error("Notation/doc mismatch: Notation was Fold but doc node was childless.")]
    FoldNotationOnChildlessDoc,
}

impl<'d, D: PrettyDoc<'d>> DelayedConsolidatedNotation<'d, D> {
    pub fn new(doc: D) -> Self {
        DelayedConsolidatedNotation {
            doc,
            notation: &doc.notation().0,
            flat: false,
            indent: 0,
            join_pos: None,
            mark: doc.whole_node_mark(),
        }
    }

    pub fn doc(&self) -> &D {
        &self.doc
    }

    /// Expand this node, to get a usable `ConsolidatedNotation` and its mark (if any).
    pub fn eval(self) -> Result<(ConsolidatedNotation<'d, D>, Option<&'d D::Mark>), PrintingError> {
        let notation = self.eval_notation()?;
        Ok((notation, self.mark))
    }

    fn eval_notation(mut self) -> Result<ConsolidatedNotation<'d, D>, PrintingError> {
        use Notation::*;

        loop {
            match self.notation {
                Empty => return Ok(ConsolidatedNotation::Empty),
                Newline => return Ok(ConsolidatedNotation::Newline(self.indent)),
                Literal(lit) => {
                    return Ok(ConsolidatedNotation::Textual(
                        lit.str(),
                        lit.width(),
                        lit.style(),
                    ))
                }
                Text(style) => {
                    if self.doc.num_children().is_some() {
                        return Err(PrintingError::TextNotationOnTextlessDoc);
                    } else {
                        let text = self.doc.unwrap_text();
                        return Ok(ConsolidatedNotation::Textual(text, str_width(text), style));
                    }
                }
                Flat(note) => {
                    self.flat = true;
                    self.notation = note;
                }
                Indent(indent, note) => {
                    self.indent += indent;
                    self.notation = note;
                }
                Concat(note1, note2) => {
                    let mut cnote1 = self;
                    cnote1.notation = note1;
                    let mut cnote2 = self;
                    cnote2.notation = note2;
                    return Ok(ConsolidatedNotation::Concat(cnote1, cnote2));
                }
                Choice(note1, _note2) if self.flat => {
                    self.notation = note1;
                }
                Choice(note1, note2) => {
                    let mut cnote1 = self;
                    cnote1.notation = note1;
                    let mut cnote2 = self;
                    cnote2.notation = note2;
                    return Ok(ConsolidatedNotation::Choice(cnote1, cnote2));
                }
                IfEmptyText(note1, note2) => {
                    if self.doc.num_children().is_some() {
                        return Err(PrintingError::IfEmptyTextNotationOnTextlessDoc);
                    }
                    if self.doc.unwrap_text().is_empty() {
                        self.notation = note1;
                    } else {
                        self.notation = note2;
                    }
                }
                Child(i) => match self.doc.num_children() {
                    None => return Err(PrintingError::ChildNotationOnChildlessDoc),
                    Some(n) if *i >= n => {
                        return Err(PrintingError::ChildIndexOutOfBounds { index: *i, len: n })
                    }
                    Some(_) => {
                        self.doc = self.doc.unwrap_child(*i);
                        self.notation = &self.doc.notation().0;
                        if let Some(mark) = self.doc.whole_node_mark() {
                            self.mark = Some(mark);
                        }
                        return Ok(ConsolidatedNotation::Child(*i, self));
                    }
                },
                Mark(mark_name, note) => {
                    if let Some(mark) = self.doc.partial_node_mark(mark_name) {
                        self.mark = Some(mark);
                    }
                    self.notation = note;
                }
                Count { zero, one, many } => match self.doc.num_children() {
                    None => return Err(PrintingError::CountNotationOnChildlessDoc),
                    Some(0) => self.notation = zero,
                    Some(1) => self.notation = one,
                    Some(_) => self.notation = many,
                },
                Fold { first, join } => match self.doc.num_children() {
                    None => return Err(PrintingError::NumChildrenChanged),
                    Some(0) => return Err(PrintingError::FoldNotationOnChildlessDoc),
                    Some(1) => self.notation = first,
                    Some(n) => {
                        self.join_pos = Some(JoinPos {
                            parent: self.doc,
                            child: self.doc.unwrap_last_child(),
                            index: n - 1,
                            first,
                            join,
                        });
                        self.notation = join;
                    }
                },
                Left => {
                    match &mut self.join_pos {
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
                            } else {
                                *child = child.unwrap_prev_sibling(*parent, *index - 1);
                                *index -= 1;
                                self.notation = *join;
                            }
                        }
                    }
                }
                Right => match &mut self.join_pos {
                    None => {
                        panic!("Bug: Right used outside of fold; should have been caught by validation")
                    }
                    Some(JoinPos { child, index, .. }) => {
                        let index = *index;
                        self.doc = *child;
                        self.notation = &child.notation().0;
                        self.join_pos = None;
                        return Ok(ConsolidatedNotation::Child(index, self));
                    }
                },
            }
        }
    }
}

// For debugging. Should match impl fmt::Display for Notation.
impl<'d, D: PrettyDoc<'d>> fmt::Display for ConsolidatedNotation<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ConsolidatedNotation::*;

        match self {
            Empty => write!(f, "ε"),
            Newline(_) => write!(f, "↵"),
            Textual(text, _, _) => write!(f, "'{}'", text),
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
