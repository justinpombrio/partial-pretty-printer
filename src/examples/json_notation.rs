use super::doc::{Doc, Sort};
use crate::notation::{Notation, RepeatInner};
use crate::notation_constructors::{child, flat, left, lit, nl, repeat, right, surrounded, text};
use crate::style::{Color, Style};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Json {
    Null,
    True,
    False,
    String,
    Number,
    List,
    Dict,
    DictEntry,
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
            let multi = punct("[") + (4 >> surrounded()) ^ punct("]");
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
            let multi = punct("{") + (4 >> left()) ^ punct("}");
            single | multi
        },
        join: left() + punct(",") + nl() + right(),
        surround: punct("{") + (4 >> surrounded()) ^ punct("}"),
    })
});

impl Sort for Json {
    fn notation(self) -> &'static Notation {
        use Json::*;

        match self {
            Null => &JSON_NULL_NOTATION,
            True => &JSON_TRUE_NOTATION,
            False => &JSON_FALSE_NOTATION,
            String => &JSON_STRING_NOTATION,
            Number => &JSON_NUMBER_NOTATION,
            List => &JSON_LIST_NOTATION,
            DictEntry => &JSON_DICT_ENTRY_NOTATION,
            Dict => &JSON_DICT_NOTATION,
        }
    }
}

pub fn json_null() -> Doc<Json> {
    Doc::new_node(Json::Null, vec![])
}

pub fn json_bool(b: bool) -> Doc<Json> {
    if b {
        Doc::new_node(Json::True, vec![])
    } else {
        Doc::new_node(Json::False, vec![])
    }
}

pub fn json_string(s: &str) -> Doc<Json> {
    Doc::new_text(Json::String, s.to_owned())
}

pub fn json_number(f: f64) -> Doc<Json> {
    Doc::new_text(Json::Number, f.to_string())
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
