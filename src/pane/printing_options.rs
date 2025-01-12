use crate::{FocusTarget, Height, Line, PrettyDoc, Row, Width};

#[cfg(doc)]
use super::pretty_window::PrettyWindow;

/// Options for how to print a document within a pane.
#[derive(Debug, Clone)]
pub struct PrintingOptions<Style> {
    /// Set the focus of the document to be at some target relative to the node at this path. Each
    /// `usize` is the index of a child node, starting from the root.
    pub focus_path: Vec<usize>,
    /// Where the focus should be relative to the node given by `focus_path`.
    pub focus_target: FocusTarget,
    /// Position the document such that the focus is at this height, where 0.0 is the top line of
    /// the pane and 1.0 is the bottom line.
    pub focus_height: f32,
    /// How to choose the document width.
    pub width_strategy: WidthStrategy,
    /// How to wrap lines that overflow the document width.
    pub line_wrapping: LineWrapping<Style>,
    /// Whether to invoke [`PrettyWindow::set_focus`] with the focus point of this document.
    pub set_focus: bool,
}

/// How to choose the document width, after learning the how much width is available.
#[derive(Debug, Clone, Copy)]
pub enum WidthStrategy {
    /// Use all available width in the pane.
    Full,
    /// Use the given width. If the pane is too narrow, the document will be printed with the given
    /// width but then truncated to fit in the pane.
    Fixed(Width),
    /// Use either the given width or the available pane width, whichever is smaller.
    NoMoreThan(Width),
}

/// How to wrap lines that exceed the available width (despite all attempts to print the document
/// within the available width).
#[derive(Debug, Clone)]
pub enum LineWrapping<Style> {
    /// Don't show the end of the line.
    Clip,
    /// Wrap the end of the line. Show `str` in `Style` at the beginning of any wrapped lines.
    Wrap(&'static str, Style),
}

impl<Style> PrintingOptions<Style> {
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

impl<Style: Clone> LineWrapping<Style> {
    pub(crate) fn wrap_line<'d, D: PrettyDoc<'d, Style = Style>>(
        &self,
        mut long_line: Line<'d, D>,
        width: Width,
    ) -> Vec<Line<'d, D>> {
        use crate::geometry::str_width;
        use crate::Segment;

        match self {
            LineWrapping::Clip => {
                let (first_line, _) = long_line.split_at(width);
                vec![first_line]
            }
            LineWrapping::Wrap(marker_str, marker_style) => {
                let marker_segment = Segment {
                    str: marker_str,
                    style: marker_style.clone(),
                    width: str_width(marker_str),
                };
                let mut lines = Vec::new();
                let mut cur_line = Line::new();
                while !long_line.segments.is_empty() {
                    let (mut first_line, rest) = long_line.split_at(width - cur_line.width());
                    cur_line.segments.append(&mut first_line.segments);
                    lines.push(cur_line);
                    cur_line = Line {
                        segments: vec![marker_segment.clone()],
                    };
                    long_line = rest;
                }
                lines
            }
        }
    }
}
