mod notation;
mod partial_print;
mod pretty_print;

// TODO: have an api
pub use notation::Notation;
pub use partial_print::{print_downward_for_testing, print_upward_for_testing};
pub use pretty_print::pretty_print;
