use crate::standard::pretty_testing::assert_pp;
use once_cell::sync::Lazy;
use partial_pretty_printer::notation_constructors::{child, flat, lit, text};
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

static VAR_NOTATION: Lazy<ValidNotation<(), ()>> = Lazy::new(|| text().validate().unwrap());
static METHOD_CALL_NOTATION: Lazy<ValidNotation<(), ()>> = Lazy::new(|| {
    let single = lit(".") + child(1) + lit(" ") + child(2);
    let two_lines = lit(".") + child(1) + lit(" ") + child(2);
    let multi = lit(".") + child(1) + (4 >> child(2));
    (child(0) + (single | (4 >> (two_lines | multi))))
        .validate()
        .unwrap()
});
static DO_LOOP_NOTATION: Lazy<ValidNotation<(), ()>> = Lazy::new(|| {
    let single = lit("do |") + child(0) + lit("| ") + flat(child(1)) + lit(" end");
    let multi = (lit("do |") + child(0) + lit("|") + (4 >> child(1))) ^ lit("end");
    (single | multi).validate().unwrap()
});

enum Contents<'d> {
    Text(&'d str),
    Children(&'d [Ruby]),
}

impl Ruby {
    fn contents(&self) -> Contents<'_> {
        use Contents::{Children, Text};
        use RubyData::*;

        match &self.data {
            Var(txt) => Text(txt),
            MethodCall(contents) => Children(&**contents),
            DoLoop(contents) => Children(&**contents),
        }
    }
}

impl<'d> PrettyDoc<'d> for &'d Ruby {
    type Id = usize;
    type Style = ();
    type StyleLabel = ();
    type Condition = ();
    type Error = std::convert::Infallible;

    fn id(self) -> Result<usize, Self::Error> {
        Ok(self.id)
    }

    fn notation(self) -> Result<&'d ValidNotation<(), ()>, Self::Error> {
        use RubyData::*;

        Ok(match self.data {
            Var(_) => &VAR_NOTATION,
            MethodCall(_) => &METHOD_CALL_NOTATION,
            DoLoop(_) => &DO_LOOP_NOTATION,
        })
    }

    fn condition(self, _condition: &()) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn node_style(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn lookup_style(self, _label: ()) -> Result<(), Self::Error> {
        Ok(())
    }

    fn num_children(self) -> Result<Option<usize>, Self::Error> {
        Ok(match self.contents() {
            Contents::Text(_) => None,
            Contents::Children(slice) => Some(slice.len()),
        })
    }

    fn unwrap_text(self) -> Result<&'d str, Self::Error> {
        Ok(match self.contents() {
            Contents::Text(txt) => txt,
            Contents::Children(_) => unreachable!(),
        })
    }

    fn unwrap_child(self, i: usize) -> Result<Self, Self::Error> {
        Ok(match self.contents() {
            Contents::Text(_) => unreachable!(),
            Contents::Children(slice) => &slice[i],
        })
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
