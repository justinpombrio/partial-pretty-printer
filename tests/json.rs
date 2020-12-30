mod common;

use common::{assert_pp, child, flat, lit, nl, Tree};
use partial_pretty_printer::Notation;

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
    let empty = lit("[]");
    let lone = |elem| lit("[") + elem + lit("]");
    let join = |elem: Notation, accum: Notation| elem + lit(",") + (lit(" ") | nl()) + accum;
    let surround = |accum: Notation| {
        let single = lit("[") + flat(accum.clone()) + lit("]");
        let multi = lit("[") + (4 >> accum) ^ lit("]");
        single | multi
    };
    let notation = Notation::repeat(elements.len(), empty, lone, join, surround);
    Tree::new_branch(notation, elements)
}

fn json_dict(entries: Vec<Tree>) -> Tree {
    let empty = lit("{}");
    let lone = |elem: Notation| {
        let single = lit("{") + elem.clone() + lit("}");
        let multi = lit("{") + (4 >> elem) ^ lit("}");
        single | multi
    };
    let join = |elem: Notation, accum: Notation| elem + lit(",") + nl() + accum;
    let surround = |accum: Notation| lit("{") + (4 >> accum) ^ lit("}");
    let notation = Notation::repeat(entries.len(), empty, lone, join, surround);
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
