use crate::{
    geometry::{is_char_full_width, str_width, Rectangle},
    pane::{
        divvy::Divvier, DocLabel, OverflowBehavior, PaneNotation, PaneSize, PrettyWindow,
        PrintingOptions,
    },
    pretty_print, Height, Line, Pos, PrettyDoc, PrintingError, Row, Size, Width,
};
use std::error::Error;
use std::iter;

/// Errors that can occur while displaying a pane.
///
/// `W` is the type of [`PrettyWindow::Error`], and `E` is the type of [`PrettyDoc::Error`].
#[derive(thiserror::Error, Debug)]
pub enum PaneError<W: Error + 'static, E: Error + 'static> {
    #[error(
        "Invalid pane notation: PaneSize::Dyanmic may only be used in a PaneNotation::Doc pane"
    )]
    InvalidUseOfDynamic,

    #[error("PrettyWindow error: {0}")]
    PrettyWindowError(#[source] W),

    #[error("PrettyDoc printing error: {0}")]
    PrintingError(#[from] PrintingError<E>),
}

/// Display a [`PaneNotation`] to a [`PrettyWindow`].
///
/// `style` is the initial style to use on the entire pane. `get_content` is a function to look up
/// a document by [`DocLabel`]. It's also given the available pane width. It must return both the
/// document and [extra information](PrintingOptions) about how to print it.
pub fn display_pane<'d, L, D, W>(
    window: &mut W,
    notation: &PaneNotation<L, D::Style>,
    style: &D::Style,
    get_content: &impl Fn(L, Width) -> Option<(D, PrintingOptions<D::Style>)>,
) -> Result<(), PaneError<W::Error, D::Error>>
where
    L: DocLabel,
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style>,
{
    let size = window.size().map_err(PaneError::PrettyWindowError)?;
    let rect = Rectangle::from_size(size);
    display_pane_rec(window, notation, style, get_content, rect)
}

fn display_pane_rec<'d, L, D, W>(
    window: &mut W,
    notation: &PaneNotation<L, D::Style>,
    style: &D::Style,
    get_content: &impl Fn(L, Width) -> Option<(D, PrintingOptions<D::Style>)>,
    rect: Rectangle,
) -> Result<(), PaneError<W::Error, D::Error>>
where
    L: DocLabel,
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style>,
{
    use crate::pretty_doc::Style;

    match notation {
        PaneNotation::Fill { ch } => {
            let is_full_width = is_char_full_width(*ch);
            let char_width = if is_full_width { 2 } else { 1 };

            for row in rect.min_row..rect.max_row {
                let mut col = rect.min_col;
                while col + char_width <= rect.max_col {
                    window
                        .display_char(*ch, Pos { row, col }, style, is_full_width)
                        .map_err(PaneError::PrettyWindowError)?;
                    col += char_width;
                }
                if col < rect.max_col {
                    window
                        .display_char(' ', Pos { row, col }, style, false)
                        .map_err(PaneError::PrettyWindowError)?;
                }
            }
        }
        PaneNotation::Doc { label } => {
            if let Some((doc, options)) = get_content(label.clone(), rect.width()) {
                let printed_doc = PrintedDoc::new(doc, &options, rect.size(), style)?;
                printed_doc.display(window, rect)?;
            }
        }
        PaneNotation::Style {
            style: inner_style,
            notation: inner_notation,
        } => {
            let combined_style = D::Style::combine(style, inner_style);
            display_pane_rec(window, inner_notation, &combined_style, get_content, rect)?;
        }
        PaneNotation::Horz(panes) => {
            let pane_sizes = panes
                .iter()
                .map(|(size, _)| size.to_owned())
                .collect::<Vec<_>>();
            let divvier = Divvier::new(rect.width() as usize, pane_sizes);
            let mut available_size = Size {
                width: divvier.remaining() as Width,
                height: rect.height(),
            };
            let mut dynamic_docs = Vec::new();
            let mut dynamic_widths: Vec<usize> = Vec::new();
            for (size, child_note) in panes {
                if *size != PaneSize::Dynamic {
                    continue;
                }

                let (label, doc_style) = extract_doc::<L, D, W>(child_note, style.clone())?;
                let printed_doc = if let Some((doc, options)) =
                    get_content(label.clone(), available_size.width)
                {
                    PrintedDoc::new(doc, &options, available_size, &doc_style)?
                } else {
                    PrintedDoc::new_empty(&doc_style)
                };

                let width = printed_doc.width().min(available_size.width);
                available_size.width -= width;
                dynamic_widths.push(width as usize);
                dynamic_docs.push(printed_doc);
            }
            let widths = divvier.finish(dynamic_widths);

            let mut col = rect.min_col;
            let mut dynamic_docs = dynamic_docs.into_iter();
            for ((size, child_note), width) in panes.iter().zip(widths.into_iter()) {
                // Split this pane's rectangle horizontally (a.k.a. vertical slices) into multiple subpanes
                let old_col = col;
                col += width as Width;
                let child_rect = Rectangle {
                    min_col: old_col,
                    max_col: col,
                    min_row: rect.min_row,
                    max_row: rect.max_row,
                };

                if let PaneSize::Dynamic = size {
                    let doc = dynamic_docs.next().unwrap();
                    doc.display(window, child_rect)?;
                } else {
                    display_pane_rec(window, child_note, style, get_content, child_rect)?;
                }
            }
        }
        PaneNotation::Vert(panes) => {
            let pane_sizes = panes
                .iter()
                .map(|(size, _)| size.to_owned())
                .collect::<Vec<_>>();
            let divvier = Divvier::new(rect.height() as usize, pane_sizes);
            let mut available_size = Size {
                width: rect.width(),
                height: divvier.remaining() as Height,
            };
            let mut dynamic_docs = Vec::new();
            let mut dynamic_heights: Vec<usize> = Vec::new();
            for (size, child_note) in panes {
                if *size != PaneSize::Dynamic {
                    continue;
                }

                let (label, doc_style) = extract_doc::<L, D, W>(child_note, style.clone())?;
                let printed_doc = if let Some((doc, options)) =
                    get_content(label.clone(), available_size.width)
                {
                    PrintedDoc::new(doc, &options, available_size, &doc_style)?
                } else {
                    PrintedDoc::new_empty(&doc_style)
                };

                let height = printed_doc.height();
                available_size.height -= height;
                dynamic_heights.push(height as usize);
                dynamic_docs.push(printed_doc);
            }
            let heights = divvier.finish(dynamic_heights);

            let mut row = rect.min_row;
            let mut dynamic_docs = dynamic_docs.into_iter();
            for ((size, child_note), height) in panes.iter().zip(heights.into_iter()) {
                // Split this pane's rectangle vertically (a.k.a. horizontal slices) into multiple subpanes
                let old_row = row;
                row += height as Row;
                let child_rect = Rectangle {
                    min_col: rect.min_col,
                    max_col: rect.max_col,
                    min_row: old_row,
                    max_row: row,
                };

                if let PaneSize::Dynamic = size {
                    let doc = dynamic_docs.next().unwrap();
                    doc.display(window, child_rect)?;
                } else {
                    display_pane_rec(window, child_note, style, get_content, child_rect)?;
                }
            }
        }
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
fn extract_doc<'d, L, D, W>(
    mut notation: &PaneNotation<L, D::Style>,
    mut style: D::Style,
) -> Result<(L, D::Style), PaneError<W::Error, D::Error>>
where
    L: DocLabel,
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style>,
{
    use crate::pretty_doc::Style;

    while let PaneNotation::Style {
        notation: inner_notation,
        style: inner_style,
    } = notation
    {
        notation = inner_notation;
        style = D::Style::combine(&style, inner_style);
    }
    if let PaneNotation::Doc { label } = notation {
        Ok((label.clone(), style))
    } else {
        Err(PaneError::InvalidUseOfDynamic)
    }
}

struct PrintedDoc<'d, D: PrettyDoc<'d>> {
    lines: Vec<Line<'d, D>>,
    /// Which line in `lines` is the focus line.
    focus_line_index: usize,
    /// Which row of the pane should the focus line be displayed on.
    focus_line_row: Row,
    /// Focus point of the document, relative to the pane.
    focus_point: Option<Pos>,
    /// Style to apply to blank space.
    blank_style: D::Style,
}

impl<'d, D: PrettyDoc<'d>> PrintedDoc<'d, D> {
    /// Construct a blank PrintedDoc (in case `getContent()` returned `None`).
    fn new_empty(root_style: &D::Style) -> Self {
        PrintedDoc {
            lines: Vec::new(),
            focus_line_index: 0,
            focus_line_row: 0,
            focus_point: None,
            blank_style: root_style.clone(),
        }
    }

    /// Pretty-print the portion of document that would fit in the given `size`,
    /// storing it as text in the `PrintedDoc`.
    fn new(
        doc: D,
        options: &PrintingOptions<D::Style>,
        size: Size,
        root_style: &D::Style,
    ) -> Result<Self, PrintingError<D::Error>> {
        if size.height == 0 || size.width == 0 {
            return Ok(PrintedDoc::new_empty(root_style));
        }

        let focus_line_row = options.choose_focus_line_row(size.height);
        let (upward_printer, focused_line, downward_printer) = pretty_print(
            doc,
            options.printing_width,
            &options.focus_path,
            options.focus_target,
            Some(root_style),
        )?;

        let focused_line_width = focused_line.width();
        let (focused_line_left, focused_line_right) = focused_line.split_at_focus();
        let mut upward_printer = LineWrapper::new(
            iter::once(Ok(focused_line_left)).chain(upward_printer),
            options.printing_width,
            options.overflow_behavior.clone(),
            true,
        );

        let left_half_of_center_line = upward_printer.next().unwrap_or_else(|| Ok(Line::new()))?;
        let left_width = left_half_of_center_line.width();
        let focus_is_shown = options
            .overflow_behavior
            .is_shown(left_width, focused_line_width);

        let center_line_not_wrapped = Line {
            segments: [
                left_half_of_center_line.segments,
                focused_line_right.segments,
            ]
            .concat(),
        };
        let mut downward_printer = LineWrapper::new(
            iter::once(Ok(center_line_not_wrapped)).chain(downward_printer),
            options.printing_width,
            options.overflow_behavior.clone(),
            false,
        );
        let center_line = downward_printer.next().unwrap_or_else(|| Ok(Line::new()))?;

        // Take the appropriate number of lines, both up and down. Using iterators ensures we don't
        // compute lines we don't need.
        let mut lines = Vec::new();
        for _ in 0..focus_line_row {
            if let Some(line) = upward_printer.next() {
                lines.push(line?);
            }
        }
        lines.reverse();
        let focus_line_index = lines.len();
        lines.push(center_line);
        for _ in (focus_line_row + 1)..size.height {
            if let Some(line) = downward_printer.next() {
                lines.push(line?);
            }
        }

        let focus_point = if options.set_focus && focus_is_shown {
            Some(Pos {
                row: focus_line_row,
                col: left_width,
            })
        } else {
            None
        };

        Ok(PrintedDoc {
            lines,
            focus_line_index,
            focus_line_row,
            focus_point,
            blank_style: root_style.to_owned(),
        })
    }

    /// The number of lines in the document.
    fn height(&self) -> Height {
        self.lines.len() as Height
    }

    /// The number of columns in the widest line of the document.
    fn width(&self) -> Width {
        self.lines
            .iter()
            .map(|line| line.width() as Width)
            .max()
            .unwrap_or(0)
    }

    /// Actually display the document to the PrettyWindow.
    fn display<W>(
        self,
        window: &mut W,
        rect: Rectangle,
    ) -> Result<(), PaneError<W::Error, D::Error>>
    where
        D: PrettyDoc<'d>,
        W: PrettyWindow<Style = D::Style>,
    {
        if let Some(focus_point) = self.focus_point {
            window
                .set_focus(Pos {
                    row: focus_point.row + rect.min_row,
                    col: focus_point.col + rect.min_col,
                })
                .map_err(PaneError::PrettyWindowError)?;
        }

        let first_row = self.focus_line_row - (self.focus_line_index as Row);
        let last_row = first_row + self.lines.len() as Row;
        for row in 0..rect.size().height {
            if row >= first_row && row < last_row {
                let line = &self.lines[(row - first_row) as usize];
                display_line(window, line, row, rect, &self.blank_style)?;
            } else {
                display_blank_line::<D, W>(window, row, rect, &self.blank_style)?;
            }
        }
        Ok(())
    }
}

struct LineWrapper<'d, D, I>
where
    D: PrettyDoc<'d>,
    I: Iterator<Item = Result<Line<'d, D>, PrintingError<D::Error>>>,
{
    iter: I,
    is_upward: bool,
    overflow_behavior: OverflowBehavior<D::Style>,
    pending_lines: Vec<Line<'d, D>>,
}

impl<'d, D, I> LineWrapper<'d, D, I>
where
    D: PrettyDoc<'d>,
    I: Iterator<Item = Result<Line<'d, D>, PrintingError<D::Error>>>,
{
    fn new(
        iter: I,
        pane_width: Width,
        overflow_behavior: OverflowBehavior<D::Style>,
        is_upward: bool,
    ) -> LineWrapper<'d, D, I> {
        use OverflowBehavior::{Clip, Wrap};

        // Never print outside of the pane.
        let overflow_behavior = match overflow_behavior {
            Clip(string, style, width) => {
                let width = width.min(pane_width);
                if width < str_width(string) + 2 {
                    // Can't fit a single full-width character before the marker. Give up and clip
                    // without a marker.
                    Clip("", style, width)
                } else {
                    Clip(string, style, width)
                }
            }
            Wrap(string, style, width) => {
                let width = width.min(pane_width);
                if width < str_width(string) + 2 {
                    // Can't fit a single full-width character after the marker. Give up and clip.
                    Clip("", style, width)
                } else {
                    Wrap(string, style, width)
                }
            }
        };

        LineWrapper {
            iter,
            overflow_behavior,
            is_upward,
            pending_lines: Vec::new(),
        }
    }
}

impl<'d, D, I> Iterator for LineWrapper<'d, D, I>
where
    D: PrettyDoc<'d>,
    I: Iterator<Item = Result<Line<'d, D>, PrintingError<D::Error>>>,
{
    type Item = Result<Line<'d, D>, PrintingError<D::Error>>;

    fn next(&mut self) -> Option<Result<Line<'d, D>, PrintingError<D::Error>>> {
        if let Some(line) = self.pending_lines.pop() {
            Some(Ok(line))
        } else {
            let long_line: Line<'d, D> = match self.iter.next() {
                None => return None,
                Some(Err(err)) => return Some(Err(err)),
                Some(Ok(long_line)) => long_line,
            };
            let mut lines = self.overflow_behavior.wrap_line(long_line);
            if !self.is_upward {
                lines.reverse();
            }
            self.pending_lines = lines;
            self.pending_lines.pop().map(Ok)
        }
    }
}

/// Display a blank line in the given window, at the given row relative to the `rect`.
/// Does not display anything that falls outside of the `rect`.
fn display_blank_line<'d, D, W>(
    window: &mut W,
    relative_row: Row,
    rect: Rectangle,
    blank_style: &D::Style,
) -> Result<(), PaneError<W::Error, D::Error>>
where
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style>,
{
    // Compute row in absolute window coords
    let absolute_row = rect.min_row + relative_row;
    if absolute_row >= rect.max_row {
        return Ok(());
    }

    for absolute_col in rect.min_col..rect.max_col {
        let absolute_pos = Pos {
            row: absolute_row,
            col: absolute_col,
        };
        window
            .display_char(' ', absolute_pos, blank_style, false)
            .map_err(PaneError::PrettyWindowError)?;
    }
    Ok(())
}

/// Display the [`Line`] in the given window, at the given row relative to the `rect`.
/// Does not display anything that falls outside of the `rect`.
fn display_line<'d, D, W>(
    window: &mut W,
    line: &Line<'d, D>,
    relative_row: Row,
    rect: Rectangle,
    blank_style: &D::Style,
) -> Result<(), PaneError<W::Error, D::Error>>
where
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style>,
{
    // Compute pos in absolute window coords
    let mut pos = Pos {
        row: rect.min_row + relative_row,
        col: rect.min_col,
    };
    if pos.row >= rect.max_row {
        return Ok(());
    }

    // Display each segment
    'segments_loop: for segment in &line.segments {
        for ch in segment.str.chars() {
            let is_full_width = is_char_full_width(ch);
            let char_width = if is_full_width { 2 } else { 1 };
            if pos.col + char_width > rect.max_col {
                // This should never happen, but clip just in case.
                break 'segments_loop;
            }
            window
                .display_char(ch, pos, &segment.style, is_full_width)
                .map_err(PaneError::PrettyWindowError)?;
            pos.col += char_width;
        }
    }
    while pos.col < rect.max_col {
        window
            .display_char(' ', pos, blank_style, false)
            .map_err(PaneError::PrettyWindowError)?;
        pos.col += 1;
    }
    Ok(())
}
