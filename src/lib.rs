mod doc;
mod notation;
mod pretty_print;

// TODO: have an api
pub use doc::Doc;
pub use notation::{Notation, RepeatInner};
pub use pretty_print::{print_downward_for_testing, print_upward_for_testing};
