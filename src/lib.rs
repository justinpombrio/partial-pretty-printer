mod doc;
mod notation;
mod pretty_print;

pub mod json_notation;
pub mod notation_constructors;
pub mod simple_doc;

// TODO: have an api
pub use doc::{Doc, DocContents};
pub use notation::{Notation, RepeatInner};
pub use pretty_print::pretty_print;
