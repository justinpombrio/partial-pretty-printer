//! A sample implementation of the [`PrettyDoc`] trait.

use crate::pretty_printing::{PrettyDoc, Style};
use crate::valid_notation::ValidNotation;
use std::cell::Cell;
use std::thread_local;

#[cfg(doc)]
use crate::Notation; // for links in rustdocs

// Ensure that each tree node will have an id that is unique within its thread. Sharing the id
// counter across threads would break unit tests that are run in parallel and expect a node to have
// a particular id.
thread_local! {
    // id 0 reserved for Default
    static ID_COUNTER: Cell<u32> = Cell::new(1);
}

fn next_id() -> u32 {
    let id = ID_COUNTER.get();
    ID_COUNTER.set(id + 1);
    id
}

/// Properties of a [`Tree`] node that can be checked with [`Notation::Check`].
#[derive(Debug, Clone, Copy)]
pub enum TreeCondition {
    /// Whether this node should be followed by a separator.
    NeedsSeparator,
    /// Whether this node is a text node containing the empty string.
    IsEmptyText,
    /// Whether this node is marked as a comment (by [`Tree::into_comment`]).
    IsComment,
}

pub type TreeStyleLabel = &'static str;
pub type TreeNotation = ValidNotation<TreeStyleLabel, TreeCondition>;

/// A sample implementation of the [`PrettyDoc`] trait. Each `Tree` node can be
/// either a leaf containing text, or a branch containing a list of child `Tree`
/// nodes. `Tree` nodes are not designed to have their contents modified after
/// they're created, but they're useful for creating and printing static
/// documents.
#[derive(Debug, Clone)]
pub struct Tree<S>
where
    S: Style + From<TreeStyleLabel> + Default + 'static,
{
    /// The text or children that the node contains.
    pub contents: Contents<S>,
    /// A unique id for this node. Each thread has a global id counter that
    /// starts at 1 and is incremented each time a node is created.
    /// The id counter can be reset to 1 with [`Tree::reset_id()`].
    pub id: u32,
    /// How to display this node.
    pub notation: &'static TreeNotation,
    /// A style to apply to this entire node.
    pub node_style: S,
    /// The style corresponding to each style label that could be applied to
    /// this node with [`Notation::Style`]. Used for [`PrettyDoc::lookup_style()`].
    pub style_overrides: Vec<(TreeStyleLabel, S)>,
    /// For checking [`TreeCondition::IsComment`].
    pub is_comment: bool,
    /// For checking [`TreeCondition::NeedsSeparator`]. A child needs a
    /// separator iff it's not a comment, and not the last non-comment child.
    /// This is automatically set when creating a branch node, and will become
    /// outdated if you manually modify the branch node's contents later.
    pub needs_separator: bool,
}

/// The contents of a [`Tree`].
#[derive(Debug, Clone)]
pub enum Contents<S>
where
    S: Style + From<TreeStyleLabel> + Default + 'static,
{
    /// A leaf node.
    Text(String),
    /// A branch node.
    Children(Vec<Tree<S>>),
}

impl<S> Tree<S>
where
    S: Style + From<TreeStyleLabel> + Default + 'static,
{
    /// Create a new node containing the given text.
    pub fn new_text(notation: &'static TreeNotation, text: String) -> Self {
        Tree {
            id: next_id(),
            notation,
            contents: Contents::Text(text),
            node_style: S::default(),
            style_overrides: Vec::new(),
            is_comment: false,
            needs_separator: false,
        }
    }

    /// Create a new node containing the given children.
    pub fn new_branch(notation: &'static TreeNotation, mut children: Vec<Self>) -> Self {
        for child in children
            .iter_mut()
            .rev()
            .filter(|child| !child.is_comment)
            .skip(1)
        {
            child.needs_separator = true;
        }

        Tree {
            id: next_id(),
            notation,
            contents: Contents::Children(children),
            node_style: S::default(),
            style_overrides: Vec::new(),
            is_comment: false,
            needs_separator: false,
        }
    }

    /// Mark this node as being a comment.
    pub fn into_comment(mut self) -> Self {
        self.is_comment = true;
        self
    }

    /// Apply the style to this node.
    pub fn with_style(mut self, style: S) -> Self {
        self.node_style = style;
        self
    }

    /// Add a label->style lookup entry for this node.
    pub fn with_style_override(mut self, label: TreeStyleLabel, style: S) -> Self {
        self.style_overrides.push((label, style));
        self
    }

    /// Reset the global id counter, so that the next `Tree` that's created will
    /// have the id `1`. This is intended for use in unit tests that rely on
    /// nodes having particular ids. It must only be called between
    /// constructions of distinct documents, to ensure ids are unique within a
    /// document.
    pub fn reset_id() {
        ID_COUNTER.set(1);
    }
}

impl<'d, S> PrettyDoc<'d> for &'d Tree<S>
where
    S: Style + From<TreeStyleLabel> + Default + 'static,
{
    type Id = u32;
    type Style = S;
    type StyleLabel = TreeStyleLabel;
    type Condition = TreeCondition;

    fn id(self) -> u32 {
        self.id
    }

    fn notation(self) -> &'d TreeNotation {
        self.notation
    }

    fn node_style(self) -> Self::Style {
        self.node_style.clone()
    }

    fn lookup_style(self, label: TreeStyleLabel) -> Self::Style {
        for (l, style) in &self.style_overrides {
            if *l == label {
                return style.clone();
            }
        }
        Self::Style::from(label)
    }

    fn num_children(self) -> Option<usize> {
        match &self.contents {
            Contents::Text(_) => None,
            Contents::Children(children) => Some(children.len()),
        }
    }

    fn unwrap_text(self) -> &'d str {
        match &self.contents {
            Contents::Text(text) => text,
            Contents::Children(_) => panic!("Tree: invalid invocation of unwrap_text"),
        }
    }

    fn unwrap_child(self, i: usize) -> Self {
        match &self.contents {
            Contents::Text(_) => panic!("Tree: invalid invocation of unwrap_child"),
            Contents::Children(children) => &children[i],
        }
    }

    fn condition(self, condition: &TreeCondition) -> bool {
        match condition {
            TreeCondition::IsEmptyText => match &self.contents {
                Contents::Text(text) if text.is_empty() => true,
                _ => false,
            },
            TreeCondition::NeedsSeparator => self.needs_separator,
            TreeCondition::IsComment => self.is_comment,
        }
    }
}
