//! Print to multiple rectangular sub-panes of a window.
//!
//! This module performs "pane printing": a way to split a window into multiple rectangular panes,
//! each of which can display a pretty-printed document (or be filled with a repeated character).
//! This is primarily meant for implementing terminal UIs.
//!
//! Pane printing involves printing a [`PaneNotation`] into a [`PrettyWindow`], using the
//! [`pane_print()`] function.
//!
//! The [`PaneNotation`] says how to divide a window into multiple rectangular panes. For example,
//! it could say to show two different documents side-by-side.
//!
//! A [`PrettyWindow`] knows how to display a styled character at a position. This library only
//! supplies a simple implementation of the [`PrettyWindow`] trait called [`PlainText`], which
//! ignores style metadata and prints to a string. If you use pane printing, you will probably want
//! to provide your own implementation of [`PrettyWindow`] for whatever medium you want to display
//! to (like a terminal window).

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
