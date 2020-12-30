mod common;

use common::{assert_pp, flat, lit, Tree};
use partial_pretty_printer::Notation;

#[test]
fn basics_empty() {
    let notation = Notation::Empty;
    assert_pp(&Tree::new_leaf(notation), 80, &[""]);
}

#[test]
fn basics_literal() {
    let notation = lit("Hello world!");
    assert_pp(&Tree::new_leaf(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_concat() {
    let notation = lit("Hello") + lit(" world!");
    assert_pp(&Tree::new_leaf(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_newline() {
    let notation = lit("Hello") ^ lit("world!");
    assert_pp(&Tree::new_leaf(notation), 80, &["Hello", "world!"]);
}

#[test]
fn basics_indent() {
    let notation = lit("Hello") + (2 >> lit("world!"));
    assert_pp(&Tree::new_leaf(notation), 80, &["Hello", "  world!"]);
}

#[test]
fn basics_flat() {
    let notation = flat((lit("a") ^ lit("b")) | lit("long"));
    assert_pp(&Tree::new_leaf(notation), 2, &["long"]);
}

#[test]
fn basics_choice() {
    let notation = lit("Hello world!") | lit("Hello") ^ lit("world!");
    assert_pp(&Tree::new_leaf(notation.clone()), 12, &["Hello world!"]);
    assert_pp(&Tree::new_leaf(notation), 11, &["Hello", "world!"]);
}
