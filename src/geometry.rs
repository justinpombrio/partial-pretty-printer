use std::fmt;
use std::ops::Add;

/// Line number
pub type Line = u32;

/// Column, measured in characters
pub type Col = u16;

/// Height, measured in lines
pub type Height = u32;

/// Width, measured in characters
pub type Width = u16;

/// A character position, relative to the screen or the document.
///
/// The origin is in the upper left, and is `(0, 0)`. I.e., this is 0-indexed.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct Pos {
    pub line: Line,
    pub col: Col,
}

/// A size, in characters.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct Size {
    pub height: Height,
    pub width: Width,
}

/// A rectangle, either on the screen, or on the document.
/// Includes its upper-left, but excludes its lower-right.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) struct Rectangle {
    pub min_line: Line,
    pub max_line: Line,
    pub min_col: Col,
    pub max_col: Col,
}

impl Pos {
    pub fn zero() -> Pos {
        Pos { line: 0, col: 0 }
    }
}

impl Rectangle {
    pub fn width(self) -> Width {
        self.max_col - self.min_col
    }

    pub fn height(self) -> Height {
        self.max_line - self.min_line
    }

    /// Does this rectangle completely cover the other rectangle?
    pub fn covers(self, other: Rectangle) -> bool {
        self.min_line <= other.min_line
            && other.max_line <= self.max_line
            && self.min_col <= other.min_col
            && other.max_col <= self.max_col
    }
}

impl Add<Size> for Pos {
    type Output = Pos;

    fn add(self, size: Size) -> Pos {
        Pos {
            line: self.line + size.height,
            col: self.col + size.width,
        }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.height, self.width)
    }
}
