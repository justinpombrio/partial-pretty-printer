mod common;

use common::{assert_pp, child, left, lit, nl, repeat, right, surrounded, Tree};
use partial_pretty_printer::RepeatInner;

fn word_flow(words: &[&str]) -> Tree {
    let elements = words
        .iter()
        .map(|w| Tree::new_leaf(lit(w)))
        .collect::<Vec<_>>();
    let soft_break = || lit(" ") | nl();
    let notation = repeat(RepeatInner {
        empty: lit(""),
        lone: lit("    ") + child(0),
        join: left() + lit(",") + soft_break() + right(),
        surround: lit("    ") + surrounded(),
    });
    Tree::new_branch(notation, elements)
}

fn mark_paragraph(paragraph: Tree) -> Tree {
    let notation = lit("¶") + child(0) + lit("□");
    Tree::new_branch(notation, vec![paragraph])
}

#[test]
fn flow_wrap() {
    // from https://github.com/rust-lang/rust/blob/master/src/test/ui/bastion-of-the-turbofish.rs

    let doc = mark_paragraph(word_flow(&[
        "Oh",
        "woe",
        "is",
        "me",
        "the",
        "turbofish",
        "remains",
        "undefeated",
    ]));

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
}
