use super::pane::{Pane, PaneError};
use super::pane_notation::{Label, PaneNotation, PaneSize};
use super::pretty_window::PrettyWindow;
use crate::geometry::{Height, Pos, Width};
use crate::pretty_printing::{pretty_print, PrettyDoc};
use crate::style::{Shade, ShadedStyle};

/// A list of child indices, describing the path from the root to a node in the document.
pub type Path = Vec<usize>;

/// Render to this pane according to the given [PaneNotation].
///
/// - `window` is the `PrettyWindow` to display to.
/// - `note` is the `PaneNotation` to render. It says how to break up the screen into rectangular
///   "panes", and which document to display in each pane. It does not contain the Documents
///   directly, instead it references them by `Label`.
/// - `get_content` is a function to look up a document by label. It returns both the document, and
///   the path to the node in the document to focus on.
pub fn pane_print<'d, L: Label, D: PrettyDoc<'d>, W: PrettyWindow>(
    window: &mut W,
    note: &PaneNotation<L>,
    get_content: &impl Fn(L) -> Option<(D, Path)>,
) -> Result<(), PaneError<W>> {
    let mut pane = Pane::new(window)?;
    pane_print_rec(&mut pane, note, get_content)
}

fn pane_print_rec<'d, L: Label, D: PrettyDoc<'d>, W: PrettyWindow>(
    pane: &mut Pane<W>,
    note: &PaneNotation<L>,
    get_content: &impl Fn(L) -> Option<(D, Path)>,
) -> Result<(), PaneError<W>> {
    match note {
        PaneNotation::Empty => (),
        PaneNotation::Fill { ch, style } => {
            for line in pane.rect.min_line..pane.rect.max_line {
                let col = pane.rect.min_col;
                let shaded_style = ShadedStyle::new(*style, Shade::background());
                let len = pane.rect.max_col - col;
                pane.fill(Pos { line, col }, *ch, len, shaded_style)?;
            }
        }
        PaneNotation::Doc {
            label,
            render_options,
        } => {
            let (doc, path) = get_content(label.clone())
                .ok_or_else(|| PaneError::MissingLabel(format!("{:?}", label)))?;
            let doc_width = render_options.choose_width(pane.rect.width());
            let focal_line = render_options.focal_line(pane.rect.height());
            let (mut upward_printer, mut downward_printer) = pretty_print(doc, doc_width, &path);
            let highlight_cursor = render_options.highlight_cursor;
            for line in (0..focal_line).into_iter().rev() {
                if let Some(contents) = upward_printer.next() {
                    pane.print_line(line, contents, highlight_cursor)?;
                } else {
                    break;
                }
            }
            for line in focal_line..pane.rect.height() {
                if let Some(contents) = downward_printer.next() {
                    pane.print_line(line, contents, highlight_cursor)?;
                } else {
                    break;
                }
            }
        }
        PaneNotation::Horz(panes) => {
            let child_notes: Vec<_> = panes.iter().map(|(_, note)| note).collect();
            let total_fixed: usize = panes.iter().filter_map(|(size, _)| size.get_fixed()).sum();
            let total_width = pane.rect.width() as usize;
            let mut available_width = total_width.saturating_sub(total_fixed);
            let child_sizes = panes
                .iter()
                .map(|(size, notation)| match *size {
                    PaneSize::Dynamic => {
                        // Convert dynamic width into a fixed width, based on the currrent document.
                        if let PaneNotation::Doc { label, .. } = notation {
                            let (doc, path) = get_content(label.clone())
                                .ok_or_else(|| PaneError::MissingLabel(format!("{:?}", label)))?;
                            let width =
                                doc_width(doc, &path, pane.rect.height(), available_width as Width);
                            let width = width as usize;
                            available_width -= width;
                            Ok(PaneSize::Fixed(width))
                        } else {
                            // Dynamic may only be used on Doc subpanes!
                            Err(PaneError::InvalidNotation)
                        }
                    }
                    size => Ok(size), // pass through all other pane sizes
                })
                .collect::<Result<Vec<_>, _>>()?;

            let widths: Vec<_> = divvy(total_width, &child_sizes)
                .into_iter()
                .map(|n| n as Width)
                .collect();

            let rects = pane.rect.horz_splits(&widths);
            for (rect, child_note) in rects.zip(child_notes.into_iter()) {
                let mut child_pane = pane.sub_pane(rect).ok_or(PaneError::NotSubPane)?;
                pane_print_rec(&mut child_pane, child_note, get_content)?;
            }
        }
        PaneNotation::Vert(panes) => {
            let child_notes: Vec<_> = panes.iter().map(|(_, note)| note).collect();
            let total_fixed: usize = panes.iter().filter_map(|(size, _)| size.get_fixed()).sum();
            let total_height = pane.rect.height() as usize;
            let mut available_height = total_height.saturating_sub(total_fixed);
            let child_sizes = panes
                .iter()
                .map(|(size, notation)| match *size {
                    PaneSize::Dynamic => {
                        // Convert dynamic height into a fixed height, based on the currrent document.
                        if let PaneNotation::Doc { label, .. } = notation {
                            let (doc, path) = get_content(label.clone())
                                .ok_or_else(|| PaneError::MissingLabel(format!("{:?}", label)))?;
                            let height = doc_height(
                                doc,
                                &path,
                                pane.rect.width(),
                                available_height as Height,
                            );
                            let height = height as usize;
                            available_height -= height;
                            Ok(PaneSize::Fixed(height))
                        } else {
                            // Dynamic may only be used on Doc subpanes!
                            Err(PaneError::InvalidNotation)
                        }
                    }
                    size => Ok(size), // pass through all other pane sizes
                })
                .collect::<Result<Vec<_>, _>>()?;

            let heights: Vec<_> = divvy(total_height, &child_sizes)
                .into_iter()
                .map(|n| n as Height)
                .collect();

            let rects = pane.rect.vert_splits(&heights);
            for (rect, child_note) in rects.zip(child_notes.into_iter()) {
                let mut child_pane = pane.sub_pane(rect).ok_or(PaneError::NotSubPane)?;
                pane_print_rec(&mut child_pane, child_note, get_content)?;
            }
        }
    };
    Ok(())
}

/// Determine how many lines a document would take to print. If it would take more than
/// `max_height` lines, stops and returns `max_height` instead.
fn doc_height<'d>(
    doc: impl PrettyDoc<'d>,
    path: &[usize],
    width: Width,
    max_height: Height,
) -> Height {
    let (_, downward_printer) = pretty_print(doc, width, path);
    downward_printer.take(max_height as usize).count() as Height
}

/// Determine how many columns a document would take to print, if given `height` lines. If it would
/// use more than `max_width` columns, returns `max_width` instead.
fn doc_width<'d>(
    doc: impl PrettyDoc<'d>,
    path: &[usize],
    height: Height,
    max_width: Width,
) -> Width {
    let (_, downward_printer) = pretty_print(doc, max_width, path);
    let lines = downward_printer.take(height as usize);
    let mut width = 0;
    for line in lines {
        width = width.max(line.to_string().chars().count() as Width);
    }
    width
}

/// Divvy space ("cookies") up among various PaneSize demands. Some demands are fixed size, and
/// some are proprortional: first satisfy all of the fixed demands, then allocate the rest of the
/// space proportionally. If there is not enough space to satisfy the fixed demands, it's
/// first-come first-served among the fixed demands and the proportional demands get nothing.
fn divvy(cookies: usize, demands: &[PaneSize]) -> Vec<usize> {
    // Allocate cookies for all the fixed demands.
    let fixed_demands = demands
        .iter()
        .filter_map(|demand| demand.get_fixed())
        .collect::<Vec<_>>();
    let (fixed_allocation, cookies) = fixedly_divide(cookies, &fixed_demands);
    let mut fixed_allocation = fixed_allocation.into_iter();

    // Now divvy up any remaining cookies among the proportional demands.
    let proportional_demands = demands
        .iter()
        .filter_map(|demand| demand.get_proportional())
        .collect::<Vec<_>>();
    let mut proportional_allocation =
        proportionally_divide(cookies, &proportional_demands).into_iter();

    // And finally merge the two allocations
    demands
        .iter()
        .map(|demand| match demand {
            PaneSize::Fixed(_) => fixed_allocation.next().expect("bug in divvy"),
            PaneSize::Proportional(_) => proportional_allocation.next().expect("bug in divvy"),
            PaneSize::Dynamic => {
                panic!("All Dynamic sizes should have been replaced by Fixed sizes by now!")
            }
        })
        .collect::<Vec<_>>()
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
