#[allow(unused)] // Not actually unused
mod common;

use common::combinators::*;
use common::oracular_pretty_print;
use partial_pretty_printer::{
    partial_pretty_print, partial_pretty_print_first, partial_pretty_print_last, pretty_print,
    Notation, Pos,
};

/********************************************************************************/
/*   Assertions                                                                 */
/********************************************************************************/

fn assert_pp(notation: Notation, width: usize, expected_lines: &[&str]) {
    notation.validate().expect("failed to validate");
    let measured_notation = notation.measure();
    let oracle_lines: Vec<String> = expand_lines(oracular_pretty_print(&notation, width)).collect();
    let actual_lines: Vec<String> = expand_lines(pretty_print(&measured_notation, width)).collect();
    if oracle_lines != expected_lines {
        eprintln!(
            "BAD TEST CASE!\n\nTEST CASE EXPECTS:\n{}\nBUT ORACLE SAYS:\n{}",
            expected_lines.join("\n"),
            oracle_lines.join("\n"),
        );
        assert_eq!(oracle_lines, expected_lines);
    }
    if actual_lines != expected_lines {
        eprintln!(
            "EXPECTED:\n{}\nACTUAL:\n{}",
            expected_lines.join("\n"),
            actual_lines.join("\n"),
        );
        assert_eq!(actual_lines, expected_lines);
    }
}

fn assert_ppp_first(
    notation: Notation,
    width: usize,
    num_first_lines: usize,
    expected_lines: &[&str],
) {
    notation.validate().expect("failed to validate");
    let measured_notation = notation.measure();
    let oracle_lines: Vec<String> = expand_lines(oracular_pretty_print(&notation, width))
        .take(num_first_lines)
        .collect();
    let actual_lines_iter = partial_pretty_print_first(&measured_notation, width);
    let actual_lines: Vec<String> =
        expand_lines(actual_lines_iter.take(num_first_lines).collect()).collect();
    if oracle_lines != expected_lines {
        eprintln!(
            "BAD TEST CASE!\n\nTEST CASE EXPECTS THE LAST {} LINES TO BE:\n{}\nBUT ORACLE SAYS THEY ARE:\n{}",
            num_first_lines,
            expected_lines.join("\n"),
            oracle_lines.join("\n"),
        );
        assert_eq!(oracle_lines, expected_lines);
    }
    if actual_lines != expected_lines {
        eprintln!(
            "EXPECTED LAST {} LINES:\n{}\nACTUAL:\n{}",
            num_first_lines,
            expected_lines.join("\n"),
            actual_lines.join("\n"),
        );
        assert_eq!(actual_lines, expected_lines);
    }
}

fn assert_ppp_last(
    notation: Notation,
    width: usize,
    num_last_lines: usize,
    expected_lines: &[&str],
) {
    notation.validate().expect("failed to validate");
    let measured_notation = notation.measure();
    let oracle_lines: Vec<String> = expand_lines(oracular_pretty_print(&notation, width))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .take(num_last_lines)
        .rev()
        .collect();
    let actual_lines_iter = partial_pretty_print_last(&measured_notation, width);
    let mut actual_lines: Vec<String> =
        expand_lines(actual_lines_iter.take(num_last_lines).collect()).collect();
    actual_lines.reverse();
    if oracle_lines != expected_lines {
        eprintln!(
            "BAD TEST CASE!\n\nTEST CASE EXPECTS THE FIRST {} LINES TO BE:\n{}\nBUT ORACLE SAYS THEY ARE:\n{}",
            num_last_lines,
            expected_lines.join("\n"),
            oracle_lines.join("\n"),
        );
        assert_eq!(oracle_lines, expected_lines);
    }
    if actual_lines != expected_lines {
        eprintln!(
            "EXPECTED FIRST {} LINES:\n{}\nACTUAL:\n{}",
            num_last_lines,
            expected_lines.join("\n"),
            actual_lines.join("\n"),
        );
        assert_eq!(actual_lines, expected_lines);
    }
}

fn assert_ppp_seek(notation: Notation, width: usize, sought_pos: Pos, expected_lines: &[&str]) {
    notation.validate().expect("failed to validate");
    let measured_notation = notation.measure();
    let oracle_lines: Vec<String> = expand_lines(oracular_pretty_print(&notation, width)).collect();
    let (bw_iter, fw_iter) = partial_pretty_print(&measured_notation, width, sought_pos);
    let lines_iter = bw_iter.collect::<Vec<_>>().into_iter().rev().chain(fw_iter);
    let actual_lines: Vec<String> = expand_lines(lines_iter.collect()).collect();
    if oracle_lines != expected_lines {
        eprintln!(
            "BAD TEST CASE!\n\nWHEN SEEKING POS {}, TEST CASE EXPECTS THE LINES TO BE:\n{}\nBUT ORACLE SAYS THEY ARE:\n{}",
            sought_pos,
            expected_lines.join("\n"),
            oracle_lines.join("\n"),
        );
        assert_eq!(oracle_lines, expected_lines);
    }
    if actual_lines != expected_lines {
        eprintln!(
            "WHEN SEEKING POS {}, EXPECTED LINES:\n{}\nACTUAL LINES:\n{}",
            sought_pos,
            expected_lines.join("\n"),
            actual_lines.join("\n"),
        );
        assert_eq!(actual_lines, expected_lines);
    }
}

/********************************************************************************/
/*   Notation Construction                                                      */
/********************************************************************************/

fn hello() -> Notation {
    lit("Hello")
}

fn goodbye() -> Notation {
    lit("Good") + nest(0, lit("Bye"))
}

fn list_one(element: Notation) -> Notation {
    let option1 = lit("[") + element.clone() + lit("]");
    let option2 = lit("[") + align(nest(0, element)) + nest(0, lit("]"));
    option1 | option2
}

fn list_align(elements: Vec<Notation>) -> Notation {
    let empty = lit("[]");
    let lone = |elem| lit("[") + elem + lit("]");
    let join = |elem: Notation, accum: Notation| {
        (elem.clone() + lit(", ") + accum.clone()) | (elem.clone() + lit(",") + nest(0, accum))
    };
    let surround = |accum: Notation| {
        let single = lit("[") + flat(accum.clone()) + lit("]");
        let multi = align(lit("[") + nest(1, accum) + nest(0, lit("]")));
        single | multi
    };
    Notation::repeat(elements, empty, lone, join, surround)
}

fn json_string(s: &str) -> Notation {
    // Using single quote instead of double quote to avoid inconvenient
    // escaping
    lit("'") + lit(s) + lit("'")
}

fn json_key(s: &str) -> Notation {
    json_string(s)
}

fn json_entry(key: &str, value: Notation) -> Notation {
    json_key(key) + lit(": ") + value
}

fn json_dict(entries: Vec<Notation>) -> Notation {
    let tab = 4;
    let empty = lit("{}");
    let lone = |elem: Notation| {
        let single = lit("{") + flat(elem.clone()) + lit("}");
        let multi = lit("{") + nest(tab, elem) + nest(0, lit("}"));
        single | multi
    };
    let join = |elem: Notation, accum: Notation| elem + lit(",") + nest(0, accum);
    let surround = |accum: Notation| {
        let single = lit("{") + flat(accum.clone()) + lit("}");
        let multi = lit("{") + nest(tab, accum) + nest(0, lit("}"));
        single | multi
    };
    Notation::repeat(entries, empty, lone, join, surround)
}

fn expand_line(indent: usize, line: String) -> String {
    format!("{:indent$}{}", "", line, indent = indent)
}

fn expand_lines(lines: Vec<(usize, String)>) -> impl Iterator<Item = String> {
    lines.into_iter().map(|(i, s)| expand_line(i, s))
}

/********************************************************************************/
/*   Unit Tests                                                                 */
/********************************************************************************/

#[test]
fn test_pp_hello() {
    let n = lit("Hello") + nest(4, lit("world!"));
    assert_pp(n, 80, &["Hello", "    world!"])
}

#[test]
fn test_pp_choice() {
    let n = (hello() | goodbye()) + lit(" world");
    assert_pp(n, 80, &["Hello world"]);

    let n = (goodbye() | hello()) + lit(" world");
    assert_pp(n, 80, &["Good", "Bye world"]);

    let n = (goodbye() | goodbye()) + lit(" world");
    assert_pp(n, 80, &["Good", "Bye world"]);

    let n = (flat(goodbye()) | hello()) + lit(" world");
    assert_pp(n, 80, &["Hello world"]);

    let n = (hello() | goodbye()) + lit(" world");
    assert_pp(n, 10, &["Good", "Bye world"]);

    let n = (hello() | goodbye()) + lit(" world");
    assert_pp(n, 3, &["Good", "Bye world"]);
}

#[test]
fn test_pp_list_one() {
    let n = list_one(hello());
    assert_pp(n, 80, &["[Hello]"]);

    let n = list_one(goodbye());
    assert_pp(n, 80, &["[Good", "Bye]"]);
    // TODO test nest case
}

#[test]
fn test_pp_list() {
    let n = list_align(vec![]);
    assert_pp(n, 80, &["[]"]);

    let n = list_align(vec![hello()]);
    assert_pp(n, 80, &["[Hello]"]);

    let n = list_align(vec![hello(), hello()]);
    assert_pp(n, 80, &["[Hello, Hello]"]);

    let n = list_align(vec![hello(), hello()]);
    assert_pp(n, 13, &["[", " Hello, Hello", "]"]);

    let n = list_align(vec![hello(), hello()]);
    assert_pp(n, 10, &["[", " Hello,", " Hello", "]"]);
    let n = list_align(vec![goodbye()]);
    assert_pp(n, 80, &["[Good", "Bye]"]);

    let n = list_align(vec![hello(), hello(), hello(), hello()]);
    assert_pp(n, 15, &["[", " Hello, Hello,", " Hello, Hello", "]"]);

    let n = list_align(vec![goodbye(), hello(), hello()]);
    assert_pp(n, 80, &["[", " Good", " Bye, Hello, Hello", "]"]);

    let n = list_align(vec![goodbye(), hello(), hello(), goodbye()]);
    assert_pp(
        n,
        80,
        &["[", " Good", " Bye, Hello, Hello, Good", " Bye", "]"],
    );
}

#[test]
fn test_pp_dict() {
    let e1 = json_entry("Name", json_string("Alice"));
    let e2 = json_entry("Age", lit("42"));
    let favorites_list = list_align(vec![
        json_string("chocolate"),
        json_string("lemon"),
        json_string("almond"),
    ]);
    let e3 = json_entry("Favorites", favorites_list.clone());

    let n = json_dict(vec![e1.clone()]);
    assert_pp(n, 80, &["{'Name': 'Alice'}"]);

    let n = json_dict(vec![e1.clone(), e2.clone()]);
    assert_pp(
        n,
        80,
        &[
            // force rustfmt
            "{",
            "    'Name': 'Alice',",
            "    'Age': 42",
            "}",
        ],
    );

    assert_pp(
        favorites_list.clone(),
        20,
        &["[", " 'chocolate',", " 'lemon', 'almond'", "]"],
    );

    //This can fit in 34, but the pretty printer puts them all on separate lines!
    assert_pp(
        e3.clone(),
        34,
        &[
            "'Favorites': [",
            "              'chocolate',",
            "              'lemon', 'almond'",
            "             ]",
        ],
    );

    // This looks totally broken
    let n = json_dict(vec![e1, e2, e3]);
    assert_pp(
        n,
        38,
        &[
            "{",
            "    'Name': 'Alice',",
            "    'Age': 42,",
            "    'Favorites': [",
            "                  'chocolate',",
            "                  'lemon', 'almond'",
            "                 ]",
            "}",
        ],
    );
}

#[test]
fn test_pp_align() {
    let n = lit("four") + list_align(vec![hello(), hello()]);
    assert_pp(
        n,
        10,
        &[
            // make rustfmt split lines
            "four[",
            "     Hello,",
            "     Hello",
            "    ]",
        ],
    );
}

/********************************************************************************/
/*   Regression Tests                                                           */
/********************************************************************************/

#[test]
fn oracle_failure_1() {
    let n = flat(lit("aa") | lit("b")) | nest(0, lit(""));
    assert_pp(n, 1, &["b"]);
}

#[test]
// NOTE: This test case originally expected `bb` instead of `a`, because that
// was the "correct" but questionable behavior of a previous pretty-printing
// algorithm.
fn oracle_failure_2() {
    let n = (lit("a") | lit("bb")) + nest(9, lit(""));
    assert_pp(n, 5, &["a", "         "]);
}

#[test]
fn oracle_failure_3() {
    let n = flat(lit("aaaaaaaa") + (lit("") | lit("cccccc")));
    assert_pp(n, 6, &["aaaaaaaacccccc"]);
}

#[test]
fn oracle_failure_4() {
    let n = nest(8, lit("")) | nest(0, lit("")) | lit("aaaaaaa");
    assert_pp(n, 5, &["", ""]);
}

#[test]
fn oracle_failure_5() {
    let n = align(nest(6, lit("aaaaaaaa"))) + align(lit("bbbbbbbbb"));
    assert_pp(n, 10, &["", "      aaaaaaaabbbbbbbbb"]);
}

#[test]
fn oracle_failure_6() {
    let n = nest(7, lit("aaaa")) + align(nest(3, lit("bbb")));
    assert_pp(n, 5, &["", "       aaaa", "              bbb"]);
}

#[test]
fn oracle_failure_7() {
    let n = flat(align(nest(2, lit("aaaa")) | lit("bbbbb")));
    assert_pp(n, 6, &["bbbbb"]);
}

#[test]
fn oracle_failure_8() {
    let n = lit("a") + align(align(nest(7, lit("bbbbbbbb"))));
    assert_pp(n, 5, &["a", "        bbbbbbbb"]);
}

#[test]
fn oracle_failure_9() {
    let n = (align(nest(6, lit("aaaaaaaaa"))) + nest(3, lit("bb"))) | lit("ccc");
    assert_pp(n, 5, &["", "      aaaaaaaaa", "   bb"]);
}

#[test]
fn oracle_failure_10() {
    let n = (flat(nest(3, lit("aaaaaa"))) + align(nest(3, lit("bb")))) | lit("c");
    assert_pp(n, 1, &["c"]);
}

#[test]
fn oracle_failure_11() {
    let n = (nest(8, lit("aaaaa")) + (nest(0, lit("bbbb")) | lit("ccc"))) | lit("ddddd");
    assert_pp(n, 4, &["", "        aaaaaccc"]);
}

#[test]
fn oracle_ppp_first_failure_1() {
    let n = (lit("aaaaa") | lit("bbbbb")) + newline();
    assert_ppp_first(n, 1, 2, &["bbbbb", ""]);
}

#[test]
fn oracle_ppp_first_failure_2() {
    let n = flat(newline() | lit("aaaaaa")) + (lit("bb") | lit(""));
    assert_ppp_first(n, 2, 1, &["aaaaaa"]);
}

#[test]
fn oracle_ppp_first_failure_3() {
    let n = newline();
    assert_ppp_first(n, 1, 3, &["", ""]);
}

#[test]
fn oracle_ppp_last_failure_1() {
    let n = lit("a") + lit("b");
    assert_ppp_last(n, 2, 1, &["ab"]);
}

#[test]
fn oracle_ppp_last_failure_2() {
    let n = indent(5, newline()) + (newline() | lit("aaaaaaaa"));
    assert_ppp_last(n, 1, 1, &["     aaaaaaaa"]);
}

#[test]
fn oracle_ppp_last_failure_3() {
    let n = indent(5, newline()) + (newline() | lit("aaaaaaaa"));
    assert_ppp_last(n, 5, 2, &["     ", ""]);
}

#[test]
fn oracle_ppp_last_failure_4() {
    let n = indent(6, newline()) + ((newline() | newline()) | indent(9, lit("a")));
    assert_ppp_last(n, 6, 2, &["      ", ""]);
}

#[test]
fn oracle_ppp_seek_failure_1() {
    let n = lit("aaaaaa") + newline();
    assert_ppp_seek(n, 1, 2, &["aaaaaa", ""]);
}
