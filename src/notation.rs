use crate::geometry::Width;
use crate::measure::Measure;
use crate::style::Style;
use std::fmt;

// ASSUMPTION:
// In every choice `X | Y`, `min_first_line_len Y <= min_first_line_len X`.

#[derive(Debug, Clone)]
pub struct MeasuredNotation {
    pub measure: Measure,
    pub notation: Box<Notation>,
}

/// Describes how to display a syntactic construct.
#[derive(Debug, Clone)]
pub enum Notation {
    /// Display nothing. Identical to `Literal("")`.
    Empty,
    /// Display a newline. If this is inside an `Indent`, the new line will be indented.
    Newline,
    /// Literal text. Cannot contain a newline.
    Literal(String, Style),
    /// Only consider single-line options of the contained notation.
    Flat(MeasuredNotation),
    /// Indent all lines of the contained notation except the first to the right by the given
    /// number of spaces.
    Indent(Width, MeasuredNotation),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. The right notation's indentation level is not
    /// affected.
    Concat(MeasuredNotation, MeasuredNotation),
    /// Display the left notation if its first line fits within the required width; otherwise
    /// display the right.
    Choice(MeasuredNotation, MeasuredNotation),
    // TODO: support children
    // /// Display a piece of text. Must be used on a texty node.
    // Text(Style),
    // /// Display the `i`th child of this node.
    // /// Must be used on a foresty node.
    // /// `i` must be less than the node's arity number.
    // Child(usize),
    // /// Display the first notation in case this tree has empty text,
    // /// otherwise show the second notation.
    // IfEmptyText(MeasuredNotation, MeasuredNotation),
}

impl fmt::Display for MeasuredNotation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Notation::*;

        match &*self.notation {
            Empty => write!(f, "ε"),
            Newline => write!(f, "↵"),
            Literal(lit, _) => write!(f, "'{}'", lit),
            Flat(note) => write!(f, "Flat({})", note),
            Indent(i, note) => write!(f, "{}⇒({})", i, note),
            Concat(left, right) => write!(f, "{} + {}", left, right),
            Choice(opt1, opt2) => write!(f, "({} | {})", opt1, opt2),
            // Child(i) => write!(f, "${}", i),
            // Text(_) => write!(f, "TEXT"),
            // IfEmptyText(opt1, opt2) => write!(f, "IfEmptyText({} | {})", opt1, opt2),
        }
    }
}
