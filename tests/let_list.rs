mod common;

use common::{assert_pp, punct};
use once_cell::sync::Lazy;
use partial_pretty_printer::examples::{Doc, Sort};
use partial_pretty_printer::notation_constructors::{
    child, left, nl, repeat, right, surrounded, text,
};
use partial_pretty_printer::{Notation, RepeatInner, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LetList {
    Num,
    Var,
    Phi,
    List,
    Let,
}

static VAR_NOTATION: Lazy<Notation> = Lazy::new(|| text(Style::plain()));
static NUM_NOTATION: Lazy<Notation> = Lazy::new(|| text(Style::plain()));
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
            let multi = punct("[") + (4 >> surrounded()) ^ punct("]");
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

fn list(elements: Vec<Doc<LetList>>) -> Doc<LetList> {
    Doc::new_node(LetList::List, elements)
}

fn make_let(var_name: &str, defn: Doc<LetList>) -> Doc<LetList> {
    Doc::new_node(LetList::Let, vec![var(var_name), defn])
}

// TODO: Add a way to get this to not share lines
fn phi() -> Doc<LetList> {
    Doc::new_node(LetList::Phi, vec![])
}

fn num(n: &str) -> Doc<LetList> {
    Doc::new_text(LetList::Num, n.to_owned())
}

fn var(v: &str) -> Doc<LetList> {
    Doc::new_text(LetList::Var, v.to_owned())
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
