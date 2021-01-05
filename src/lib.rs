mod notation;
mod pretty_printing;

pub mod examples;
pub mod notation_constructors;

pub use notation::{Notation, RepeatInner};
pub use pretty_printing::{pretty_print, PrettyDoc, PrettyDocContents};
