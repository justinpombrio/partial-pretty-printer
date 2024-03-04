//! Print to multiple rectangular sub-panes of a window.
//!
//! This module performs "pane printing": a way to split a window into multiple rectangular
//! panes, each of which can display a document via pretty printing. This is primarily meant
//! for implementing a terminal UI.
//!
//! Pane printing involves rendering a _pane notation_ onto a _window_.
//!
//! The [`PaneNotation`] says how to divide a window into multiple rectangular panes. For example,
//! it could say to show two different documents side by side.
//!
//! A [`PrettyWindow`] knows how to render styled strings. This library only supplies a simple
//! implementation of `PrettyWindow` called [`PlainText`], which ignores styling and prints to
//! a string. If you use pane printing, you will probably want to provide your own
//! implementation of `PrettyWindow`, for whatever medium you want to display to.
//!
//! With a pane notation and window, you can _pane print_:
//!
//! ```ignore
//! pub fn pane_print<'d, L, D, W>(
//!     window: &mut W,
//!     notation: &PaneNotation<L, D::Style>,
//!     get_content: &impl Fn(L) -> Option<(D, RenderOptions)>,
//! ) -> Result<(), PaneError<W>>
//! where
//!     L: DocLabel,
//!     D: PrettyDoc<'d>,
//!     W: PrettyWindow<Style = D::Style>;
//! ```
//!
//! - `window` is the `PrettyWindow` to display to.
//! - `notation` is the `PaneNotation` to render. It says how to break up
//!   the screen into rectangular "panes", and which document to display in
//!   each pane. It does not contain the Documents directly, instead it
//!   references them by `DocLabel`.
//! - `get_content()` is a function to look up a document by label. It
//!   returns both the document, and information about how to render it.

mod divvy;
mod pane_notation;
mod pane_print;
mod plain_text;
mod pretty_window;
mod render_options;

pub use pane_notation::{DocLabel, PaneNotation, PaneSize};
pub use pane_print::{pane_print, PaneError};
pub use plain_text::PlainText;
pub use pretty_window::PrettyWindow;
pub use render_options::{FocusSide, RenderOptions, WidthStrategy};
