use crate::pretty_printing::PrettyDoc;
use crate::valid_notation::ValidNotation;
use std::cell::Cell;
use std::fmt;
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

#[derive(Debug)]
pub struct Tree<Style, Mark>
where
    Style: fmt::Debug + Default + 'static,
    Mark: fmt::Debug,
{
    pub id: u32,
    pub notation: &'static ValidNotation<Style>,
    pub contents: Contents<Style, Mark>,
    pub whole_node_mark: Option<Mark>,
    pub partial_node_marks: Vec<(String, Mark)>,
}

#[derive(Debug)]
pub enum Contents<Style, Mark>
where
    Style: fmt::Debug + Default + 'static,
    Mark: fmt::Debug,
{
    Text(String),
    Children(Vec<Tree<Style, Mark>>),
}

impl<Style, Mark> Tree<Style, Mark>
where
    Style: fmt::Debug + Default + 'static,
    Mark: fmt::Debug,
{
    pub fn new_text(notation: &'static ValidNotation<Style>, text: String) -> Self {
        Tree {
            id: next_id(),
            notation,
            contents: Contents::Text(text),
            whole_node_mark: None,
            partial_node_marks: Vec::new(),
        }
    }

    pub fn new_branch(notation: &'static ValidNotation<Style>, children: Vec<Self>) -> Self {
        Tree {
            id: next_id(),
            notation,
            contents: Contents::Children(children),
            whole_node_mark: None,
            partial_node_marks: Vec::new(),
        }
    }

    pub fn whole_node_mark(mut self, mark: Mark) -> Self {
        self.whole_node_mark = Some(mark);
        self
    }

    pub fn partial_node_mark(mut self, name: &str, mark: Mark) -> Self {
        self.partial_node_marks.push((name.to_owned(), mark));
        self
    }

    /// Must only be called between constructions of distinct Json docs.
    pub fn reset_id() {
        ID_COUNTER.set(1);
    }
}

impl<'d, Style, Mark> PrettyDoc<'d> for &'d Tree<Style, Mark>
where
    Style: fmt::Debug + Default + 'static,
    Mark: fmt::Debug,
{
    type Id = u32;
    type Style = Style;
    type Mark = Mark;

    fn id(self) -> u32 {
        self.id
    }

    fn notation(self) -> &'d ValidNotation<Style> {
        self.notation
    }

    fn whole_node_mark(self) -> Option<&'d Self::Mark> {
        self.whole_node_mark.as_ref()
    }

    fn partial_node_mark(self, mark_name: &'static str) -> Option<&'d Self::Mark> {
        for (name, mark) in &self.partial_node_marks {
            if name == mark_name {
                return Some(mark);
            }
        }
        None
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
