#![feature(test)]

extern crate test;

mod common;

use common::{assert_pp, assert_pp_seek, print_region};
use partial_pretty_printer::examples::json::{
    json_bool, json_dict, json_dict_entry, json_list, json_null, json_number, json_string, Json,
};
use partial_pretty_printer::examples::Doc;
use test::Bencher;

fn entry_1() -> Doc<Json> {
    json_dict_entry("Name", json_string("Alice"))
}

fn entry_2() -> Doc<Json> {
    json_dict_entry("Age", json_number(42))
}

fn favorites_list() -> Doc<Json> {
    json_list(vec![
        json_string("chocolate"),
        json_string("lemon"),
        json_string("almond"),
    ])
}

fn entry_3() -> Doc<Json> {
    json_dict_entry("Favorites", favorites_list())
}

fn dictionary() -> Doc<Json> {
    json_dict(vec![entry_1(), entry_2(), entry_3()])
}

#[test]
fn json_constants() {
    let doc = json_list(vec![json_bool(true), json_null(), json_bool(false)]);
    assert_pp(&doc, 80, &["[true, null, false]"]);
}

#[test]
fn json_small_dict() {
    let doc = json_dict(vec![entry_1(), entry_2()]);
    assert_pp_seek(
        &doc,
        80,
        &[],
        &[],
        &[
            // force rustfmt
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42",
            "}",
        ],
    );
    assert_pp_seek(
        &doc,
        80,
        &[0],
        &[
            // force rustfmt
            "{",
        ],
        &[
            // force rustfmt
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42",
            "}",
        ],
    );
    assert_pp_seek(
        &doc,
        80,
        &[0, 0],
        &[
            // force rustfmt
            "{",
        ],
        &[
            // force rustfmt
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42",
            "}",
        ],
    );
    assert_pp_seek(
        &doc,
        80,
        &[1],
        &[
            // force rustfmt
            "{",
            "    \"Name\": \"Alice\",",
        ],
        &[
            // force rustfmt
            "    \"Age\": 42",
            "}",
        ],
    );
    assert_pp_seek(
        &doc,
        80,
        &[1, 0],
        &[
            // force rustfmt
            "{",
            "    \"Name\": \"Alice\",",
        ],
        &[
            // force rustfmt
            "    \"Age\": 42",
            "}",
        ],
    );
    assert_pp(
        &doc,
        80,
        &[
            // force rustfmt
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42",
            "}",
        ],
    );
}

#[test]
#[should_panic(expected = "Missing child (2)")]
fn json_invalid_path() {
    let doc = json_dict(vec![entry_1(), entry_2()]);
    assert_pp_seek(&doc, 80, &[0, 2], &[], &[]);
}

#[test]
fn json_flow_wrapped_list() {
    assert_pp(
        &favorites_list(),
        24,
        &[
            // force rustfmt
            "[",
            "    \"chocolate\",",
            "    \"lemon\", \"almond\"",
            "]",
        ],
    );

    assert_pp(
        &entry_3(),
        27,
        &[
            "\"Favorites\": [",
            "    \"chocolate\", \"lemon\",",
            "    \"almond\"",
            "]",
        ],
    );
}

#[test]
fn json_big_dict() {
    assert_pp(
        &dictionary(),
        27,
        &[
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42,",
            "    \"Favorites\": [",
            "        \"chocolate\",",
            "        \"lemon\", \"almond\"",
            "    ]",
            "}",
        ],
    );

    assert_pp(
        &dictionary(),
        60,
        &[
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42,",
            "    \"Favorites\": [\"chocolate\", \"lemon\", \"almond\"]",
            "}",
        ],
    );

    assert_pp(
        &dictionary(),
        40,
        &[
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42,",
            "    \"Favorites\": [",
            "        \"chocolate\", \"lemon\", \"almond\"",
            "    ]",
            "}",
        ],
    );
}

#[bench]
fn json_long_list_bench(bencher: &mut Bencher) {
    let num_elems = 1000;
    let numbers = (0..num_elems).map(|n| json_number(n)).collect::<Vec<_>>();
    let list = json_list(numbers);

    //let lines = print_region(&list, 80, &[num_elems / 2], 60);
    //assert_eq!(lines, &[""]);

    bencher.iter(|| {
        print_region(&list, 80, &[num_elems / 2], 60);
    });
}
