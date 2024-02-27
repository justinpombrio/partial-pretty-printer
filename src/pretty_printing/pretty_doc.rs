use crate::valid_notation::ValidNotation;
use std::fmt;
use std::hash::Hash;

/// A node in a "document" that supports the necessary methods to be pretty-printed.
///
/// Consider implementing `unwrap_last_child` and `unwrap_prev_sibling`. Depending on
/// your representation of documents, they may be much more efficient than their default
/// implementations, which call `unwrap_child`.
pub trait PrettyDoc<'d>: Copy {
    type Id: Eq + Hash + Copy + Default + fmt::Debug;
    /// The style used in this document's notation.
    type Style: Style + 'd;
    // TODO doc
    type StyleLabel: fmt::Debug + Clone + 'd;

    /// An id that uniquely identifies this node. It should not be `Id::default()`.
    fn id(self) -> Self::Id;

    /// The node's notation.
    fn notation(self) -> &'d ValidNotation<Self::StyleLabel>;

    // TODO doc
    fn lookup_style(self, style_label: Self::StyleLabel) -> Self::Style;

    // TODO doc
    /// The mark on this node, if any. This method is called once per document node. If it returns
    /// `Some(mark)`, the mark is applied to that node.
    /// Look up a mark that applies to only part of this node. Whenever `Notation::Mark(mark_name,
    /// notation)` is encountered while printing, `partial_node_mark(mark_name)` is invoked. If it
    /// returns `Some(mark)`, then that mark is applied to `notation`.
    fn node_style(self) -> Self::Style;

    /// Get this node's number of children, or `None` if it contains text instead. `Some(0)` means
    /// that this node contains no children, and no text.
    fn num_children(self) -> Option<usize>;

    /// Get this node's text, or panic. The pretty printer will only call this method if
    /// `num_children()` returns `None` - it is ok to make this method panic otherwise.
    fn unwrap_text(self) -> &'d str;

    /// Get this node's i'th child, or panic. The pretty printer will only call this method if
    /// `num_children()` returns `Some(n)` for `n > i` - it is ok to make this method panic
    /// otherwise.
    fn unwrap_child(self, i: usize) -> Self;

    /// Get this node's last child, or panic. The pretty printer will only call this method if
    /// `num_children()` returns `Some(n)` where `n > 0` - it is ok to make this method panic
    /// otherwise.
    ///
    /// This method is redundant with `unwrap_child`, but depending on your document representation
    /// it could have a more efficient implementation.
    fn unwrap_last_child(self) -> Self {
        match self.num_children() {
            None => panic!("Bug in PrettyDoc impl: num_children's return value changed"),
            Some(n) => self.unwrap_child(n - 1),
        }
    }

    /// Access this node's previous sibling, or panic. `parent` is this node's parent, and `i` is
    /// the index of its previous sibling. The pretty printer will only call this method if:
    ///
    /// - `parent.num_children()` returned `Some(n)` for `n > i + 1`, and
    /// - the index of `self` is `i + 1`.
    ///
    /// It is ok to make this method panic otherwise.
    ///
    /// This method is redundant with `unwrap_child`, but depending on your document representation
    /// it could have a more efficient implementation.
    fn unwrap_prev_sibling(self, parent: Self, i: usize) -> Self {
        parent.unwrap_child(i)
    }
}

// TODO docs
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
