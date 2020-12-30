use crate::notation::Notation;
use std::fmt;

pub trait Doc {
    fn child(&self, index: usize) -> &Self;
    fn notation(&self) -> &Notation;
}

#[derive(Debug)]
pub struct NotationRef<'d, D: Doc> {
    pub notation: &'d Notation,
    pub doc: &'d D,
}

pub enum NotationCase<'d, D: Doc> {
    Empty,
    Literal(&'d str),
    Newline,
    Flat(NotationRef<'d, D>),
    Indent(usize, NotationRef<'d, D>),
    Concat(NotationRef<'d, D>, NotationRef<'d, D>),
    Choice(NotationRef<'d, D>, NotationRef<'d, D>),
    Child(usize, NotationRef<'d, D>),
}

impl<'d, D: Doc> Clone for NotationRef<'d, D> {
    fn clone(&self) -> NotationRef<'d, D> {
        NotationRef {
            notation: &self.notation,
            doc: &self.doc,
        }
    }
}
impl<'d, D: Doc> Copy for NotationRef<'d, D> {}

impl<'d, D: Doc> NotationRef<'d, D> {
    pub fn case(self) -> NotationCase<'d, D> {
        match &self.notation {
            Notation::Empty => NotationCase::Empty,
            Notation::Literal(lit) => NotationCase::Literal(lit),
            Notation::Newline => NotationCase::Newline,
            Notation::Flat(note) => NotationCase::Flat(self.subnotation(note)),
            Notation::Indent(i, note) => NotationCase::Indent(*i, self.subnotation(note)),
            Notation::Concat(left, right) => {
                NotationCase::Concat(self.subnotation(left), self.subnotation(right))
            }
            Notation::Choice(left, right) => {
                NotationCase::Choice(self.subnotation(left), self.subnotation(right))
            }
            Notation::Child(i) => NotationCase::Child(*i, self.child(*i)),
        }
    }

    fn subnotation(&self, notation: &'d Notation) -> NotationRef<'d, D> {
        NotationRef {
            notation,
            doc: self.doc,
        }
    }

    fn child(&self, index: usize) -> NotationRef<'d, D> {
        let child = self.doc.child(index);
        NotationRef {
            notation: child.notation(),
            doc: child,
        }
    }
}

impl<'d, D: Doc> fmt::Display for NotationRef<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.notation)
    }
}
