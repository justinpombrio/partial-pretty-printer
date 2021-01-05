use crate::{Notation, PrettyDoc, PrettyDocContents};
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub trait Sort: Copy + Eq + Debug {
    fn notation(self) -> &'static Notation;
}

#[derive(Debug, Clone)]
pub struct Doc<S: Sort> {
    id: usize,
    sort: S,
    contents: DocContents<S>,
}

#[derive(Debug, Clone)]
enum DocContents<S: Sort> {
    Text(String),
    Node(Vec<Doc<S>>),
}

impl<S: Sort> Doc<S> {
    pub fn new_text(sort: S, text: String) -> Doc<S> {
        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Doc {
            id,
            sort,
            contents: DocContents::Text(text),
        }
    }

    pub fn new_node(sort: S, children: Vec<Doc<S>>) -> Doc<S> {
        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Doc {
            id,
            sort,
            contents: DocContents::Node(children),
        }
    }
}

impl<S: Sort> PrettyDoc for Doc<S> {
    type Id = usize;

    fn id(&self) -> usize {
        self.id
    }

    fn notation(&self) -> &Notation {
        S::notation(self.sort)
    }

    fn contents(&self) -> PrettyDocContents<Self> {
        match &self.contents {
            DocContents::Node(children) => PrettyDocContents::Children(children),
            DocContents::Text(text) => PrettyDocContents::Text(text),
        }
    }
}
