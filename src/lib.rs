//! # Partial Pretty Printer
//!
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
//! ## Installation
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
//! ## Usage
//!
//! In order to pretty print a document, it must implement the [`PrettyDoc`] trait:
//!
//! ```ignore
//! pub trait PrettyDoc: Sized {
//!     type Id: Eq + Copy;
//!
//!     fn id(&self) -> Self::Id;
//!     fn notation(&self) -> &Notation;
//!     fn num_children(&self) -> Option<usize>;
//!     fn unwrap_text(&self) -> &str;
//!     fn unwrap_child(&self, i: usize) -> Self;
//! }
//! ```
//!
//! For each node in the document, this requires:
//!
//! - a unique id for that node,
//! - a notation for displaying that sort of node, and
//! - an accessor for the contents of the node, which is either a slice of contained nodes, of a
//!   string.
//!
//! ### Pretty Printing Functions
//!
//! There are two pretty printing functions. The simpler one is [`pretty_print_to_string`], which
//! prints the entirety of the document, with a preferred line width, to a string:
//!
//! ```ignore
//! fn pretty_print_to_string<D: PrettyDoc>(doc: &'d D, width: Width) -> String;
//! ```
//!
//! This provides a simple interface, but does not take full advantage of this library. To do so,
//! you can use the more versatile [`pretty_print`] function:
//!     
//! ```ignore
//! fn pretty_print<'d, D: PrettyDoc>(
//!     doc: &'d D,
//!     width: Width,
//!     path: &[usize],
//! ) -> (
//!     impl Iterator<Item = LineContents<'d>> + 'd,
//!     impl Iterator<Item = LineContents<'d>> + 'd,
//! );
//! ```
//!
//! This exposes two additional features.
//!
//! First, instead of printing the entire document, it lets you print just part of the document.
//! `path` is a sequence of indices, that leads from the root of the document to a node buried
//! inside of it. The return value is a pair of iterators over line contents: the first prints lines
//! _above_ the buried node going up; the second prints lines _from its first line_ going down. If
//! you exhaust both iterators, you will print the entire document, but if you take fewer you can
//! save the pretty printer some work.
//!
//! Secondly, notice that the iterators contain [`LineContents`] instead of strings:
//!
//! ```ignore
//! struct LineContents<'d> {
//!     spaces: (Width, Shade),
//!     contents: Vec<(&'d str, Style, Shade)>,
//! }
//! ```
//!
//! The contents of the line can contain styles, which can be set in the [`Notation`]s returned by
//! [`PrettyDoc`]. Additionally, there cursor highlighting information in the form of a _shade_ for
//! the background color. `spaces` is the indentation level of the line, and the shade of those
//! spaces.
//!
//! ### Pane Printing
//!
//! Besides pretty printing, this library can also perform "pane printing": a simple mechanism for
//! splitting a window into multiple rectangular panes, each of which can display a document via
//! pretty printing. This is meant for implementing terminal UIs. For more details see the [`pane`]
//! module.
//!
//! ### Notation Design
//!
//! TODO
//!
//! ### Other Types
//!
//! A character position [`Pos`] has a [`Line`] and [`Col`]. `Line` and `Col` are type aliases for
//! integer types.
//!
//! A size [`Size`] has a [`Width`] and [`Height`]. `Width` and `Height` are type aliases for
//! integer types.
//!
//! Everything is measured in characters and 0-indexed.

mod geometry;
mod notation;
mod pane_printing;
mod pretty_printing;
mod style;

pub mod examples;
pub mod notation_constructors;

pub use geometry::{Col, Height, Line, Pos, Size, Width};
pub use notation::{Notation, RepeatInner};
pub use pretty_printing::{pretty_print, pretty_print_to_string, LineContents, PrettyDoc};
pub use style::{Color, ShadedStyle, Style};

pub mod pane {
    //! Print to multiple rectangular sub-panes of a window.
    //!
    //! This module performs "pane printing": a way to split a window into multiple rectangular
    //! panes, each of which can display a document via pretty printing. This is primarily meant
    //! for implementing a terminal UI.
    //!
    //! Pane printing involves rendering a _pane notation_ onto a _window_.
    //!
    //! The [`PaneNotation`] says how to divide a window into multiple rectangular panes. For example,
    //! it could say to show two different documents side by side.
    //!
    //! A [`PrettyWindow`] knows how to render styled strings. This library only supplies a simple
    //! implementation of `PrettyWindow` called [`PlainText`], which ignores styling and prints to
    //! a string. If you use pane printing, you will probably want to provide your own
    //! implementation of `PrettyWindow`, for whatever medium you want to display to.
    //!
    //! With a pane notation and window, you can _pane print_:
    //!
    //! ```ignore
    //! fn pane_print<L: Label, D: PrettyDoc, W: PrettyWindow>(
    //!     window: &mut W,
    //!     note: &PaneNotation<L>,
    //!     get_content: &impl Fn(L) -> Option<(D, Path)>,
    //! ) -> Result<(), PaneError<W>>;
    //! ```
    //!
    //! - `window` is the `PrettyWindow` to display to.
    //! - `note` is the `PaneNotation` to render. It says how to break up the screen into rectangular
    //!   "panes", and which document to display in each pane. It does not contain the Documents
    //!   directly, instead it references them by `Label`. (`Label` is a trait alias: `trait Label:
    //!   Clone + Debug {}`.)
    //! - `get_content` is a function to look up a document by label. It returns both the document, and
    //!   the path to the node in the document to focus on. (The empty path `vec![]` will focus on the
    //!   top of the document.)
    pub use super::pane_printing::{
        pane_print, Label, PaneNotation, PaneSize, PlainText, PrettyWindow, RenderOptions,
        WidthStrategy,
    };
}
