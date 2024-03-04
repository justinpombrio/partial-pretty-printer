// TODO: After rewrite, ensure that these docs are up to date.
//       Especially, e.g., referenced code like `PrettyDoc`.
//! This is a pretty printing library for tree-shaped documents, such as ASTs. Given declarative
//! rules for how to display each sort of node in a document, it prints the document with a
//! desired line width.
//!
//! The combinators it uses for pretty printing are similar to existing approaches like Wadler's
//! [prettier printer](http://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf). However,
//! the algorithm it uses is unique, and lets you display just _part of_ a document. If you ask it
//! to print 50 lines in the middle of a 100,000 line document, it can typically do that in ~50
//! units of work, rather than ~50,000 units of work.
//!
//! # Installation
//!
//! This crate [TODO: will be] on crates.io, and can be imported with:
//!
//! ```text
//! [dependencies]
//! partial-pretty-printer = "*"
//! ```
//!
//! It is currently unstable, and if you use it you should expect to encounter breaking changes.
//!
//! # Quick Reference
//!
//! You can:
//!
//! - Print a [`PrettyDoc`] to a `String` using [`pretty_print_to_string`].
//! - Print a node in a [`PrettyDoc`] to get lazy iterators over [`Line`] using
//! [`pretty_print`]. This lets you (i) use styles and (ii) print just part of a
//! document for efficiency.
//! - Make a character-grid based UI with nested panes, using the [`pane`] module.
//!
//! Keep reading for details.
//!
//! # Usage
//!
//! In order to pretty print a document, the document must implement the [`PrettyDoc`] trait:
//!
//! ```ignore
//! pub trait PrettyDoc<'d>: Copy {
//!     type Id: Eq + Hash + Copy + Default + fmt::Debug;
//!     type Style: Style + 'd;
//!     type StyleLabel: fmt::Debug + Clone + 'd;
//!
//!     fn id(self) -> Self::Id;
//!     fn notation(self) -> &'d ValidNotation<Self::StyleLabel>;
//!     fn lookup_style(self, style_label: Self::StyleLabel) -> Self::Style;
//!     fn node_style(self) -> Self::Style;
//!     fn num_children(self) -> Option<usize>;
//!     fn unwrap_text(self) -> &'d str;
//!     fn unwrap_child(self, i: usize) -> Self;
//! }
//! ```
//!
//! For each node in the document, this requires:
//!
//! - a unique id for that node,
//! - a notation for displaying that sort of node, and
//! - accessors for the contents of that node, which is either a sequence of contained nodes
//!   (children), or a string (text).
//!
//! [Read more](trait.PrettyDoc.html)
//!
//! ## Pretty Printing Functions
//!
//! There are two ways to pretty print a `PrettyDoc`.
//! The simpler one is [`pretty_print_to_string`], which prints the entirety of
//! the document to a string, with a preferred line width:
//!
//! ```ignore
//! pub fn pretty_print_to_string<'d, D: PrettyDoc<'d>>(
//!     doc: D,
//!     width: Width,
//! ) -> Result<String, PrintingError>;
//! ```
//!
//! ([`Width`] is just an alias for an integer type, currently `u16`.)
//!
//! This provides a simple interface, but does not take full advantage of this library. To do so,
//! you can use the more versatile [`pretty_print`] function:
//!
//! ```ignore
//! pub fn pretty_print<'d, D: PrettyDoc<'d>>(
//!     doc: D,
//!     width: Width,
//!     path: &[usize],
//!     seek_end: bool,
//! ) -> Result<
//!     (
//!         impl Iterator<Item = Result<Line<'d, D>, PrintingError>>,
//!         FocusedLine<'d, D>,
//!         impl Iterator<Item = Result<Line<'d, D>, PrintingError>>,
//!     ),
//!     PrintingError,
//! >;
//! ```
//!
//! This exposes two additional features.
//!
//! First, instead of printing the entire document, it lets you print just part
//! of the document. `path` (and `seek_end`) specify which line of the document
//! to focus on. The return value gives that focused line, and iterators that
//! yield the lines above and below it. If you exhaust both iterators, you will
//! print the entire document, but if you take fewer lines you can save the
//! pretty printer a lot of work.
//!
//! Second, notice that the iterators yield [`Line`]s instead of strings. A
//! `Line` contains additional metadata such as styles.
//!
//! ## Notation Design
//!
//! TODO
//!
//! ## Other Types
//!
//! A character position [`Pos`] has a [`Row`] and [`Col`]. `Row` and `Col` are type aliases for
//! integer types.
//!
//! A size [`Size`] has a [`Width`] and [`Height`]. `Width` and `Height` are type aliases for
//! integer types.
//!
//! Everything is measured in characters and is 0-indexed.
//!
//! ## Pane Printing
//!
//! Besides pretty printing, this library can also perform "pane printing": a simple mechanism for
//! splitting a window into multiple rectangular panes, each of which can display a document via
//! pretty printing. This is meant for implementing terminal UIs. For more details see the [`pane`]
//! module.

mod consolidated_notation;
mod geometry;
mod infra;
mod notation;
mod oracle;
mod pretty_doc;
mod pretty_print;
mod valid_notation;

pub mod examples;
pub mod notation_constructors;
pub mod pane;

pub use consolidated_notation::{PrintingError, Segment};
pub use geometry::{Col, Height, Pos, Row, Size, Width};
pub use notation::{CheckPos, Condition, Literal, Notation, StyleLabel};
pub use pretty_doc::{PrettyDoc, Style};
pub use pretty_print::{pretty_print, pretty_print_to_string, FocusedLine, Line};
pub use valid_notation::{NotationError, ValidNotation};

pub mod testing {
    pub use super::geometry::str_width;
    pub use super::oracle::oracular_pretty_print;
}
