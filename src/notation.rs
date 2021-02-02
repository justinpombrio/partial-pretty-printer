use crate::geometry::Width;
use crate::style::Style;
use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

// ASSUMPTION:
// In every choice `X | Y`, `min_first_line_len(Y) <= min_first_line_len(X)`.

/// Describes how to display a syntactic construct.
#[derive(Clone, Debug)]
pub enum Notation {
    /// Display nothing. Identical to `Literal("")`.
    Empty,
    /// Display a newline. If this is inside an `Indent`, the new line will be indented.
    Newline,
    /// Display a piece of text. Must be used on a texty node.
    Text(Style),
    /// Literal text. Cannot contain a newline.
    Literal(Box<Literal>),
    /// Only consider single-line options of the contained notation.
    Flat(Box<Notation>),
    /// Indent all lines of the contained notation except the first to the right by the given
    /// number of spaces.
    Indent(Width, Box<Notation>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. The right notation's indentation level is not
    /// affected.
    Concat(Box<Notation>, Box<Notation>),
    /// Display the left notation if its first line fits within the required width; otherwise
    /// display the right.
    Choice(Box<Notation>, Box<Notation>),
    /// Display the first notation in case this tree has empty text,
    /// otherwise show the second notation.
    IfEmptyText(Box<Notation>, Box<Notation>),
    /// Display the `i`th child of this node.
    /// Must be used on a foresty node.
    /// `i` must be less than the node's arity number.
    Child(usize),
    /// Determines what to display based on the arity of this node.
    /// Used for syntactic constructs that have extendable arity.
    Repeat(Box<RepeatInner>),
    /// Used in [`Repeat`](Notation::Repeat) to refer to the accumulated Notation
    /// in `join`.
    Left,
    /// Used in [`Repeat`](Notation::Repeat) to refer to the next child in `join`.
    Right,
    /// Used in [`Repeat`](Notation::Repeat) to refer to the Notation inside of
    /// `surround`.
    Surrounded,
}

#[derive(Clone, Debug)]
pub struct Literal {
    string: String,
    /// Number of characters (*not* num bytes!)
    len: Width,
    style: Style,
}

/// Describes how to display the extra children of a syntactic
/// construct with extendable arity.
#[derive(Clone, Debug)]
pub struct RepeatInner {
    /// If the sequence is empty, use this notation.
    pub empty: Notation,
    /// If the sequence has length one, use this notation.
    pub lone: Notation,
    /// If the sequence has length 2 or more, (left-)fold elements together with
    /// this notation. [`Left`](Notation::Left) holds the notation so far, while
    /// [`Right`](Notation::Right) holds the next child to be folded.
    pub join: Notation,
    /// If the sequence has length 2 or more, surround the folded notation with
    /// this notation. [`Surrounded`](Notation::Surrounded) holds the folded notation.
    pub surround: Notation,
}

impl Literal {
    pub fn new(s: &str, style: Style) -> Literal {
        Literal {
            string: s.to_owned(),
            len: s.chars().count() as Width,
            style,
        }
    }

    pub fn len(&self) -> Width {
        self.len
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
            Literal(lit) => write!(f, "{}", lit.string),
            Flat(note) => write!(f, "Flat({})", note),
            Indent(i, note) => write!(f, "⇒{}({})", i, note),
            Concat(left, right) => write!(f, "({} + {})", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            IfEmptyText(opt1, opt2) => write!(f, "IfEmptyText({} | {})", opt1, opt2),
            Child(i) => write!(f, "${}", i),
            Repeat(repeat) => write!(
                f,
                "Repeat{{empty={} lone={} join={} surround={}",
                repeat.empty, repeat.lone, repeat.join, repeat.surround
            ),
            Left => write!(f, "$Left"),
            Right => write!(f, "$Right"),
            Surrounded => write!(f, "$Surrounded"),
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
        Notation::Indent(self, Box::new(Notation::Newline + notation))
    }
}
