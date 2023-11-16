use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::notation::{Literal, Notation, RepeatInner};
use crate::style::Style;
use std::fmt;

const START_OF_DOC: &'static Notation = &Notation::Newline;

#[derive(Debug)]
pub struct NotationRef<'d, D: PrettyDoc<'d>> {
    doc: D,
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
pub enum NotationCase<'d, D: PrettyDoc<'d>> {
    Empty,
    Literal(&'d Literal),
    Newline,
    Text(&'d str, Style),
    Flat(NotationRef<'d, D>),
    Indent(Width, NotationRef<'d, D>),
    Concat(NotationRef<'d, D>, NotationRef<'d, D>),
    Choice(NotationRef<'d, D>, NotationRef<'d, D>),
    Child(usize, NotationRef<'d, D>),
}

impl<'d, D: PrettyDoc<'d>> Clone for NotationRef<'d, D> {
    fn clone(&self) -> NotationRef<'d, D> {
        NotationRef {
            doc: self.doc,
            notation: self.notation,
            repeat_pos: self.repeat_pos,
        }
    }
}
impl<'d, D: PrettyDoc<'d>> Copy for NotationRef<'d, D> {}

impl<'d, D: PrettyDoc<'d>> NotationRef<'d, D> {
    pub fn case(self) -> NotationCase<'d, D> {
        match self.notation {
            Notation::Empty => NotationCase::Empty,
            Notation::Literal(lit) => NotationCase::Literal(lit),
            Notation::Newline => NotationCase::Newline,
            Notation::Text(style) => NotationCase::Text(self.doc.unwrap_text(), *style),
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

    pub fn new(doc: D) -> NotationRef<'d, D> {
        NotationRef::from_parts(doc, &doc.notation().0, RepeatPos::None)
    }

    // Turns out it's _really_ convenient to put a fake newline at the start of the document.
    pub fn make_fake_start_of_doc_newline(&self) -> NotationRef<'d, D> {
        NotationRef {
            doc: self.doc,
            notation: START_OF_DOC,
            repeat_pos: RepeatPos::None,
        }
    }

    pub fn doc_id(&self) -> D::Id {
        self.doc.id()
    }

    fn from_parts(
        doc: D,
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
                    refn.notation = match doc.num_children().unwrap() {
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
                        if i + 2 == refn.doc.num_children().unwrap() {
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
                    if refn.doc.unwrap_text().is_empty() {
                        refn.notation = opt1;
                    } else {
                        refn.notation = opt2;
                    }
                }
                Empty
                | Literal(_)
                | Newline
                | Text(_)
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
        let child = self.doc.unwrap_child(index);
        NotationRef::from_parts(child, &child.notation().0, RepeatPos::None)
    }
}

impl<'d, D: PrettyDoc<'d>> fmt::Display for NotationRef<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.notation)
    }
}
