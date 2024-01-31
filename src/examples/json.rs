use super::{BasicStyle, Color};
use crate::notation::Notation;
use crate::notation_constructors::{
    child, count, empty, flat, fold, left, lit, nl, right, text, Count, Fold,
};
use crate::pretty_printing::PrettyDoc;
use crate::valid_notation::ValidNotation;
use once_cell::sync::Lazy;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Json {
    id: u32,
    comment: Option<String>,
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
    Dict(Vec<(String, Json)>),
}

fn punct(s: &'static str) -> Notation<BasicStyle> {
    lit(s, BasicStyle::new())
}

fn constant(s: &'static str) -> Notation<BasicStyle> {
    lit(s, BasicStyle::new().color(Color::Green).bold())
}

static JSON_NULL_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| constant("null").validate().unwrap());

static JSON_TRUE_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| constant("true").validate().unwrap());

static JSON_FALSE_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| constant("false").validate().unwrap());

static JSON_STRING_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let style = BasicStyle::new().color(Color::Magenta);
    (punct("\"") + text(style) + punct("\""))
        .validate()
        .unwrap()
});

static JSON_NUMBER_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let style = BasicStyle::new().color(Color::Blue);
    text(style).validate().unwrap()
});

static JSON_LIST_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let single_seq = fold(Fold {
        first: flat(child(0)),
        join: left() + punct(", ") + flat(right()),
    });
    let single = punct("[") + single_seq + punct("]");

    let multi_seq = 4
        >> fold(Fold {
            first: child(0),
            join: left() + punct(",") ^ right(),
        });
    let multi = punct("[") + multi_seq ^ punct("]");

    let list = single | multi;

    count(Count {
        zero: punct("[]"),
        one: list.clone(),
        many: list,
    })
    .validate()
    .unwrap()
});

static JSON_DICT_ENTRY_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| (child(0) + punct(": ") + child(1)).validate().unwrap());

static JSON_DICT_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let single_seq = fold(Fold {
        first: flat(child(0)),
        join: left() + punct(", ") + flat(right()),
    });
    let single = punct("{") + single_seq + punct("}");

    let multi_seq = fold(Fold {
        first: child(0),
        join: left() + punct(",") ^ right(),
    });
    let multi = punct("{") + (4 >> multi_seq) ^ punct("}");

    let dict = single | multi;

    count(Count {
        zero: punct("{}"),
        one: dict.clone(),
        many: dict,
    })
    .validate()
    .unwrap()
});

static JSON_COMMENTED_NOTATION: Lazy<ValidNotation<BasicStyle>> =
    Lazy::new(|| (child(0) ^ child(1)).validate().unwrap());

static JSON_COMMENT_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let style = BasicStyle::new().color(Color::Yellow);
    let notation = lit("// ", style)
        + count(Count {
            zero: empty(),
            one: child(0),
            many: fold(Fold {
                first: child(0),
                join: left() + (punct(" ") | nl() + lit("// ", style)) + right(),
            }),
        });
    notation.validate().unwrap()
});

static JSON_COMMENT_WORD_NOTATION: Lazy<ValidNotation<BasicStyle>> = Lazy::new(|| {
    let style = BasicStyle::new().color(Color::Yellow);
    text(style).validate().unwrap()
});

#[derive(Debug, Clone, Copy)]
pub enum JsonRef<'a> {
    /// e.g. // some comment \n 17
    Commented(&'a str, &'a Json),
    /// e.g. 17
    Json(&'a Json),
    /// e.g. // some comment
    Comment(&'a str, &'a Json),
    /// e.g. some
    CommentWord(&'a str, &'a Json, usize),
    /// e.g. "key": 17
    DictEntry(&'a str, &'a Json),
    /// e.g. "key"
    DictKey(&'a str, &'a Json),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonId {
    Commented(u32),
    Json(u32),
    Comment(u32),
    CommentWord(u32, usize),
    DictEntry(u32),
    DictKey(u32),
}

impl Default for JsonId {
    fn default() -> JsonId {
        // id 0 is otherwise unused
        JsonId::Json(0)
    }
}

impl<'a> JsonRef<'a> {
    fn new(json: &'a Json) -> JsonRef<'a> {
        if let Some(comment) = &json.comment {
            JsonRef::Commented(comment, json)
        } else {
            JsonRef::Json(json)
        }
    }
}

impl<'a> PrettyDoc<'a> for JsonRef<'a> {
    type Id = JsonId;
    type Style = BasicStyle;
    type Mark = ();

    fn id(self) -> JsonId {
        use JsonRef::*;

        match self {
            Commented(_, json) => JsonId::Commented(json.id),
            Json(json) => JsonId::Json(json.id),
            Comment(_, json) => JsonId::Comment(json.id),
            CommentWord(_, json, index) => JsonId::CommentWord(json.id, index),
            DictEntry(_, val) => JsonId::DictEntry(val.id),
            DictKey(_, val) => JsonId::DictKey(val.id),
        }
    }

    fn notation(self) -> &'a ValidNotation<BasicStyle> {
        use JsonData::*;
        use JsonRef::*;

        match self {
            Commented(_, _) => &JSON_COMMENTED_NOTATION,
            Json(json) => match &json.data {
                Null => &JSON_NULL_NOTATION,
                True => &JSON_TRUE_NOTATION,
                False => &JSON_FALSE_NOTATION,
                String(_) => &JSON_STRING_NOTATION,
                Number(_) => &JSON_NUMBER_NOTATION,
                List(_) => &JSON_LIST_NOTATION,
                Dict(_) => &JSON_DICT_NOTATION,
            },
            Comment(_, _) => &JSON_COMMENT_NOTATION,
            CommentWord(_, _, _) => &JSON_COMMENT_WORD_NOTATION,
            DictEntry(_, _) => &JSON_DICT_ENTRY_NOTATION,
            DictKey(_, _) => &JSON_STRING_NOTATION,
        }
    }

    fn mark(self) -> Option<&'a ()> {
        None
    }

    fn num_children(self) -> Option<usize> {
        use JsonData::*;
        use JsonRef::*;

        match self {
            Commented(_, _) => Some(2),
            Json(json) => match &json.data {
                Null | True | False => Some(0),
                String(_) => None,
                Number(_) => None,
                List(list) => Some(list.len()),
                Dict(dict) => Some(dict.len()),
            },
            Comment(text, _) => Some(text.split(' ').count()),
            CommentWord(_, _, _) => None,
            DictEntry(_, _) => Some(2),
            DictKey(_, _) => None,
        }
    }

    fn unwrap_text(self) -> &'a str {
        use JsonData::*;
        use JsonRef::*;

        match self {
            Json(json) => match &json.data {
                Null | True | False | List(_) | Dict(_) => {
                    panic!("Json: not text")
                }
                String(text) => text,
                Number(text) => text,
            },
            Commented(_, _) | DictEntry(_, _) | Comment(_, _) => panic!("Json: not text"),
            CommentWord(word, _, _) => word,
            DictKey(text, _) => text,
        }
    }

    fn unwrap_child(self, i: usize) -> Self {
        use JsonData::*;
        use JsonRef::*;

        match self {
            CommentWord(_, _, _) | DictKey(_, _) => panic!("Json: no children"),
            Comment(comment, json) => {
                // inefficient
                let word = comment.split(' ').nth(i).unwrap();
                CommentWord(word, json, i)
            }
            Commented(text, json) => match i {
                0 => Comment(text, json),
                1 => Json(json),
                _ => panic!("Json: invalid index (commented)"),
            },
            Json(json) => match &json.data {
                Null | True | False | String(_) | Number(_) => panic!("Json: no children"),
                List(elems) => JsonRef::new(&elems[i]),
                Dict(pairs) => {
                    let (key, val) = &pairs[i];
                    DictEntry(key, val)
                }
            },
            DictEntry(key, val) => match i {
                0 => DictKey(key, val),
                1 => JsonRef::new(val),
                _ => panic!("Json: invalid index (dict entry)"),
            },
        }
    }
}

static ID_COUNTER: AtomicU32 = AtomicU32::new(1); // id 0 reserved for Default

fn new_node(data: JsonData) -> Json {
    Json {
        id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
        comment: None,
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

pub fn json_list(elements: Vec<Json>) -> Json {
    new_node(JsonData::List(elements))
}

pub fn json_dict(entries: Vec<(&str, Json)>) -> Json {
    new_node(JsonData::Dict(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect::<Vec<_>>(),
    ))
}

impl Json {
    pub fn as_ref(&self) -> JsonRef {
        JsonRef::new(self)
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}
