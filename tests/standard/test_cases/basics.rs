use crate::standard::pretty_testing::{all_paths, assert_pp, punct, SimpleDoc};
use partial_pretty_printer::{
    notation_constructors::{flat, group, if_flat, nl, ws},
    Notation,
};

#[test]
fn basics_empty() {
    let notation = Notation::Empty;
    assert_pp(&SimpleDoc::new(notation), 80, &[""]);
}

#[test]
fn basics_literal() {
    let notation = punct("Hello world!");
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_concat() {
    let notation = punct("Hello") + punct(" world!");
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_newline() {
    let notation = punct("Hello") ^ punct("world!");
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello", "world!"]);
}

#[test]
fn basics_indent() {
    let notation = punct("Hello") + (2 >> punct("world!"));
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello", "  world!"]);
}

#[test]
fn basics_flat() {
    let notation = flat(punct("long") | (punct("a") ^ punct("b")));
    assert_pp(&SimpleDoc::new(notation), 2, &["long"]);
}

#[test]
fn basics_choice() {
    let notation = punct("Hello world!") | punct("Hello") ^ punct("world!");
    assert_pp(&SimpleDoc::new(notation.clone()), 12, &["Hello world!"]);
    assert_pp(&SimpleDoc::new(notation), 11, &["Hello", "world!"]);
}

#[test]
fn basics_if_flat() {
    let notation = if_flat(punct("aaa"), punct("bbb"));
    assert_pp(&SimpleDoc::new(notation.clone()), 5, &["bbb"]);
    assert_pp(&SimpleDoc::new(notation), 2, &["bbb"]);

    let notation = flat(if_flat(punct("aaa"), punct("bbb")));
    assert_pp(&SimpleDoc::new(notation.clone()), 5, &["aaa"]);
    assert_pp(&SimpleDoc::new(notation), 2, &["aaa"]);
}

#[test]
fn basics_ws() {
    assert_pp(&SimpleDoc::new(ws("  ")), 5, &["", ""]);
    assert_pp(&SimpleDoc::new(ws("  ")), 1, &["", ""]);
    assert_pp(&SimpleDoc::new(flat(ws("  "))), 5, &["  "]);
    assert_pp(&SimpleDoc::new(flat(ws("  "))), 1, &["  "]);
}

#[test]
fn basics_group() {
    let notation = group(punct("Hello") + ws(" ") + punct("dear") + ws(" ") + punct("world"));
    assert_pp(&SimpleDoc::new(notation.clone()), 50, &["Hello dear world"]);
    assert_pp(&SimpleDoc::new(notation), 10, &["Hello", "dear", "world"]);

    let notation = punct("(") + group(ws(" ")) + punct(")");
    assert_pp(&SimpleDoc::new(notation.clone()), 50, &["( )"]);
    assert_pp(&SimpleDoc::new(notation), 1, &["(", ")"]);
}

#[test]
fn test_all_paths_fn() {
    use partial_pretty_printer::examples::json::{json_list, json_string};
    let doc = json_list(vec![
        json_list(vec![json_string("0.0"), json_string("0.1")]),
        json_string("1"),
        json_list(vec![
            json_list(vec![json_string("2.0.0")]),
            json_string("2.1"),
        ]),
    ]);
    assert_eq!(
        all_paths(&doc),
        vec![
            vec![],
            vec![0],
            vec![0, 0],
            vec![0, 1],
            vec![1],
            vec![2],
            vec![2, 0],
            vec![2, 0, 0],
            vec![2, 1]
        ]
    );
}

// Implementation purposefully disagrees with Oracle
#[test]
#[ignore]
fn hidden_error() {
    let notation = punct("x") ^ flat(nl()) | punct("ok");
    assert_pp(&SimpleDoc::new(notation), 3, &["x", "", ""]);
}

// Implementation purposefully disagrees with Oracle
#[test]
#[ignore]
fn tricky_suffix() {
    let notation = (punct("a") | punct("bb")) + ((punct("x") + nl() + flat(nl())) | punct("yy"));
    assert_pp(&SimpleDoc::new(notation), 3, &["ax", "", ""]);
}
