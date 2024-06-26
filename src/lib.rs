//! This is a pretty printing library for formatting source code in any language.
//!
//! You provide declarative rules for how to display each sort of node in a
//! document, including line break options, indentation, and coloring. The Partial
//! Pretty Printer prints the document, picking a good layout that fits in your
//! desired line width if possible.
//!
//! The [`Notation`] combinators that it uses are similar to existing approaches like Wadler's
//! [prettier printer](http://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf).
//! However, this library's algorithm is unique in that it is peephole-efficient: it lets you
//! display just _part of_ a document. If you ask it to print 50 lines in the middle of a 100,000
//! line document, it can typically do that in ~50 units of work, rather than ~50,000 units of work.
//!
//! ## Quick Reference
//!
//! You can:
//!
//! - Print an entire [`PrettyDoc`] to a `String` using [`pretty_print_to_string()`].
//! - Print part of a [`PrettyDoc`] using [`pretty_print()`]. This lets you (i) use styles and (ii)
//!   improve performance by only printing what you need.
//! - Make a terminal UI with multiple side-by-side documents, using the [`pane`] module.
//!
//! Keep reading for details.
//!
//! ## The `PrettyDoc` trait
//!
//! Each node of your document must implement the [`PrettyDoc`] trait.
//! This trait lets the pretty printer get:
//!
//! - The node's contents, which are either a piece of text or 0 or more child
//!   nodes.
//! - The node's [`Notation`], which describes how to display that sort of node.
//!   It can express choices like "if the whole list won't fit on one line, put newlines between
//!   each element". For an example of how to write realistic [`Notation`]s, see
//!   [`doc_examples::Json`](https://github.com/justinpombrio/partial-pretty-printer/blob/master/src/examples/json.rs).
//! - Other associated data like a unique ID.
//!
//! ## Pretty Printing Functions
//!
//! There are two ways to pretty print a [`PrettyDoc`].
//!
//! The simpler one is [`pretty_print_to_string()`], which prints the entire document to a
//! plain-text string, with some preferred line width. This provides a simple interface but does not
//! take full advantage of this library.
//!
//! The more versatile [`pretty_print()`] function exposes two additional features. First, it
//! produces text with [`Style`] metadata instead of plain-text. Second, it lets you efficiently
//! print just part of the document. You ask the printer to focus on the start or end of a specific
//! document node, and it returns a pair of lazy iterators that will print lines above and below
//! that focus. If the document is large, you will save a lot of time by taking only as many lines
//! as you need from the iterators. For example, a text editor would only take as many lines as fit
//! on the screen.
//!
//! ## Panes
//!
//! Besides pretty printing a single document, this library has a mechanism for splitting a window
//! into multiple rectangular panes, and displaying a different document in each one. This is meant
//! for implementing terminal UIs. For more details see the [`pane`] module.

mod consolidated_notation;
mod geometry;
mod infra;
mod notation;
mod oracle;
mod pretty_doc;
mod pretty_print;
mod valid_notation;

pub mod doc_examples;
pub mod notation_constructors;
pub mod pane;

pub use consolidated_notation::{PrintingError, Segment};
pub use geometry::{Col, Height, Pos, Row, Size, Width};
pub use notation::{CheckPos, Condition, Notation, StyleLabel};
pub use pretty_doc::{PrettyDoc, Style};
pub use pretty_print::{pretty_print, pretty_print_to_string, FocusTarget, FocusedLine, Line};
pub use valid_notation::{NotationError, ValidNotation};

/// For testing only. Not core functionality.
pub mod testing {
    pub use super::geometry::str_width;
    pub use super::oracle::oracular_pretty_print;
}
