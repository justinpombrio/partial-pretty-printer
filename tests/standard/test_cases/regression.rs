//! Test cases for previous bugs.

use crate::standard::pretty_testing::{assert_pp, assert_pp_without_expectation, punct, SimpleDoc};
use partial_pretty_printer::{notation_constructors::nl, Notation};

#[test]
fn regression_1() {
    let notation = punct("bb") + (Notation::Empty | nl());
    assert_pp(&SimpleDoc(notation), 1, &["bb", ""]);
}

#[test]
fn regression_2() {
    let notation = (Notation::Empty | punct("bb")) | nl();
    assert_pp(&SimpleDoc(notation), 1, &[""]);
}

#[test]
fn regression_3() {
    let notation = (punct("a") | punct("bb")) + punct("a") | punct("cccc");
    assert_pp_without_expectation(&SimpleDoc(notation), 2);
}
