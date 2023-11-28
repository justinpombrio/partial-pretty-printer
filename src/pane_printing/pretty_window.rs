use crate::geometry::{Pos, Size, Width};

/// A "window" that supports the methods necessary to render a set of
/// [PrettyDocument](crate::PrettyDoc)s.
pub trait PrettyWindow<S>: Sized {
    // Forbid the Error type from containing non-static references so we can use
    // `PrettyWindow` as a trait object.
    type Error: std::error::Error + 'static;

    /// Get the size of this window.
    fn size(&self) -> Result<Size, Self::Error>;

    /// Print the given character at the given window position in the given style.
    /// `width` is the width of the character in columns (either 1 or 2). The character
    /// is guaranteed to fit in the window.
    fn print_char(
        &mut self,
        ch: char,
        pos: Pos,
        style: &S,
        width: usize,
    ) -> Result<(), Self::Error>;
}
