mod common;

use common::assert_pp;
use once_cell::sync::Lazy;
use partial_pretty_printer::examples::{Doc, Sort};
use partial_pretty_printer::notation_constructors::{child, flat, lit, text};
use partial_pretty_printer::Notation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IterChain {
    Var,
    MethodCall,
    Closure,
    Times,
}

impl Sort for IterChain {
    fn notation(self) -> &'static Notation {
        use IterChain::*;
        match self {
            Var => &VAR_NOTATION,
            MethodCall => &METHOD_CALL_NOTATION,
            Closure => &CLOSURE_NOTATION,
            Times => &TIMES_NOTATION,
        }
    }
}

static VAR_NOTATION: Lazy<Notation> = Lazy::new(|| text());
static METHOD_CALL_NOTATION: Lazy<Notation> = Lazy::new(|| {
    // foobaxxle.bar(arg)
    //
    // -- Disallowing this layout:
    // foobaxxle.bar(
    //     arg
    // )
    //
    // foobaxxle
    //     .bar(arg)
    //
    // foobaxxle
    //     .bar(
    //         arg
    //      )
    let single = lit(".") + child(1) + lit("(") + flat(child(2).clone()) + lit(")");
    let two_lines = lit(".") + child(1) + lit("(") + flat(child(2).clone()) + lit(")");
    let multi = lit(".") + child(1) + lit("(") + (4 >> child(2)) ^ lit(")");
    child(0) + (single | (4 >> (two_lines | multi)))
});
static CLOSURE_NOTATION: Lazy<Notation> = Lazy::new(|| {
    let single = lit("|") + child(0) + lit("| { ") + child(1) + lit(" }");
    let multi = lit("|") + child(0) + lit("| {") + (4 >> child(1)) ^ lit("}");
    single | multi
});
static TIMES_NOTATION: Lazy<Notation> = Lazy::new(|| child(0) + lit(" * ") + child(1));

fn method_call(obj: Doc<IterChain>, method: &str, arg: Doc<IterChain>) -> Doc<IterChain> {
    Doc::new_node(IterChain::MethodCall, vec![obj, var(method), arg])
}

fn closure(var_name: &str, body: Doc<IterChain>) -> Doc<IterChain> {
    Doc::new_node(IterChain::Closure, vec![var(var_name), body])
}

fn times(arg1: Doc<IterChain>, arg2: Doc<IterChain>) -> Doc<IterChain> {
    Doc::new_node(IterChain::Times, vec![arg1, arg2])
}

fn var(var: &str) -> Doc<IterChain> {
    Doc::new_text(IterChain::Var, var.to_owned())
}

#[test]
fn iter_chain_iter_map_collect() {
    let doc = var("some_vec");
    let doc = method_call(doc, "iter", var(""));
    let doc = method_call(doc, "map", closure("elem", times(var("elem"), var("elem"))));
    let doc = method_call(doc, "collect", var(""));

    assert_pp(
        &doc,
        80,
        //0    5   10   15   20   25   30   35   40   45   50   55   60
        &["some_vec.iter().map(|elem| { elem * elem }).collect()"],
    );
    assert_pp(
        &doc,
        50,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec.iter().map(|elem| { elem * elem })",
            "    .collect()",
        ],
    );
    assert_pp(
        &doc,
        40,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec.iter()",
            "    .map(|elem| { elem * elem })",
            "    .collect()",
        ],
    );
    assert_pp(
        &doc,
        30,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec.iter()",
            "    .map(",
            "        |elem| { elem * elem }",
            "    ).collect()",
        ],
    );
}

#[test]
fn iter_chain_long_method_body() {
    let doc = var("some_vec");
    let doc = method_call(doc, "map", closure("elem", times(var("elem"), var("elem"))));

    assert_pp(
        &doc,
        31,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec",
            "    .map(",
            "        |elem| { elem * elem }",
            "    )",
        ],
    );
}

#[test]
fn iter_chain_long_methods() {
    let doc = var("some_vec");
    let doc = method_call(doc, "call_the_map_method", closure("elem", var("elem")));
    let doc = method_call(doc, "call_the_map_method", closure("elem", var("elem")));

    assert_pp(
        &doc,
        41,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec",
            "    .call_the_map_method(|elem| { elem })",
            "    .call_the_map_method(|elem| { elem })",
        ],
    );
    assert_pp(
        &doc,
        35,
        //  0    5   10   15   20   25   30   35   40   45   50   55   60
        &[
            // force rustfmt
            "some_vec",
            "    .call_the_map_method(",
            "        |elem| { elem }",
            "    )",
            "    .call_the_map_method(",
            "        |elem| { elem }",
            "    )",
        ],
    );
}
