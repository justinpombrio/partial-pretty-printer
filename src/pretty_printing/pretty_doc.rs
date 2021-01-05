use crate::notation::Notation;

/// A "document" that supports the necessary methods to be pretty-printed.
pub trait PrettyDoc: Sized {
    type Id: Eq + Copy;

    fn id(&self) -> Self::Id;
    fn notation(&self) -> &Notation;
    fn contents(&self) -> PrettyDocContents<Self>;

    fn unwrap_text(&self) -> &str {
        match self.contents() {
            PrettyDocContents::Text(text) => text,
            PrettyDocContents::Children(_) => panic!("PrettyDoc: expected text"),
        }
    }

    fn num_children(&self) -> Option<usize> {
        match self.contents() {
            PrettyDocContents::Text(_) => None,
            PrettyDocContents::Children(children) => Some(children.len()),
        }
    }

    fn unwrap_child(&self, i: usize) -> &Self {
        match self.contents() {
            PrettyDocContents::Text(_) => panic!("PrettyDoc: expected children"),
            PrettyDocContents::Children(children) => children
                .get(i)
                .expect("PrettyDoc: child index out of bounds"),
        }
    }
}

#[derive(Debug)]
pub enum PrettyDocContents<'d, D: PrettyDoc> {
    Text(&'d str),
    Children(&'d [D]),
}
