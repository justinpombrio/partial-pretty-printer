//! Convenient functions for constructing [`Notation`]s.
//!
//! There are also shorthand operators for the most common constructors:
//!
//! - `x + y` is shorthand for [`Concat`](Notation::Concat)`(x, y)`.
//! - `x | y` is shorthand for [`Choice`](Notation::Choice)`(x, y)`.
//! - `x ^ y` is shorthand for `x + `[`Newline`](Notation::Newline)` + y`.
//! - `i >> x` is shorthand for [`Indent`](Notation::Indent)`(i_spaces,
//!   `[`Newline`](Notation::Newline)` + x)` (sometimes called "nesting").

use crate::{CheckPos, Condition, Literal, Notation, StyleLabel};

/// Construct a [`Notation::Empty`].
pub fn empty<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Empty
}

/// Construct a [`Notation::Newline`].
pub fn nl<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Newline
}

/// Construct a [`Notation::EndOfLine`].
pub fn eol<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::EndOfLine
}

/// Construct a [`Notation::Child`].
pub fn child<L: StyleLabel, C: Condition>(i: isize) -> Notation<L, C> {
    Notation::Child(i)
}

/// Construct a [`Notation::Style`].
pub fn style<L: StyleLabel, C: Condition>(style_label: L, n: Notation<L, C>) -> Notation<L, C> {
    Notation::Style(style_label, Box::new(n))
}

/// Construct a [`Notation::Text`].
pub fn text<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Text
}

/// Construct a [`Notation::Literal`].
pub fn lit<L: StyleLabel, C: Condition>(s: &str) -> Notation<L, C> {
    Notation::Literal(Literal::new(s))
}

/// Construct a [`Notation::Flat`].
pub fn flat<L: StyleLabel, C: Condition>(n: Notation<L, C>) -> Notation<L, C> {
    Notation::Flat(Box::new(n))
}

/// Construct a [`Notation::Indent`].
pub fn indent<L: StyleLabel, C: Condition>(
    s: &str,
    style_label: Option<L>,
    n: Notation<L, C>,
) -> Notation<L, C> {
    Notation::Indent(Literal::new(s), style_label, Box::new(n))
}

/// Construct a [`Notation::Check`].
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

/// The arguments to [`count()`].
pub struct Count<L: StyleLabel, C: Condition> {
    pub zero: Notation<L, C>,
    pub one: Notation<L, C>,
    pub many: Notation<L, C>,
}

/// Construct a [`Notation::Count`].
pub fn count<L: StyleLabel, C: Condition>(count: Count<L, C>) -> Notation<L, C> {
    Notation::Count {
        zero: Box::new(count.zero),
        one: Box::new(count.one),
        many: Box::new(count.many),
    }
}

/// The arguments to [`fold()`].
pub struct Fold<L: StyleLabel, C: Condition> {
    pub first: Notation<L, C>,
    pub join: Notation<L, C>,
}

/// Construct a [`Notation::Fold`].
pub fn fold<L: StyleLabel, C: Condition>(fold: Fold<L, C>) -> Notation<L, C> {
    Notation::Fold {
        first: Box::new(fold.first),
        join: Box::new(fold.join),
    }
}

/// Construct a [`Notation::Left`].
pub fn left<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Left
}

/// Construct a [`Notation::Right`].
pub fn right<L: StyleLabel, C: Condition>() -> Notation<L, C> {
    Notation::Right
}
