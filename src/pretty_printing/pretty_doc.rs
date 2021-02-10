use crate::notation::Notation;

/// A node in a "document" that supports the necessary methods to be pretty-printed.
pub trait PrettyDoc<'d>: Copy {
    type Id: Eq + Copy;

    /// An id that uniquely identifies this node.
    fn id(self) -> Self::Id;

    /// The node's notation.
    fn notation(self) -> &'d Notation;

    /// Get this node's number of children, or `None` if it contains text instead.
    fn num_children(self) -> Option<usize>;

    /// Get this node's text, or panic.
    fn unwrap_text(self) -> &'d str;

    /// Get this node's i'th child, or panic.
    fn unwrap_child(self, i: usize) -> Self;
}
