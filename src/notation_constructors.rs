use crate::geometry::Width;
use crate::measure::Measure;
use crate::notation::{MeasuredNotation, Notation};
use crate::style::Style;
use std::ops::{Add, BitOr, BitXor, Shr};

/// Display nothing. Identical to `lit("")`.
pub fn empty() -> MeasuredNotation {
    MeasuredNotation {
        measure: Measure::flat(0),
        notation: Box::new(Notation::Empty),
    }
}

/// Display a newline. If this is inside an `indent`, the new line will be indented.
pub fn nl() -> MeasuredNotation {
    MeasuredNotation {
        measure: Measure::nonflat(0),
        notation: Box::new(Notation::Newline),
    }
}

/// Literal text. Cannot contain a newline.
pub fn lit(s: &str, style: Style) -> MeasuredNotation {
    let len = s.chars().count() as Width;
    MeasuredNotation {
        measure: Measure::flat(len),
        notation: Box::new(Notation::Literal(s.to_owned(), style)),
    }
}

/// Only consider single-line options of the contained notation.
pub fn flat(note: MeasuredNotation) -> MeasuredNotation {
    MeasuredNotation {
        measure: note.measure.flattened(),
        notation: Box::new(Notation::Flat(note)),
    }
}

/// Indent all lines of the contained notation except the first to the right by the given
/// number of spaces.
pub fn indent(ind: Width, note: MeasuredNotation) -> MeasuredNotation {
    MeasuredNotation {
        measure: note.measure,
        notation: Box::new(Notation::Indent(ind, note)),
    }
}

/// Display both notations. The first character of the right notation immediately follows the
/// last character of the left notation. The right notation's indentation level is not
/// affected.
pub fn concat(left: MeasuredNotation, right: MeasuredNotation) -> MeasuredNotation {
    MeasuredNotation {
        measure: left.measure.concat(right.measure),
        notation: Box::new(Notation::Concat(left, right)),
    }
}

/// Display the left notation if its first line fits within the required width; otherwise
/// display the right.
pub fn choice(left: MeasuredNotation, right: MeasuredNotation) -> MeasuredNotation {
    MeasuredNotation {
        measure: left.measure.choice(right.measure),
        notation: Box::new(Notation::Choice(left, right)),
    }
}

impl Add<MeasuredNotation> for MeasuredNotation {
    type Output = MeasuredNotation;

    /// Shorthand for `concat`.
    fn add(self, other: MeasuredNotation) -> MeasuredNotation {
        concat(self, other)
    }
}

impl BitOr<MeasuredNotation> for MeasuredNotation {
    type Output = MeasuredNotation;

    /// Shorthand for `choice`.
    fn bitor(self, other: MeasuredNotation) -> MeasuredNotation {
        choice(self, other)
    }
}

impl BitXor<MeasuredNotation> for MeasuredNotation {
    type Output = MeasuredNotation;

    /// Shorthand for `X + newline() + Y`.
    fn bitxor(self, other: MeasuredNotation) -> MeasuredNotation {
        self + nl() + other
    }
}

impl Shr<MeasuredNotation> for Width {
    type Output = MeasuredNotation;

    /// Shorthand for nesting (indented newline)
    fn shr(self, notation: MeasuredNotation) -> MeasuredNotation {
        indent(self, concat(nl(), notation))
    }
}
