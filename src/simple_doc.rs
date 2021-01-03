use super::doc::{Doc, DocContents};
use super::notation::Notation;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub trait Sort: Copy + Eq + Debug {
    fn notation(self) -> &'static Notation;
}

#[derive(Debug, Clone)]
pub struct SimpleDoc<S: Sort> {
    id: usize,
    sort: S,
    contents: SimpleDocContents<S>,
}

#[derive(Debug, Clone)]
enum SimpleDocContents<S: Sort> {
    Text(String),
    Node(Vec<SimpleDoc<S>>),
}

impl<S: Sort> SimpleDoc<S> {
    pub fn new_text(sort: S, text: String) -> SimpleDoc<S> {
        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        SimpleDoc {
            id,
            sort,
            contents: SimpleDocContents::Text(text),
        }
    }

    pub fn new_node(sort: S, children: Vec<SimpleDoc<S>>) -> SimpleDoc<S> {
        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        SimpleDoc {
            id,
            sort,
            contents: SimpleDocContents::Node(children),
        }
    }
}

impl<S: Sort> Doc for SimpleDoc<S> {
    type Id = usize;

    fn id(&self) -> usize {
        self.id
    }

    fn notation(&self) -> &Notation {
        S::notation(self.sort)
    }

    fn contents(&self) -> DocContents<Self> {
        match &self.contents {
            SimpleDocContents::Node(children) => DocContents::Children(children),
            SimpleDocContents::Text(text) => DocContents::Text(text),
        }
    }
}
