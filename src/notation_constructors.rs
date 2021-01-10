use super::notation::{Literal, Notation, RepeatInner};
use super::style::Style;

pub fn nl() -> Notation {
    Notation::Newline
}

pub fn child(i: usize) -> Notation {
    Notation::Child(i)
}

pub fn text() -> Notation {
    Notation::Text(Style::default())
}

pub fn lit(s: &str) -> Notation {
    let literal = Literal::new(s, Style::default());
    Notation::Literal(Box::new(literal))
}

pub fn flat(n: Notation) -> Notation {
    Notation::Flat(Box::new(n))
}

pub fn left() -> Notation {
    Notation::Left
}

pub fn right() -> Notation {
    Notation::Right
}

pub fn surrounded() -> Notation {
    Notation::Surrounded
}

pub fn repeat(repeat: RepeatInner) -> Notation {
    Notation::Repeat(Box::new(repeat))
}
