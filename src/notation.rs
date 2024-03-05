use crate::geometry::{str_width, Width};
use std::fmt;
use std::ops::{Add, BitOr, BitXor, Shr};

#[cfg(doc)]
use crate::notation_constructors;
#[cfg(doc)]
use crate::PrettyDoc; // for links in rustdocs // for links in rustdocs

/// A label used to look up a style in the document.
/// It corresponds to the [`PrettyDoc::StyleLabel`] associated type.
pub trait StyleLabel: fmt::Debug + Clone {}
impl<T: fmt::Debug + Clone> StyleLabel for T {}

/// Arbitrary property of a document node that can be checked with [`Notation::Check`].
/// It corresponds to the [`PrettyDoc::Condition`] associated type.
pub trait Condition: fmt::Debug + Clone {}
impl<T: fmt::Debug + Clone> Condition for T {}

/// Describes how to display a document node.
///
/// The [`notation_constructors`] module has convenient functions for constructing these notations.
#[derive(Clone, Debug)]
pub enum Notation<L: StyleLabel, C: Condition> {
    /// Display nothing.
    Empty,
    /// Display a newline followed by the current indentation. (See [`Notation::Indent`]).
    Newline,
    /// The printer will try to resolve choices such that this `EndOfLine` is followed by a
    /// [`Newline`](Notation::Newline) (or the end of the document), and not by a
    /// [`Text`](Notation::Text) or [`Literal`](Notation::Literal). If that's not possible, it will
    /// fail with a [`PrintingError::TextAfterEndOfLine`](crate::PrintingError::TextAfterEndOfLine).
    EndOfLine,
    /// Display this constant text. It must not contain a newline character.
    Literal(Literal),
    /// Display a dynamic piece of text from the document. It must not contain a newline character.
    /// It can only be used in the notation for a document node that contains text (indicated by
    /// [`PrettyDoc::num_children()`] returning `None`).
    Text,
    /// Pick the first option of every [`Choice`](Notation::Choice) in the contained notation.
    /// If the notation author followed the recommendation of not putting
    /// [`Newline`](Notation::Newline)s in the first options of [`Choice`](Notation::Choice)s,
    /// then everything inside this `Flat` will be displayed on one line.
    Flat(Box<Notation<L, C>>),
    /// Append a string to the indentation of the contained notation. If the `Option<L>` is `Some`,
    /// that style will be applied to the indentation string.
    ///
    /// All of the indentation strings will be displayed after each [`Newline`](Notation::Newline).
    /// (They therefore don't affect the first line of a notation.) Indentation strings will
    /// typically contain one indentation level's worth of whitespace characters (e.g. 4 spaces), but
    /// can also be used for other purposes like placing comment syntax at the start of a line.
    Indent(Literal, Option<L>, Box<Notation<L, C>>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. Note that the column at which the right notation starts
    /// does not affect its indentation level.
    Concat(Box<Notation<L, C>>, Box<Notation<L, C>>),
    /// Display the first notation if we're inside a [`Flat`](Notation::Flat) or if it "fits on the
    /// current line". Otherwise, display the second.
    ///
    /// A notation "fits on the current line" if both of the following are true:
    ///
    /// - The first line of the notation does not cause the current line to exceed the width limit.
    /// - The first line of the notation does not cause the current line to contain an
    ///   [`EndOfLine`](Notation::EndOfLine) followed by a [`Text`](Notation::Text) or
    ///   [`Literal`](Notation::Literal).
    ///
    /// You must obey an important requirement when constructing [`Choice`](Notation::Choice)s. If
    /// you do not, the pretty printer may choose poor layouts or produce an error.
    ///
    /// > For every choice `(x | y)`, if `x` "fits on the current line" then `y` must too.
    ///
    /// Additionally, whenever possible, `x` should not contain newlines. This allows notations to
    /// use [`Flat`](Notation::Flat) to attempt to fit `(x | y)` all on one line.
    Choice(Box<Notation<L, C>>, Box<Notation<L, C>>),
    /// Check whether the [`Condition`](PrettyDoc::Condition) `C` is true for the document node
    /// located at [`CheckPos`]. If so, display the first notation, otherwise display the second.
    Check(C, CheckPos, Box<Notation<L, C>>, Box<Notation<L, C>>),
    /// Display the i'th child of the current document node. If the index is negative, the number
    /// of children is added to it (so that -1 accesses the last child). Can only be used on a node
    /// for which [`PrettyDoc::num_children()`] returns `Some(n)`, with `-n <= i < n`.
    Child(isize),
    /// Look up the style with the given label in the current document node (via
    /// [`PrettyDoc::lookup_style()`]), and apply it to this notation. It will be combined with any
    /// other styles that were previously applied to this subtree using
    /// [`Style::combine()`](crate::Style::combine).
    Style(L, Box<Notation<L, C>>),
    /// Display one of these notations, depending how many children the current document node has.
    Count {
        zero: Box<Notation<L, C>>,
        one: Box<Notation<L, C>>,
        many: Box<Notation<L, C>>,
    },
    /// [Left-fold](https://en.wikipedia.org/wiki/Fold_(higher-order_function)) over the node's
    /// children. This lets you specify how an indeterminate number of children should be
    /// displayed. For example, to separate the children by commas on a single line:
    ///
    /// ```
    /// use partial_pretty_printer::Notation;
    /// use partial_pretty_printer::notation_constructors::{
    ///     fold, Fold, child, left, right, lit
    /// };
    ///
    /// let notation: Notation<(), ()> = fold(Fold {
    ///     first: child(0),
    ///     join: left() + lit(", ") + right()
    /// });
    /// ```
    ///
    /// `Fold` should typically be used within [`Count`](Notation::Count)'s `many` case, where you
    /// know there are multiple children. But if there are zero children, it will display nothing.
    // Note: Folds must be left-folds for flow wrap to work.
    Fold {
        /// How to display the first child on its own.
        first: Box<Notation<L, C>>,
        /// How to append an additional child onto a partially displayed sequence of children.
        /// Within this notation, [`Left`](Notation::Left) refers to the children displayed so far,
        /// and [`Right`](Notation::Right) refers to the next child to be appended.
        join: Box<Notation<L, C>>,
    },
    /// Used in [`Fold`](Notation::Fold)'s `join` case to refer to the accumulated notation. Illegal
    /// outside of `Fold`.
    Left,
    /// Used in [`Fold`](Notation::Fold)'s `join` case to refer to the next child's notation.
    /// Illegal outside of `Fold`.
    Right,
}

/// Which document node to check a [`Condition`] on.
#[derive(Clone, Debug)]
pub enum CheckPos {
    /// The current document node.
    Here,
    /// The i'th child of the current document node. If the index is negative, the length of the
    /// list is added to it (so that -1 accesses the last child).
    Child(isize),
    /// The previous child. May only be used inside of [`Notation::Fold`]'s `join` case.
    LeftChild,
    /// The next child. May only be used inside of [`Notation::Fold`]'s `join` case.
    RightChild,
}

/// Literal text in a [`Notation`], to be displayed as-is. Must not contain a newline.
#[derive(Clone, Debug)]
pub struct Literal {
    string: String,
    /// Width of the string in columns.
    width: Width,
}

/// Normalizes the index so that negative indices count back from the end of the list.
/// Returns `None` if the index would be out of bounds.
pub fn normalize_child_index(signed_index: isize, num_children: usize) -> Option<usize> {
    let len = num_children as isize;
    let index: isize = if signed_index < 0 {
        signed_index + len
    } else {
        signed_index
    };
    if index < 0 || index >= len {
        None
    } else {
        Some(index as usize)
    }
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
            EndOfLine => write!(f, "EOL"),
            Text => write!(f, "TEXT"),
            Literal(lit) => write!(f, "'{}'", lit.string),
            Flat(note) => write!(f, "Flat({})", note),
            Indent(lit, _style_label, note) => write!(f, "'{}'⇒({})", lit.string, note),
            Concat(left, right) => write!(f, "{} + {}", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            Check(cond, pos, opt1, opt2) => {
                write!(f, "({:?}@{:?} ? {} | {})", cond, pos, opt1, opt2)
            }
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

    /// `x + y` is shorthand for [`Concat`](Notation::Concat)`(x, y)`.
    fn add(self, other: Notation<L, C>) -> Notation<L, C> {
        Notation::Concat(Box::new(self), Box::new(other))
    }
}

impl<L: StyleLabel, C: Condition> BitOr<Notation<L, C>> for Notation<L, C> {
    type Output = Notation<L, C>;

    /// `x | y` is shorthand for [`Choice`](Notation::Choice)`(x, y)`.
    fn bitor(self, other: Notation<L, C>) -> Notation<L, C> {
        Notation::Choice(Box::new(self), Box::new(other))
    }
}

impl<L: StyleLabel, C: Condition> BitXor<Notation<L, C>> for Notation<L, C> {
    type Output = Notation<L, C>;

    /// `x ^ y` is shorthand for `x + `[`Newline`](Notation::Newline)` + y`.
    fn bitxor(self, other: Notation<L, C>) -> Notation<L, C> {
        self + Notation::Newline + other
    }
}

impl<L: StyleLabel, C: Condition> Shr<Notation<L, C>> for Width {
    type Output = Notation<L, C>;

    /// `i >> x` is shorthand for
    /// [`Indent`](Notation::Indent)`(i_spaces, `[`Newline`](Notation::Newline)` + x)`
    /// (sometimes called "nesting").
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
