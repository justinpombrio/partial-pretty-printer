use crate::{CompiledNotation, NotationCache};

/// A "document" that supports the necessary methods to be pretty-printed.
pub trait PrettyDocument: Sized + Clone {
    type ChildIter: Iterator<Item = Self>;

    /// This node's parent, together with the index of this node (or `None` if
    /// this is the root node).
    fn parent(&self) -> Option<(Self, usize)>;
    /// The node's `i`th child. `i` will always be valid.
    fn child(&self, i: usize) -> Self;
    /// All of the node's (immediate) children.
    fn children(&self) -> Self::ChildIter;
    /// The node's notation.
    fn notation(&self) -> &CompiledNotation;
    /// If the node contains text, that text. Otherwise `None`.
    fn text(&self) -> Option<&str>;

    /// Get cached information about this node's notation. This can be computed via
    /// `NotationCache::compute`. **However, it is an expensive operation, that should only be
    /// re-computed when the document is edited.** Specifically, when a node in the document is
    /// edited, that invalidates its cache and all of its ancestors' caches.
    fn cache(&self) -> &NotationCache;
}
