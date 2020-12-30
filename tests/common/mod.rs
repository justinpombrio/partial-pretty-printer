#![allow(unused)]

use partial_pretty_printer::{print_downward_for_testing, print_upward_for_testing, Doc, Notation};

#[derive(Debug, Clone)]
pub struct Tree {
    notation: Notation,
    children: Vec<Tree>,
}

impl Tree {
    pub fn new_branch(notation: Notation, children: Vec<Tree>) -> Tree {
        Tree { notation, children }
    }

    pub fn new_leaf(notation: Notation) -> Tree {
        Tree {
            notation,
            children: vec![],
        }
    }
}

impl Doc for Tree {
    fn notation(&self) -> &Notation {
        &self.notation
    }

    fn child(&self, i: usize) -> &Tree {
        &self.children[i]
    }
}

pub fn nl() -> Notation {
    Notation::Newline
}

pub fn child(i: usize) -> Notation {
    Notation::Child(i)
}

pub fn lit(s: &str) -> Notation {
    Notation::Literal(s.to_string())
}

pub fn flat(n: Notation) -> Notation {
    Notation::Flat(Box::new(n))
}

fn compare_lines(message: &str, actual: &[String], expected: &[&str]) {
    if actual != expected {
        eprintln!(
            "{}\nEXPECTED:\n{}\nACTUAL:\n{}\n=========",
            message,
            expected.join("\n"),
            actual.join("\n"),
        );
        assert_eq!(actual, expected);
    }
}

#[track_caller]
pub fn assert_pp<D: Doc>(doc: &D, width: usize, expected_lines: &[&str]) {
    let downward_lines = print_downward_for_testing(doc, width);
    compare_lines("IN DOWNWARD PRINTING", &downward_lines, expected_lines);
    let upward_lines = print_upward_for_testing(doc, width);
    compare_lines("IN UPWARD PRINTING", &upward_lines, expected_lines);
}
