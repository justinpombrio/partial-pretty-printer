use super::pane_notation::{Label, PaneNotation, PaneSize};
use super::pretty_window::PrettyWindow;
use crate::geometry::{is_char_full_width, Height, Pos, Rectangle, Width};
use crate::pretty_printing::{pretty_print, LineContents, PrettyDoc, PrintingError};

/// A list of child indices describing the path from the root to a node in the document.
pub type Path = Vec<usize>;

/// Errors that can occur while attempting to render to a `Pane`.
#[derive(thiserror::Error, Debug)]
pub enum PaneError<W: PrettyWindow> {
    #[error(
        "invalid pane notation: PaneSize::Dyanmic may only be used in a PaneNotation::Doc pane"
    )]
    InvalidUseOfDynamic,
    #[error("missing document in pane notation: {0}")]
    MissingLabel(String),

    #[error("Window error: {0}")]
    PrettyWindowError(#[source] W::Error),

    #[error("Printing error: {0}")]
    PrintingError(#[from] PrintingError),
}

/// Render to this pane according to the given [PaneNotation].
///
/// - `window` is the `PrettyWindow` to display to.
/// - `notation` is the `PaneNotation` to render. It says how to break up the screen into rectangular
///   "panes", and which document to display in each pane. It does not contain the Documents
///   directly, instead it references them by `Label`.
/// - `get_content` is a function to look up a document by label. It returns both the document, and
///   the path to the node in the document to focus on.
pub fn pane_print<'d, L, D, W>(
    window: &mut W,
    notation: &PaneNotation<L, D::Style>,
    get_content: &impl Fn(L) -> Option<(D, Path)>,
) -> Result<(), PaneError<W>>
where
    L: Label,
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style, Mark = D::Mark>,
{
    let size = window.size().map_err(PaneError::PrettyWindowError)?;
    let rect = Rectangle::from_size(size);
    pane_print_rec(window, notation, get_content, rect)
}

type DynSizeFn<'l, W> = Box<dyn FnOnce(usize) -> Result<usize, PaneError<W>> + 'l>;

fn pane_print_rec<'d, L, D, W>(
    window: &mut W,
    notation: &PaneNotation<L, D::Style>,
    get_content: &impl Fn(L) -> Option<(D, Path)>,
    rect: Rectangle,
) -> Result<(), PaneError<W>>
where
    L: Label,
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style, Mark = D::Mark>,
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
                        .print_char(*ch, Pos { row, col }, None, style, is_full_width)
                        .map_err(PaneError::PrettyWindowError)?;
                    col += char_width;
                }
            }
        }
        PaneNotation::Doc {
            label,
            render_options,
        } => {
            let (doc, path) = get_content(label.clone())
                .ok_or_else(|| PaneError::MissingLabel(format!("{:?}", label)))?;
            let doc_width = render_options.choose_width(rect.width());
            let focal_line = render_options.focal_line(rect.height());
            let (mut upward_printer, mut downward_printer) = pretty_print(doc, doc_width, &path)?;
            for row in (0..focal_line).into_iter().rev() {
                if let Some(contents) = upward_printer.next() {
                    print_line_contents(window, contents?, Pos { row, col: 0 }, rect)?;
                } else {
                    break;
                }
            }
            for row in focal_line..rect.height() {
                if let Some(contents) = downward_printer.next() {
                    print_line_contents(window, contents?, Pos { row, col: 0 }, rect)?;
                } else {
                    break;
                }
            }
        }
        PaneNotation::Horz(panes) => {
            let mut dynamic_sizes: Vec<DynSizeFn<W>> = vec![];
            for (size, notation) in panes {
                if let PaneSize::Dynamic = size {
                    let label = if let PaneNotation::Doc { label, .. } = notation {
                        label.clone()
                    } else {
                        return Err(PaneError::InvalidUseOfDynamic);
                    };
                    let height = rect.height();
                    let func = move |available_width: usize| {
                        let (doc, path) = get_content(label.clone())
                            .ok_or_else(|| PaneError::MissingLabel(format!("{:?}", label)))?;
                        Ok(doc_width(doc, &path, height, available_width as Width)? as usize)
                    };
                    dynamic_sizes.push(Box::new(func) as DynSizeFn<W>);
                }
            }

            let child_sizes = &panes.iter().map(|(size, _)| *size).collect::<Vec<_>>();
            let widths: Vec<_> = divvy(rect.width() as usize, &child_sizes, dynamic_sizes)?
                .into_iter()
                .map(|n| n as Width)
                .collect();

            // Split this pane's rectangle horizontally (a.k.a. vertical slices) into multiple subpanes.
            let mut col = rect.min_col;
            let rects = widths.into_iter().map(|width| {
                let old_col = col;
                col += width;
                Rectangle {
                    min_col: old_col,
                    max_col: col,
                    min_row: rect.min_row,
                    max_row: rect.max_row,
                }
            });

            let child_notes = &panes.iter().map(|(_, note)| note).collect::<Vec<_>>();
            for (child_rect, child_note) in rects.into_iter().zip(child_notes.iter()) {
                pane_print_rec(window, child_note, get_content, child_rect)?;
            }
        }
        PaneNotation::Vert(panes) => {
            let mut dynamic_sizes: Vec<DynSizeFn<W>> = vec![];
            for (size, notation) in panes {
                if let PaneSize::Dynamic = size {
                    let label = if let PaneNotation::Doc { label, .. } = notation {
                        label.clone()
                    } else {
                        return Err(PaneError::InvalidUseOfDynamic);
                    };
                    let width = rect.width();
                    let func = move |available_height: usize| {
                        let (doc, path) = get_content(label.clone())
                            .ok_or_else(|| PaneError::MissingLabel(format!("{:?}", label)))?;
                        Ok(doc_height(doc, &path, width, available_height as Height)? as usize)
                    };
                    dynamic_sizes.push(Box::new(func) as DynSizeFn<W>);
                }
            }

            let child_sizes = &panes.iter().map(|(size, _)| *size).collect::<Vec<_>>();
            let heights: Vec<_> = divvy(rect.height() as usize, &child_sizes, dynamic_sizes)?
                .into_iter()
                .map(|n| n as Height)
                .collect();

            // Split this pane's rectangle vertically (a.k.a. horizontal slices) into multiple subpanes.
            let mut row = rect.min_row;
            let pane_rect = rect;
            let rects = heights.into_iter().map(|height| {
                let old_row = row;
                row += height;
                Rectangle {
                    min_col: pane_rect.min_col,
                    max_col: pane_rect.max_col,
                    min_row: old_row,
                    max_row: row,
                }
            });

            let child_notes = &panes.iter().map(|(_, note)| note).collect::<Vec<_>>();
            for (child_rect, child_note) in rects.zip(child_notes.iter()) {
                pane_print_rec(window, child_note, get_content, child_rect)?;
            }
        }
    }
    Ok(())
}

/// Determine how many lines a document would take to print. If it would take more than
/// `max_height` lines, stops and returns `max_height` instead.
fn doc_height<'d>(
    doc: impl PrettyDoc<'d>,
    path: &[usize],
    width: Width,
    max_height: Height,
) -> Result<Height, PrintingError> {
    let (_, downward_printer) = pretty_print(doc, width, path)?;
    Ok(downward_printer.take(max_height as usize).count() as Height)
}

/// Determine how many columns a document would take to print, if given `height` lines. If it would
/// use more than `max_width` columns, returns `max_width` instead.
fn doc_width<'d>(
    doc: impl PrettyDoc<'d>,
    path: &[usize],
    height: Height,
    max_width: Width,
) -> Result<Width, PrintingError> {
    let (_, downward_printer) = pretty_print(doc, max_width, path)?;
    let lines = downward_printer.take(height as usize);
    let mut width = 0;
    for line in lines {
        let line = line?;
        width = width.max(line.width());
    }
    Ok(width)
}

/// Displays LineContents in the given window, at the given position relative to the rect.
/// Does not display anything that falls outside of the rect.
fn print_line_contents<'d, D, W>(
    window: &mut W,
    contents: LineContents<'d, D>,
    relative_pos: Pos,
    rect: Rectangle,
) -> Result<(), PaneError<W>>
where
    D: PrettyDoc<'d>,
    W: PrettyWindow<Style = D::Style, Mark = D::Mark>,
{
    // Compute pos in absolute window coords
    let mut pos = Pos {
        row: rect.min_row + relative_pos.row,
        col: rect.min_col + relative_pos.col,
    };
    if pos.row >= rect.max_row {
        return Ok(());
    }

    // Print indentation
    for _ in 0..contents.indentation.num_spaces {
        if pos.col >= rect.max_col {
            return Ok(());
        }
        let mark = contents.indentation.mark;
        window
            .print_char(' ', pos, mark, &D::Style::default(), false)
            .map_err(PaneError::PrettyWindowError)?;
        pos.col += 1;
    }

    // Print each piece
    for piece in contents.pieces {
        for ch in piece.str.chars() {
            let is_full_width = is_char_full_width(ch);
            let char_width = if is_full_width { 2 } else { 1 };
            if pos.col + char_width > rect.max_col {
                return Ok(());
            }
            window
                .print_char(ch, pos, piece.mark, piece.style, is_full_width)
                .map_err(PaneError::PrettyWindowError)?;
            pos.col += char_width;
        }
    }
    Ok(())
}

/// Divvy space ("cookies") up among various PaneSize demands. Some demands are fixed size, and
/// some are proprortional: first satisfy all of the fixed demands, then allocate the rest of the
/// space proportionally. If there is not enough space to satisfy the fixed demands, it's
/// first-come first-served among the fixed demands and the proportional demands get nothing.
fn divvy<'d, W: PrettyWindow>(
    cookies: usize,
    demands: &[PaneSize],
    dynamic_sizes: Vec<DynSizeFn<W>>,
) -> Result<Vec<usize>, PaneError<W>> {
    // Allocate cookies for all the fixed demands.
    let fixed_demands = demands
        .iter()
        .filter_map(|demand| demand.get_fixed())
        .collect::<Vec<_>>();
    let (fixed_allocation, mut cookies) = fixedly_divide(cookies, &fixed_demands);
    let mut fixed_allocation = fixed_allocation.into_iter();

    // Allocate remaining cookies among the dynamic demands.
    let mut dynamic_allocation = vec![];
    for func in dynamic_sizes {
        let eaten_cookies = func(cookies)?;
        cookies -= eaten_cookies;
        dynamic_allocation.push(eaten_cookies);
    }
    let mut dynamic_allocation = dynamic_allocation.into_iter();

    // Allocate remaining cookies among the proportional demands.
    let proportional_demands = demands
        .iter()
        .filter_map(|demand| demand.get_proportional())
        .collect::<Vec<_>>();
    let mut proportional_allocation =
        proportionally_divide(cookies, &proportional_demands).into_iter();

    // And finally merge the two allocations
    Ok(demands
        .iter()
        .map(|demand| match demand {
            PaneSize::Fixed(_) => fixed_allocation.next().expect("bug in divvy (fixed)"),
            PaneSize::Proportional(_) => proportional_allocation
                .next()
                .expect("bug in divvy (proportional)"),
            PaneSize::Dynamic => dynamic_allocation.next().expect("bug in divvy (dynamic)"),
        })
        .collect::<Vec<_>>())
}

/// Divvy `cookies` up among children, where each child requires a fixed number of cookies,
/// returning the allocation and the number of remaining cookies. If there aren't enough cookies,
/// it's first-come first-serve.
fn fixedly_divide(cookies: usize, child_hungers: &[usize]) -> (Vec<usize>, usize) {
    let mut cookies = cookies;
    let cookie_allocation = child_hungers
        .iter()
        .map(|hunger| {
            let cookies_given = cookies.min(*hunger);
            cookies -= cookies_given;
            cookies_given
        })
        .collect::<Vec<_>>();
    (cookie_allocation, cookies)
}

/// Divvy `cookies` up among children as fairly as possible, where the `i`th
/// child has `child_hungers[i]` hunger. Children should receive cookies in proportion
/// to their hunger, with the difficulty that cookies cannot be split into
/// pieces. Exact ties go to the leftmost tied child.
fn proportionally_divide(cookies: usize, child_hungers: &[usize]) -> Vec<usize> {
    let total_hunger: usize = child_hungers.iter().sum();
    // Start by allocating each child a guaranteed minimum number of cookies,
    // found as the floor of the real number of cookies they deserve.
    let mut cookie_allocation: Vec<usize> = child_hungers
        .iter()
        .map(|hunger| cookies * hunger / total_hunger)
        .collect();
    // Compute the number of cookies still remaining.
    let allocated_cookies: usize = cookie_allocation.iter().sum();
    let leftover: usize = cookies - allocated_cookies;
    // Determine what fraction of a cookie each child still deserves, found as
    // the remainder of the above division. Then hand out the remaining cookies
    // to the children with the largest remainders.
    let mut remainders: Vec<(usize, usize)> = child_hungers
        .iter()
        .map(|hunger| cookies * hunger % total_hunger)
        .enumerate()
        .collect();
    remainders.sort_by(|(_, r1), (_, r2)| r2.cmp(r1));
    remainders
        .into_iter()
        .take(leftover)
        .for_each(|(i, _)| cookie_allocation[i] += 1);
    // Return the maximally-fair cookie allocation.
    cookie_allocation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proportional_division() {
        assert_eq!(proportionally_divide(0, &[1, 1]), vec!(0, 0));
        assert_eq!(proportionally_divide(1, &[1, 1]), vec!(1, 0));
        assert_eq!(proportionally_divide(2, &[1, 1]), vec!(1, 1));
        assert_eq!(proportionally_divide(3, &[1, 1]), vec!(2, 1));
        assert_eq!(proportionally_divide(4, &[10, 11, 12]), vec!(1, 1, 2));
        assert_eq!(proportionally_divide(5, &[17]), vec!(5));
        assert_eq!(proportionally_divide(5, &[12, 10, 11]), vec!(2, 1, 2));
        assert_eq!(proportionally_divide(5, &[10, 10, 11]), vec!(2, 1, 2));
        assert_eq!(proportionally_divide(5, &[2, 0, 1]), vec!(3, 0, 2));
        assert_eq!(proportionally_divide(61, &[1, 2, 3]), vec!(10, 20, 31));
        assert_eq!(
            proportionally_divide(34583, &[55, 98, 55, 7, 12, 200]),
            vec!(4455, 7937, 4454, 567, 972, 16198)
        );
    }
}
