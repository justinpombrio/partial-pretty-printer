use crate::{Condition, StyleLabel, ValidNotation};
use std::fmt;
use std::hash::Hash;

#[cfg(doc)]
use crate::Notation; // for links in rustdocs

/// A reference of lifetime `'d` to a node in a "document", that supports the necessary methods to
/// be pretty-printed.
///
/// A node is expected to contain either a piece of text, or 0 or more child nodes.
///
/// Consider implementing [`PrettyDoc::unwrap_last_child()`] and [`PrettyDoc::unwrap_prev_sibling()`]
/// even though default implementations are provided. Depending on your representation of documents,
/// you may be able to write much more efficient implementations.
pub trait PrettyDoc<'d>: Copy {
    /// Used to uniquely identify a node.
    type Id: Eq + Hash + Copy + fmt::Debug;
    /// Arbitrary metadata that's applied to regions of the document.
    type Style: Style + 'd;
    /// Used to look up a style. It should be small and cheap to clone.
    type StyleLabel: StyleLabel + 'd;
    /// Arbitrary property of a node that can be checked with
    /// [`PrettyDoc::condition()`]/[`Notation::Check`].
    type Condition: Condition + 'd;

    /// Get the id that uniquely identifies this node.
    fn id(self) -> Self::Id;

    /// Get this node's notation.
    fn notation(self) -> &'d ValidNotation<Self::StyleLabel, Self::Condition>;

    /// Check whether the given condition holds for this node. The pretty printer will only call
    /// this method with conditions that were used in [`Notation::Check`].
    fn condition(self, condition: &Self::Condition) -> bool;

    /// Get the style associated with this label, in the context of this node.
    /// The pretty printer will only call this method with labels that were used in
    /// [`Notation::Style`].
    fn lookup_style(self, style_label: Self::StyleLabel) -> Self::Style;

    /// Get the style to apply to this node. This method is called once per document node and applies
    /// to the whole node. It will be [`combined`](Style::combine) with any overlapping styles.
    fn node_style(self) -> Self::Style;

    /// Get the number of children this node has, or `None` if it contains text instead. `Some(0)` means
    /// that this node contains no children and no text.
    fn num_children(self) -> Option<usize>;

    /// Get this node's text, or panic. The pretty printer will only call this method if
    /// [`num_children()`](PrettyDoc::num_children) returns `None` - it is ok to make this method
    /// panic otherwise.
    fn unwrap_text(self) -> &'d str;

    /// Get this node's i'th child, or panic. The pretty printer will only call this method if
    /// [`num_children()`](PrettyDoc::num_children) returns `Some(n)` for `n > i` - it is ok to make
    /// this method panic otherwise.
    fn unwrap_child(self, i: usize) -> Self;

    /// Get this node's last child, or panic. The pretty printer will only call this method if
    /// [`num_children()`](PrettyDoc::num_children) returns `Some(n)` where `n > 0` - it is ok to
    /// make this method panic otherwise.
    ///
    /// This method is redundant with [`unwrap_child()`](PrettyDoc::unwrap_child), but depending on
    /// your document representation it could have a more efficient implementation.
    fn unwrap_last_child(self) -> Self {
        match self.num_children() {
            None => panic!("Bug in PrettyDoc impl: num_children's return value changed"),
            Some(n) => self.unwrap_child(n - 1),
        }
    }

    /// Get this node's previous sibling, or panic. `parent` is this node's parent and `i` is
    /// the index of this node's previous sibling. The pretty printer will only call this method if:
    ///
    /// - `parent.num_children()` returned `Some(n)` for `n > i + 1`, and
    /// - the index of `self` is `i + 1`.
    ///
    /// It is ok to make this method panic otherwise.
    ///
    /// This method is redundant with [`unwrap_child()`](PrettyDoc::unwrap_child), but depending on
    /// your document representation it could have a more efficient implementation.
    fn unwrap_prev_sibling(self, parent: Self, i: usize) -> Self {
        parent.unwrap_child(i)
    }
}

/// Styles are arbitrary metadata that are applied to regions of the document. When multiple styles
/// overlap, they are merged into a single style with [`Style::combine()`].
pub trait Style: fmt::Debug + Clone {
    /// Produce a new Style by combining the `outer_style` with an `inner_style`
    /// that applies to a subregion.
    fn combine(outer_style: &Self, inner_style: &Self) -> Self;
}

impl Style for () {
    fn combine(_outer_style: &Self, _inner_style: &Self) -> Self {
        ()
    }
}
