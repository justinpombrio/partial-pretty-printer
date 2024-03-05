use crate::{Pos, Size, Style};

/// A "window" that can display a set of pretty-printed [`PrettyDoc`](crate::PrettyDoc)s.
pub trait PrettyWindow: Sized {
    /// An error that can happen when displaying to the window. It is forbidden from containing
    /// non-static references, so that `PrettyWindow` can be used as a trait object.
    type Error: std::error::Error + 'static;

    /// The style metadata used in the document(s).
    type Style: Style;

    /// Get the size of this window.
    fn size(&self) -> Result<Size, Self::Error>;

    /// Display a character at the given window position in the given style. `full_width` indicates
    /// whether the character is 1 (`false`) or 2 (`true`) columns wide. The character is guaranteed
    /// to fit in the window.
    fn display_char(
        &mut self,
        ch: char,
        pos: Pos,
        style: &Self::Style,
        full_width: bool,
    ) -> Result<(), Self::Error>;
}
