use crate::standard::pretty_testing::{assert_pp, assert_pp_seek, punct};
use once_cell::sync::Lazy;
use partial_pretty_printer::notation_constructors::{
    child, count, fold, left, nl, right, text, Count, Fold,
};
use partial_pretty_printer::{PrettyDoc, ValidNotation};
use std::sync::atomic::{AtomicUsize, Ordering};

// TODO: test seek_end = true

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

const START: &str = "始";
const END: &str = "端";

static WORD_NOTATION: Lazy<ValidNotation<()>> = Lazy::new(|| text(()).validate().unwrap());
static WORDS_NOTATION: Lazy<ValidNotation<()>> = Lazy::new(|| {
    let soft_break = punct(" ") | nl();
    count(Count {
        zero: punct(""),
        one: punct("    ") + child(0),
        many: punct("    ")
            + fold(Fold {
                first: child(0),
                join: left() + punct(",") + soft_break + right(),
            }),
    })
    .validate()
    .unwrap()
});
static PARAGRAPH_NOTATION: Lazy<ValidNotation<()>> =
    Lazy::new(|| (punct(START) + child(0) + punct(END)).validate().unwrap());

enum Contents<'d> {
    Text(&'d str),
    Children(&'d [FlowWrap]),
}

impl FlowWrap {
    fn contents(&self) -> Contents {
        use FlowWrapData::*;

        match &self.data {
            Word(txt) => Contents::Text(txt),
            Words(words) => Contents::Children(words),
            Paragraph(para) => Contents::Children(&**para),
        }
    }
}

impl<'d> PrettyDoc<'d> for &'d FlowWrap {
    type Id = usize;
    type Style = ();
    type Mark = ();

    fn id(self) -> usize {
        self.id
    }

    fn notation(self) -> &'d ValidNotation<()> {
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

    use partial_pretty_printer::testing::str_width;
    assert_eq!(str_width("始"), 2);
    assert_eq!(str_width("端"), 2);

    assert_pp(
        &doc,
        80,
        //0    5   10   15   20   25   30   35   40   45   50   55   60
        &["始    Oh, woe, is, me, the, turbofish, remains, undefeated端"],
    );
    assert_pp(
        &doc,
        60,
        //0    5   10   15   20   25   30   35   40   45   50   55   60
        &["始    Oh, woe, is, me, the, turbofish, remains, undefeated端"],
    );
    assert_pp(
        &doc,
        59,
        //0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "始    Oh, woe, is, me, the, turbofish, remains,",
            "undefeated端",
        ],
    );
    assert_pp(
        &doc,
        47,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "始    Oh, woe, is, me, the, turbofish, remains,",
            "undefeated端",
        ],
    );
    assert_pp(
        &doc,
        45,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "始    Oh, woe, is, me, the, turbofish,",
            "remains, undefeated端",
        ],
    );
    assert_pp(
        &doc,
        21,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "始    Oh, woe, is,",
            "me, the, turbofish,",
            "remains, undefeated端",
        ],
    );
    assert_pp(
        &doc,
        19,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "始    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp(
        &doc,
        18,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "始    Oh, woe, is,",
            "me, the,",
            "turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp(
        &doc,
        14,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "始    Oh, woe,",
            "is, me, the,",
            "turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp(
        &doc,
        0,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            "始    Oh,",
            "woe,",
            "is,",
            "me,",
            "the,",
            "turbofish,",
            "remains,",
            "undefeated端",
        ],
    );

    // Explicit seeking tests
    assert_pp_seek(
        &doc,
        19,
        &[0, 2],
        false,
        &[],
        &[
            // force rustfmt
            "始    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 3],
        false,
        &[
            // force rustfmt
            "始    Oh, woe, is,",
        ],
        &[
            // force rustfmt
            "me, the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 4],
        false,
        &[
            // force rustfmt
            "始    Oh, woe, is,",
        ],
        &[
            // force rustfmt
            "me, the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 5],
        false,
        &[
            // force rustfmt
            "始    Oh, woe, is,",
        ],
        &[
            // force rustfmt
            "me, the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 6],
        false,
        &[
            // force rustfmt
            "始    Oh, woe, is,",
            "me, the, turbofish,",
        ],
        &[
            // force rustfmt
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 7],
        false,
        &[
            // force rustfmt
            "始    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
        ],
        &[
            // force rustfmt
            "undefeated端",
        ],
    );
}
