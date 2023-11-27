use super::{BasicStyle, Color};
use crate::notation::Notation;
use crate::notation_constructors::{
    child, count, flat, fold, left, lit, nl, right, text, Count, Fold,
};
use crate::pretty_printing::PrettyDoc;
use crate::valid_notation::ValidNotation;
use once_cell::sync::Lazy;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Json {
    id: usize,
    data: JsonData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonData {
    Null,
    True,
    False,
    String(String),
    Number(String),
    List(Vec<Json>),
    // Must contain DictEntries
    Dict(Vec<Json>),
    // First Json must be a Json::String
    DictEntry(Box<[Json; 2]>),
}

fn punct(s: &'static str) -> Notation<BasicStyle> {
    lit(
        s,
        BasicStyle {
            color: Color::White,
            bold: false,
        },
    )
}

fn constant(s: &'static str) -> Notation<BasicStyle> {
    lit(
        s,
        BasicStyle {
            color: Color::Green,
            bold: true,
        },
    )
}

static JSON_NULL_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| constant("null").validate().unwrap());
static JSON_TRUE_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| constant("true").validate().unwrap());
static JSON_FALSE_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| constant("false").validate().unwrap());
static JSON_STRING_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let style = BasicStyle {
        color: Color::Magenta,
        bold: false,
    };
    (punct("\"") + text(style) + punct("\""))
        .validate()
        .unwrap()
});
static JSON_NUMBER_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let style = BasicStyle {
        color: Color::Yellow,
        bold: false,
    };
    text(style).validate().unwrap()
});
static JSON_LIST_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let seq = fold(Fold {
        first: child(0),
        join: left() + punct(",") + (punct(" ") | nl()) + right(),
    });
    let single = punct("[") + flat(seq.clone()) + punct("]");
    let multi = (punct("[") + (4 >> seq)) ^ punct("]");

    count(Count {
        zero: punct("[]"),
        one: punct("[") + child(0) + punct("]"),
        many: single | multi,
    })
    .validate()
    .unwrap()
});
static JSON_DICT_ENTRY_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| (child(0) + punct(": ") + child(1)).validate().unwrap());
static JSON_DICT_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    count(Count {
        zero: punct("{}"),
        one: {
            let single = punct("{") + child(0) + punct("}");
            let multi = (punct("{") + (4 >> child(0))) ^ punct("}");
            single | multi
        },
        many: punct("{")
            + (4 >> fold(Fold {
                first: child(0),
                join: left() + punct(",") ^ right(),
            }))
            ^ punct("}"),
    })
    .validate()
    .unwrap()
});

enum JsonContents<'a> {
    Text(&'a str),
    Children(&'a [Json]),
}

impl Json {
    fn contents(&self) -> JsonContents {
        use JsonContents::{Children, Text};
        use JsonData::*;

        match &self.data {
            Null => Children(&[]),
            True => Children(&[]),
            False => Children(&[]),
            String(txt) => Text(txt),
            Number(txt) => Text(txt),
            List(children) => Children(children),
            DictEntry(entry) => Children(&**entry),
            Dict(children) => Children(children),
        }
    }
}

impl<'a> PrettyDoc<'a, BasicStyle> for &'a Json {
    type Id = usize;

    fn id(self) -> usize {
        self.id
    }

    fn notation(self) -> &'a ValidNotation<BasicStyle> {
        use JsonData::*;

        match &self.data {
            Null => &JSON_NULL_NOTATION,
            True => &JSON_TRUE_NOTATION,
            False => &JSON_FALSE_NOTATION,
            String(_) => &JSON_STRING_NOTATION,
            Number(_) => &JSON_NUMBER_NOTATION,
            List(_) => &JSON_LIST_NOTATION,
            DictEntry(_) => &JSON_DICT_ENTRY_NOTATION,
            Dict(_) => &JSON_DICT_NOTATION,
        }
    }

    fn num_children(self) -> Option<usize> {
        match self.contents() {
            JsonContents::Text(_) => None,
            JsonContents::Children(slice) => Some(slice.len()),
        }
    }

    fn unwrap_text(self) -> &'a str {
        match self.contents() {
            JsonContents::Text(txt) => txt,
            JsonContents::Children(_) => panic!("Json: not text"),
        }
    }

    fn unwrap_child(self, i: usize) -> Self {
        match self.contents() {
            JsonContents::Text(_) => panic!("Json: no children"),
            JsonContents::Children(slice) => &slice[i],
        }
    }

    fn unwrap_last_child(self) -> Self {
        match self.contents() {
            JsonContents::Text(_) => panic!("Json: no children"),
            JsonContents::Children(slice) => slice.last().expect("Json: zero children"),
        }
    }

    fn unwrap_prev_sibling(self, parent: Self, i: usize) -> Self {
        parent.unwrap_child(i)
    }
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn new_node(data: JsonData) -> Json {
    Json {
        id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
        data,
    }
}

pub fn json_null() -> Json {
    new_node(JsonData::Null)
}

pub fn json_bool(b: bool) -> Json {
    if b {
        new_node(JsonData::True)
    } else {
        new_node(JsonData::False)
    }
}

pub fn json_string(s: &str) -> Json {
    new_node(JsonData::String(s.to_owned()))
}

pub fn json_number(f: f64) -> Json {
    new_node(JsonData::Number(f.to_string()))
}

pub fn json_dict_entry(key: &str, value: Json) -> Json {
    new_node(JsonData::DictEntry(Box::new([json_string(key), value])))
}

pub fn json_list(elements: Vec<Json>) -> Json {
    new_node(JsonData::List(elements))
}

pub fn json_dict(entries: Vec<Json>) -> Json {
    new_node(JsonData::Dict(entries))
}
