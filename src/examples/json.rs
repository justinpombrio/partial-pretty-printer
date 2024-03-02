#![allow(clippy::precedence)]

use super::style::BasicStyle;
use super::tree::{Tree, TreeCondition, TreeNotation};
use crate::notation::CheckPos;
use crate::notation_constructors::{
    check, child, count, empty, eol, flat, fold, indent, left, lit, nl, right, style, text, Count,
    Fold,
};
use once_cell::sync::Lazy;

const CONSTANT_STYLE: &str = "green_bold";
const STRING_STYLE: &str = "magenta";
const NUMBER_STYLE: &str = "blue";
const COMMENT_STYLE: &str = "yellow";

static JSON_NULL_NOTATION: Lazy<TreeNotation> =
    Lazy::new(|| style(CONSTANT_STYLE, lit("null")).validate().unwrap());

static JSON_TRUE_NOTATION: Lazy<TreeNotation> =
    Lazy::new(|| style(CONSTANT_STYLE, lit("true")).validate().unwrap());

static JSON_FALSE_NOTATION: Lazy<TreeNotation> =
    Lazy::new(|| style(CONSTANT_STYLE, lit("false")).validate().unwrap());

static JSON_STRING_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
    style(STRING_STYLE, lit("\"") + text() + lit("\""))
        .validate()
        .unwrap()
});

static JSON_NUMBER_NOTATION: Lazy<TreeNotation> =
    Lazy::new(|| style(NUMBER_STYLE, text()).validate().unwrap());

static JSON_ARRAY_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
    let single_seq = fold(Fold {
        first: flat(child(0)),
        join: left() + lit(", ") + flat(right()),
    });
    let single = style("open", lit("[")) + single_seq + style("close", lit("]"));

    let separator = check(
        TreeCondition::NeedsSeparator,
        CheckPos::LeftChild,
        lit(","),
        empty(),
    );
    let multi_seq = 4
        >> fold(Fold {
            first: child(0),
            join: left() + separator ^ right(),
        });
    let multi = style("open", lit("[")) + multi_seq ^ style("close", lit("]"));

    let array = single | multi;

    count(Count {
        zero: style("open", lit("[")) + style("close", lit("]")),
        one: array.clone(),
        many: array,
    })
    .validate()
    .unwrap()
});

static JSON_OBJECT_PAIR_NOTATION: Lazy<TreeNotation> =
    Lazy::new(|| (child(0) + lit(": ") + child(1)).validate().unwrap());

static JSON_OBJECT_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
    let single_seq = fold(Fold {
        first: flat(child(0)),
        join: left() + lit(", ") + flat(right()),
    });
    let single = style("open", lit("{")) + single_seq + style("close", lit("}"));

    let separator = check(
        TreeCondition::NeedsSeparator,
        CheckPos::LeftChild,
        lit(","),
        empty(),
    );
    let multi_seq = fold(Fold {
        first: child(0),
        join: left() + separator ^ right(),
    });
    let multi = style("open", lit("{")) + (4 >> multi_seq) ^ style("close", lit("}"));

    let object = single | multi;

    count(Count {
        zero: style("open", lit("{")) + style("close", lit("}")),
        one: object.clone(),
        many: object,
    })
    .validate()
    .unwrap()
});

static JSON_COMMENT_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
    let comment_body = count(Count {
        zero: empty(),
        one: child(0),
        many: fold(Fold {
            first: child(0),
            join: left() + (lit(" ") | nl()) + right(),
        }),
    });
    let notation = style(
        COMMENT_STYLE,
        lit("// ") + indent("// ", None, comment_body) + eol(),
    );
    notation.validate().unwrap()
});

static JSON_COMMENT_WORD_NOTATION: Lazy<TreeNotation> = Lazy::new(|| text().validate().unwrap());

static JSON_ROOTS_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
    let notation = count(Count {
        zero: empty(),
        one: child(0),
        many: fold(Fold {
            first: child(0),
            join: left() ^ right(),
        }),
    });
    notation.validate().unwrap()
});

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

pub fn json_array(elements: Vec<Json>) -> Json {
    Tree::new_branch(&JSON_ARRAY_NOTATION, elements)
}

pub fn json_roots(values: Vec<Json>) -> Json {
    Tree::new_branch(&JSON_ROOTS_NOTATION, values)
}

pub fn json_object_pair(key: &str, value: Json) -> Json {
    Tree::new_branch(&JSON_OBJECT_PAIR_NOTATION, vec![json_string(key), value])
}

pub fn json_object(entries: Vec<Json>) -> Json {
    Tree::new_branch(&JSON_OBJECT_NOTATION, entries)
}

pub fn json_comment(comment: &str) -> Json {
    Tree::new_branch(
        &JSON_COMMENT_NOTATION,
        comment
            .split_whitespace()
            .map(|word| Tree::new_text(&JSON_COMMENT_WORD_NOTATION, word.to_owned()))
            .collect::<Vec<_>>(),
    )
    .into_comment()
}
