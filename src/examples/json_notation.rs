use crate::notation::{Notation, RepeatInner};
use crate::notation_constructors::{child, flat, left, lit, nl, repeat, right, surrounded, text};
use crate::pretty_printing::{PrettyDoc, PrettyDocContents};
use crate::style::{Color, Style};
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

fn punct(s: &'static str) -> Notation {
    lit(s, Style::plain())
}

fn constant(s: &'static str) -> Notation {
    lit(
        s,
        Style {
            color: Color::Base09,
            bold: true,
            underlined: false,
            reversed: false,
        },
    )
}

static JSON_NULL_NOTATION: Lazy<Notation> = Lazy::new(|| constant("null"));
static JSON_TRUE_NOTATION: Lazy<Notation> = Lazy::new(|| constant("true"));
static JSON_FALSE_NOTATION: Lazy<Notation> = Lazy::new(|| constant("false"));
static JSON_STRING_NOTATION: Lazy<Notation> =
    Lazy::new(|| punct("\"") + text(Style::color(Color::Base0B)) + punct("\""));
static JSON_NUMBER_NOTATION: Lazy<Notation> = Lazy::new(|| text(Style::color(Color::Base09)));
static JSON_LIST_NOTATION: Lazy<Notation> = Lazy::new(|| {
    repeat(RepeatInner {
        empty: punct("[]"),
        lone: punct("[") + child(0) + punct("]"),
        join: left() + punct(",") + (punct(" ") | nl()) + right(),
        surround: {
            let single = punct("[") + flat(surrounded()) + punct("]");
            let multi = (punct("[") + (4 >> surrounded())) ^ punct("]");
            single | multi
        },
    })
});
static JSON_DICT_ENTRY_NOTATION: Lazy<Notation> = Lazy::new(|| child(0) + punct(": ") + child(1));
static JSON_DICT_NOTATION: Lazy<Notation> = Lazy::new(|| {
    repeat(RepeatInner {
        empty: punct("{}"),
        lone: {
            let single = punct("{") + left() + punct("}");
            let multi = (punct("{") + (4 >> left())) ^ punct("}");
            single | multi
        },
        join: left() + punct(",") + nl() + right(),
        surround: (punct("{") + (4 >> surrounded())) ^ punct("}"),
    })
});

impl PrettyDoc for Json {
    type Id = usize;

    fn id(&self) -> usize {
        self.id
    }

    fn notation(&self) -> &Notation {
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

    fn contents(&self) -> PrettyDocContents<Self> {
        use JsonData::*;
        use PrettyDocContents::{Children, Text};

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
