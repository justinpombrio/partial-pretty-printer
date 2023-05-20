use crate::geometry::{Pos, Size, Width};
use crate::style::ShadedStyle;

/// A "window" that supports the methods necessary to render a set of [PrettyDocument](crate::PrettyDoc)s.
pub trait PrettyWindow: Sized {
    // Forbid the Error type from containing non-static references so we can use
    // `PrettyWindow` as a trait object.
    type Error: std::error::Error + 'static;

    /// The size of the window.
    fn size(&self) -> Result<Size, Self::Error>;

    /// Render the string at the given position. The position is relative to the window, not
    /// relative to the document. The string is guaranteed to fit in the window, assuming one
    /// column per unicode code point.
    fn print(&mut self, pos: Pos, string: &str, style: ShadedStyle) -> Result<(), Self::Error>;

    /// Fill a section of a line with a character. `len` is the number of times to repeat the
    /// character. The position is relative to the window, not relative to the document. The
    /// filled region is guaranteed to fit in the window, assuming one column per unicode code
    /// point.
    fn fill(
        &mut self,
        pos: Pos,
        ch: char,
        len: Width,
        style: ShadedStyle,
    ) -> Result<(), Self::Error>;
}
