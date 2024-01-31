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
    type Style: fmt::Debug + 'd;
    /// Arbitrary data associated with some nodes in the document. Returned as part of
    /// `LineContents` when pretty printing.
    type Mark: fmt::Debug + 'd;

    /// An id that uniquely identifies this node. It should not be `Id::default()`.
    fn id(self) -> Self::Id;

    /// The node's notation.
    fn notation(self) -> &'d ValidNotation<Self::Style>;

    /// The mark on this node, if any. This method is called once per document node. If it returns
    /// `Some(mark)`, the mark is applied to that node.
    fn whole_node_mark(self) -> Option<&'d Self::Mark> {
        None
    }

    /// Look up a mark that applies to only part of this node. Whenever `Notation::Mark(mark_name,
    /// notation)` is encountered while printing, `partial_node_mark(mark_name)` is invoked. If it
    /// returns `Some(mark)`, then that mark is applied to `notation`.
    fn partial_node_mark(self, mark_name: &'static str) -> Option<&'d Self::Mark> {
        None
    }

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
