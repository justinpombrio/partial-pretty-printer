use crate::geometry::{Height, Line, Width};

/// Options for how to display a document within a `Pane`.
#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub highlight_cursor: bool,
    /// Position the document such that the top of the cursor is at this height,
    /// where 1 is the top line of the Pane and 0 is the bottom line.
    pub cursor_height: f32,
    pub width_strategy: WidthStrategy,
}

/// How to choose the document width, after learning the how much width is available.
#[derive(Debug, Clone, Copy)]
pub enum WidthStrategy {
    /// Use all available width in the pane.
    Full,
    /// Use the given width.
    Fixed(Width),
    /// Try to use the given width. But if the pane width is smaller, use that width instead.
    NoMoreThan(Width),
}

impl RenderOptions {
    pub fn focal_line(self, available_height: Height) -> Line {
        assert!(self.cursor_height >= 0.0);
        assert!(self.cursor_height <= 1.0);
        let offset_from_top =
            f32::round((available_height - 1) as f32 * (1.0 - self.cursor_height)) as Line;
        offset_from_top
    }

    pub fn choose_width(self, available_width: Width) -> Width {
        match self.width_strategy {
            WidthStrategy::Full => available_width,
            WidthStrategy::Fixed(width) => width,
            WidthStrategy::NoMoreThan(width) => width.min(available_width),
        }
    }
}
