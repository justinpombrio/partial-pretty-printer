// TODO: docs

use super::notation::{Literal, Notation};
use std::fmt;

pub fn empty<L: fmt::Debug + Clone>() -> Notation<L> {
    Notation::Empty
}

pub fn nl<L: fmt::Debug + Clone>() -> Notation<L> {
    Notation::Newline
}

pub fn child<L: fmt::Debug + Clone>(i: usize) -> Notation<L> {
    Notation::Child(i)
}

pub fn style<L: fmt::Debug + Clone>(style_label: L, n: Notation<L>) -> Notation<L> {
    Notation::Style(style_label, Box::new(n))
}

pub fn text<L: fmt::Debug + Clone>() -> Notation<L> {
    Notation::Text
}

pub fn lit<L: fmt::Debug + Clone>(s: &str) -> Notation<L> {
    Notation::Literal(Literal::new(s))
}

pub fn flat<L: fmt::Debug + Clone>(n: Notation<L>) -> Notation<L> {
    Notation::Flat(Box::new(n))
}

pub fn indent<L: fmt::Debug + Clone>(
    s: &str,
    style_label: Option<L>,
    n: Notation<L>,
) -> Notation<L> {
    Notation::Indent(Literal::new(s), style_label, Box::new(n))
}

/* Count */

pub struct Count<L: fmt::Debug + Clone> {
    pub zero: Notation<L>,
    pub one: Notation<L>,
    pub many: Notation<L>,
}

pub fn count<L: fmt::Debug + Clone>(count: Count<L>) -> Notation<L> {
    Notation::Count {
        zero: Box::new(count.zero),
        one: Box::new(count.one),
        many: Box::new(count.many),
    }
}

/* Fold */

pub struct Fold<L: fmt::Debug + Clone> {
    pub first: Notation<L>,
    pub join: Notation<L>,
}

pub fn fold<L: fmt::Debug + Clone>(fold: Fold<L>) -> Notation<L> {
    Notation::Fold {
        first: Box::new(fold.first),
        join: Box::new(fold.join),
    }
}

pub fn left<L: fmt::Debug + Clone>() -> Notation<L> {
    Notation::Left
}

pub fn right<L: fmt::Debug + Clone>() -> Notation<L> {
    Notation::Right
}
