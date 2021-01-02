// TODO: Put most of this somewhere shared

use partial_pretty_printer::notation_constructors::{
    child, flat, left, lit, nl, repeat, right, surrounded,
};
use partial_pretty_printer::{pretty_print, Doc, Notation, RepeatInner};
use std::sync::atomic::{AtomicUsize, Ordering};

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
        join: left() + lit(",") + (lit(" ") | nl()) + right(),
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

fn json_long_list_example() {
    let num_elems = 1000;
    let numbers = (0..num_elems)
        .map(|n| json_number(&format!("{}", n)))
        .collect::<Vec<_>>();
    let list = json_list(numbers);

    for i in 0..100 {
        let lines = print_region(&list, 80, &[num_elems / 2], 60);
        assert_eq!(lines.len(), 60);
    }
}

fn main() {
    json_long_list_example();
}
