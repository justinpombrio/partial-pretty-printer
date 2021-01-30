mod common;

use common::{all_paths, assert_pp, punct, SimpleDoc};
use partial_pretty_printer::notation_constructors::flat;
use partial_pretty_printer::Notation;

#[test]
fn basics_empty() {
    let notation = Notation::Empty;
    assert_pp(&SimpleDoc(notation), 80, &[""]);
}

#[test]
fn basics_literal() {
    let notation = punct("Hello world!");
    assert_pp(&SimpleDoc(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_concat() {
    let notation = punct("Hello") + punct(" world!");
    assert_pp(&SimpleDoc(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_newline() {
    let notation = punct("Hello") ^ punct("world!");
    assert_pp(&SimpleDoc(notation), 80, &["Hello", "world!"]);
}

#[test]
fn basics_indent() {
    let notation = punct("Hello") + (2 >> punct("world!"));
    assert_pp(&SimpleDoc(notation), 80, &["Hello", "  world!"]);
}

#[test]
fn basics_flat() {
    let notation = flat((punct("a") ^ punct("b")) | punct("long"));
    assert_pp(&SimpleDoc(notation), 2, &["long"]);
}

#[test]
fn basics_choice() {
    let notation = punct("Hello world!") | punct("Hello") ^ punct("world!");
    assert_pp(&SimpleDoc(notation.clone()), 12, &["Hello world!"]);
    assert_pp(&SimpleDoc(notation), 11, &["Hello", "world!"]);
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
