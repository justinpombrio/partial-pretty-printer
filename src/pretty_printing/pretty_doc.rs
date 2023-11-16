use crate::valid_notation::ValidNotation;

/// A node in a "document" that supports the necessary methods to be pretty-printed.
pub trait PrettyDoc<'d>: Copy {
    type Id: Eq + Copy;

    /// An id that uniquely identifies this node.
    fn id(self) -> Self::Id;

    /// The node's notation.
    fn notation(self) -> &'d ValidNotation;

    /// Get this node's number of children, or `None` if it contains text instead. `Some(0)` means
    /// that this node contains no children, and no text.
    fn num_children(self) -> Option<usize>;

    /// Get this node's text, or panic. The pretty printer will only call this method if
    /// `num_children()` returns `None`. It is safe to panic otherwise.
    fn unwrap_text(self) -> &'d str;

    /// Get this node's i'th child, or panic. The pretty printer will only call this method if
    /// `num_children()` returns `Some(n)` for `n > i`. It is safe to panic otherwise.
    fn unwrap_child(self, i: usize) -> Self;

    /// Get this node's last child, or panic. The pretty printer will only call this method if
    /// `num_children()` returns `Some(_)`. It is safe to panic otherwise.
    fn unwrap_last_child(self) -> Self;

    /// Access this node's previous sibling, or panic. `parent` is this node's parent, and `i` is
    /// the index of its previous sibling. The pretty printer will only call this method if:
    ///
    /// - `parent.num_children()` returned `Some(n)` for `n > i + 1`, and
    /// - the index of `self` is `i + 1`.
    ///
    /// It is safe to panic otherwise.
    fn unwrap_prev_sibling(self, parent: Self, i: usize) -> Self {
        parent.unwrap_child(i)
    }
}
