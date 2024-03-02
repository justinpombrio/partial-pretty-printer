use crate::standard::pretty_testing::{assert_pp, SimpleDoc};
use partial_pretty_printer::notation_constructors::{empty, eol, lit, nl};

// NOTE: We tried to have EOL + text add a line break before, and it was awful.
// Two tricky test cases that failed:
// - EOL + ('a' | ε)
// - 'a' + EOL + ('a' | ε)
// Additionally, see test_regression_1. What is even the correct behavior?
// What if 'bb' is instead an opaque child that might contain 'a' or 'bb'?

#[test]
fn test_regression_1() {
    // 'bb' + EOL + (ε | ↵)
    let notation = lit("bb") + eol() + (empty() | nl());
    assert_pp(&SimpleDoc::new(notation), 1, &["bb", ""]);
}
