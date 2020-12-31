mod common;

use common::{assert_pp, child, flat, left, lit, nl, repeat, right, surrounded, Tree};
use partial_pretty_printer::RepeatInner;

fn json_string(s: &str) -> Tree {
    // Using single quote instead of double quote to avoid inconvenient
    // escaping
    Tree::new_leaf(lit("'") + lit(s) + lit("'"))
}

fn json_number(n: &str) -> Tree {
    Tree::new_leaf(lit(n))
}

fn json_entry(key: &str, value: Tree) -> Tree {
    let notation = lit("'") + lit(key) + lit("': ") + child(0);
    Tree::new_branch(notation, vec![value])
}

fn json_list(elements: Vec<Tree>) -> Tree {
    let notation = repeat(RepeatInner {
        empty: lit("[]"),
        lone: lit("[") + child(0) + lit("]"),
        join: "(" + left() + lit(",") + (lit(" ") | nl()) + right() + ")",
        surround: {
            let single = lit("[") + flat(surrounded()) + lit("]");
            let multi = lit("[") + (4 >> surrounded()) ^ lit("]");
            single | multi
        },
    });
    Tree::new_branch(notation, elements)
}

fn json_dict(entries: Vec<Tree>) -> Tree {
    let notation = repeat(RepeatInner {
        empty: lit("{}"),
        lone: {
            let single = lit("{") + left() + lit("}");
            let multi = lit("{") + (4 >> left()) ^ lit("}");
            single | multi
        },
        join: left() + lit(",") + nl() + right(),
        surround: lit("{") + (4 >> surrounded()) ^ lit("}"),
    });
    Tree::new_branch(notation, entries)
}

fn entry_1() -> Tree {
    json_entry("Name", json_string("Alice"))
}

fn entry_2() -> Tree {
    json_entry("Age", json_number("42"))
}

fn favorites_list() -> Tree {
    json_list(vec![
        json_string("chocolate"),
        json_string("lemon"),
        json_string("almond"),
    ])
}

fn entry_3() -> Tree {
    json_entry("Favorites", favorites_list())
}

fn dictionary() -> Tree {
    json_dict(vec![entry_1(), entry_2(), entry_3()])
}

#[test]
fn json_small_dict() {
    assert_pp(
        &json_dict(vec![entry_1(), entry_2()]),
        80,
        &[
            // force rustfmt
            "{",
            "    'Name': 'Alice',",
            "    'Age': 42",
            "}",
        ],
    );
}

#[test]
fn json_flow_wrapped_list() {
    assert_pp(
        &favorites_list(),
        24,
        &[
            // force rustfmt
            "[",
            "    'chocolate',",
            "    'lemon', 'almond'",
            "]",
        ],
    );

    assert_pp(
        &entry_3(),
        27,
        &[
            "'Favorites': [",
            "    'chocolate', 'lemon',",
            "    'almond'",
            "]",
        ],
    );
}

#[test]
fn json_big_dict() {
    assert_pp(
        &dictionary(),
        27,
        &[
            "{",
            "    'Name': 'Alice',",
            "    'Age': 42,",
            "    'Favorites': [",
            "        'chocolate',",
            "        'lemon', 'almond'",
            "    ]",
            "}",
        ],
    );

    assert_pp(
        &dictionary(),
        60,
        &[
            "{",
            "    'Name': 'Alice',",
            "    'Age': 42,",
            "    'Favorites': ['chocolate', 'lemon', 'almond']",
            "}",
        ],
    );

    assert_pp(
        &dictionary(),
        40,
        &[
            "{",
            "    'Name': 'Alice',",
            "    'Age': 42,",
            "    'Favorites': [",
            "        'chocolate', 'lemon', 'almond'",
            "    ]",
            "}",
        ],
    );
}
