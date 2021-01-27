use super::pane_notation::{PaneNotation, PaneSize};
use super::pretty_window::PrettyWindow;
use crate::geometry::{Height, Line, Pos, Rectangle, Width};
use crate::pretty_printing::LineContents;
use crate::style::Style;
use std::hash::Hash;

/// A rectangular area of a window. You can pretty-print to it, or get sub-panes
/// of it and pretty-print to those.
pub struct Pane<'w, W>
where
    W: PrettyWindow,
{
    pub(crate) window: &'w mut W,
    pub(crate) rect: Rectangle,
}

/// Errors that can occur while attempting to render to a `Pane`.
#[derive(thiserror::Error, Debug)]
pub enum PaneError<W: PrettyWindow> {
    #[error("requested pane is not a subpane of the current pane")]
    NotSubPane,

    #[error("pane notation layout demands cannot be satisfied")]
    ImpossibleDemands,

    #[error("invalid pane notation")]
    InvalidNotation,

    #[error("missing document in pane notation: {0}")]
    MissingLabel(String),

    #[error("window error: {0}")]
    PrettyWindowErr(#[source] W::Error),
}

impl<'w, W> Pane<'w, W>
where
    W: PrettyWindow,
{
    /// Get a new `Pane` representing only the given sub-region of this `Pane`.
    /// Returns `None` if `rect` is not fully contained within this `Pane`.
    /// `rect` is specified in the same absolute coordinate system as the full
    /// `PrettyWindow` (not specified relative to this `Pane`!).
    fn sub_pane(&mut self, rect: Rectangle) -> Option<Pane<'_, W>> {
        if !self.rect.covers(rect) {
            return None;
        }
        Some(Pane {
            window: self.window,
            rect,
        })
    }

    fn print_line(&mut self, pos: Pos, contents: LineContents) -> Result<(), W::Error> {
        let mut pos = pos;
        pos.col += Width(contents.spaces as u16);
        for (string, style) in contents.contents {
            self.print(pos, string, style)?;
            pos.col += Width(string.chars().count() as u16);
        }
        Ok(())
    }

    fn print(&mut self, pos: Pos, string: &str, style: Style) -> Result<(), W::Error> {
        if pos.col >= self.rect.max_col {
            // Trying to print outside the pane.
            return Ok(());
        }
        let max_len = (self.rect.max_col - pos.col).0 as usize;
        if string.chars().count() > max_len {
            let (last_index, last_char) = string.char_indices().take(max_len).last().unwrap();
            let end_index = last_index + last_char.len_utf8();
            let truncated_string = &string[0..end_index];
            self.window.print(pos, truncated_string, style)
        } else {
            self.window.print(pos, string, style)
        }
    }

    fn fill(&mut self, pos: Pos, ch: char, style: Style) -> Result<(), W::Error> {
        if pos.col >= self.rect.max_col {
            // Trying to print outside the pane.
            return Ok(());
        }
        let len = (self.rect.max_col - pos.col).0 as usize;
        self.window.fill(pos, ch, len, style)
    }

    /// Render to this pane according to the given [PaneNotation]. Use the `get_content` closure to
    /// map the document labels used in any `PaneNotation::Doc` variants to actual documents.
    pub fn render<L: Copy + Eq + Hash, D>(
        &mut self,
        note: &PaneNotation<L>,
        get_content: impl Fn(L) -> Option<D>,
    ) -> Result<(), PaneError<W>> {
        match note {
            PaneNotation::Fill { ch, style } => {
                for line in 0..self.rect.height().0 {
                    let line = Line(line);
                    let col = self.rect.min_col;
                    self.fill(Pos { line, col }, *ch, *style)
                        .map_err(PaneError::PrettyWindowErr)?;
                }
            }
            PaneNotation::Empty => (),
            _ => unimplemented!(),
        };
        Ok(())
    }
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
