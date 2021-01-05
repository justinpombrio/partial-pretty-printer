use super::doc::{Doc, Sort};
use crate::notation::{Notation, RepeatInner};
use crate::notation_constructors::{child, flat, left, lit, nl, repeat, right, surrounded, text};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Json {
    String,
    Number,
    List,
    Dict,
    DictEntry,
}

static JSON_STRING_NOTATION: Lazy<Notation> = Lazy::new(|| lit("\"") + text() + lit("\""));
static JSON_NUMBER_NOTATION: Lazy<Notation> = Lazy::new(|| text());
static JSON_LIST_NOTATION: Lazy<Notation> = Lazy::new(|| {
    repeat(RepeatInner {
        empty: lit("[]"),
        lone: lit("[") + child(0) + lit("]"),
        join: left() + lit(",") + (lit(" ") | nl()) + right(),
        surround: {
            let single = lit("[") + flat(surrounded()) + lit("]");
            let multi = lit("[") + (4 >> surrounded()) ^ lit("]");
            single | multi
        },
    })
});
static JSON_DICT_ENTRY_NOTATION: Lazy<Notation> = Lazy::new(|| child(0) + lit(": ") + child(1));
static JSON_DICT_NOTATION: Lazy<Notation> = Lazy::new(|| {
    repeat(RepeatInner {
        empty: lit("{}"),
        lone: {
            let single = lit("{") + left() + lit("}");
            let multi = lit("{") + (4 >> left()) ^ lit("}");
            single | multi
        },
        join: left() + lit(",") + nl() + right(),
        surround: lit("{") + (4 >> surrounded()) ^ lit("}"),
    })
});

impl Sort for Json {
    fn notation(self) -> &'static Notation {
        use Json::*;

        match self {
            String => &JSON_STRING_NOTATION,
            Number => &JSON_NUMBER_NOTATION,
            List => &JSON_LIST_NOTATION,
            DictEntry => &JSON_DICT_ENTRY_NOTATION,
            Dict => &JSON_DICT_NOTATION,
        }
    }
}

pub fn json_string(s: &str) -> Doc<Json> {
    Doc::new_text(Json::String, s.to_owned())
}

pub fn json_number(n: usize) -> Doc<Json> {
    Doc::new_text(Json::Number, n.to_string())
}

pub fn json_dict_entry(key: &str, value: Doc<Json>) -> Doc<Json> {
    Doc::new_node(Json::DictEntry, vec![json_string(key), value])
}

pub fn json_list(elements: Vec<Doc<Json>>) -> Doc<Json> {
    Doc::new_node(Json::List, elements)
}

pub fn json_dict(entries: Vec<Doc<Json>>) -> Doc<Json> {
    Doc::new_node(Json::Dict, entries)
}