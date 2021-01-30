mod common;

use common::{assert_pp, assert_pp_seek, punct};
use once_cell::sync::Lazy;
use partial_pretty_printer::examples::{Doc, Sort};
use partial_pretty_printer::notation_constructors::{
    child, left, nl, repeat, right, surrounded, text,
};
use partial_pretty_printer::{Notation, RepeatInner, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowWrap {
    Word,
    Words,
    Paragraph,
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

impl Sort for FlowWrap {
    fn notation(self) -> &'static Notation {
        use FlowWrap::*;
        match self {
            Word => &WORD_NOTATION,
            Words => &WORDS_NOTATION,
            Paragraph => &PARAGRAPH_NOTATION,
        }
    }
}

fn paragraph(words: &[&str]) -> Doc<FlowWrap> {
    let children = words
        .into_iter()
        .map(|w| Doc::new_text(FlowWrap::Word, (*w).to_owned()))
        .collect::<Vec<_>>();
    Doc::new_node(
        FlowWrap::Paragraph,
        vec![Doc::new_node(FlowWrap::Words, children)],
    )
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
