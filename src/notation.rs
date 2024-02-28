use crate::geometry::{str_width, Width};
use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

pub trait StyleLabel: fmt::Debug + Clone {}
impl<T: fmt::Debug + Clone> StyleLabel for T {}

pub trait Condition: fmt::Debug + Clone {}
impl<T: fmt::Debug + Clone> Condition for T {}

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
pub enum Notation<L: StyleLabel, C: Condition> {
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
    Flat(Box<Notation<L, C>>),
    /// Append a string to the indentation of the contained notation. All of the
    /// indentation strings will be displayed after `Newline`s. (They therefore
    /// don't affect the first line of a notation.) Indentation strings will
    /// typically contain one indentation level's worth of whitespace characters
    /// (eg. 4 spaces), but can also be used for other purposes like placing comment
    /// syntax at the start of a line. If the `Option<L>` is `Some`, that style
    /// will be applied to the indentation string.
    Indent(Literal, Option<L>, Box<Notation<L, C>>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. Note that the column at which the right notation
    /// starts does not affect its indentation level.
    Concat(Box<Notation<L, C>>, Box<Notation<L, C>>),
    /// If we're inside a `Flat`, _or_ the first line of the left notation fits within the required
    /// width, then display the left notation. Otherwise, display the right notation.
    Choice(Box<Notation<L, C>>, Box<Notation<L, C>>),
    // TODO: doc
    If(C, Box<Notation<L, C>>, Box<Notation<L, C>>),
    /// Display the `i`th child of this node.  Can only be used on a [`PrettyDoc`] node for which
    /// `.num_children()` returns `Some(n)`, with `i < n`.
    Child(usize),
    /// Look up the style with the given label in the current document node and apply it to this
    /// notation. (The lookup happens via `PrettyDoc::lookup_style()`.)
    Style(L, Box<Notation<L, C>>),
    /// Determines what to display based on the number of children this [`PrettyDoc`] node has.
    Count {
        zero: Box<Notation<L, C>>,
        one: Box<Notation<L, C>>,
        many: Box<Notation<L, C>>,
    },
    /// Fold (a.k.a. reduce) over the node's children. This is a left-fold. May only be used in
    /// `Notation::Count.many`.
    Fold {
        /// How to display the first child on its own.
        first: Box<Notation<L, C>>,
        /// How to append an additional child onto a partially displayed sequence of children.
        /// Within this notation, `Notation::Left` refers to the children displayed so far,
        /// and `Notation::Right` refers to the next child to be appended.
        join: Box<Notation<L, C>>,
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
impl<L: StyleLabel, C: Condition> fmt::Display for Notation<L, C> {
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
            If(cond, opt1, opt2) => write!(f, "({:?} ? {} | {})", cond, opt1, opt2),
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

impl<L: StyleLabel, C: Condition> Add<Notation<L, C>> for Notation<L, C> {
    type Output = Notation<L, C>;

    /// `x + y` is shorthand for `Concat(x, y)`.
    fn add(self, other: Notation<L, C>) -> Notation<L, C> {
        Notation::Concat(Box::new(self), Box::new(other))
    }
}

impl<L: StyleLabel, C: Condition> BitOr<Notation<L, C>> for Notation<L, C> {
    type Output = Notation<L, C>;

    /// `x | y` is shorthand for `Choice(x, y)`.
    fn bitor(self, other: Notation<L, C>) -> Notation<L, C> {
        Notation::Choice(Box::new(self), Box::new(other))
    }
}

impl<L: StyleLabel, C: Condition> BitXor<Notation<L, C>> for Notation<L, C> {
    type Output = Notation<L, C>;

    /// `x ^ y` is shorthand for `x + Newline + y`.
    fn bitxor(self, other: Notation<L, C>) -> Notation<L, C> {
        self + Notation::Newline + other
    }
}

impl<L: StyleLabel, C: Condition> Shr<Notation<L, C>> for Width {
    type Output = Notation<L, C>;

    /// `i >> x` is shorthand for `Indent(i_spaces, Newline + x)` (sometimes called "nesting").
    fn shr(self, notation: Notation<L, C>) -> Notation<L, C> {
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
