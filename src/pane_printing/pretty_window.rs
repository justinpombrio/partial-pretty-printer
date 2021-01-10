use crate::geometry::{Col, Line, Size};
use crate::style::Shade;
use crate::LineContents;

/// A "window" that supports the methods necessary to render a set of [PrettyDocument](crate::PrettyDocument)s.
pub trait PrettyWindow: Sized {
    // Forbid the Error type from containing non-static references so we can use
    // `PrettyWindow` as a trait object.
    type Error: std::error::Error + 'static;

    /// The size of the window.
    fn size(&self) -> Result<Size, Self::Error>;

    /// Render the contents of a line. The line consists of some spaces at the start of the line,
    /// followed by a sequence of `(string, style)` pairs to render in order. `line_num` is
    /// relative to the window, not relative to the document.
    fn print_line(&mut self, line_num: Line, contents: LineContents) -> Result<(), Self::Error>;

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
