mod common;

use common::assert_pp;
use partial_pretty_printer::notation_constructors::{flat, lit};
use partial_pretty_printer::{Notation, PrettyDoc, PrettyDocContents};

struct SimpleDoc(Notation);

impl PrettyDoc for SimpleDoc {
    type Id = usize;

    fn id(&self) -> usize {
        0
    }

    fn notation(&self) -> &Notation {
        &self.0
    }

    fn contents(&self) -> PrettyDocContents<SimpleDoc> {
        PrettyDocContents::Children(&[])
    }
}

#[test]
fn basics_empty() {
    let notation = Notation::Empty;
    assert_pp(&SimpleDoc(notation), 80, &[""]);
}

#[test]
fn basics_literal() {
    let notation = lit("Hello world!");
    assert_pp(&SimpleDoc(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_concat() {
    let notation = lit("Hello") + lit(" world!");
    assert_pp(&SimpleDoc(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_newline() {
    let notation = lit("Hello") ^ lit("world!");
    assert_pp(&SimpleDoc(notation), 80, &["Hello", "world!"]);
}

#[test]
fn basics_indent() {
    let notation = lit("Hello") + (2 >> lit("world!"));
    assert_pp(&SimpleDoc(notation), 80, &["Hello", "  world!"]);
}

#[test]
fn basics_flat() {
    let notation = flat((lit("a") ^ lit("b")) | lit("long"));
    assert_pp(&SimpleDoc(notation), 2, &["long"]);
}

#[test]
fn basics_choice() {
    let notation = lit("Hello world!") | lit("Hello") ^ lit("world!");
    assert_pp(&SimpleDoc(notation.clone()), 12, &["Hello world!"]);
    assert_pp(&SimpleDoc(notation), 11, &["Hello", "world!"]);
}
