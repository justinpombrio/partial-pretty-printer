use std::fmt;
use std::ops::Add;

/// Line number. 0-indexed.
pub type Row = u32;

/// Zero-indexed column number. A typical ascii character is half-width and takes up one column.
/// Some Unicode characters, especially in East-Asian languages, are full-width and take up two
/// columns.
pub type Col = u16;

/// Height, measured in lines.
pub type Height = u32;

/// Width, measured in columns.
pub type Width = u16;

/// A position relative to the screen or the document.
///
/// The origin is in the upper left, and is `(0, 0)`. I.e., this is 0-indexed.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct Pos {
    pub row: Row,
    pub col: Col,
}

/// The size of a two-dimensional rectangular region.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct Size {
    pub height: Height,
    pub width: Width,
}

/// A rectangle, either on the screen, or on the document.
/// Includes its upper-left, but excludes its lower-right.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) struct Rectangle {
    pub min_row: Row,
    pub max_row: Row,
    pub min_col: Col,
    pub max_col: Col,
}

impl Pos {
    pub fn zero() -> Pos {
        Pos { row: 0, col: 0 }
    }
}

impl Rectangle {
    pub fn from_size(size: Size) -> Rectangle {
        Rectangle {
            min_row: 0,
            min_col: 0,
            max_row: size.height,
            max_col: size.width,
        }
    }

    pub fn width(self) -> Width {
        self.max_col - self.min_col
    }

    pub fn height(self) -> Height {
        self.max_row - self.min_row
    }
}

impl Add<Size> for Pos {
    type Output = Pos;

    fn add(self, size: Size) -> Pos {
        Pos {
            row: self.row + size.height,
            col: self.col + size.width,
        }
    }
}

/// The width of a string in columns. May be an overestimate. Not to be confused with number of
/// bytes, Unicode code points, or Unicode grapheme clusters.
pub fn str_width(s: &str) -> Width {
    unicode_width::UnicodeWidthStr::width(s) as Width
}

/// Like [`str_width`] but for a single character.
pub fn is_char_full_width(ch: char) -> bool {
    unicode_width::UnicodeWidthChar::width(ch) == Some(2)
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.height, self.width)
    }
}
