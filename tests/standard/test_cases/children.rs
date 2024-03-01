use crate::standard::pretty_testing::assert_pp;
use once_cell::sync::Lazy;
use partial_pretty_printer::examples::tree::{Tree, TreeCondition, TreeNotation};
use partial_pretty_printer::examples::BasicStyle;
use partial_pretty_printer::notation_constructors::{
    check, child, count, empty, fold, left, lit, right, text, Count, Fold,
};
use partial_pretty_printer::CheckPos;

static TEXT_NOTATION: Lazy<TreeNotation> = Lazy::new(|| text().validate().unwrap());

fn element(s: &str) -> Tree<BasicStyle> {
    Tree::new_text(&TEXT_NOTATION, s.to_owned())
}

#[test]
fn test_count() {
    static COUNT_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
        count(Count {
            zero: lit("0"),
            one: lit("1: ") + child(0),
            many: lit("n: ")
                + fold(Fold {
                    first: child(0),
                    join: left() + lit(", ") + right(),
                }),
        })
        .validate()
        .unwrap()
    });

    assert_pp(
        &Tree::<BasicStyle>::new_branch(&COUNT_NOTATION, vec![]),
        80,
        &["0"],
    );
    assert_pp(
        &Tree::new_branch(&COUNT_NOTATION, vec![element("aaaa")]),
        80,
        &["1: aaaa"],
    );
    assert_pp(
        &Tree::new_branch(&COUNT_NOTATION, vec![element("aaaa"), element("bbbb")]),
        80,
        &["n: aaaa, bbbb"],
    );
}

#[test]
fn test_fold() {
    static FOLD_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
        (lit("[")
            + (2 >> fold(Fold {
                first: lit("first: ") + child(0),
                join: left() ^ lit("later: ") + right(),
            }))
            ^ lit("]"))
        .validate()
        .unwrap()
    });

    assert_pp(
        &Tree::new_branch(
            &FOLD_NOTATION,
            vec![element("aaaa"), element("bbbb"), element("cccc")],
        ),
        80,
        &[
            // force rustfmt
            "[",
            "  first: aaaa",
            "  later: bbbb",
            "  later: cccc",
            "]",
        ],
    );

    assert_pp(
        &Tree::new_branch(&FOLD_NOTATION, vec![element("aaaa")]),
        80,
        &[
            // force rustfmt
            "[",
            "  first: aaaa",
            "]",
        ],
    );

    assert_pp(
        &Tree::<BasicStyle>::new_branch(&FOLD_NOTATION, vec![]),
        80,
        // This is why you should use fold inside count...
        &["[", "  ", "]"],
    );
}

#[test]
fn test_condition_positions() {
    static COMMENT_LIST_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
        let cond = TreeCondition::IsComment;
        let list_seq = fold(Fold {
            first: check(cond, CheckPos::Child(0), lit("/* "), empty()) + child(0),
            join: left() + check(cond, CheckPos::LeftChild, lit(" */"), empty())
                ^ check(cond, CheckPos::RightChild, lit("/* "), empty()) + right(),
        });
        let list_note = list_seq + check(cond, CheckPos::Child(-1), lit(" */"), empty());
        list_note.validate().unwrap()
    });

    let doc = Tree::new_branch(
        &COMMENT_LIST_NOTATION,
        vec![
            element("aaaa").into_comment(),
            element("bbbb"),
            element("cccc"),
            element("dddd").into_comment(),
        ],
    );
    assert_pp(
        &doc,
        80,
        &[
            // force rustfmt
            "/* aaaa */",
            "bbbb",
            "cccc",
            "/* dddd */",
        ],
    );

    let one_comment =
        Tree::new_branch(&COMMENT_LIST_NOTATION, vec![element("aaaa").into_comment()]);
    assert_pp(
        &one_comment,
        80,
        &[
            // force rustfmt
            "/* aaaa */",
        ],
    );
}
