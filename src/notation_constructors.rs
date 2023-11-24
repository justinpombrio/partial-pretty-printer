// TODO: docs

use super::geometry::Width;
use super::notation::{Literal, Notation};
use super::style::Style;

pub fn empty() -> Notation {
    Notation::Empty
}

pub fn nl() -> Notation {
    Notation::Newline
}

pub fn child(i: usize) -> Notation {
    Notation::Child(i)
}

pub fn text(style: Style) -> Notation {
    Notation::Text(style)
}

pub fn lit(s: &str, style: Style) -> Notation {
    let literal = Literal::new(s, style);
    Notation::Literal(literal)
}

pub fn flat(n: Notation) -> Notation {
    Notation::Flat(Box::new(n))
}

pub fn indent(i: Width, n: Notation) -> Notation {
    Notation::Indent(i, Box::new(n))
}

/* Count */

pub struct Count {
    zero: Notation,
    one: Notation,
    many: Notation,
}

pub fn count(count: Count) -> Notation {
    Notation::Count {
        zero: Box::new(count.zero),
        one: Box::new(count.one),
        many: Box::new(count.many),
    }
}

/* Fold */

pub struct Fold {
    first: Notation,
    join: Notation,
}

pub fn fold(fold: Fold) -> Notation {
    Notation::Fold {
        first: Box::new(fold.first),
        join: Box::new(fold.join),
    }
}

pub fn left() -> Notation {
    Notation::Left
}

pub fn right() -> Notation {
    Notation::Right
}
