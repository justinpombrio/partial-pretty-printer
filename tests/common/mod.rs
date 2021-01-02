#![allow(unused)]

use partial_pretty_printer::{pretty_print, Doc, Notation, RepeatInner};
use std::sync::atomic::{AtomicUsize, Ordering};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct Tree {
    id: usize,
    notation: Notation,
    children: Vec<Tree>,
}

impl Tree {
    pub fn new_branch(notation: Notation, children: Vec<Tree>) -> Tree {
        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Tree {
            id,
            notation,
            children,
        }
    }

    pub fn new_leaf(notation: Notation) -> Tree {
        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Tree {
            id,
            notation,
            children: vec![],
        }
    }
}

impl Doc for Tree {
    type Id = usize;

    fn id(&self) -> usize {
        self.id
    }

    fn notation(&self) -> &Notation {
        &self.notation
    }

    fn child(&self, i: usize) -> &Tree {
        &self.children[i]
    }

    fn num_children(&self) -> usize {
        self.children.len()
    }
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

fn print_above_and_below<D: Doc>(
    doc: &D,
    width: usize,
    path: &[usize],
) -> (Vec<String>, Vec<String>) {
    let path_iter = path.into_iter().map(|i| *i);
    let (upward_printer, downward_printer) = pretty_print(doc, width, path_iter);
    let mut lines_above = upward_printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .collect::<Vec<_>>();
    lines_above.reverse();
    let lines_below = downward_printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .collect::<Vec<_>>();
    (lines_above, lines_below)
}

fn all_paths<D: Doc>(doc: &D) -> Vec<Vec<usize>> {
    fn recur<D: Doc>(doc: &D, path: &mut Vec<usize>, paths: &mut Vec<Vec<usize>>) {
        paths.push(path.clone());
        for i in 0..doc.num_children() {
            path.push(i);
            recur(doc.child(i), path, paths);
            path.pop();
        }
    }
    let mut paths = vec![];
    recur(doc, &mut vec![], &mut paths);
    paths
}

pub fn print_region<D: Doc>(doc: &D, width: usize, path: &[usize], rows: usize) -> Vec<String> {
    let path_iter = path.into_iter().map(|i| *i);
    let (upward_printer, downward_printer) = pretty_print(doc, width, path_iter);
    let mut lines = upward_printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .take(rows / 2)
        .collect::<Vec<_>>();
    lines.reverse();
    let mut lines_below = downward_printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .take(rows / 2)
        .collect::<Vec<_>>();
    lines.append(&mut lines_below);
    lines
}

#[track_caller]
pub fn assert_pp<D: Doc>(doc: &D, width: usize, expected_lines: &[&str]) {
    for path in all_paths(doc) {
        let (lines_above, mut lines_below) = print_above_and_below(doc, width, &path);
        let mut lines = lines_above;
        lines.append(&mut lines_below);
        compare_lines(
            &format!("IN PRETTY PRINTING AT PATH {:?}", path),
            &lines,
            expected_lines,
        );
    }
}

#[track_caller]
pub fn assert_pp_seek<D: Doc>(
    doc: &D,
    width: usize,
    path: &[usize],
    expected_lines_above: &[&str],
    expected_lines_below: &[&str],
) {
    let (lines_above, lines_below) = print_above_and_below(doc, width, path);
    compare_lines(
        &format!("IN DOWNWARD PRINTING AT PATH {:?}", path),
        &lines_below,
        expected_lines_below,
    );
    compare_lines(
        &format!("IN UPWARD PRINTING AT PATH {:?}", path),
        &lines_above,
        expected_lines_above,
    );
}

#[test]
fn test_all_paths_fn() {
    let br = |children: Vec<Tree>| -> Tree { Tree::new_branch(Notation::Empty, children) };
    let leaf = || -> Tree { Tree::new_leaf(Notation::Empty) };
    let doc = br(vec![
        br(vec![leaf(), leaf()]),
        leaf(),
        br(vec![br(vec![leaf()]), leaf()]),
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
