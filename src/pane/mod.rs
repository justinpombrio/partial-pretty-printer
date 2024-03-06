//! This module provides a way to split a window into multiple rectangular panes, each of which can
//! display a pretty-printed document (or be filled with a repeated character).
//! This is primarily meant for implementing terminal UIs.
//!
//! You use the [`display_pane()`] function to display a [`PaneNotation`] in a [`PrettyWindow`].
//!
//! The [`PaneNotation`] says how to divide a window into multiple rectangular panes. For example,
//! it could say to show two different documents side-by-side.
//!
//! A [`PrettyWindow`] knows how to display a styled character at a position. This library only
//! supplies a simple implementation of the [`PrettyWindow`] trait called [`PlainText`], which
//! ignores style metadata and just writes the window contents to a string. You will need to provide
//! your own implementation of [`PrettyWindow`] for whatever medium you want to display to (like a
//! terminal window).

mod display_pane;
mod divvy;
mod pane_notation;
mod plain_text;
mod pretty_window;
mod printing_options;

pub use display_pane::{display_pane, PaneError};
pub use pane_notation::{DocLabel, PaneNotation, PaneSize};
pub use plain_text::PlainText;
pub use pretty_window::PrettyWindow;
pub use printing_options::{FocusSide, PrintingOptions, WidthStrategy};
