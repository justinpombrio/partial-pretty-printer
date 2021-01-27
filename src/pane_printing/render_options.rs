use crate::geometry::Width;
use std::cmp;

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub cursor_visibility: CursorVisibility,
    pub scroll_strategy: ScrollStrategy,
    pub width_strategy: WidthStrategy,
}

/// The visibility of the cursor in some document.
#[derive(Debug, Clone, Copy)]
pub enum CursorVisibility {
    Show,
    Hide,
}

/// How to choose the document width, after learning the how much width is available.
#[derive(Debug, Clone, Copy)]
pub enum WidthStrategy {
    /// Use all available width.
    Full,
    /// Use the given width.
    Fixed(Width),
    /// Try to use the given width. If that's not available, use as much width is available.
    NoMoreThan(Width),
}

impl WidthStrategy {
    pub fn choose(&self, available_width: Width) -> Width {
        match self {
            WidthStrategy::Full => available_width,
            WidthStrategy::Fixed(width) => *width,
            WidthStrategy::NoMoreThan(width) => cmp::min(*width, available_width),
        }
    }
}

/// What part of the document to show, which may depend on the cursor position.
#[derive(Debug, Clone, Copy)]
pub struct ScrollStrategy {
    /// Position the document such that the top of the cursor is at this height,
    /// where 1 is the top line of the Pane and 0 is the bottom line.
    cursor_height: f32,
}
