mod measure;
mod notation;
mod pretty_print;
mod staircase;
mod validate;

pub mod if_flat;

pub use notation::Notation;
pub use pretty_print::{pretty_print, pretty_print_at, pretty_print_first, pretty_print_last};

// TODO: Make these private
pub use measure::{MeasuredNotation, Pos, Shapes};
