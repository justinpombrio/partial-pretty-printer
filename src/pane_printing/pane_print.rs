use super::pane::{Pane, PaneError};
use super::pane_notation::{PaneNotation, PaneSize};
use super::pretty_window::PrettyWindow;
use crate::geometry::{Height, Line, Pos, Width};
use crate::pretty_printing::{pretty_print, PrettyDoc};
use crate::style::{Shade, ShadedStyle};
use std::fmt;
use std::hash::Hash;

pub trait Label: Copy + Eq + Hash + fmt::Debug {}

/// A list of child indices, describing the path from the root to a node in the document.
pub type Path = Vec<usize>;

/// Render to this pane according to the given [PaneNotation]. Use the `get_content` closure to
/// map the document labels used in any `PaneNotation::Doc` variants to actual documents.
pub fn pane_print<L: Label, D: PrettyDoc, W: PrettyWindow>(
    mut window: W,
    note: &PaneNotation<L>,
    get_content: &impl Fn(L) -> Option<(D, Path)>,
) -> Result<(), PaneError<W>> {
    let mut pane = Pane::new(&mut window)?;
    pane_print_rec(&mut pane, note, get_content)
}

fn pane_print_rec<L: Label, D: PrettyDoc, W: PrettyWindow>(
    pane: &mut Pane<W>,
    note: &PaneNotation<L>,
    get_content: &impl Fn(L) -> Option<(D, Path)>,
) -> Result<(), PaneError<W>> {
    match note {
        PaneNotation::Empty => (),
        PaneNotation::Fill { ch, style } => {
            for line in 0..pane.rect.height().0 {
                let line = Line(line);
                let col = pane.rect.min_col;
                let shaded_style = ShadedStyle::new(*style, Shade::background());
                pane.fill(Pos { line, col }, *ch, shaded_style)?;
            }
        }
        PaneNotation::Doc {
            label,
            render_options,
        } => {
            let (doc, path) = get_content(*label)
                .ok_or_else(|| PaneError::MissingLabel(format!("{:?}", label)))?;
            let doc_width = render_options.width_strategy.choose(pane.rect.width()).0 as usize;
            let focal_line = render_options
                .scroll_strategy
                .focal_line(pane.rect.height());
            let (mut upward_printer, mut downward_printer) = pretty_print(&doc, doc_width, &path);
            let highlight_cursor = render_options.highlight_cursor;
            for line in (0..focal_line.0).into_iter().rev() {
                if let Some(contents) = upward_printer.next() {
                    pane.print_line(Line(line), contents, highlight_cursor)?;
                } else {
                    break;
                }
            }
            for line in focal_line.0..pane.rect.height().0 {
                if let Some(contents) = downward_printer.next() {
                    pane.print_line(Line(line), contents, highlight_cursor)?;
                } else {
                    break;
                }
            }
        }
        PaneNotation::Horz { panes } => {
            let child_notes: Vec<_> = panes.iter().map(|p| &p.1).collect();
            let child_sizes: Vec<_> = panes.iter().map(|p| p.0).collect();
            let total_width = pane.rect.width().0 as usize;
            let widths: Vec<_> = divvy(total_width, &child_sizes)
                .ok_or(PaneError::ImpossibleDemands)?
                .into_iter()
                .map(|n| Width(n as u16))
                .collect();
            let rects = pane.rect.horz_splits(&widths);
            for (rect, child_note) in rects.zip(child_notes.into_iter()) {
                let mut child_pane = pane.sub_pane(rect).ok_or(PaneError::NotSubPane)?;
                pane_print_rec(&mut child_pane, child_note, get_content)?;
            }
        }
        PaneNotation::Vert { panes } => {
            let child_notes: Vec<_> = panes.iter().map(|p| &p.1).collect();
            let total_fixed: usize = panes.iter().filter_map(|p| p.0.get_fixed()).sum();
            let total_height = pane.rect.height().0 as usize;
            let mut available_height = total_height.saturating_sub(total_fixed);
            let child_sizes = panes
                .iter()
                .map(|p| match p.0 {
                    PaneSize::DynHeight => {
                        // Convert dynamic height into a fixed height, based on the currrent document.
                        if let PaneNotation::Doc { label, .. } = &p.1 {
                            let f = get_content.clone();
                            let _doc = f(*label)
                                .ok_or_else(|| PaneError::MissingLabel(format!("{:?}", label)))?;
                            // TODO
                            unimplemented!();
                            let height = 0;
                            // let height = available_height.min(doc.required_height(pane.rect.width()));
                            available_height -= height;
                            Ok(PaneSize::Fixed(height))
                        } else {
                            // DynHeight is only implemented for Doc subpanes!
                            Err(PaneError::InvalidNotation)
                        }
                    }
                    size => Ok(size), // pass through all other pane sizes
                })
                .collect::<Result<Vec<_>, _>>()?;

            let heights: Vec<_> = divvy(total_height, &child_sizes)
                .ok_or(PaneError::ImpossibleDemands)?
                .into_iter()
                .map(|n| Height(n as u32))
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

fn divvy(cookies: usize, demands: &[PaneSize]) -> Option<Vec<usize>> {
    let total_fixed: usize = demands.iter().filter_map(|demand| demand.get_fixed()).sum();
    if total_fixed > cookies {
        return None; // Impossible to satisfy the demands!
    }

    let hungers: Vec<_> = demands
        .iter()
        .filter_map(|demand| demand.get_proportional())
        .collect();

    let mut proportional_allocation =
        proportionally_divide(cookies - total_fixed, &hungers).into_iter();

    Some(
        demands
            .iter()
            .map(|demand| match demand {
                PaneSize::Fixed(n) => *n,
                PaneSize::Proportional(_) => proportional_allocation.next().expect("bug in divvy"),
                PaneSize::DynHeight => {
                    panic!("All DynHeight sizes should have been replaced by Fixed sizes by now!")
                }
            })
            .collect(),
    )
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
