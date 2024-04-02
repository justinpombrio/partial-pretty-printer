//! This uses Json (with comments) as an example of how to create notations and documents.  Each
//! type of Json value has a color applied for syntax highlighting.

#![allow(clippy::precedence)]

use super::{
    style::BasicStyle,
    tree::{Tree, TreeCondition, TreeNotation},
};
use crate::notation_constructors::{
    check, child, count, empty, eol, flat, fold, indent, left, lit, mark, nl, right, style, text,
    Count, Fold,
};
use crate::CheckPos;
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
        zero: style("open", lit("[")) + mark() + style("close", lit("]")),
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
        zero: style("open", lit("{")) + mark() + style("close", lit("}")),
        one: object.clone(),
        many: object,
    })
    .validate()
    .unwrap()
});

static JSON_COMMENT_NOTATION: Lazy<TreeNotation> = Lazy::new(|| {
    let comment_body = count(Count {
        zero: mark(),
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
        zero: mark(),
        one: child(0),
        many: fold(Fold {
            first: child(0),
            join: left() ^ right(),
        }),
    });
    notation.validate().unwrap()
});

/// A [`Tree`] that stores a Json document (with comments).
pub type Json = Tree<BasicStyle>;

/// Create a document that can contain multiple Json values or comments at the
/// top-level.
pub fn json_roots(values: Vec<Json>) -> Json {
    Tree::new_branch(&JSON_ROOTS_NOTATION, values)
}

/// Create the value `null`.
pub fn json_null() -> Json {
    Tree::new_branch(&JSON_NULL_NOTATION, Vec::new())
}

/// Create the value `true` or `false`.
pub fn json_bool(b: bool) -> Json {
    if b {
        Tree::new_branch(&JSON_TRUE_NOTATION, Vec::new())
    } else {
        Tree::new_branch(&JSON_FALSE_NOTATION, Vec::new())
    }
}

/// Create a string.
pub fn json_string(s: &str) -> Json {
    Tree::new_text(&JSON_STRING_NOTATION, s.to_owned())
}

/// Create a number.
pub fn json_number(f: f64) -> Json {
    Tree::new_text(&JSON_NUMBER_NOTATION, f.to_string())
}

/// Create an array containing the given values.
pub fn json_array(elements: Vec<Json>) -> Json {
    Tree::new_branch(&JSON_ARRAY_NOTATION, elements)
}

/// Create a key-value pair for an object.
pub fn json_object_pair(key: &str, value: Json) -> Json {
    Tree::new_branch(&JSON_OBJECT_PAIR_NOTATION, vec![json_string(key), value])
}

/// Create a Json object. The entries must be either [`json_object_pair`]s or [`json_comment`]s.
pub fn json_object(entries: Vec<Json>) -> Json {
    Tree::new_branch(&JSON_OBJECT_NOTATION, entries)
}

/// Create a comment containing text.
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
