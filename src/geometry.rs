use std::fmt;
use std::ops::{Add, AddAssign, Sub};

/// Line number
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line(pub u32);
/// Column, measured in characters
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Col(pub u16);
/// Height, measured in lines
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Height(pub u32);
/// Width, measured in characters
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Width(pub u16);

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
pub struct Rectangle {
    pub min_line: Line,
    pub max_line: Line,
    pub min_col: Col,
    pub max_col: Col,
}

impl Pos {
    pub fn zero() -> Pos {
        Pos {
            line: Line(0),
            col: Col(0),
        }
    }
}

impl Rectangle {
    #[cfg(test)]
    pub const fn new(min_col: u16, max_col: u16, min_line: u32, max_line: u32) -> Rectangle {
        Rectangle {
            min_col: Col(min_col),
            max_col: Col(max_col),
            min_line: Line(min_line),
            max_line: Line(max_line),
        }
    }

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

    /// Given N `widths`, returns an iterator over N sub-rectangles with those
    /// widths, in order from left to right. `.next()` will panic if the next
    /// width is larger than the width of the remaining rectangle.
    pub fn horz_splits<'a>(self, widths: &'a [Width]) -> impl Iterator<Item = Rectangle> + 'a {
        HorzSplits { rect: self, widths }
    }

    /// Given N `heights`, returns an iterator over N sub-rectangles with those
    /// heights, in order from top to bottom. `.next()` will panic if the next
    /// height is greater than the height of the remaining rectangle.
    pub fn vert_splits<'a>(self, heights: &'a [Height]) -> impl Iterator<Item = Rectangle> + 'a {
        VertSplits {
            rect: self,
            heights,
        }
    }
}

struct HorzSplits<'a> {
    rect: Rectangle,
    widths: &'a [Width],
}

impl<'a> Iterator for HorzSplits<'a> {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        match self.widths {
            [] => None,
            [w, ws @ ..] => {
                let split = self.rect.min_col + *w;
                let result = Rectangle {
                    max_col: split,
                    ..self.rect
                };
                self.rect = Rectangle {
                    min_col: split,
                    ..self.rect
                };
                assert!(self.rect.min_col <= self.rect.max_col);
                self.widths = ws;
                Some(result)
            }
        }
    }
}

struct VertSplits<'a> {
    rect: Rectangle,
    heights: &'a [Height],
}

impl<'a> Iterator for VertSplits<'a> {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        match self.heights {
            [] => None,
            [h, hs @ ..] => {
                let split = self.rect.min_line + *h;
                let result = Rectangle {
                    max_line: split,
                    ..self.rect
                };
                self.rect = Rectangle {
                    min_line: split,
                    ..self.rect
                };
                assert!(self.rect.min_line <= self.rect.max_line);
                self.heights = hs;
                Some(result)
            }
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Col {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl fmt::Debug for Col {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Width {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl fmt::Debug for Width {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl fmt::Debug for Height {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<Width> for Col {
    type Output = Col;

    fn add(self, width: Width) -> Col {
        Col(self.0 + width.0)
    }
}

impl Add<Width> for Width {
    type Output = Width;

    fn add(self, width: Width) -> Width {
        Width(self.0 + width.0)
    }
}

impl AddAssign<Width> for Col {
    fn add_assign(&mut self, width: Width) {
        *self = *self + width
    }
}

impl Add<Height> for Line {
    type Output = Line;

    fn add(self, height: Height) -> Line {
        Line(self.0 + height.0)
    }
}

impl Add<Height> for Height {
    type Output = Height;

    fn add(self, height: Height) -> Height {
        Height(self.0 + height.0)
    }
}

impl AddAssign<Height> for Line {
    fn add_assign(&mut self, height: Height) {
        *self = *self + height
    }
}

impl Sub<Col> for Col {
    type Output = Width;

    fn sub(self, other: Col) -> Width {
        Width(self.0 - other.0)
    }
}

impl Sub<Line> for Line {
    type Output = Height;

    fn sub(self, other: Line) -> Height {
        Height(self.0 - other.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: Rectangle = Rectangle::new(1, 5, 2, 4);

    const BIG: Rectangle = Rectangle::new(1, 5, 1, 4);

    #[test]
    fn test_split_horz1() {
        let mut it = RECT.horz_splits(&[Width(1), Width(3)]);
        assert_eq!(it.next(), Some(Rectangle::new(1, 2, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(2, 5, 2, 4)));
        assert_eq!(it.next(), None)
    }

    #[test]
    fn test_split_horz2() {
        let mut it = RECT.horz_splits(&[
            Width(0),
            Width(1),
            Width(0),
            Width(1),
            Width(0),
            Width(1),
            Width(0),
            Width(1),
            Width(0),
        ]);
        assert_eq!(it.next(), Some(Rectangle::new(1, 1, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 2, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(2, 2, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(2, 3, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(3, 3, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(3, 4, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(4, 4, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(4, 5, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(5, 5, 2, 4)));
        assert_eq!(it.next(), None)
    }

    #[test]
    #[should_panic]
    fn test_split_horz3() {
        let mut it = RECT.horz_splits(&[Width(5), Width(1)]);
        it.next();
    }

    #[test]
    #[should_panic]
    fn test_split_horz4() {
        let mut it = RECT.horz_splits(&[Width(1), Width(5)]);
        it.next();
        it.next();
    }

    #[test]
    fn test_split_horz5() {
        // It's ok to leave some leftover width
        let mut it = RECT.horz_splits(&[Width(1), Width(1)]);
        assert_eq!(it.next(), Some(Rectangle::new(1, 2, 2, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(2, 3, 2, 4)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_split_vert1() {
        let mut it = BIG.vert_splits(&[Height(1), Height(2)]);
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 1, 2)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 2, 4)));
        assert_eq!(it.next(), None)
    }

    #[test]
    fn test_split_vert2() {
        let mut it = BIG.vert_splits(&[
            Height(0),
            Height(1),
            Height(0),
            Height(1),
            Height(0),
            Height(1),
            Height(0),
        ]);
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 1, 1)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 1, 2)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 2, 2)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 2, 3)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 3, 3)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 3, 4)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 4, 4)));
        assert_eq!(it.next(), None)
    }

    #[test]
    #[should_panic]
    fn test_split_vert3() {
        let mut it = BIG.vert_splits(&[Height(4), Height(1)]);
        it.next();
    }

    #[test]
    #[should_panic]
    fn test_split_vert4() {
        let mut it = BIG.vert_splits(&[Height(1), Height(4)]);
        it.next();
        it.next();
    }

    #[test]
    fn test_split_vert5() {
        // It's ok to leave some leftover height
        let mut it = BIG.vert_splits(&[Height(1), Height(1)]);
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 1, 2)));
        assert_eq!(it.next(), Some(Rectangle::new(1, 5, 2, 3)));
        assert_eq!(it.next(), None);
    }
}
