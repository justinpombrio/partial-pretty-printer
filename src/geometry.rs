use std::fmt;
use std::ops::Add;

/// Line number
pub type Line = u32;
/// Column, as measured in characters.
pub type Col = u16;

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
    pub height: Line,
    pub width: Col,
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
