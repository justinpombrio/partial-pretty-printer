use std::fmt;

/// A label that can be used to look up a document.
pub trait DocLabel: fmt::Debug + Clone {}
impl<T: fmt::Debug + Clone> DocLabel for T {}

/// Specify the content of a pane.
#[derive(Clone, Debug)]
pub enum PaneNotation<L: DocLabel, S> {
    /// Split the pane into multiple subpanes from left to right, each with its own `PaneNotation`.
    /// Each subpane has the same height as this pane, and a width determined by its [`PaneSize`].
    Horz(Vec<(PaneSize, PaneNotation<L, S>)>),
    /// Split the pane into multiple subpanes from top to bottom, each with its own `PaneNotation`.
    /// Each subpane has the same width as this pane, and a height determined by its [`PaneSize`].
    Vert(Vec<(PaneSize, PaneNotation<L, S>)>),
    /// Pretty print a document and display it in this pane. The given [`DocLabel`] will be used to dynamically look up
    /// the [`PrettyDoc`](crate::PrettyDoc) when the pane is displayed.
    Doc { label: L },
    /// Fill the entire pane by repeating the given character with the given style.
    Fill { ch: char, style: S },
    /// Leave the pane empty.
    Empty,
}

/// Specify the size of a subpane within a vertically ([`PaneNotation::Vert`]) or horizontally
/// ([`PaneNotation::Horz`]) concatenated list of subpanes. Space is divvied up among all the panes
/// in a `Vert` or `Horz` in this priority order:
///
/// 1. `Fixed`
/// 2. `Dynamic`
/// 3. `Proportional`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneSize {
    /// Give the subpane exactly this number of rows of height (for [`PaneNotation::Vert`]) or
    /// columns of width (for [`PaneNotation::Horz`]).
    Fixed(usize),

    /// Try to give the subpane exactly the amount of height or width needed to fit its content. If
    /// that's not possible, give it all of the remaining height or width. Note that documents are
    /// typically very greedy for width, so you should only use `Dynamic` width in unusual
    /// circumstances.
    ///
    /// If there are multiple `Dynamic` subpanes and not enough space to satisfy all of them, the
    /// ones earlier in the list get priority. `Dynamic` subpanes get priority over `Proportional`
    /// subpanes, regardless of order.
    ///
    /// A `Dynamic` subpane can only contain a [`PaneNotation::Doc`], not more nested subpanes.
    Dynamic,

    /// After `Fixed` and `Dynamic` subpanes have been assigned a width/height, divide up the
    /// remaining available width/height between the `Proportional` subpanes according to their
    /// given weights. For example, a subpane with weight 2 will be twice as large as one with
    /// weight 1.
    Proportional(usize),
}
