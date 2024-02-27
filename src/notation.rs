use crate::geometry::{str_width, Width};
use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

pub trait StyleLabel: fmt::Debug + Clone {}
impl<T: fmt::Debug + Clone> StyleLabel for T {}

/// Describes how to display a syntactic construct. When constructing a Notation, you must obey one
/// requirement. If you do not, the pretty printer may choose poor layouts.
///
/// > For every choice `(x | y)`, the first line of `x` is shorter than (or equal to) the first
///   line of `y`.
///
/// Additionally, whenever possible, `x` should be flat (contain no newlines). This allows
/// notations to use the `Flat` variant to attempt to fit `(x | y)` all on one line.
///
/// Type parameter `L` is a label used to look up a style in the document. It
/// corresponds to the `PrettyDoc::StyleLabel` associated type.
#[derive(Clone, Debug)]
pub enum Notation<L: StyleLabel> {
    /// Display nothing.
    Empty,
    /// Display a newline followed by the current indentation. (See [`Notation::Indent`]).
    Newline,
    /// Display a piece of text. Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `None` (implying that it contains text).
    Text,
    /// Literal text. Cannot contain a newline.
    Literal(Literal),
    /// Use the leftmost option of every choice in the contained notation. If the notation author
    /// followed the recommendation of not putting `Newline`s in the left-most options of choices,
    /// then this `Flat` will be displayed all on one line.
    Flat(Box<Notation<L>>),
    /// Append a string to the indentation of the contained notation. All of the
    /// indentation strings will be displayed after `Newline`s. (They therefore
    /// don't affect the first line of a notation.) Indentation strings will
    /// typically contain one indentation level's worth of whitespace characters
    /// (eg. 4 spaces), but can also be used for other purposes like placing comment
    /// syntax at the start of a line. If the `Option<L>` is `Some`, that style
    /// will be applied to the indentation string.
    Indent(Literal, Option<L>, Box<Notation<L>>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. Note that the column at which the right notation
    /// starts does not affect its indentation level.
    Concat(Box<Notation<L>>, Box<Notation<L>>),
    /// If we're inside a `Flat`, _or_ the first line of the left notation fits within the required
    /// width, then display the left notation. Otherwise, display the right notation.
    Choice(Box<Notation<L>>, Box<Notation<L>>),
    /// If this [`PrettyDoc`] node is a text node containing the empty string, display the left
    /// notation, otherwise show the right notation.
    IfEmptyText(Box<Notation<L>>, Box<Notation<L>>),
    /// Display the `i`th child of this node.  Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `Some(n)`, with `i < n`.
    Child(usize),
    /// Look up the style with the given label in the current document node and apply it to this
    /// notation. (The lookup happens via `PrettyDoc::lookup_style()`.)
    Style(L, Box<Notation<L>>),
    /// Determines what to display based on the number of children this [`PrettyDoc`] node has.
    Count {
        zero: Box<Notation<L>>,
        one: Box<Notation<L>>,
        many: Box<Notation<L>>,
    },
    /// Fold (a.k.a. reduce) over the node's children. This is a left-fold. May only be used in
    /// `Notation::Count.many`.
    Fold {
        /// How to display the first child on its own.
        first: Box<Notation<L>>,
        /// How to append an additional child onto a partially displayed sequence of children.
        /// Within this notation, `Notation::Left` refers to the children displayed so far,
        /// and `Notation::Right` refers to the next child to be appended.
        join: Box<Notation<L>>,
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
}

impl Literal {
    pub fn new(s: &str) -> Literal {
        Literal {
            string: s.to_owned(),
            width: str_width(s),
        }
    }

    pub fn width(&self) -> Width {
        self.width
    }

    pub fn str(&self) -> &str {
        &self.string
    }
}

// For debugging. Should match impl fmt::Display for ConsolidatedNotation.
impl<L: StyleLabel> fmt::Display for Notation<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Notation::*;

        match self {
            Empty => write!(f, "ε"),
            Newline => write!(f, "↵"),
            Text => write!(f, "TEXT"),
            Literal(lit) => write!(f, "'{}'", lit.string),
            Flat(note) => write!(f, "Flat({})", note),
            Indent(lit, _style_label, note) => write!(f, "'{}'⇒({})", lit.string, note),
            Concat(left, right) => write!(f, "{} + {}", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            IfEmptyText(opt1, opt2) => write!(f, "IfEmptyText({} | {})", opt1, opt2),
            Child(i) => write!(f, "${}", i),
            Style(style_label, note) => write!(f, "Style({:?}, {})", style_label, note),
            Count { zero, one, many } => {
                write!(f, "Count(zero={}, one={}, many={})", zero, one, many)
            }
            Fold { first, join } => write!(f, "Fold(first={}, join={})", first, join),
            Left => write!(f, "$Left"),
            Right => write!(f, "$Right"),
        }
    }
}

impl<L: StyleLabel> Add<Notation<L>> for Notation<L> {
    type Output = Notation<L>;

    /// `x + y` is shorthand for `Concat(x, y)`.
    fn add(self, other: Notation<L>) -> Notation<L> {
        Notation::Concat(Box::new(self), Box::new(other))
    }
}

impl<L: StyleLabel> BitOr<Notation<L>> for Notation<L> {
    type Output = Notation<L>;

    /// `x | y` is shorthand for `Choice(x, y)`.
    fn bitor(self, other: Notation<L>) -> Notation<L> {
        Notation::Choice(Box::new(self), Box::new(other))
    }
}

impl<L: StyleLabel> BitXor<Notation<L>> for Notation<L> {
    type Output = Notation<L>;

    /// `x ^ y` is shorthand for `x + Newline + y`.
    fn bitxor(self, other: Notation<L>) -> Notation<L> {
        self + Notation::Newline + other
    }
}

impl<L: StyleLabel> Shr<Notation<L>> for Width {
    type Output = Notation<L>;

    /// `i >> x` is shorthand for `Indent(i_spaces, Newline + x)` (sometimes called "nesting").
    fn shr(self, notation: Notation<L>) -> Notation<L> {
        Notation::Indent(
            Literal::new(&format!("{:spaces$}", "", spaces = self as usize)),
            None,
            Box::new(Notation::Concat(
                Box::new(Notation::Newline),
                Box::new(notation),
            )),
        )
    }
}
