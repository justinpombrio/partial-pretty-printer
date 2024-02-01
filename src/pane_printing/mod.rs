mod pane_notation;
mod pretty_window;
mod render_options;

pub use pane_notation::{Label, PaneNotation, PaneSize};
pub use pretty_window::PrettyWindow;
pub use render_options::{RenderOptions, WidthStrategy};

mod plain_text;

/*
mod pane;
mod pane_print;

pub use pane::PaneError;
pub use pane_print::{pane_print, Path};
pub use plain_text::PlainText;
*/
