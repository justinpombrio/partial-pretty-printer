use super::{assert_pp, assert_pp_seek, punct};
use once_cell::sync::Lazy;
use partial_pretty_printer::notation_constructors::{
    child, left, nl, repeat, right, surrounded, text,
};
use partial_pretty_printer::{Notation, PrettyDoc, RepeatInner, Style};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlowWrap {
    id: usize,
    data: FlowWrapData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FlowWrapData {
    Word(String),
    Words(Vec<FlowWrap>),
    Paragraph(Box<[FlowWrap; 1]>),
}

static WORD_NOTATION: Lazy<Notation> = Lazy::new(|| text(Style::plain()));
static WORDS_NOTATION: Lazy<Notation> = Lazy::new(|| {
    let soft_break = || punct(" ") | nl();
    repeat(RepeatInner {
        empty: punct(""),
        lone: punct("    ") + child(0),
        join: left() + punct(",") + soft_break() + right(),
        surround: punct("    ") + surrounded(),
    })
});
static PARAGRAPH_NOTATION: Lazy<Notation> = Lazy::new(|| punct("¶") + child(0) + punct("□"));

enum Contents<'d> {
    Text(&'d str),
    Children(&'d [FlowWrap]),
}

impl FlowWrap {
    fn contents(&self) -> Contents {
        use Contents::{Children, Text};
        use FlowWrapData::*;

        match &self.data {
            Word(txt) => Text(txt),
            Words(words) => Children(words),
            Paragraph(para) => Children(&**para),
        }
    }
}

impl<'d> PrettyDoc<'d> for &'d FlowWrap {
    type Id = usize;

    fn id(self) -> usize {
        self.id
    }

    fn notation(self) -> &'d Notation {
        use FlowWrapData::*;

        match &self.data {
            Word(_) => &WORD_NOTATION,
            Words(_) => &WORDS_NOTATION,
            Paragraph(_) => &PARAGRAPH_NOTATION,
        }
    }

    fn num_children(self) -> Option<usize> {
        match self.contents() {
            Contents::Text(_) => None,
            Contents::Children(slice) => Some(slice.len()),
        }
    }

    fn unwrap_text(self) -> &'d str {
        match self.contents() {
            Contents::Text(txt) => txt,
            Contents::Children(_) => unreachable!(),
        }
    }

    fn unwrap_child(self, i: usize) -> Self {
        match self.contents() {
            Contents::Text(_) => unreachable!(),
            Contents::Children(slice) => &slice[i],
        }
    }
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn new_node(data: FlowWrapData) -> FlowWrap {
    FlowWrap {
        id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
        data,
    }
}

fn paragraph(words: &[&str]) -> FlowWrap {
    use FlowWrapData::*;

    let children = words
        .iter()
        .map(|w| new_node(Word((*w).to_owned())))
        .collect::<Vec<_>>();
    new_node(Paragraph(Box::new([new_node(Words(children))])))
}

#[test]
fn flow_wrap() {
    // from https://github.com/rust-lang/rust/blob/master/src/test/ui/bastion-of-the-turbofish.rs

    let doc = paragraph(&[
        "Oh",
        "woe",
        "is",
        "me",
        "the",
        "turbofish",
        "remains",
        "undefeated",
    ]);

    assert_pp(
        &doc,
        80,
        //0    5   10   15   20   25   30   35   40   45   50   55   60
        &["¶    Oh, woe, is, me, the, turbofish, remains, undefeated□"],
    );
    assert_pp(
        &doc,
        46,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "¶    Oh, woe, is, me, the, turbofish, remains,",
            "undefeated□",
        ],
    );
    assert_pp(
        &doc,
        45,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "¶    Oh, woe, is, me, the, turbofish,",
            "remains, undefeated□",
        ],
    );
    assert_pp(
        &doc,
        20,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "¶    Oh, woe, is,",
            "me, the, turbofish,",
            "remains, undefeated□",
        ],
    );
    assert_pp(
        &doc,
        19,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "¶    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
            "undefeated□",
        ],
    );
    assert_pp(
        &doc,
        18,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "¶    Oh, woe, is,",
            "me, the,",
            "turbofish,",
            "remains,",
            "undefeated□",
        ],
    );
    assert_pp(
        &doc,
        15,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "¶    Oh, woe,",
            "is, me, the,",
            "turbofish,",
            "remains,",
            "undefeated□",
        ],
    );
    assert_pp(
        &doc,
        0,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "¶    Oh,",
            "woe,",
            "is,",
            "me,",
            "the,",
            "turbofish,",
            "remains,",
            "undefeated□",
        ],
    );

    // Explicit seeking tests
    assert_pp_seek(
        &doc,
        19,
        &[0, 2],
        &[],
        &[
            // force rustfmt
            "¶    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
            "undefeated□",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 3],
        &[
            // force rustfmt
            "¶    Oh, woe, is,",
        ],
        &[
            // force rustfmt
            "me, the, turbofish,",
            "remains,",
            "undefeated□",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 4],
        &[
            // force rustfmt
            "¶    Oh, woe, is,",
        ],
        &[
            // force rustfmt
            "me, the, turbofish,",
            "remains,",
            "undefeated□",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 5],
        &[
            // force rustfmt
            "¶    Oh, woe, is,",
        ],
        &[
            // force rustfmt
            "me, the, turbofish,",
            "remains,",
            "undefeated□",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 6],
        &[
            // force rustfmt
            "¶    Oh, woe, is,",
            "me, the, turbofish,",
        ],
        &[
            // force rustfmt
            "remains,",
            "undefeated□",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 7],
        &[
            // force rustfmt
            "¶    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
        ],
        &[
            // force rustfmt
            "undefeated□",
        ],
    );
}
