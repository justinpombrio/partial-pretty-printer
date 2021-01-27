use crate::geometry::{Col, Line, Pos, Size};
use crate::style::{Shade, Style};
use std::fmt;

/// A "window" that supports the methods necessary to render a set of [PrettyDocument](crate::PrettyDocument)s.
pub trait PrettyWindow: Sized + fmt::Debug {
    // Forbid the Error type from containing non-static references so we can use
    // `PrettyWindow` as a trait object.
    type Error: std::error::Error + 'static;

    /// The size of the window.
    fn size(&self) -> Result<Size, Self::Error>;

    /// Render the string at the given position. The position is relative to the window, not
    /// relative to the document.
    fn print(&mut self, pos: Pos, string: &str, style: Style) -> Result<(), Self::Error>;

    /// Fill a section of a line with a character. `len` is the number of times to repeat the
    /// character. The position is relative to the window, not relative to the document.
    fn fill(&mut self, pos: Pos, ch: char, len: usize, style: Style) -> Result<(), Self::Error>;

    /// Highlight part of a line by shading and/or reversing it. If `shade` is `Some`,
    /// set the region's background color to that `Shade`. If `reverse`
    /// is true, toggle whether the foreground and background colors are swapped
    /// within the region.
    fn highlight(
        &mut self,
        line: Line,
        cols: (Col, Col),
        shade: Option<Shade>,
        reverse: bool,
    ) -> Result<(), Self::Error>;
}
