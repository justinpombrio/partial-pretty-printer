use crate::notation::Notation;

/// A node in a "document" that supports the necessary methods to be pretty-printed.
pub trait PrettyDoc: Sized {
    type Id: Eq + Copy;

    /// An id that uniquely identifies this node.
    fn id(&self) -> Self::Id;
    /// The node's notation.
    fn notation(&self) -> &Notation;
    /// Get the contents of this document node. It may contain text, or children, but not both.
    fn contents(&self) -> PrettyDocContents<Self>;

    /// Get this node's text, or panic.
    fn unwrap_text(&self) -> &str {
        match self.contents() {
            PrettyDocContents::Text(text) => text,
            PrettyDocContents::Children(_) => panic!("PrettyDoc: expected text"),
        }
    }

    /// Get this node's number of children, or `None` if it contains text instead.
    fn num_children(&self) -> Option<usize> {
        match self.contents() {
            PrettyDocContents::Text(_) => None,
            PrettyDocContents::Children(children) => Some(children.len()),
        }
    }

    /// Get this node's i'th child, or panic.
    fn unwrap_child(&self, i: usize) -> &Self {
        match self.contents() {
            PrettyDocContents::Text(_) => panic!("PrettyDoc: expected children"),
            PrettyDocContents::Children(children) => children
                .get(i)
                .expect("PrettyDoc: child index out of bounds"),
        }
    }
}

/// The contents of a node in a document. It may contain text, or children, but not both.
#[derive(Debug)]
pub enum PrettyDocContents<'d, D: PrettyDoc> {
    Text(&'d str),
    Children(&'d [D]),
}
