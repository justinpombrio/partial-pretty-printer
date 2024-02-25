use crate::geometry::{str_width, Width};
use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

/// Describes how to display a syntactic construct. When constructing a Notation, you must obey one
/// requirement. If you do not, the pretty printer may choose poor layouts.
///
/// > For every choice `(x | y)`, the first line of `x` is shorter than (or equal to) the first
///   line of `y`.
///
/// Additionally, whenever possible, `x` should be flat (contain no newlines). This allows
/// notations to use the `Flat` variant to attempt to fit `(x | y)` all on one line.
#[derive(Clone, Debug)]
pub enum Notation<S> {
    /// Display nothing.
    Empty,
    /// Display a newline followed by the current indentation. (See [`Notation::Indent`]).
    Newline,
    /// Display a piece of text. Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `None` (implying that it contains text).
    Text(S),
    /// Literal text. Cannot contain a newline.
    Literal(Literal<S>),
    /// Use the leftmost option of every choice in the contained notation. If the notation author
    /// followed the recommendation of not putting `Newline`s in the left-most options of choices,
    /// then this `Flat` will be displayed all on one line.
    Flat(Box<Notation<S>>),
    /// Append a string to the indentation of the contained notation. All of the
    /// indentation strings will be displayed after `Newline`s. (They therefore
    /// don't affect the first line of a notation.) Indentation strings will
    /// typically contain one indentation level's worth of whitespace characters
    /// (eg. 4 spaces), but can also be used for other purposes like placing comment
    /// syntax at the start of a line.
    Indent(Literal<S>, Box<Notation<S>>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. Note that the column at which the right notation
    /// starts does not affect its indentation level.
    Concat(Box<Notation<S>>, Box<Notation<S>>),
    /// If we're inside a `Flat`, _or_ the first line of the left notation fits within the required
    /// width, then display the left notation. Otherwise, display the right notation.
    Choice(Box<Notation<S>>, Box<Notation<S>>),
    /// If this [`PrettyDoc`] node is a text node containing the empty string, display the left
    /// notation, otherwise show the right notation.
    IfEmptyText(Box<Notation<S>>, Box<Notation<S>>),
    /// Display the `i`th child of this node.  Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `Some(n)`, with `i < n`.
    Child(usize),
    /// Look up the mark with the given name in the current node, if any, and apply it to this
    /// notation. (The lookup happens via `PrettyDoc::partial_node_mark()`.)
    Mark(&'static str, Box<Notation<S>>),
    /// Determines what to display based on the number of children this [`PrettyDoc`] node has.
    Count {
        zero: Box<Notation<S>>,
        one: Box<Notation<S>>,
        many: Box<Notation<S>>,
    },
    /// Fold (a.k.a. reduce) over the node's children. This is a left-fold. May only be used in
    /// `Notation::Count.many`.
    Fold {
        /// How to display the first child on its own.
        first: Box<Notation<S>>,
        /// How to append an additional child onto a partially displayed sequence of children.
        /// Within this notation, `Notation::Left` refers to the children displayed so far,
        /// and `Notation::Right` refers to the next child to be appended.
        join: Box<Notation<S>>,
    },
    /// Used in `Fold.join` to refer to the accumulated Notation.
    /// Illegal outside of `Fold`.
    Left,
    /// Used in `Fold.join` to refer to the next child.
    /// Illegal outside of `Fold`.
    Right,
}

/// Literal text, to be displayed as-is. Cannot contain a newline.
#[derive(Clone, Debug)]
pub struct Literal<S> {
    string: String,
    /// Width of the string in [`Col`]s. See [`Width`].
    width: Width,
    style: S,
}

impl<S> Literal<S> {
    pub fn new(s: &str, style: S) -> Literal<S> {
        Literal {
            string: s.to_owned(),
            width: str_width(s),
            style,
        }
    }

    pub fn width(&self) -> Width {
        self.width
    }

    pub fn str(&self) -> &str {
        &self.string
    }

    pub fn style(&self) -> &S {
        &self.style
    }
}

// For debugging. Should match impl fmt::Display for ConsolidatedNotation.
impl<S> fmt::Display for Notation<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Notation::*;

        match self {
            Empty => write!(f, "ε"),
            Newline => write!(f, "↵"),
            Text(_) => write!(f, "TEXT"),
            Literal(lit) => write!(f, "'{}'", lit.string),
            Flat(note) => write!(f, "Flat({})", note),
            Indent(lit, note) => write!(f, "'{}'⇒({})", lit.string, note),
            Concat(left, right) => write!(f, "{} + {}", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            IfEmptyText(opt1, opt2) => write!(f, "IfEmptyText({} | {})", opt1, opt2),
            Child(i) => write!(f, "${}", i),
            Mark(mark_name, note) => write!(f, "Mark({}, {})", mark_name, note),
            Count { zero, one, many } => {
                write!(f, "Count(zero={}, one={}, many={})", zero, one, many)
            }
            Fold { first, join } => write!(f, "Fold(first={}, join={})", first, join),
            Left => write!(f, "$Left"),
            Right => write!(f, "$Right"),
        }
    }
}

impl<S> Add<Notation<S>> for Notation<S> {
    type Output = Notation<S>;

    /// `x + y` is shorthand for `Concat(x, y)`.
    fn add(self, other: Notation<S>) -> Notation<S> {
        Notation::Concat(Box::new(self), Box::new(other))
    }
}

impl<S> BitOr<Notation<S>> for Notation<S> {
    type Output = Notation<S>;

    /// `x | y` is shorthand for `Choice(x, y)`.
    fn bitor(self, other: Notation<S>) -> Notation<S> {
        Notation::Choice(Box::new(self), Box::new(other))
    }
}

impl<S> BitXor<Notation<S>> for Notation<S> {
    type Output = Notation<S>;

    /// `x ^ y` is shorthand for `x + Newline + y`.
    fn bitxor(self, other: Notation<S>) -> Notation<S> {
        self + Notation::Newline + other
    }
}

impl<S> Shr<Notation<S>> for Width
where
    S: Default,
{
    type Output = Notation<S>;

    /// `i >> x` is shorthand for `Indent(i_spaces, Newline + x)` (sometimes called "nesting").
    fn shr(self, notation: Notation<S>) -> Notation<S> {
        Notation::Indent(
            Literal::new(
                &format!("{:spaces$}", "", spaces = self as usize),
                S::default(),
            ),
            Box::new(Notation::Concat(
                Box::new(Notation::Newline),
                Box::new(notation),
            )),
        )
    }
}
