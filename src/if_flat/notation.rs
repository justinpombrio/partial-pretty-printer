use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

// ASSUMPTION:
// In every choice `X | Y`, `min_first_line_len(Y) <= min_first_line_len(X)`.

#[derive(Clone, Debug)]
pub enum Notation {
    /// Display nothing. Identical to `Literal("")`.
    Empty,
    /// Literal text. Cannot contain a newline.
    Literal(String),
    /// Display a newline. If this is inside an `Indent`, the new line will be indented.
    Newline,
    /// Only consider single-line options of the contained notation.
    Flat(Box<Notation>),
    /// Indent all lines of the contained notation except the first to the right by the given
    /// number of spaces.
    Indent(usize, Box<Notation>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. The right notation's indentation level is not
    /// affected.
    Concat(Box<Notation>, Box<Notation>),
    /// Display the left notation if it fits on one line within the required width; otherwise the
    /// right.
    Choice(Box<Notation>, Box<Notation>),
    /// Display the `i`th child of this node.
    /// Must be used on a foresty node.
    /// `i` must be less than the node's arity number.
    Child(usize),
}

impl Notation {
    // TODO: build this into the notation. This can be exponentially large!
    pub fn repeat(
        num_elements: usize,
        empty: Notation,
        lone: impl Fn(Notation) -> Notation,
        join: impl Fn(Notation, Notation) -> Notation,
        surround: impl Fn(Notation) -> Notation,
    ) -> Notation {
        let elements = (0..num_elements)
            .map(|i| Notation::Child(i))
            .collect::<Vec<_>>();
        let mut iter = elements.into_iter();
        match num_elements {
            0 => empty,
            1 => lone(iter.next().unwrap()),
            _ => {
                let mut iter = iter.rev();
                let mut accumulator = iter.next().unwrap();
                for elem in iter {
                    accumulator = join(elem, accumulator);
                }
                surround(accumulator)
            }
        }
    }
}

impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Notation::*;

        match self {
            Empty => write!(f, "ε"),
            Newline => write!(f, "↵"),
            Literal(lit) => write!(f, "{}", lit),
            Flat(note) => write!(f, "Flat({})", note),
            Indent(i, note) => write!(f, "⇒{}({})", i, note),
            Concat(left, right) => write!(f, "({} + {})", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            Child(i) => write!(f, "${}", i),
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

impl Shr<Notation> for usize {
    type Output = Notation;

    /// Shorthand for nesting (indented newline)
    fn shr(self, notation: Notation) -> Notation {
        Notation::Indent(self, Box::new(Notation::Newline + notation))
    }
}
