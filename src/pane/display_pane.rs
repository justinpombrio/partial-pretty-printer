use crate::{
    geometry::{is_char_full_width, Rectangle},
    pane::{divvy::Divvier, DocLabel, PaneNotation, PaneSize, PrettyWindow, PrintingOptions},
    pretty_print, Height, Line, Pos, PrettyDoc, PrintingError, Row, Size, Width,
};

/// Errors that can occur while displaying a pane.
#[derive(thiserror::Error, Debug)]
pub enum PaneError<W: PrettyWindow, E: std::error::Error + 'static> {
    #[error(
        "Invalid pane notation: PaneSize::Dyanmic may only be used in a PaneNotation::Doc pane"
    )]
    InvalidUseOfDynamic,

    #[error("No document found for label in PaneNotation::Doc: {0}")]
    MissingDocument(String),

    #[error("PrettyWindow error: {0}")]
    PrettyWindowError(#[source] W::Error),

    #[error("PrettyDoc printing error: {0}")]
    PrintingError(#[from] PrintingError<E>),
}

/// Display a [`PaneNotation`] to a [`PrettyWindow`].
///
/// `get_content` is a function to look up a document by [`DocLabel`]. It returns both the document
/// and [extra information](PrintingOptions) about how to print it.
pub fn display_pane<'d, L, D, W>(
    window: &mut W,
    notation: &PaneNotation<L, D::Style>,
    get_content: &impl Fn(L) -> Option<(D, PrintingOptions)>,
) -> Result<(), PaneError<W, D::Error>>
where
    L: DocLabel,
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style>,
{
    let size = window.size().map_err(PaneError::PrettyWindowError)?;
    let rect = Rectangle::from_size(size);
    display_pane_rec(window, notation, get_content, rect)
}

fn display_pane_rec<'d, L, D, W>(
    window: &mut W,
    notation: &PaneNotation<L, D::Style>,
    get_content: &impl Fn(L) -> Option<(D, PrintingOptions)>,
    rect: Rectangle,
) -> Result<(), PaneError<W, D::Error>>
where
    L: DocLabel,
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style>,
{
    match notation {
        PaneNotation::Empty => (),
        PaneNotation::Fill { ch, style } => {
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
            }
        }
        PaneNotation::Doc { label } => {
            let (doc, options) = get_content(label.clone())
                .ok_or_else(|| PaneError::MissingDocument(format!("{:?}", label)))?;
            let printed_doc = PrintedDoc::new(doc, &options, rect.size())?;
            printed_doc.display(window, rect)?;
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
                let label = if let PaneNotation::Doc { label } = child_note {
                    label.clone()
                } else {
                    return Err(PaneError::InvalidUseOfDynamic);
                };
                let (doc, options) = get_content(label.clone())
                    .ok_or_else(|| PaneError::MissingDocument(format!("{:?}", label)))?;

                let printed_doc = PrintedDoc::new(doc, &options, available_size)?;
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
                    display_pane_rec(window, child_note, get_content, child_rect)?;
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
                let label = if let PaneNotation::Doc { label } = child_note {
                    label.clone()
                } else {
                    return Err(PaneError::InvalidUseOfDynamic);
                };
                let (doc, options) = get_content(label.clone())
                    .ok_or_else(|| PaneError::MissingDocument(format!("{:?}", label)))?;

                let printed_doc = PrintedDoc::new(doc, &options, available_size)?;
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
                    display_pane_rec(window, child_note, get_content, child_rect)?;
                }
            }
        }
    }
    Ok(())
}

struct PrintedDoc<'d, D: PrettyDoc<'d>> {
    lines: Vec<Line<'d, D>>,
    /// Which line in `lines` is the focus line.
    focus_line_index: usize,
    /// Which row of the pane should the focus line be displayed on.
    focus_line_row: Row,
    /// Focus point of the document, relative to the pane.
    focus_point: Option<Pos>,
}

impl<'d, D: PrettyDoc<'d>> PrintedDoc<'d, D> {
    /// Pretty-print the portion of document that would fit in the given `size`,
    /// storing it as text in the `PrintedDoc`.
    fn new(doc: D, options: &PrintingOptions, size: Size) -> Result<Self, PrintingError<D::Error>> {
        if size.height == 0 || size.width == 0 {
            return Ok(PrintedDoc {
                lines: Vec::new(),
                focus_line_index: 0,
                focus_line_row: 0,
                focus_point: None,
            });
        }

        let printing_width = options.choose_width(size.width);
        let focus_line_row = options.choose_focus_line_row(size.height);
        let (mut upward_printer, focused_line, mut downward_printer) = pretty_print(
            doc,
            printing_width,
            &options.focus_path,
            options.focus_target,
        )?;

        let focus_point = if options.set_focus {
            Some(Pos {
                row: focus_line_row,
                col: focused_line.left_width(),
            })
        } else {
            None
        };

        let mut lines = Vec::new();
        for _ in 0..focus_line_row {
            if let Some(line) = upward_printer.next() {
                lines.push(line?);
            }
        }
        lines.reverse();
        let focus_line_index = lines.len();
        lines.push(Line::from(focused_line));
        for _ in (focus_line_row + 1)..size.height {
            if let Some(line) = downward_printer.next() {
                lines.push(line?);
            }
        }

        Ok(PrintedDoc {
            lines,
            focus_line_index,
            focus_line_row,
            focus_point,
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
    fn display<W>(self, window: &mut W, rect: Rectangle) -> Result<(), PaneError<W, D::Error>>
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
        let mut relative_pos = Pos {
            col: 0,
            row: self.focus_line_row - (self.focus_line_index as Row),
        };
        for line in self.lines {
            display_line(window, line, relative_pos, rect)?;
            relative_pos.row += 1;
        }
        Ok(())
    }
}

/// Display the [`Line`] in the given window, at the given position relative to the `rect`.
/// Does not display anything that falls outside of the `rect`.
fn display_line<'d, D, W>(
    window: &mut W,
    line: Line<'d, D>,
    relative_pos: Pos,
    rect: Rectangle,
) -> Result<(), PaneError<W, D::Error>>
where
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style>,
{
    // Compute pos in absolute window coords
    let mut pos = Pos {
        row: rect.min_row + relative_pos.row,
        col: rect.min_col + relative_pos.col,
    };
    if pos.row >= rect.max_row {
        return Ok(());
    }

    // Display each segment
    for segment in line.segments {
        for ch in segment.str.chars() {
            let is_full_width = is_char_full_width(ch);
            let char_width = if is_full_width { 2 } else { 1 };
            if pos.col + char_width > rect.max_col {
                return Ok(());
            }
            window
                .display_char(ch, pos, &segment.style, is_full_width)
                .map_err(PaneError::PrettyWindowError)?;
            pos.col += char_width;
        }
    }
    Ok(())
}
