use crate::pretty_printing::{PrettyDoc, Style};
use crate::valid_notation::ValidNotation;
use std::cell::Cell;
use std::thread_local;

thread_local! {
    // id 0 reserved for Default
    static ID_COUNTER: Cell<u32> = Cell::new(1);
}

fn next_id() -> u32 {
    let id = ID_COUNTER.get();
    ID_COUNTER.set(id + 1);
    id
}

pub type StyleLabel = &'static str;

#[derive(Debug)]
pub struct Tree<S>
where
    S: Style + From<StyleLabel> + Default + 'static,
{
    pub id: u32,
    pub notation: &'static ValidNotation<StyleLabel>,
    pub contents: Contents<S>,
    pub node_style: S,
    pub style_overrides: Vec<(StyleLabel, S)>,
}

#[derive(Debug)]
pub enum Contents<S>
where
    S: Style + From<StyleLabel> + Default + 'static,
{
    Text(String),
    Children(Vec<Tree<S>>),
}

impl<S> Tree<S>
where
    S: Style + From<StyleLabel> + Default + 'static,
{
    pub fn new_text(notation: &'static ValidNotation<StyleLabel>, text: String) -> Self {
        Tree {
            id: next_id(),
            notation,
            contents: Contents::Text(text),
            node_style: S::default(),
            style_overrides: Vec::new(),
        }
    }

    pub fn new_branch(notation: &'static ValidNotation<StyleLabel>, children: Vec<Self>) -> Self {
        Tree {
            id: next_id(),
            notation,
            contents: Contents::Children(children),
            node_style: S::default(),
            style_overrides: Vec::new(),
        }
    }

    pub fn with_style(mut self, style: S) -> Self {
        self.node_style = style;
        self
    }

    pub fn with_style_override(mut self, label: StyleLabel, style: S) -> Self {
        self.style_overrides.push((label, style));
        self
    }

    /// Must only be called between constructions of distinct Json docs.
    pub fn reset_id() {
        ID_COUNTER.set(1);
    }
}

impl<'d, S> PrettyDoc<'d> for &'d Tree<S>
where
    S: Style + From<StyleLabel> + Default + 'static,
{
    type Id = u32;
    type Style = S;
    type StyleLabel = StyleLabel;

    fn id(self) -> u32 {
        self.id
    }

    fn notation(self) -> &'d ValidNotation<StyleLabel> {
        self.notation
    }

    fn node_style(self) -> Self::Style {
        self.node_style.clone()
    }

    fn lookup_style(self, label: StyleLabel) -> Self::Style {
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
}
