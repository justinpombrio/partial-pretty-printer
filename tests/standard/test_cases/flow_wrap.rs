use crate::standard::pretty_testing::{assert_pp, assert_pp_seek};
use once_cell::sync::Lazy;
use partial_pretty_printer::notation_constructors::{
    child, count, fold, left, lit, nl, right, text, Count, Fold,
};
use partial_pretty_printer::{PrettyDoc, ValidNotation};
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

const START: &str = "始";
const END: &str = "端";

static WORD_NOTATION: Lazy<ValidNotation<(), ()>> = Lazy::new(|| text().validate().unwrap());
static WORDS_NOTATION: Lazy<ValidNotation<(), ()>> = Lazy::new(|| {
    let soft_break = lit(" ") | nl();
    count(Count {
        zero: lit(""),
        one: lit("    ") + child(0),
        many: lit("    ")
            + fold(Fold {
                first: child(0),
                join: left() + lit(",") + soft_break + right(),
            }),
    })
    .validate()
    .unwrap()
});
static PARAGRAPH_NOTATION: Lazy<ValidNotation<(), ()>> =
    Lazy::new(|| (lit(START) + child(0) + lit(END)).validate().unwrap());

enum Contents<'d> {
    Text(&'d str),
    Children(&'d [FlowWrap]),
}

impl FlowWrap {
    fn contents(&self) -> Contents<'_> {
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
    type StyleLabel = ();
    type Condition = ();
    type Error = std::convert::Infallible;

    fn id(self) -> Result<usize, Self::Error> {
        Ok(self.id)
    }

    fn notation(self) -> Result<&'d ValidNotation<(), ()>, Self::Error> {
        use FlowWrapData::*;

        Ok(match &self.data {
            Word(_) => &WORD_NOTATION,
            Words(_) => &WORDS_NOTATION,
            Paragraph(_) => &PARAGRAPH_NOTATION,
        })
    }

    fn condition(self, _condition: &()) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn node_style(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn lookup_style(self, _label: ()) -> Result<(), Self::Error> {
        Ok(())
    }

    fn num_children(self) -> Result<Option<usize>, Self::Error> {
        Ok(match self.contents() {
            Contents::Text(_) => None,
            Contents::Children(slice) => Some(slice.len()),
        })
    }

    fn unwrap_text(self) -> Result<&'d str, Self::Error> {
        Ok(match self.contents() {
            Contents::Text(txt) => txt,
            Contents::Children(_) => unreachable!(),
        })
    }

    fn unwrap_child(self, i: usize) -> Result<Self, Self::Error> {
        Ok(match self.contents() {
            Contents::Text(_) => unreachable!(),
            Contents::Children(slice) => &slice[i],
        })
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
        &[],
        &[
            // force rustfmt
            "(始    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
            "undefeated端)",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0],
        &[
            // force rustfmt
            "始(    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
            "undefeated)端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 0],
        &[
            // force rustfmt
            "始    (Oh), woe, is,",
            "me, the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 1],
        &[
            // force rustfmt
            "始    Oh, (woe), is,",
            "me, the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 2],
        &[
            // force rustfmt
            "始    Oh, woe, (is),",
            "me, the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 3],
        &[
            // force rustfmt
            "始    Oh, woe, is,",
            "(me), the, turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 4],
        &[
            // force rustfmt
            "始    Oh, woe, is,",
            "me, (the), turbofish,",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 5],
        &[
            // force rustfmt
            "始    Oh, woe, is,",
            "me, the, (turbofish),",
            "remains,",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 6],
        &[
            // force rustfmt
            "始    Oh, woe, is,",
            "me, the, turbofish,",
            "(remains),",
            "undefeated端",
        ],
    );
    assert_pp_seek(
        &doc,
        19,
        &[0, 7],
        &[
            // force rustfmt
            "始    Oh, woe, is,",
            "me, the, turbofish,",
            "remains,",
            "(undefeated)端",
        ],
    );
}
