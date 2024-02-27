use crate::standard::pretty_testing::{all_paths, assert_pp, SimpleDoc};
use partial_pretty_printer::notation_constructors::{empty, flat, indent, lit, nl};

#[test]
fn basics_empty() {
    let notation = empty::<()>();
    assert_pp(&SimpleDoc::new(notation), 80, &[""]);
}

#[test]
fn basics_literal() {
    let notation = lit("Hello world!");
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_concat() {
    let notation = lit("Hello") + lit(" world!");
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello world!"]);
}

#[test]
fn basics_newline() {
    let notation = lit("Hello") ^ lit("world!");
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello", "world!"]);
}

#[test]
fn basics_indent() {
    let notation = lit("Hello") + (2 >> lit("world!"));
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello", "  world!"]);
}

#[test]
fn basics_non_whitespace_indent() {
    let notation = lit("Hello") + indent("// ", None, nl() + lit("world!"));
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello", "// world!"]);
}

#[test]
fn basics_flat() {
    let notation = flat(lit("long") | (lit("a") ^ lit("b")));
    assert_pp(&SimpleDoc::new(notation), 2, &["long"]);
}

#[test]
fn basics_choice() {
    let notation = lit("Hello world!") | lit("Hello") ^ lit("world!");
    assert_pp(&SimpleDoc::new(notation.clone()), 12, &["Hello world!"]);
    assert_pp(&SimpleDoc::new(notation), 11, &["Hello", "world!"]);
}

#[test]
fn test_all_paths_fn() {
    use partial_pretty_printer::examples::json::{json_list, json_string};

    let doc = json_list(vec![
        json_list(vec![json_string("0.0"), json_string("0.1")]),
        json_string("1"),
        json_list(vec![
            json_list(vec![json_string("2.0.0")]),
            json_string("2.1"),
        ]),
    ]);
    assert_eq!(
        all_paths(&doc),
        vec![
            vec![],
            vec![0],
            vec![0, 0],
            vec![0, 1],
            vec![1],
            vec![2],
            vec![2, 0],
            vec![2, 0, 0],
            vec![2, 1]
        ]
    );
}

#[test]
fn hidden_error() {
    let notation = lit("x") ^ flat(nl()) | lit("ok");
    assert_pp(&SimpleDoc::new(notation), 3, &["x", "", ""]);
}

#[test]
fn tricky_suffix() {
    let notation = (lit("a") | lit("bb")) + ((lit("x") + nl() + flat(nl())) | lit("yy"));
    assert_pp(&SimpleDoc::new(notation), 3, &["ax", "", ""]);
}
