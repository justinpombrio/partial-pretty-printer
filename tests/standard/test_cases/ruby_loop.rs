use crate::standard::pretty_testing::{assert_pp, punct};
use once_cell::sync::Lazy;
use partial_pretty_printer::notation_constructors::{child, flat, text};
use partial_pretty_printer::{PrettyDoc, ValidNotation};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Ruby {
    id: usize,
    data: RubyData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RubyData {
    Var(String),
    MethodCall(Box<[Ruby; 3]>),
    DoLoop(Box<[Ruby; 2]>),
}

static VAR_NOTATION: Lazy<ValidNotation<()>> = Lazy::new(|| text(()).validate().unwrap());
static METHOD_CALL_NOTATION: Lazy<ValidNotation<()>> = Lazy::new(|| {
    let single = punct(".") + child(1) + punct(" ") + child(2);
    let two_lines = punct(".") + child(1) + punct(" ") + child(2);
    let multi = punct(".") + child(1) + (4 >> child(2));
    (child(0) + (single | (4 >> (two_lines | multi))))
        .validate()
        .unwrap()
});
static DO_LOOP_NOTATION: Lazy<ValidNotation<()>> = Lazy::new(|| {
    let single = punct("do |") + child(0) + punct("| ") + flat(child(1)) + punct(" end");
    let multi = (punct("do |") + child(0) + punct("|") + (4 >> child(1))) ^ punct("end");
    (single | multi).validate().unwrap()
});

enum Contents<'d> {
    Text(&'d str),
    Children(&'d [Ruby]),
}

impl Ruby {
    fn contents(&self) -> Contents {
        use Contents::{Children, Text};
        use RubyData::*;

        match &self.data {
            Var(txt) => Text(txt),
            MethodCall(contents) => Children(&**contents),
            DoLoop(contents) => Children(&**contents),
        }
    }
}

impl<'d> PrettyDoc<'d, ()> for &'d Ruby {
    type Id = usize;

    fn id(self) -> usize {
        self.id
    }

    fn notation(self) -> &'d ValidNotation<()> {
        use RubyData::*;

        match self.data {
            Var(_) => &VAR_NOTATION,
            MethodCall(_) => &METHOD_CALL_NOTATION,
            DoLoop(_) => &DO_LOOP_NOTATION,
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

    fn unwrap_last_child(self) -> Self {
        match self.contents() {
            Contents::Text(_) => unreachable!(),
            Contents::Children(slice) => slice.last().unwrap(),
        }
    }

    fn unwrap_prev_sibling(self, parent: Self, i: usize) -> Self {
        parent.unwrap_child(i)
    }
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn new_node(data: RubyData) -> Ruby {
    Ruby {
        id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
        data,
    }
}

fn var(var_name: &str) -> Ruby {
    new_node(RubyData::Var(var_name.to_owned()))
}

fn method_call(obj: Ruby, method: &str, arg: Ruby) -> Ruby {
    new_node(RubyData::MethodCall(Box::new([obj, var(method), arg])))
}

fn do_loop(var_name: &str, body: Ruby) -> Ruby {
    new_node(RubyData::DoLoop(Box::new([var(var_name), body])))
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
