#![allow(clippy::precedence)]

use super::tree::Tree;
use super::{BasicStyle, Color};
use crate::notation::Notation;
use crate::notation_constructors::{
    child, count, empty, flat, fold, left, lit, mark, nl, right, text, Count, Fold,
};
use crate::valid_notation::ValidNotation;
use once_cell::sync::Lazy;

fn punct(s: &'static str) -> Notation<BasicStyle> {
    lit(s, BasicStyle::new())
}

fn open(s: &'static str) -> Notation<BasicStyle> {
    // For testing partial node marks; doesn't really effect printing
    mark("open", lit(s, BasicStyle::new()))
}

fn close(s: &'static str) -> Notation<BasicStyle> {
    // For testing partial node marks; doesn't really effect printing
    mark("close", lit(s, BasicStyle::new()))
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
    (lit("\"", style) + text(style) + lit("\"", style))
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
    let single = open("[") + single_seq + close("]");

    let multi_seq = 4
        >> fold(Fold {
            first: child(0),
            join: left() + punct(",") ^ right(),
        });
    let multi = open("[") + multi_seq ^ close("]");

    let list = single | multi;

    count(Count {
        zero: open("[") + close("]"),
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
    let single = open("{") + single_seq + close("}");

    let multi_seq = fold(Fold {
        first: child(0),
        join: left() + punct(",") ^ right(),
    });
    let multi = open("{") + (4 >> multi_seq) ^ close("}");

    let dict = single | multi;

    count(Count {
        zero: open("{") + close("}"),
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

pub type Json = Tree<BasicStyle, char>;

pub fn json_null() -> Json {
    Tree::new_branch(&JSON_NULL_NOTATION, Vec::new())
}

pub fn json_bool(b: bool) -> Json {
    if b {
        Tree::new_branch(&JSON_TRUE_NOTATION, Vec::new())
    } else {
        Tree::new_branch(&JSON_FALSE_NOTATION, Vec::new())
    }
}

pub fn json_string(s: &str) -> Json {
    Tree::new_text(&JSON_STRING_NOTATION, s.to_owned())
}

pub fn json_number(f: f64) -> Json {
    Tree::new_text(&JSON_NUMBER_NOTATION, f.to_string())
}

pub fn json_list(elements: Vec<Json>) -> Json {
    Tree::new_branch(&JSON_LIST_NOTATION, elements)
}

pub fn json_dict(entries: Vec<(&str, Json)>) -> Json {
    Tree::new_branch(
        &JSON_DICT_NOTATION,
        entries
            .into_iter()
            .map(|(key, val)| {
                Tree::new_branch(&JSON_DICT_ENTRY_NOTATION, vec![json_string(key), val])
            })
            .collect::<Vec<_>>(),
    )
}

pub fn json_comment(comment: &str, value: Json) -> Json {
    let comment = Tree::new_branch(
        &JSON_COMMENT_NOTATION,
        comment
            .split_whitespace()
            .map(|word| Tree::new_text(&JSON_COMMENT_WORD_NOTATION, word.to_owned()))
            .collect::<Vec<_>>(),
    );
    Tree::new_branch(&JSON_COMMENTED_NOTATION, vec![comment, value])
}
