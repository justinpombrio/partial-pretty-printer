mod geometry;
mod notation;
mod pane_printing;
mod pretty_printing;
mod style;

pub mod examples;
pub mod notation_constructors;

pub use notation::{Notation, RepeatInner};
pub use pane_printing::{CursorVisibility, PaneNotation, PlainText, PrettyWindow, ScrollStrategy};
pub use pretty_printing::{
    pretty_print, pretty_print_to_string, LineContents, PrettyDoc, PrettyDocContents,
};
pub use style::{Color, Emph, Style};
