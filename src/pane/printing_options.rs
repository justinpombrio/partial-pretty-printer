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
    /// Whether to invoke [`PrettyWindow::set_focus`] with the focus point of this document.
    pub set_focus: bool,
    /// The width to print the document at (Choices in the Notation will be picked so as to not
    /// exceed this width, if at all possible.)
    pub printing_width: Width,
    /// What to do with lines that exceed the printing width, despite our best attempts.
    pub overflow_behavior: OverflowBehavior<Style>,
}

/// How to display lines that exceed the printing width (despite all attempts to print the
/// document within the printing width).
#[derive(Debug, Clone)]
pub enum OverflowBehavior<Style> {
    /// Clip lines longer than this. Show `str` in `Style` at the end of clipped lines.
    Clip(&'static str, Style, Width),
    /// Wrap the end of the line at this width. Show `str` in `Style` at the beginning of the next
    /// line.
    Wrap(&'static str, Style, Width),
}

impl<Style> PrintingOptions<Style> {
    /// Choose which row of the pane the focus line should be displayed on.
    pub(crate) fn choose_focus_line_row(&self, pane_height: Height) -> Row {
        assert!(self.focus_height >= 0.0);
        assert!(self.focus_height <= 1.0);
        f32::round((pane_height - 1) as f32 * self.focus_height) as Row
    }
}

impl<Style: Clone> OverflowBehavior<Style> {
    pub(crate) fn wrap_line<'d, D: PrettyDoc<'d, Style = Style>>(
        &self,
        mut long_line: Line<'d, D>,
    ) -> Vec<Line<'d, D>> {
        use crate::geometry::str_width;
        use crate::Segment;

        match self {
            OverflowBehavior::Clip(marker_str, marker_style, width) => {
                if long_line.width() <= *width {
                    vec![long_line]
                } else {
                    let marker_width = str_width(marker_str);
                    let clipped_width = *width - marker_width;
                    let (mut first_line, _) = long_line.split_at(clipped_width);
                    if first_line.width() + 1 == clipped_width {
                        first_line.segments.push(Segment {
                            str: " ",
                            style: marker_style.clone(),
                            width: 1,
                        });
                    }
                    first_line.segments.push(Segment {
                        str: marker_str,
                        style: marker_style.clone(),
                        width: marker_width,
                    });
                    vec![first_line]
                }
            }
            OverflowBehavior::Wrap(marker_str, marker_style, width) => {
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

    pub(crate) fn is_shown(&self, pos: Width, line_width: Width) -> bool {
        use crate::geometry::str_width;

        if pos > line_width {
            return false;
        }

        match self {
            OverflowBehavior::Wrap(_, _, _) => true,
            OverflowBehavior::Clip(marker_str, _, clip_width) => {
                line_width <= *clip_width || pos + str_width(marker_str) <= *clip_width
            }
        }
    }
}
