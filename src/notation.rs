use crate::geometry::{str_width, Width};
use crate::style::Style;
use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

// TODO: Make generic over Style

// TODO: Nail down this assumption further, I don't think this is _quite_ right.

/// Describes how to display a syntactic construct. When constructing a Notation, you must obey one
/// requirement. If you do not, the pretty printer may choose poor layouts.
///
/// > For every choice `(x | y)`, the first line of `x` is shorter than (or equal to) the first
///   line of `y`.
///
/// Additionally, whenever possible, `x` should be flat (contain no newlines). This allows
/// notations to use the `Flat` variant to attempt to fit `(x | y)` all on one line.
#[derive(Clone, Debug)]
pub enum Notation {
    /// Display nothing.
    Empty,
    /// Display a newline followed a number of spaces determined by the indentation level. (See
    /// [`Notation::Indent`]).
    Newline,
    /// Display a piece of text. Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `None` (implying that it contains text).
    Text(Style),
    /// Literal text. Cannot contain a newline.
    Literal(Literal),
    /// Use the leftmost option of every choice in the contained notation. If the notation author
    /// followed the recommendation of not putting `Newline`s in the left-most options of choices,
    /// then this `Flat` will be displayed all on one line.
    Flat(Box<Notation>),
    /// Increase the indentation level of the contained notation by the given width. The
    /// indentation level determines the number of spaces put after `Newline`s. (It therefore
    /// doesn't affect the first line of a notation.)
    Indent(Width, Box<Notation>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. Note that the column at which the right notation
    /// starts does not affect its indentation level.
    Concat(Box<Notation>, Box<Notation>),
    /// If we're inside a `Flat`, _or_ the first line of the left notation fits within the required
    /// width, then display the left notation. Otherwise, display the right notation.
    Choice(Box<Notation>, Box<Notation>),
    /// If this [`PrettyDoc`] node is a text node containing the empty string, display the left
    /// notation, otherwise show the right notation.
    IfEmptyText(Box<Notation>, Box<Notation>),
    /// Display the `i`th child of this node.  Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `Some(n)`, with `i < n`.
    Child(usize),
    /// Determines what to display based on the number of children this [`PrettyDoc`] node has.
    Count {
        zero: Box<Notation>,
        one: Box<Notation>,
        many: Box<Notation>,
    },
    /// Fold (a.k.a. reduce) over the node's children. This is a left-fold. May only be used in
    /// `Notation::Count.many`.
    Fold {
        /// How to display the first child on its own.
        first: Box<Notation>,
        /// How to append an additional child onto a partially displayed sequence of children.
        /// Within this notation, `Notation::Left` refers to the children displayed so far,
        /// and `Notation::Right` refers to the next child to be appended.
        join: Box<Notation>,
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
pub struct Literal {
    string: String,
    /// Width of the string in [`Col`]s. See [`Width`].
    width: Width,
    style: Style,
}

impl Literal {
    pub fn new(s: &str, style: Style) -> Literal {
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

    pub fn style(&self) -> Style {
        self.style
    }
}

// For debugging
impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Notation::*;

        match self {
            Empty => write!(f, "ε"),
            Newline => write!(f, "↵"),
            Text(_) => write!(f, "TEXT"),
            Literal(lit) => write!(f, "'{}'", lit.string),
            Flat(note) => write!(f, "Flat({})", note),
            Indent(i, note) => write!(f, "{}⇒({})", i, note),
            Concat(left, right) => write!(f, "{} + {}", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            IfEmptyText(opt1, opt2) => write!(f, "IfEmptyText({} | {})", opt1, opt2),
            Child(i) => write!(f, "${}", i),
            Count { zero, one, many } => {
                write!(f, "Count(zero={}, one={}, many={})", zero, one, many)
            }
            Fold { first, join } => write!(f, "Fold(first={}, join={})", first, join),
            Left => write!(f, "$Left"),
            Right => write!(f, "$Right"),
        }
    }
}

impl Add<Notation> for Notation {
    type Output = Notation;

    /// `x + y` is shorthand for `Concat(x, y)`.
    fn add(self, other: Notation) -> Notation {
        Notation::Concat(Box::new(self), Box::new(other))
    }
}

impl BitOr<Notation> for Notation {
    type Output = Notation;

    /// `x | y` is shorthand for `Choice(x, y)`.
    fn bitor(self, other: Notation) -> Notation {
        Notation::Choice(Box::new(self), Box::new(other))
    }
}

impl BitXor<Notation> for Notation {
    type Output = Notation;

    /// `x ^ y` is shorthand for `x + Newline + y`.
    fn bitxor(self, other: Notation) -> Notation {
        self + Notation::Newline + other
    }
}

impl Shr<Notation> for Width {
    type Output = Notation;

    /// `i >> x` is shorthand for `Indent(i, Newline + x)` (sometimes called "nesting").
    fn shr(self, notation: Notation) -> Notation {
        Notation::Indent(
            self,
            Box::new(Notation::Concat(
                Box::new(Notation::Newline),
                Box::new(notation),
            )),
        )
    }
}
