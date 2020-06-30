use crate::style::Style;
use std::ops::{Add, BitOr};

/// Describes how to display a syntactic construct.
#[derive(Clone, Debug)]
pub enum Notation {
    /// Display nothing. The same as `Literal("")`.
    Empty,
    /// Display a literal string. Cannot contain a newline.
    Literal(String, Style),
    /// Display a newline. If this is inside an `Indent`, the new line will be
    /// indented.
    Newline,
    /// Only consider single-line options of the contained notation.
    Flat(Box<Notation>),
    /// Indent all lines of the contained notation except the first to the right
    /// by the given number of spaces.
    Indent(usize, Box<Notation>),
    /// Display the second notation after the first, and so forth. The last character of one of the
    /// notations is immediately followed by the first character of the next. The notations'
    /// indentation levels are not affected.
    Follow(Vec<Notation>),
    /// Pick exactly one of the notations to display: either the first one that fits in the allowed
    /// width or, failing that, the last one.
    Choice(Vec<Notation>),
    /// Display a piece of text. Must be used on a texty node.
    Text(Style),
    /// Determines what to display based on the arity of this node.
    /// Used for syntactic constructs that have extendable arity.
    Repeat(Box<Repeat>),
    /// Used in [`Repeat`](Repeat) to refer to the accumulated Notation
    /// in `join`.
    Left,
    /// Used in [`Repeat`](Repeat) to refer to the next child in `join`.
    Right,
    /// Used in [`Repeat`](Repeat) to refer to the Notation inside of
    /// `surround`.
    Surrounded,
}

/// Describes how to display the extra children of a syntactic
/// construct with extendable arity.
#[derive(Clone, Debug)]
pub struct Repeat {
    /// If the sequence is empty, use this notation.
    pub empty: Notation,
    /// If the sequence has length one, use this notation.
    pub lone: Notation,
    /// If the sequence has length 2 or more, (right-)fold elements together with
    /// this notation. [`Right`](Right) holds the notation so far, while
    /// [`Left`](Left) holds the next child to be folded.
    pub join: Notation,
    /// If the sequence has length 2 or more, surround the folded notation with
    /// this notation. [`Surrounded`](Surrounded) holds the folded notation.
    pub surround: Notation,
}

impl Add<Notation> for Notation {
    type Output = Notation;
    /// Shorthand for `Concat`.
    fn add(self, other: Notation) -> Notation {
        Notation::Follow(vec![self, other])
    }
}

impl BitOr<Notation> for Notation {
    type Output = Notation;
    /// Shorthand for `Choice`.
    fn bitor(self, other: Notation) -> Notation {
        Notation::Choice(vec![self, other])
    }
}
