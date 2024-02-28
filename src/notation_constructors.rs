// TODO: docs

use super::notation::{CheckPos, Condition, Literal, Notation, StyleLabel};

pub fn empty<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Empty
}

pub fn nl<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Newline
}

pub fn child<L: StyleLabel, C: Condition>(i: usize) -> Notation<L, C> {
    Notation::Child(i)
}

pub fn style<L: StyleLabel, C: Condition>(style_label: L, n: Notation<L, C>) -> Notation<L, C> {
    Notation::Style(style_label, Box::new(n))
}

pub fn text<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Text
}

pub fn lit<L: StyleLabel, C: Condition>(s: &str) -> Notation<L, C> {
    Notation::Literal(Literal::new(s))
}

pub fn flat<L: StyleLabel, C: Condition>(n: Notation<L, C>) -> Notation<L, C> {
    Notation::Flat(Box::new(n))
}

pub fn indent<L: StyleLabel, C: Condition>(
    s: &str,
    style_label: Option<L>,
    n: Notation<L, C>,
) -> Notation<L, C> {
    Notation::Indent(Literal::new(s), style_label, Box::new(n))
}

pub fn check<L: StyleLabel, C: Condition>(
    condition: C,
    pos: CheckPos,
    then_notation: Notation<L, C>,
    else_notation: Notation<L, C>,
) -> Notation<L, C> {
    Notation::Check(
        condition,
        pos,
        Box::new(then_notation),
        Box::new(else_notation),
    )
}

/* Count */

pub struct Count<L: StyleLabel, C: Condition> {
    pub zero: Notation<L, C>,
    pub one: Notation<L, C>,
    pub many: Notation<L, C>,
}

pub fn count<L: StyleLabel, C: Condition>(count: Count<L, C>) -> Notation<L, C> {
    Notation::Count {
        zero: Box::new(count.zero),
        one: Box::new(count.one),
        many: Box::new(count.many),
    }
}

/* Fold */

pub struct Fold<L: StyleLabel, C: Condition> {
    pub first: Notation<L, C>,
    pub join: Notation<L, C>,
}

pub fn fold<L: StyleLabel, C: Condition>(fold: Fold<L, C>) -> Notation<L, C> {
    Notation::Fold {
        first: Box::new(fold.first),
        join: Box::new(fold.join),
    }
}

pub fn left<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Left
}

pub fn right<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Right
}
