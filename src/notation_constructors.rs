// TODO: docs

use super::notation::{Literal, Notation};

pub fn empty<S>() -> Notation<S> {
    Notation::Empty
}

pub fn nl<S>() -> Notation<S> {
    Notation::Newline
}

pub fn child<S>(i: usize) -> Notation<S> {
    Notation::Child(i)
}

pub fn text<S>(style: S) -> Notation<S> {
    Notation::Text(style)
}

pub fn lit<S>(s: &str, style: S) -> Notation<S> {
    let literal = Literal::new(s, style);
    Notation::Literal(literal)
}

pub fn flat<S>(n: Notation<S>) -> Notation<S> {
    Notation::Flat(Box::new(n))
}

// TODO: new method with int indent
pub fn indent<S>(indent: &str, style: S, n: Notation<S>) -> Notation<S> {
    Notation::Indent(Literal::new(indent, style), Box::new(n))
}

pub fn mark<S>(mark_name: &'static str, n: Notation<S>) -> Notation<S> {
    Notation::Mark(mark_name, Box::new(n))
}

/* Count */

pub struct Count<S> {
    pub zero: Notation<S>,
    pub one: Notation<S>,
    pub many: Notation<S>,
}

pub fn count<S>(count: Count<S>) -> Notation<S> {
    Notation::Count {
        zero: Box::new(count.zero),
        one: Box::new(count.one),
        many: Box::new(count.many),
    }
}

/* Fold */

pub struct Fold<S> {
    pub first: Notation<S>,
    pub join: Notation<S>,
}

pub fn fold<S>(fold: Fold<S>) -> Notation<S> {
    Notation::Fold {
        first: Box::new(fold.first),
        join: Box::new(fold.join),
    }
}

pub fn left<S>() -> Notation<S> {
    Notation::Left
}

pub fn right<S>() -> Notation<S> {
    Notation::Right
}
