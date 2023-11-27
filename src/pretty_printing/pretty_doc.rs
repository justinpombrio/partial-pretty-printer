use crate::valid_notation::ValidNotation;
use std::fmt;
use std::hash::Hash;

/// A node in a "document" that supports the necessary methods to be pretty-printed.
pub trait PrettyDoc<'d, S>: Copy {
    type Id: Eq + Hash + Copy + Default + fmt::Debug;

    /// An id that uniquely identifies this node. It should not be `Id::default()`.
    fn id(self) -> Self::Id;

    /// The node's notation.
    fn notation(self) -> &'d ValidNotation<S>;

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
    fn unwrap_last_child(self) -> Self;

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
    fn unwrap_prev_sibling(self, parent: Self, i: usize) -> Self;
}
