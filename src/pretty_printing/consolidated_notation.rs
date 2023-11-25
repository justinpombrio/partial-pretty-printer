//! Walk along the notation tree, skipping the boring parts.

use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::notation::{Literal, Notation};
use crate::style::Style;
use std::fmt;

/// A `Notation` says how to print a _single_ node in a document. The _notation tree_ is what you
/// get from gluing together the `Notation`s for every node in the document. A
/// `ConsolidatedNotation` is a node in a _simplified_ version of the notation tree, in which many
/// of the kinds of `Notation`s have been automatically resolved away. The remaining kinds are
/// listed in `ConsolidatedNotation`.
///
/// (Note that this is not a "visitor". It does not invoke a callback on every node. Instead, it
///  allows you to walk the (simplified) tree yourself.)
#[derive(Debug, Clone, Copy)]
pub enum ConsolidatedNotation<'d, D: PrettyDoc<'d>> {
    Empty,
    Newline(Width),
    Literal(&'d Literal),
    Text(&'d str, Style),
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
#[derive(Debug, Clone, Copy)]
pub struct DelayedConsolidatedNotation<'d, D: PrettyDoc<'d>> {
    doc: D,
    notation: &'d Notation,
    flat: bool,
    indent: Width,
    join_pos: Option<JoinPos<'d, D>>,
}

#[derive(Debug, Clone, Copy)]
struct JoinPos<'d, D: PrettyDoc<'d>> {
    parent: D,
    child: D,
    index: usize,
    first: &'d Notation,
    join: &'d Notation,
}

impl<'d, D: PrettyDoc<'d>> ConsolidatedNotation<'d, D> {
    pub fn new(doc: D) -> Result<ConsolidatedNotation<'d, D>, NotationMismatchError> {
        DelayedConsolidatedNotation::new(doc).eval()
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum NotationMismatchError {
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
        }
    }

    pub fn id(&self) -> D::Id {
        self.doc.id()
    }

    /// Expand this node, to get a usable `ConsolidatedNotation`.
    pub fn eval(mut self) -> Result<ConsolidatedNotation<'d, D>, NotationMismatchError> {
        use Notation::*;

        loop {
            match self.notation {
                Empty => return Ok(ConsolidatedNotation::Empty),
                Newline => return Ok(ConsolidatedNotation::Newline(self.indent)),
                Literal(lit) => return Ok(ConsolidatedNotation::Literal(lit)),
                Text(style) => {
                    if self.doc.num_children().is_some() {
                        return Err(NotationMismatchError::TextNotationOnTextlessDoc);
                    } else {
                        return Ok(ConsolidatedNotation::Text(self.doc.unwrap_text(), *style));
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
                        return Err(NotationMismatchError::IfEmptyTextNotationOnTextlessDoc);
                    }
                    if self.doc.unwrap_text().is_empty() {
                        self.notation = note1;
                    } else {
                        self.notation = note2;
                    }
                }
                Child(i) => match self.doc.num_children() {
                    None => return Err(NotationMismatchError::ChildNotationOnChildlessDoc),
                    Some(n) if *i >= n => {
                        return Err(NotationMismatchError::ChildIndexOutOfBounds {
                            index: *i,
                            len: n,
                        })
                    }
                    Some(n) => {
                        self.doc = self.doc.unwrap_child(n);
                        self.notation = &self.doc.notation().0;
                        return Ok(ConsolidatedNotation::Child(*i, self));
                    }
                },
                Count { zero, one, many } => match self.doc.num_children() {
                    None => return Err(NotationMismatchError::CountNotationOnChildlessDoc),
                    Some(0) => self.notation = zero,
                    Some(1) => self.notation = one,
                    Some(_) => self.notation = many,
                },
                Fold { first, join } => match self.doc.num_children() {
                    None => return Err(NotationMismatchError::NumChildrenChanged),
                    Some(0) => return Err(NotationMismatchError::FoldNotationOnChildlessDoc),
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

impl<'d, D: PrettyDoc<'d>> fmt::Display for DelayedConsolidatedNotation<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.notation)
    }
}
