use crate::notation::{Notation, RepeatInner};
use std::fmt;

/// A "document" that supports the necessary methods to be pretty-printed.
pub trait Doc {
    type Id: Eq + Copy;
    type TextRef: AsRef<str>;

    fn id(&self) -> Self::Id;
    fn num_children(&self) -> usize;
    fn child(&self, index: usize) -> &Self;
    fn notation(&self) -> &Notation;
    fn text(&self) -> Option<Self::TextRef>;
}

#[derive(Debug)]
pub struct NotationRef<'d, D: Doc> {
    doc: &'d D,
    notation: &'d Notation,
    repeat_pos: RepeatPos<'d>,
}

#[derive(Debug, Clone, Copy)]
enum RepeatPos<'d> {
    None,
    Surround(&'d RepeatInner),
    Join(&'d RepeatInner, usize),
}

#[derive(Debug)]
pub enum NotationCase<'d, D: Doc> {
    Empty,
    Literal(&'d str),
    Newline,
    Text(D::TextRef),
    Flat(NotationRef<'d, D>),
    Indent(usize, NotationRef<'d, D>),
    Concat(NotationRef<'d, D>, NotationRef<'d, D>),
    Choice(NotationRef<'d, D>, NotationRef<'d, D>),
    Child(usize, NotationRef<'d, D>),
}

impl<'d, D: Doc> Clone for NotationRef<'d, D> {
    fn clone(&self) -> NotationRef<'d, D> {
        NotationRef {
            doc: self.doc,
            notation: self.notation,
            repeat_pos: self.repeat_pos,
        }
    }
}
impl<'d, D: Doc> Copy for NotationRef<'d, D> {}

impl<'d, D: Doc> NotationRef<'d, D> {
    pub fn case(self) -> NotationCase<'d, D> {
        match self.notation {
            Notation::Empty => NotationCase::Empty,
            Notation::Literal(lit) => NotationCase::Literal(lit),
            Notation::Newline => NotationCase::Newline,
            Notation::Text => NotationCase::Text(self.doc.text().unwrap()),
            Notation::Flat(note) => NotationCase::Flat(self.subnotation(note)),
            Notation::Indent(i, note) => NotationCase::Indent(*i, self.subnotation(note)),
            Notation::Concat(left, right) => {
                NotationCase::Concat(self.subnotation(left), self.subnotation(right))
            }
            Notation::Choice(left, right) => {
                NotationCase::Choice(self.subnotation(left), self.subnotation(right))
            }
            Notation::Child(i) => NotationCase::Child(*i, self.child(*i)),
            Notation::Left => {
                if let RepeatPos::Join(_, i) = self.repeat_pos {
                    NotationCase::Child(i, self.child(i))
                } else {
                    unreachable!()
                }
            }
            Notation::Right => {
                if let RepeatPos::Join(_, i) = self.repeat_pos {
                    NotationCase::Child(i + 1, self.child(i + 1))
                } else {
                    unreachable!()
                }
            }
            Notation::Repeat(_) | Notation::Surrounded | Notation::IfEmptyText(_, _) => {
                unreachable!()
            }
        }
    }

    pub fn new(doc: &'d D) -> NotationRef<'d, D> {
        NotationRef::from_parts(doc, doc.notation(), RepeatPos::None)
    }

    pub fn doc_id(&self) -> D::Id {
        self.doc.id()
    }

    fn from_parts(
        doc: &'d D,
        notation: &'d Notation,
        parent_repeat_pos: RepeatPos<'d>,
    ) -> NotationRef<'d, D> {
        use Notation::*;

        let mut refn = NotationRef {
            doc,
            notation,
            repeat_pos: parent_repeat_pos,
        };

        loop {
            match refn.notation {
                Repeat(repeat) => {
                    assert!(
                        matches!(parent_repeat_pos, RepeatPos::None),
                        "Can't handle nested repeats"
                    );
                    refn.notation = match doc.num_children() {
                        0 => &repeat.empty,
                        1 => &repeat.lone,
                        _ => &repeat.surround,
                    };
                    refn.repeat_pos = RepeatPos::Surround(repeat);
                }
                Surrounded => {
                    if let RepeatPos::Surround(repeat) = refn.repeat_pos {
                        refn.notation = &repeat.join;
                        refn.repeat_pos = RepeatPos::Join(repeat, 0);
                    } else {
                        panic!("`Surrounded` is only allowed in `RepeatInner::surround`");
                    }
                }
                Left => {
                    if let RepeatPos::Join(_, _) = refn.repeat_pos {
                        // Semantically, this notation is equivalent to `Child(i)`.
                        // But there's no allocated `Child(i)` Notation to reference,
                        // so w'll break and let the `case` method deal with it.
                        break;
                    } else {
                        panic!("`Left` is only allowed in `RepeatInner::join`");
                    }
                }
                Right => {
                    if let RepeatPos::Join(repeat, i) = refn.repeat_pos {
                        if i + 2 == refn.doc.num_children() {
                            // Similar to the Left case: equivalent to Child(i)
                            break;
                        } else {
                            refn.notation = &repeat.join;
                            refn.repeat_pos = RepeatPos::Join(repeat, i + 1);
                        }
                    } else {
                        panic!("`Right` is only allowed in `RepeatInner::join`");
                    }
                }
                IfEmptyText(opt1, opt2) => {
                    if refn.doc.text().is_none() {
                        refn.notation = opt1;
                    } else {
                        refn.notation = opt2;
                    }
                }
                Empty
                | Literal(_)
                | Newline
                | Text
                | Indent(_, _)
                | Flat(_)
                | Concat(_, _)
                | Choice(_, _)
                | Child(_) => break,
            }
        }

        refn
    }

    fn subnotation(&self, notation: &'d Notation) -> NotationRef<'d, D> {
        NotationRef::from_parts(self.doc, notation, self.repeat_pos)
    }

    fn child(&self, index: usize) -> NotationRef<'d, D> {
        let child = self.doc.child(index);
        NotationRef::from_parts(child, child.notation(), RepeatPos::None)
    }
}

impl<'d, D: Doc> fmt::Display for NotationRef<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.notation)
    }
}
