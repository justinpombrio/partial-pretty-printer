#![allow(clippy::precedence)]

use super::style::BasicStyle;
use super::tree::{StyleLabel, Tree};
use crate::notation_constructors::{
    child, count, empty, flat, fold, left, lit, nl, right, style, text, Count, Fold,
};
use crate::valid_notation::ValidNotation;
use once_cell::sync::Lazy;

const CONSTANT_STYLE: &str = "green_bold";
const STRING_STYLE: &str = "magenta";
const NUMBER_STYLE: &str = "blue";
const COMMENT_STYLE: &str = "yellow";

static JSON_NULL_NOTATION: Lazy<ValidNotation<StyleLabel>> =
    Lazy::new(|| style(CONSTANT_STYLE, lit("null")).validate().unwrap());

static JSON_TRUE_NOTATION: Lazy<ValidNotation<StyleLabel>> =
    Lazy::new(|| style(CONSTANT_STYLE, lit("true")).validate().unwrap());

static JSON_FALSE_NOTATION: Lazy<ValidNotation<StyleLabel>> =
    Lazy::new(|| style(CONSTANT_STYLE, lit("false")).validate().unwrap());

static JSON_STRING_NOTATION: Lazy<ValidNotation<StyleLabel>> = Lazy::new(|| {
    style(STRING_STYLE, lit("\"") + text() + lit("\""))
        .validate()
        .unwrap()
});

static JSON_NUMBER_NOTATION: Lazy<ValidNotation<StyleLabel>> =
    Lazy::new(|| style(NUMBER_STYLE, text()).validate().unwrap());

static JSON_LIST_NOTATION: Lazy<ValidNotation<StyleLabel>> = Lazy::new(|| {
    let single_seq = fold(Fold {
        first: flat(child(0)),
        join: left() + lit(", ") + flat(right()),
    });
    let single = style("open", lit("[")) + single_seq + style("close", lit("]"));

    let multi_seq = 4
        >> fold(Fold {
            first: child(0),
            join: left() + lit(",") ^ right(),
        });
    let multi = style("open", lit("[")) + multi_seq ^ style("close", lit("]"));

    let list = single | multi;

    count(Count {
        zero: style("open", lit("[")) + style("close", lit("]")),
        one: list.clone(),
        many: list,
    })
    .validate()
    .unwrap()
});

static JSON_DICT_ENTRY_NOTATION: Lazy<ValidNotation<StyleLabel>> =
    Lazy::new(|| (child(0) + lit(": ") + child(1)).validate().unwrap());

static JSON_DICT_NOTATION: Lazy<ValidNotation<StyleLabel>> = Lazy::new(|| {
    let single_seq = fold(Fold {
        first: flat(child(0)),
        join: left() + lit(", ") + flat(right()),
    });
    let single = style("open", lit("{")) + single_seq + style("close", lit("}"));

    let multi_seq = fold(Fold {
        first: child(0),
        join: left() + lit(",") ^ right(),
    });
    let multi = style("open", lit("{")) + (4 >> multi_seq) ^ style("close", lit("}"));

    let dict = single | multi;

    count(Count {
        zero: style("open", lit("{")) + style("close", lit("}")),
        one: dict.clone(),
        many: dict,
    })
    .validate()
    .unwrap()
});

static JSON_COMMENTED_NOTATION: Lazy<ValidNotation<StyleLabel>> =
    Lazy::new(|| (child(0) ^ child(1)).validate().unwrap());

static JSON_COMMENT_NOTATION: Lazy<ValidNotation<StyleLabel>> = Lazy::new(|| {
    let notation = style(
        COMMENT_STYLE,
        lit("// ")
            + count(Count {
                zero: empty(),
                one: child(0),
                many: fold(Fold {
                    first: child(0),
                    join: left() + (lit(" ") | nl() + lit("// ")) + right(),
                }),
            }),
    );
    notation.validate().unwrap()
});

static JSON_COMMENT_WORD_NOTATION: Lazy<ValidNotation<StyleLabel>> =
    Lazy::new(|| text().validate().unwrap());

pub type Json = Tree<BasicStyle>;

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
