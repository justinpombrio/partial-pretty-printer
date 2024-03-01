use crate::standard::pretty_testing::assert_pp;
use partial_pretty_printer::examples::{Tree, TreeCondition};
use partial_pretty_printer::notation_constructors::{empty, flat, indent, lit, nl};

// Use Tree.into_comment() to set conditions to true

#[test]
fn test_cond_here() {
    let notation = lit("Hello") + lit(" world!");
    assert_pp(&SimpleDoc::new(notation), 80, &["Hello world!"]);
}
