// TODO: docs

use super::notation::{Literal, Notation, StyleLabel};

pub fn empty<L: StyleLabel>() -> Notation<L> {
    Notation::Empty
}

pub fn nl<L: StyleLabel>() -> Notation<L> {
    Notation::Newline
}

pub fn child<L: StyleLabel>(i: usize) -> Notation<L> {
    Notation::Child(i)
}

pub fn style<L: StyleLabel>(style_label: L, n: Notation<L>) -> Notation<L> {
    Notation::Style(style_label, Box::new(n))
}

pub fn text<L: StyleLabel>() -> Notation<L> {
    Notation::Text
}

pub fn lit<L: StyleLabel>(s: &str) -> Notation<L> {
    Notation::Literal(Literal::new(s))
}

pub fn flat<L: StyleLabel>(n: Notation<L>) -> Notation<L> {
    Notation::Flat(Box::new(n))
}

pub fn indent<L: StyleLabel>(s: &str, style_label: Option<L>, n: Notation<L>) -> Notation<L> {
    Notation::Indent(Literal::new(s), style_label, Box::new(n))
}

/* Count */

pub struct Count<L: StyleLabel> {
    pub zero: Notation<L>,
    pub one: Notation<L>,
    pub many: Notation<L>,
}

pub fn count<L: StyleLabel>(count: Count<L>) -> Notation<L> {
    Notation::Count {
        zero: Box::new(count.zero),
        one: Box::new(count.one),
        many: Box::new(count.many),
    }
}

/* Fold */

pub struct Fold<L: StyleLabel> {
    pub first: Notation<L>,
    pub join: Notation<L>,
}

pub fn fold<L: StyleLabel>(fold: Fold<L>) -> Notation<L> {
    Notation::Fold {
        first: Box::new(fold.first),
        join: Box::new(fold.join),
    }
}

pub fn left<L: StyleLabel>() -> Notation<L> {
    Notation::Left
}

pub fn right<L: StyleLabel>() -> Notation<L> {
    Notation::Right
}
