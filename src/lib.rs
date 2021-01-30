mod geometry;
mod notation;
mod pane_printing;
mod pretty_printing;
mod style;

pub mod examples;
pub mod notation_constructors;

pub use geometry::{Col, Height, Line, Pos, Size, Width};
pub use notation::{Notation, RepeatInner};
pub use pane_printing::{
    pane_print, Label, PaneNotation, PaneSize, PlainText, PrettyWindow, RenderOptions,
    WidthStrategy,
};
pub use pretty_printing::{
    pretty_print, pretty_print_to_string, LineContents, PrettyDoc, PrettyDocContents,
};
pub use style::{Color, ShadedStyle, Style};
