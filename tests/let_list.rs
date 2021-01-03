mod common;

use common::assert_pp;
use once_cell::sync::Lazy;
use partial_pretty_printer::notation_constructors::{
    child, left, lit, nl, repeat, right, surrounded, text,
};
use partial_pretty_printer::simple_doc::{SimpleDoc, Sort};
use partial_pretty_printer::{Notation, RepeatInner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LetList {
    Num,
    Var,
    Phi,
    List,
    Let,
}

static VAR_NOTATION: Lazy<Notation> = Lazy::new(|| text());
static NUM_NOTATION: Lazy<Notation> = Lazy::new(|| text());
static PHI_NOTATION: Lazy<Notation> =
    Lazy::new(|| lit("1 + sqrt(5)") ^ lit("-----------") ^ lit("     2"));
static LET_NOTATION: Lazy<Notation> =
    Lazy::new(|| lit("let ") + child(0) + lit(" =") + (lit(" ") | nl()) + child(1) + lit(";"));
static LIST_NOTATION: Lazy<Notation> = Lazy::new(|| {
    repeat(RepeatInner {
        empty: lit("[]"),
        lone: lit("[") + child(0) + lit("]"),
        join: left() + lit(",") + (lit(" ") | nl()) + right(),
        surround: {
            let single = lit("[") + surrounded() + lit("]");
            let multi = lit("[") + (4 >> surrounded()) ^ lit("]");
            single | multi
        },
    })
});

impl Sort for LetList {
    fn notation(self) -> &'static Notation {
        use LetList::*;
        match self {
            Var => &VAR_NOTATION,
            Num => &NUM_NOTATION,
            Phi => &PHI_NOTATION,
            Let => &LET_NOTATION,
            List => &LIST_NOTATION,
        }
    }
}

fn list(elements: Vec<SimpleDoc<LetList>>) -> SimpleDoc<LetList> {
    SimpleDoc::new_node(LetList::List, elements)
}

fn make_let(var_name: &str, defn: SimpleDoc<LetList>) -> SimpleDoc<LetList> {
    SimpleDoc::new_node(LetList::Let, vec![var(var_name), defn])
}

// TODO: Add a way to get this to not share lines
fn phi() -> SimpleDoc<LetList> {
    SimpleDoc::new_node(LetList::Phi, vec![])
}

fn num(n: &str) -> SimpleDoc<LetList> {
    SimpleDoc::new_text(LetList::Num, n.to_owned())
}

fn var(v: &str) -> SimpleDoc<LetList> {
    SimpleDoc::new_text(LetList::Var, v.to_owned())
}

#[test]
#[ignore]
fn let_list() {
    let doc = make_let(
        "best_numbers",
        list(vec![
            num("1025"),
            num("-58"),
            num("33297"),
            phi(),
            num("1.618281828"),
            num("23"),
        ]),
    );

    assert_pp(&doc, 80, &[""]);
}
