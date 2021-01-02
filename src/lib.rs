mod doc;
mod notation;
mod pretty_print;

pub mod notation_constructors;

// TODO: have an api
pub use doc::Doc;
pub use notation::{Notation, RepeatInner};
pub use pretty_print::pretty_print;
