use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::notation::{Literal, Notation, RepeatInner, EMPTY_LITERAL};
use crate::style::Style;
use std::fmt;

const START_OF_DOC: &'static Notation = &Notation::Newline;

/// Walk all of the notations in a Doc. (Kind of like an Iterator, but tree shaped.) `Child` is
/// replaced by that child's notation, `IfEmptyText` is replaced by its left or right option, and
/// `Repeat` is replaced by the notation it defines.
pub struct NotationRef<'d, D: PrettyDoc<'d>> {
    doc: D,
    flat: bool,
    indent: Width,
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
    Literal(&'d Literal),
    Newline,
    Text(&'d str, Style),
    Concat(NotationRef<'d, D>, NotationRef<'d, D>),
    Choice(NotationRef<'d, D>, NotationRef<'d, D>),
    Child(usize, NotationRef<'d, D>),
}

impl<'d, D: PrettyDoc<'d>> fmt::Debug for NotationRef<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NotationRef {{ flat:{}, indent:{}, notation: {} }}",
            self.flat, self.indent, self.notation
        )
    }
}

impl<'d, D: PrettyDoc<'d>> Clone for NotationRef<'d, D> {
    fn clone(&self) -> NotationRef<'d, D> {
        NotationRef {
            flat: self.flat,
            indent: self.indent,
            doc: self.doc,
            notation: self.notation,
            repeat_pos: self.repeat_pos,
        }
    }
}
impl<'d, D: PrettyDoc<'d>> Copy for NotationRef<'d, D> {}

impl<'d, D: PrettyDoc<'d>> NotationRef<'d, D> {
    pub fn is_flat(self) -> bool {
        self.flat
    }

    pub fn indentation(self) -> Width {
        self.indent
    }

    pub fn case(self) -> NotationCase<'d, D> {
        match self.notation {
            Notation::Empty => NotationCase::Literal(EMPTY_LITERAL),
            Notation::Literal(lit) => NotationCase::Literal(lit),
            Notation::Newline => NotationCase::Newline,
            Notation::Text(style) => NotationCase::Text(self.doc.unwrap_text(), *style),
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
            Notation::Repeat(_)
            | Notation::Surrounded
            | Notation::IfEmptyText(_, _)
            | Notation::Flat(_)
            | Notation::Indent(_, _) => {
                unreachable!()
            }
        }
    }

    pub fn new(doc: D) -> NotationRef<'d, D> {
        NotationRef::from_parts(false, 0, doc, &doc.notation().0, RepeatPos::None)
    }

    // Turns out it's _really_ convenient to put a fake newline at the start of the document.
    pub fn make_fake_start_of_doc_newline(&self) -> NotationRef<'d, D> {
        NotationRef {
            flat: false,
            indent: 0,
            doc: self.doc,
            notation: START_OF_DOC,
            repeat_pos: RepeatPos::None,
        }
    }

    pub fn doc_id(&self) -> D::Id {
        self.doc.id()
    }

    fn from_parts(
        flat: bool,
        indent: Width,
        doc: D,
        notation: &'d Notation,
        parent_repeat_pos: RepeatPos<'d>,
    ) -> NotationRef<'d, D> {
        use Notation::*;

        let mut refn = NotationRef {
            flat,
            indent,
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
                        unreachable!("`Surrounded` is only allowed in `RepeatInner::surround`");
                    }
                }
                Left => {
                    if let RepeatPos::Join(_, _) = refn.repeat_pos {
                        // Semantically, this notation is equivalent to `Child(i)`.
                        // But there's no allocated `Child(i)` Notation to reference,
                        // so w'll break and let the `case` method deal with it.
                        break;
                    } else {
                        unreachable!("`Left` is only allowed in `RepeatInner::join`");
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
                        unreachable!("`Right` is only allowed in `RepeatInner::join`");
                    }
                }
                IfEmptyText(opt1, opt2) => {
                    if refn.doc.unwrap_text().is_empty() {
                        refn.notation = opt1;
                    } else {
                        refn.notation = opt2;
                    }
                }
                Flat(note) => {
                    refn.flat = true;
                    refn.notation = note;
                }
                Indent(i, note) => {
                    refn.indent += i;
                    refn.notation = note;
                }
                Empty | Literal(_) | Newline | Text(_) | Concat(_, _) | Choice(_, _) | Child(_) => {
                    break
                }
            }
        }

        refn
    }

    fn subnotation(&self, notation: &'d Notation) -> NotationRef<'d, D> {
        NotationRef::from_parts(self.flat, self.indent, self.doc, notation, self.repeat_pos)
    }

    fn child(&self, index: usize) -> NotationRef<'d, D> {
        let child = self.doc.unwrap_child(index);
        NotationRef::from_parts(
            self.flat,
            self.indent,
            child,
            &child.notation().0,
            RepeatPos::None,
        )
    }
}

impl<'d, D: PrettyDoc<'d>> fmt::Display for NotationRef<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.notation)
    }
}
