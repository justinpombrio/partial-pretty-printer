use crate::standard::pretty_testing::{assert_pp, punct};
use once_cell::sync::Lazy;
use partial_pretty_printer::notation_constructors::{child, flat, text};
use partial_pretty_printer::{PrettyDoc, ValidNotation};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
struct IterChain {
    id: usize,
    data: IterChainData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IterChainData {
    Var(String),
    MethodCall(Box<[IterChain; 3]>),
    Closure(Box<[IterChain; 2]>),
    Times(Box<[IterChain; 2]>),
}

static VAR_NOTATION: Lazy<ValidNotation<()>> = Lazy::new(|| text(()).validate().unwrap());
static METHOD_CALL_NOTATION: Lazy<ValidNotation<()>> = Lazy::new(|| {
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
    let single = punct(".") + child(1) + punct("(") + flat(child(2)) + punct(")");
    let two_lines = punct(".") + child(1) + punct("(") + flat(child(2)) + punct(")");
    let multi = (punct(".") + child(1) + punct("(") + (4 >> child(2))) ^ punct(")");
    (child(0) + (single | (4 >> (two_lines | multi))))
        .validate()
        .unwrap()
});
static CLOSURE_NOTATION: Lazy<ValidNotation<()>> = Lazy::new(|| {
    let single = punct("|") + child(0) + punct("| { ") + child(1) + punct(" }");
    let multi = (punct("|") + child(0) + punct("| {") + (4 >> child(1))) ^ punct("}");
    (single | multi).validate().unwrap()
});
static TIMES_NOTATION: Lazy<ValidNotation<()>> =
    Lazy::new(|| (child(0) + punct(" * ") + child(1)).validate().unwrap());

enum Contents<'d> {
    Text(&'d str),
    Children(&'d [IterChain]),
}

impl IterChain {
    fn contents(&self) -> Contents {
        use Contents::{Children, Text};
        use IterChainData::*;

        match &self.data {
            Var(txt) => Text(txt),
            MethodCall(contents) => Children(&**contents),
            Closure(contents) => Children(&**contents),
            Times(contents) => Children(&**contents),
        }
    }
}

impl<'d> PrettyDoc<'d> for &'d IterChain {
    type Id = usize;
    type Style = ();
    type Mark = ();

    fn id(self) -> usize {
        self.id
    }

    fn notation(self) -> &'d ValidNotation<()> {
        use IterChainData::*;

        match self.data {
            Var(_) => &VAR_NOTATION,
            MethodCall(_) => &METHOD_CALL_NOTATION,
            Closure(_) => &CLOSURE_NOTATION,
            Times(_) => &TIMES_NOTATION,
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

fn new_node(data: IterChainData) -> IterChain {
    IterChain {
        id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
        data,
    }
}

fn var(var_name: &str) -> IterChain {
    new_node(IterChainData::Var(var_name.to_owned()))
}

fn method_call(obj: IterChain, method: &str, arg: IterChain) -> IterChain {
    new_node(IterChainData::MethodCall(Box::new([obj, var(method), arg])))
}

fn closure(var_name: &str, body: IterChain) -> IterChain {
    new_node(IterChainData::Closure(Box::new([var(var_name), body])))
}

fn times(arg1: IterChain, arg2: IterChain) -> IterChain {
    new_node(IterChainData::Times(Box::new([arg1, arg2])))
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
