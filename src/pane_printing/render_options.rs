use crate::geometry::{Height, Row, Width};

/// Options for how to display a document within a pane.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Set the focus of the document to be at the start or end of this node. Each `usize` is the
    /// index of a child node, starting from the root.
    pub focus_path: Vec<usize>,
    /// Whether the focus should be at the start or end of the node.
    pub focus_side: FocusSide,
    /// Position the document such that the focus is at this height, where 0.0 is the top line of
    /// the pane and 1.0 is the bottom line.
    pub focus_height: f32,
    /// How to choose the document width.
    pub width_strategy: WidthStrategy,
}

/// Whether the focus should be at the start or end of the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSide {
    Start,
    End,
}

/// How to choose the document width, after learning the how much width is available.
#[derive(Debug, Clone, Copy)]
pub enum WidthStrategy {
    /// Use all available width in the pane.
    Full,
    /// Use the given width.
    Fixed(Width),
    /// Use either the given width or the available pane width, whichever is smaller.
    NoMoreThan(Width),
}

impl RenderOptions {
    /// Choose which row of the pane the focus line should be displayed on.
    pub(crate) fn choose_focus_line_row(&self, pane_height: Height) -> Row {
        assert!(self.focus_height >= 0.0);
        assert!(self.focus_height <= 1.0);
        f32::round((pane_height - 1) as f32 * self.focus_height) as Row
    }

    /// Choose what width to use when pretty-printing the document.
    pub(crate) fn choose_width(&self, available_width: Width) -> Width {
        match self.width_strategy {
            WidthStrategy::Full => available_width,
            WidthStrategy::Fixed(width) => width,
            WidthStrategy::NoMoreThan(width) => width.min(available_width),
        }
    }
}
