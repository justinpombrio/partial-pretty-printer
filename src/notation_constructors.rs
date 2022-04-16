use super::geometry::Width;
use super::notation::{Literal, Notation, RepeatInner};
use super::style::Style;

pub fn empty() -> Notation {
    Notation::Empty
}

pub fn nl() -> Notation {
    Notation::Newline
}

pub fn ws(spaces: &str) -> Notation {
    if_flat(lit(spaces, Style::plain()), nl())
}

pub fn child(i: usize) -> Notation {
    Notation::Child(i)
}

pub fn text(style: Style) -> Notation {
    Notation::Text(style)
}

pub fn lit(s: &str, style: Style) -> Notation {
    let literal = Literal::new(s, style);
    Notation::Literal(Box::new(literal))
}

pub fn half_nestled(ind: Width, left_ws: &str, n: Notation) -> Notation {
    group(indent(ind, ws(left_ws) + n))
}

pub fn nestled(ind: Width, left_ws: &str, n: Notation, right_ws: &str) -> Notation {
    group(indent(ind, ws(left_ws) + n) + ws(right_ws))
}

pub fn indent(i: Width, n: Notation) -> Notation {
    Notation::Indent(i, Box::new(n))
}

// Equivalent to `Choice(Flat(n), n)`
pub fn group(n: Notation) -> Notation {
    Notation::Group(Box::new(n))
}

pub fn flat(n: Notation) -> Notation {
    Notation::Flat(Box::new(n))
}

pub fn if_flat(left: Notation, right: Notation) -> Notation {
    Notation::IfFlat(Box::new(left), Box::new(right))
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
