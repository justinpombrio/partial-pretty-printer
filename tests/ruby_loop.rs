mod common;

use common::{assert_pp, punct};
use once_cell::sync::Lazy;
use partial_pretty_printer::examples::{Doc, Sort};
use partial_pretty_printer::notation_constructors::{child, flat, text};
use partial_pretty_printer::{Notation, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ruby {
    Var,
    MethodCall,
    DoLoop,
}

impl Sort for Ruby {
    fn notation(self) -> &'static Notation {
        use Ruby::*;
        match self {
            Var => &VAR_NOTATION,
            MethodCall => &METHOD_CALL_NOTATION,
            DoLoop => &DO_LOOP_NOTATION,
        }
    }
}

static VAR_NOTATION: Lazy<Notation> = Lazy::new(|| text(Style::plain()));
static METHOD_CALL_NOTATION: Lazy<Notation> = Lazy::new(|| {
    let single = punct(".") + child(1) + punct(" ") + child(2);
    let two_lines = punct(".") + child(1) + punct(" ") + child(2);
    let multi = punct(".") + child(1) + (4 >> child(2));
    child(0) + (single | (4 >> (two_lines | multi)))
});
static DO_LOOP_NOTATION: Lazy<Notation> = Lazy::new(|| {
    let single = punct("do |") + child(0) + punct("| ") + flat(child(1)) + punct(" end");
    let multi = punct("do |") + child(0) + punct("|") + (4 >> child(1)) ^ punct("end");
    single | multi
});

fn method_call(obj: Doc<Ruby>, method: &str, arg: Doc<Ruby>) -> Doc<Ruby> {
    Doc::new_node(Ruby::MethodCall, vec![obj, var(method), arg])
}

fn do_loop(var_name: &str, body: Doc<Ruby>) -> Doc<Ruby> {
    Doc::new_node(Ruby::DoLoop, vec![var(var_name), body])
}

fn var(var_name: &str) -> Doc<Ruby> {
    Doc::new_text(Ruby::Var, var_name.to_owned())
}

#[test]
fn ruby_loop() {
    let doc = method_call(var("(1..5)"), "each", do_loop("i", var("puts i")));
    assert_pp(
        &doc,
        30,
        //  0    5   10   15   20   25   30   35   40
        &[
            // force rustfmt
            "(1..5).each do |i| puts i end",
        ],
    );
    assert_pp(
        &doc,
        20,
        //  0    5   10   15   20   25   30   35   40
        &[
            // force rustfmt
            "(1..5).each do |i|",
            "    puts i",
            "end",
        ],
    );
    assert_pp(
        &doc,
        15,
        //  0    5   10   15   20   25   30   35   40
        &[
            // force rustfmt
            "(1..5)",
            "    .each",
            "        do |i|",
            "            puts i",
            "        end",
        ],
    );
}
