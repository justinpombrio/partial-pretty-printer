mod common;

use common::assert_pp;
use once_cell::sync::Lazy;
use partial_pretty_printer::examples::{Doc, Sort};
use partial_pretty_printer::notation_constructors::{child, flat, lit, text};
use partial_pretty_printer::Notation;

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

static VAR_NOTATION: Lazy<Notation> = Lazy::new(|| text());
static METHOD_CALL_NOTATION: Lazy<Notation> = Lazy::new(|| {
    let single = lit(".") + child(1) + lit(" ") + child(2);
    let two_lines = lit(".") + child(1) + lit(" ") + child(2);
    let multi = lit(".") + child(1) + (4 >> child(2));
    child(0) + (single | (4 >> (two_lines | multi)))
});
static DO_LOOP_NOTATION: Lazy<Notation> = Lazy::new(|| {
    let single = lit("do |") + child(0) + lit("| ") + flat(child(1)) + lit(" end");
    let multi = lit("do |") + child(0) + lit("|") + (4 >> child(1)) ^ lit("end");
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
