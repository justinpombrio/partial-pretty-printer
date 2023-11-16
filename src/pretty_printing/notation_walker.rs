use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::notation::{Literal, Notation};
use crate::style::Style;
use std::fmt;

// TODO: Don't panic on bad PrettyDoc/Notation combos

/// Walk along the notation tree, skipping the boring parts.
///
/// A `Notation` says how to print a _single_ node in a document. The _notation tree_ is what you
/// get from gluing together the `Notation`s for every node in the document. A `NotationWalker`
/// allows you to navigate a _simplified_ version of the notation tree, in which ~half of the
/// kinds of `Notation`s have been automatically resolved away. The remaining kinds are listed in
/// `NotationCase`.
///
/// (Note that this is not a "visitor". It does not invoke a callback on every node. Instead, it
///  provides a `.case()` method that allows you to walk the (simplified) tree yourself.)
#[derive(Debug, Clone, Copy)]
pub enum NotationWalker<'d, D: PrettyDoc<'d>> {
    Empty,
    Newline(Width),
    Literal(&'d Literal),
    Text(&'d str, Style),
    Concat(DelayedNotationWalker<'d, D>, DelayedNotationWalker<'d, D>),
    Choice(DelayedNotationWalker<'d, D>, DelayedNotationWalker<'d, D>),
    Child(usize, DelayedNotationWalker<'d, D>),
}

#[derive(Debug, Clone, Copy)]
pub struct DelayedNotationWalker<'d, D: PrettyDoc<'d>> {
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

impl<'d, D: PrettyDoc<'d>> NotationWalker<'d, D> {
    pub fn new(doc: D) -> NotationWalker<'d, D> {
        DelayedNotationWalker {
            doc,
            notation: &doc.notation().0,
            flat: false,
            indent: 0,
            join_pos: None,
        }
        .force()
    }
}

impl<'d, D: PrettyDoc<'d>> DelayedNotationWalker<'d, D> {
    pub fn force(mut self) -> NotationWalker<'d, D> {
        use Notation::*;

        match self.notation {
            Empty => NotationWalker::Empty,
            Newline => NotationWalker::Newline(self.indent),
            Literal(lit) => NotationWalker::Literal(lit),
            Text(style) => NotationWalker::Text(self.doc.unwrap_text(), *style),
            Flat(note) => {
                self.flat = true;
                self.notation = note;
                self.force()
            }
            Indent(indent, note) => {
                self.indent += indent;
                self.notation = note;
                self.force()
            }
            Concat(left, right) => {
                NotationWalker::Concat(self.with_notation(left), self.with_notation(right))
            }
            Choice(left, right) if self.flat => self.with_notation(left).force(),
            Choice(left, right) => {
                NotationWalker::Choice(self.with_notation(left), self.with_notation(right))
            }
            IfEmptyText(note1, note2) => {
                if self.doc.num_children().is_some() {
                    panic!("IfEmptyText used on PrettyDoc with children")
                }
                if self.doc.unwrap_text().is_empty() {
                    self.with_notation(note1).force()
                } else {
                    self.with_notation(note2).force()
                }
            }
            Child(i) => {
                let n = match self.doc.num_children() {
                    None => panic!("Attempt to access child {} in texty PrettyDoc node", *i),
                    Some(n) if *i >= n => panic!("Child {} out of range {}", *i, n),
                    Some(n) => n,
                };
                self.doc = self.doc.unwrap_child(n);
                self.notation = &self.doc.notation().0;
                NotationWalker::Child(*i, self)
            }
            Count { zero, one, many } => match self.doc.num_children() {
                None => panic!("Count used on texty doc node"),
                Some(0) => self.with_notation(zero).force(),
                Some(1) => self.with_notation(one).force(),
                Some(_) => self.with_notation(many).force(),
            },
            Fold { first, join } => match self.doc.num_children() {
                None => panic!("Fold used on texty doc node"),
                Some(0) => self.with_notation(first).force(),
                Some(n) => {
                    self.join_pos = Some(JoinPos {
                        parent: self.doc,
                        child: self.doc.unwrap_last_child(),
                        index: n - 1,
                        first,
                        join,
                    });
                    self.notation = join;
                    self.force()
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
                    if *index - 1 == 0 {
                        self.notation = *first;
                        self.join_pos = None;
                        self.force()
                    } else {
                        *child = child.unwrap_prev_sibling(*parent, *index - 1);
                        *index -= 1;
                        self.notation = *join;
                        self.force()
                    }
                }
            },
            Right => match &mut self.join_pos {
                None => {
                    panic!("Bug: Right used outside of fold; should have been caught by validation")
                }
                Some(JoinPos { child, .. }) => {
                    self.doc = *child;
                    self.join_pos = None;
                    self.force()
                }
            },
        }
    }

    fn with_notation(self, notation: &'d Notation) -> Self {
        let mut result = self;
        result.notation = notation;
        result
    }
}
