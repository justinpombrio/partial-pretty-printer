use crate::geometry::Width;
use crate::style::Style;
use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

/// Describes how to display a syntactic construct. When constructing a Notation, you must obey one
/// requirement. If you do not, the pretty printer may behave unpredictably and choose poor layouts.
///
/// > For every choice `(x | y)`, the first line of `x` is shorter than (or equal to) the first
///   line of `y`.
#[derive(Clone, Debug)]
pub enum Notation {
    /// Display nothing. Identical to Literal of an empty string.
    Empty,
    /// Display a newline. If this is inside an `Indent`, the new line will be indented.
    Newline,
    /// Display a piece of text. Must be used on a texty node (a node for which
    /// [`crate::PrettyDoc::num_children()`] returns `None`).
    Text(Style),
    /// Literal text. Contains a fixed string and a style. The string cannot contain a newline.
    Literal(Box<Literal>),
    /// `Flat(n)` means "display `n`, attempting to keep it flat (free of newlines)".
    /// Specifically:
    ///
    /// - For every choice `x | y` in `n`, choose `x`.
    /// - For every `IfFlat(x, y)` in `n`, choose `x`.
    Flat(Box<Notation>),
    /// Indent all lines of the contained notation except the first, by the given number of spaces.
    Indent(Width, Box<Notation>),
    /// Display the two notations in order. The first character of the right notation immediately
    /// follows the last character of the left notation (that is, there's no space between). The
    /// right notation's indentation level is not affected.
    Concat(Box<Notation>, Box<Notation>),
    /// Typically, you want to use `Group` instead. This is a more powerful alternative, but you
    /// must ensure that you obey the Invariant:
    ///
    /// > For every choice `(x | y)`, the first line of `x` is shorter than (or equal to) the first
    ///
    /// Pick between two options. Generally, pick the first option if its first line fits within
    /// the width limit, otherwise pick the second. However, there are exceptions:
    ///
    /// - If we're inside a `Flat`, always pick the first option.
    /// - In the unlikely event that the first option contains an unconditional newline inside of
    ///   a `Flat`, pick the second option instead.
    Choice(Box<Notation>, Box<Notation>),
    /// Either flatten everything in the group, or don't. `Group(n)` is equivalent to
    /// `Choice(Flat(n), n)`.
    Group(Box<Notation>),
    /// Display the first notation if we're inside a Flat, otherwise display the second notation.
    IfFlat(Box<Notation>, Box<Notation>),
    /// Display the first notation in case this tree has empty text,
    /// otherwise show the second notation. Must be used on a texty node.
    IfEmptyText(Box<Notation>, Box<Notation>),
    /// Display the `i`th child of this node.  Must be used on a foresty node (a node for which
    /// [`crate::PrettyDoc::num_children()`] returns `Some`).  `i` must be less than the node's
    /// arity number.
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

pub static EMPTY_LITERAL: &Literal = &Literal {
    string: String::new(),
    len: 0,
    style: Style::plain(),
};

#[derive(Clone, Debug)]
pub struct Literal {
    string: String,
    /// Number of characters (*not* num bytes!)
    len: Width,
    style: Style,
}

/// Describes how to display the extra children of a syntactic
/// construct with extendable arity. Similar to a `fold` operation:
///
/// - If the children are `[]`, return `empty`.
/// - If the children are `[x]`, return `lone`.
/// - If the chidlren are `[x, y, z]`, return `surrouned(join(join(x, y), z))`.
#[derive(Clone, Debug)]
pub struct RepeatInner {
    /// If the sequence is empty, use this notation.
    pub empty: Notation,
    /// If the sequence has length one, use this notation. You can refer to the single child with
    /// `Child(0)`.
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
            Literal(lit) => write!(f, "'{}'", lit.string),
            Flat(note) => write!(f, "Flat({})", note),
            Indent(i, note) => write!(f, "{}⇒({})", i, note),
            Concat(left, right) => write!(f, "{} + {}", left, right),
            Group(note) => write!(f, "Group({})", note),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            IfFlat(opt1, opt2) => write!(f, "ifFlat({}, {})", opt1, opt2),
            IfEmptyText(opt1, opt2) => write!(f, "IfEmptyText({} | {})", opt1, opt2),
            Child(i) => write!(f, "${}", i),
            Repeat(repeat) => write!(
                f,
                "Repeat{{empty={} lone={} join={} surround={}",
                repeat.empty, repeat.lone, repeat.join, repeat.surround
            ),
            Left => write!(f, "$Left"),
            Right => write!(f, "$Right"),
            Surrounded => write!(f, "$Surr"),
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
