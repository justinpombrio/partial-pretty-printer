//! Walk along the notation tree, skipping the boring parts.

use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::notation::{Literal, Notation};
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
pub enum ConsolidatedNotation<'d, S, D: PrettyDoc<'d, S>> {
    Empty,
    Newline(Width),
    Literal(&'d Literal<S>),
    Text(&'d str, &'d S),
    Concat(
        DelayedConsolidatedNotation<'d, S, D>,
        DelayedConsolidatedNotation<'d, S, D>,
    ),
    Choice(
        DelayedConsolidatedNotation<'d, S, D>,
        DelayedConsolidatedNotation<'d, S, D>,
    ),
    Child(usize, DelayedConsolidatedNotation<'d, S, D>),
}

/// A `ConsolidatedNotation` that has not yet been evaluated, to prevent the entire notation tree
/// from being in memory at once. Call `.eval()` to get a `ConsolidatedNotation`.
#[derive(Debug)]
pub struct DelayedConsolidatedNotation<'d, S, D: PrettyDoc<'d, S>> {
    doc: D,
    notation: &'d Notation<S>,
    flat: bool,
    indent: Width,
    join_pos: Option<JoinPos<'d, S, D>>,
}

#[derive(Debug)]
struct JoinPos<'d, S, D: PrettyDoc<'d, S>> {
    parent: D,
    child: D,
    index: usize,
    first: &'d Notation<S>,
    join: &'d Notation<S>,
}

impl<'d, S, D: PrettyDoc<'d, S>> Clone for ConsolidatedNotation<'d, S, D> {
    fn clone(&self) -> Self {
        use ConsolidatedNotation::*;

        match self {
            Empty => Empty,
            Newline(ind) => Newline(*ind),
            Literal(lit) => Literal(*lit),
            Text(str, style) => Text(*str, *style),
            Concat(note1, note2) => Concat(*note1, *note2),
            Choice(note1, note2) => Choice(*note1, *note2),
            Child(i, child) => Child(*i, *child),
        }
    }
}
impl<'d, S, D: PrettyDoc<'d, S>> Copy for ConsolidatedNotation<'d, S, D> {}

impl<'d, S, D: PrettyDoc<'d, S>> Clone for DelayedConsolidatedNotation<'d, S, D> {
    fn clone(&self) -> Self {
        DelayedConsolidatedNotation {
            doc: self.doc,
            notation: self.notation,
            flat: self.flat,
            indent: self.indent,
            join_pos: self.join_pos,
        }
    }
}
impl<'d, S, D: PrettyDoc<'d, S>> Copy for DelayedConsolidatedNotation<'d, S, D> {}

impl<'d, S, D: PrettyDoc<'d, S>> Clone for JoinPos<'d, S, D> {
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
impl<'d, S, D: PrettyDoc<'d, S>> Copy for JoinPos<'d, S, D> {}

impl<'d, S, D: PrettyDoc<'d, S>> ConsolidatedNotation<'d, S, D> {
    pub fn new(doc: D) -> Result<ConsolidatedNotation<'d, S, D>, PrintingError> {
        DelayedConsolidatedNotation::new(doc).eval()
    }
}

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

impl<'d, S, D: PrettyDoc<'d, S>> DelayedConsolidatedNotation<'d, S, D> {
    pub fn new(doc: D) -> Self {
        DelayedConsolidatedNotation {
            doc,
            notation: &doc.notation().0,
            flat: false,
            indent: 0,
            join_pos: None,
        }
    }

    pub fn id(&self) -> D::Id {
        self.doc.id()
    }

    /// Expand this node, to get a usable `ConsolidatedNotation`.
    pub fn eval(mut self) -> Result<ConsolidatedNotation<'d, S, D>, PrintingError> {
        use Notation::*;

        loop {
            match self.notation {
                Empty => return Ok(ConsolidatedNotation::Empty),
                Newline => return Ok(ConsolidatedNotation::Newline(self.indent)),
                Literal(lit) => return Ok(ConsolidatedNotation::Literal(lit)),
                Text(style) => {
                    if self.doc.num_children().is_some() {
                        return Err(PrintingError::TextNotationOnTextlessDoc);
                    } else {
                        return Ok(ConsolidatedNotation::Text(self.doc.unwrap_text(), style));
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
                Choice(note1, note2) if self.flat => {
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
                    Some(n) => {
                        self.doc = self.doc.unwrap_child(n);
                        self.notation = &self.doc.notation().0;
                        return Ok(ConsolidatedNotation::Child(*i, self));
                    }
                },
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
                    Some(JoinPos { child, .. }) => {
                        self.doc = *child;
                        self.join_pos = None;
                    }
                },
            }
        }
    }
}

// For debugging. Should match impl fmt::Display for Notation.
impl<'d, S, D: PrettyDoc<'d, S>> fmt::Display for ConsolidatedNotation<'d, S, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ConsolidatedNotation::*;

        match self {
            Empty => write!(f, "ε"),
            Newline(_) => write!(f, "↵"),
            Text(_, _) => write!(f, "TEXT"),
            Literal(lit) => write!(f, "'{}'", lit.str()),
            Concat(left, right) => write!(f, "{} + {}", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            Child(i, _) => write!(f, "${}", i),
        }
    }
}

impl<'d, S, D: PrettyDoc<'d, S>> fmt::Display for DelayedConsolidatedNotation<'d, S, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.notation)
    }
}
