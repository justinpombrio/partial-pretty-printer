use crate::geometry::{str_width, Width};
use crate::style::Style;
use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

// TODO: Nail down this assumption further, I don't think this is _quite_ right.

/// Describes how to display a syntactic construct. When constructing a Notation, you must obey one
/// requirement. If you do not, the pretty printer may choose poor layouts.
///
/// > For every choice `(x | y)`, the first line of `x` is shorter than (or equal to) the first
///   line of `y`.
///
/// Additionally, whenever possible the leftmost option of a choice should be flat (contain no
/// newlines). This allows containing notations to use the `Flat` constructor to attempt to fit it
/// in one line.
#[derive(Clone, Debug)]
pub enum Notation {
    /// Display nothing. Identical to `Literal("")`.
    Empty,
    /// Display a newline. If this is inside an `Indent`, the new line will be indented.
    Newline,
    /// Display a piece of text. Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `None` (implying that it contains text).
    Text(Style),
    /// Literal text. Cannot contain a newline.
    Literal(Literal),
    /// Use the leftmost option of every choice in the contained notation. This should typically
    /// mean not displaying any Newlines.
    Flat(Box<Notation>),
    /// Indent all lines of the contained notation except the first to the right by the given
    /// number of spaces.
    Indent(Width, Box<Notation>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. The right notation's indentation level is not
    /// affected.
    Concat(Box<Notation>, Box<Notation>),
    /// If we're inside a `Flat`, _or_ the first line of the left notation fits within the required
    /// width, then display the left notation, otherwise display the right.
    Choice(Box<Notation>, Box<Notation>),
    /// Display the first notation in case this tree has empty text,
    /// otherwise show the second notation.
    IfEmptyText(Box<Notation>, Box<Notation>),
    /// Display the `i`th child of this node.  Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `Some(n)`, with `i < n`.
    Child(usize),
    /// Determines what to display based on the arity of this node. Used for nodes that have
    /// variable arity.
    Count {
        zero: Box<Notation>,
        one: Box<Notation>,
        many: Box<Notation>,
    },
    /// Fold (a.k.a. reduce) over the node's children. This is a left-fold.
    Fold {
        /// How to display the first child on its own.
        base: Box<Notation>,
        /// How to append an additional child onto a partially displayed sequence of children.
        /// Within this notation, `Notation::Left` refers to the children displayed so far,
        /// and `Notation::Right` refers to the next child to be appended. For example, when
        /// displaying a comma separated list "1, 2, 3", `Notation::Left` would be "1, 2", while
        /// `Notation::Right` would be "3".
        join: Box<Notation>,
    },
    /// Used in [`Fold`](Notation::Fold) to refer to the accumulated Notation in `join`.
    /// Illegal outside of `Fold`.
    Left,
    /// Used in [`Fold`](Notation::Fold) to refer to the next child in `join`.
    /// Illegal outside of `Fold`.
    Right,
}

/// Literal text, to be displayed as-is. Cannot contain a newline.
#[derive(Clone, Debug)]
pub struct Literal {
    string: String,
    /// Width of the string in terminal columns. See [`Width`].
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
            Fold { base, join } => write!(f, "Fold(base={}, join={})", base, join),
            Left => write!(f, "$Left"),
            Right => write!(f, "$Right"),
        }
    }
}

impl Add<Notation> for Notation {
    type Output = Notation;

    /// Shorthand for `Concat`.
    fn add(self, other: Notation) -> Notation {
        Notation::Concat(Box::new(self), Box::new(other))
    }
}

impl BitOr<Notation> for Notation {
    type Output = Notation;

    /// Shorthand for `Choice`.
    fn bitor(self, other: Notation) -> Notation {
        Notation::Choice(Box::new(self), Box::new(other))
    }
}

impl BitXor<Notation> for Notation {
    type Output = Notation;

    /// Shorthand for `X + newline() + Y`.
    fn bitxor(self, other: Notation) -> Notation {
        self + Notation::Newline + other
    }
}

impl Shr<Notation> for Width {
    type Output = Notation;

    /// Shorthand for nesting (indented newline)
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
