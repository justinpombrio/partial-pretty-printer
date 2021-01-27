use super::pretty_window::PrettyWindow;
use crate::geometry::{Col, Line, Pos, Rectangle, Size, Width};
use crate::pretty_printing::LineContents;
use crate::style::{Shade, ShadedStyle};

/// A rectangular area of a window. You can pretty-print to it, or get sub-panes
/// of it and pretty-print to those.
pub struct Pane<'w, W>
where
    W: PrettyWindow,
{
    pub(crate) window: &'w mut W,
    pub(crate) rect: Rectangle,
}

/// Errors that can occur while attempting to render to a `Pane`.
#[derive(thiserror::Error, Debug)]
pub enum PaneError<W: PrettyWindow> {
    #[error("requested pane is not a subpane of the current pane")]
    NotSubPane,

    #[error("pane notation layout demands cannot be satisfied")]
    ImpossibleDemands,

    #[error("invalid pane notation")]
    InvalidNotation,

    #[error("missing document in pane notation: {0}")]
    MissingLabel(String),

    #[error("window error: {0}")]
    PrettyWindowErr(#[source] W::Error),
}

impl<'w, W> Pane<'w, W>
where
    W: PrettyWindow,
{
    pub fn new(window: &mut W) -> Result<Pane<W>, PaneError<W>> {
        let Size { width, height } = window.size().map_err(PaneError::PrettyWindowErr)?;
        let rect = Rectangle {
            min_line: Line(0),
            min_col: Col(0),
            max_line: Line(0) + height,
            max_col: Col(0) + width,
        };
        Ok(Pane { window, rect })
    }

    /// Get a new `Pane` representing only the given sub-region of this `Pane`.
    /// Returns `None` if `rect` is not fully contained within this `Pane`.
    /// `rect` is specified in the same absolute coordinate system as the full
    /// `PrettyWindow` (not specified relative to this `Pane`!).
    pub fn sub_pane(&mut self, rect: Rectangle) -> Option<Pane<'_, W>> {
        if !self.rect.covers(rect) {
            return None;
        }
        Some(Pane {
            window: self.window,
            rect,
        })
    }

    pub fn print_line(
        &mut self,
        line: Line,
        contents: LineContents,
        highlight_cursor: bool,
    ) -> Result<(), PaneError<W>> {
        let mut pos = Pos { line, col: Col(0) };
        pos.col += Width(contents.spaces as u16);
        for (string, style, hl) in contents.contents {
            let shade = if highlight_cursor && hl {
                Shade::highlight()
            } else {
                Shade::background()
            };
            let shaded_style = ShadedStyle::new(style, shade);
            self.print(pos, string, shaded_style)?;
            pos.col += Width(string.chars().count() as u16);
        }
        Ok(())
    }

    fn print(&mut self, pos: Pos, string: &str, style: ShadedStyle) -> Result<(), PaneError<W>> {
        if pos.col >= self.rect.max_col {
            // Trying to print outside the pane.
            return Ok(());
        }
        let max_len = (self.rect.max_col - pos.col).0 as usize;
        if string.chars().count() > max_len {
            let (last_index, last_char) = string.char_indices().take(max_len).last().unwrap();
            let end_index = last_index + last_char.len_utf8();
            let truncated_string = &string[0..end_index];
            self.window
                .print(pos, truncated_string, style)
                .map_err(PaneError::PrettyWindowErr)
        } else {
            self.window
                .print(pos, string, style)
                .map_err(PaneError::PrettyWindowErr)
        }
    }

    pub fn fill(&mut self, pos: Pos, ch: char, style: ShadedStyle) -> Result<(), PaneError<W>> {
        if pos.col >= self.rect.max_col {
            // Trying to print outside the pane.
            return Ok(());
        }
        let len = (self.rect.max_col - pos.col).0 as usize;
        self.window
            .fill(pos, ch, len, style)
            .map_err(PaneError::PrettyWindowErr)
    }
}
