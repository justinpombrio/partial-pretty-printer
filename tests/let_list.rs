mod common;

use common::{assert_pp, punct};
use once_cell::sync::Lazy;
use partial_pretty_printer::notation_constructors::{
    child, left, nl, repeat, right, surrounded, text,
};
use partial_pretty_printer::{Notation, PrettyDoc, RepeatInner, Style};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
struct LetList {
    id: usize,
    data: LetListData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LetListData {
    Num(String),
    Var(String),
    Phi,
    Let(Box<[LetList; 2]>),
    List(Vec<LetList>),
}

static NUM_NOTATION: Lazy<Notation> = Lazy::new(|| text(Style::plain()));
static VAR_NOTATION: Lazy<Notation> = Lazy::new(|| text(Style::plain()));
static PHI_NOTATION: Lazy<Notation> =
    Lazy::new(|| punct("1 + sqrt(5)") ^ punct("-----------") ^ punct("     2"));
static LET_NOTATION: Lazy<Notation> = Lazy::new(|| {
    punct("let ") + child(0) + punct(" =") + (punct(" ") | nl()) + child(1) + punct(";")
});
static LIST_NOTATION: Lazy<Notation> = Lazy::new(|| {
    repeat(RepeatInner {
        empty: punct("[]"),
        lone: punct("[") + child(0) + punct("]"),
        join: left() + punct(",") + (punct(" ") | nl()) + right(),
        surround: {
            let single = punct("[") + surrounded() + punct("]");
            let multi = (punct("[") + (4 >> surrounded())) ^ punct("]");
            single | multi
        },
    })
});

enum Contents<'d> {
    Text(&'d str),
    Children(&'d [LetList]),
}

impl LetList {
    fn contents(&self) -> Contents {
        use Contents::{Children, Text};
        use LetListData::*;

        match &self.data {
            Num(txt) => Text(txt),
            Var(txt) => Text(txt),
            Phi => Children(&[]),
            List(elems) => Children(elems),
            Let(bind) => Children(&**bind),
        }
    }
}

impl<'d> PrettyDoc<'d> for &'d LetList {
    type Id = usize;

    fn id(self) -> usize {
        self.id
    }

    fn notation(self) -> &'d Notation {
        use LetListData::*;

        match &self.data {
            Num(_) => &NUM_NOTATION,
            Var(_) => &VAR_NOTATION,
            Phi => &PHI_NOTATION,
            List(_) => &LIST_NOTATION,
            Let(_) => &LET_NOTATION,
        }
    }

    fn num_children(self) -> Option<usize> {
        match self.contents() {
            Contents::Text(_) => None,
            Contents::Children(slice) => Some(slice.len()),
        }
    }

    fn unwrap_text(self) -> &'d str {
        match self.contents() {
            Contents::Text(txt) => txt,
            Contents::Children(_) => unreachable!(),
        }
    }

    fn unwrap_child(self, i: usize) -> Self {
        match self.contents() {
            Contents::Text(_) => unreachable!(),
            Contents::Children(slice) => &slice[i],
        }
    }
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn new_node(data: LetListData) -> LetList {
    LetList {
        id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
        data,
    }
}

fn list(elements: Vec<LetList>) -> LetList {
    new_node(LetListData::List(elements))
}

fn make_let(var_name: &str, defn: LetList) -> LetList {
    new_node(LetListData::Let(Box::new([var(var_name), defn])))
}

// TODO: Add a way to get this to not share lines
fn phi() -> LetList {
    new_node(LetListData::Phi)
}

fn num(n: f64) -> LetList {
    new_node(LetListData::Num(n.to_string()))
}

fn var(v: &str) -> LetList {
    new_node(LetListData::Var(v.to_owned()))
}

#[test]
#[ignore]
fn let_list() {
    let doc = make_let(
        "best_numbers",
        list(vec![
            num(1025.0),
            num(-58.0),
            num(33297.0),
            phi(),
            num(1.618281828),
            num(23.0),
        ]),
    );

    assert_pp(&doc, 80, &[""]);
}
