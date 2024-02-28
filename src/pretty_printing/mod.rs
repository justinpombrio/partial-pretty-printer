mod consolidated_notation;
mod oracle;
mod pretty_doc;
mod pretty_print;

pub use consolidated_notation::{PrintingError, Segment};
pub use oracle::oracular_pretty_print;
pub use pretty_doc::{PrettyDoc, Style};
pub use pretty_print::{pretty_print, pretty_print_to_string, FocusedLine, Line};
