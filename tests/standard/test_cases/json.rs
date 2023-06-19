extern crate test;

use crate::standard::pretty_testing::{assert_pp, assert_pp_region, assert_pp_seek, print_region};
use partial_pretty_printer::examples::json::{
    json_bool, json_dict, json_dict_entry, json_list, json_null, json_number, json_string, Json,
};
use test::Bencher;

static NUMERALS: &[&str] = &[
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
    "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
];

fn entry_1() -> Json {
    json_dict_entry("Name", json_string("Alice"))
}

fn entry_2() -> Json {
    json_dict_entry("Age", json_number(42.0))
}

fn favorites_list() -> Json {
    json_list(vec![
        json_string("chocolate"),
        json_string("lemon"),
        json_string("almond"),
    ])
}

fn entry_3() -> Json {
    json_dict_entry("Favorites", favorites_list())
}

fn dictionary() -> Json {
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
fn json_list_of_dicts() {
    assert_pp(
        &json_list(vec![dictionary(), dictionary()]),
        40,
        &[
            "[",
            "    {",
            "        \"Name\": \"Alice\",",
            "        \"Age\": 42,",
            "        \"Favorites\": [",
            "            \"chocolate\", \"lemon\",",
            "            \"almond\"",
            "        ]",
            "    }, {",
            "        \"Name\": \"Alice\",",
            "        \"Age\": 42,",
            "        \"Favorites\": [",
            "            \"chocolate\", \"lemon\",",
            "            \"almond\"",
            "        ]",
            "    }",
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

fn make_json_tree(id: u32, size: usize) -> Json {
    let children = (0..size)
        .map(|n| make_json_tree(2u32.pow(n as u32) + id, n))
        .collect::<Vec<_>>();
    let children_lengths = (0..size)
        .map(|n| json_string(&format!("child_number_{}", NUMERALS[n])))
        .collect::<Vec<_>>();
    json_dict(vec![
        json_dict_entry("id", json_number(id as f64)),
        json_dict_entry("children_lengths", json_list(children_lengths)),
        json_dict_entry("number_of_children", json_number(size as f64)),
        json_dict_entry("children", json_list(children)),
    ])
}

#[bench]
fn json_tree_bench(bencher: &mut Bencher) {
    let little_tree = make_json_tree(0, 2);
    assert_pp(
        &little_tree,
        80,
        &[
            "{",
            "    \"id\": 0,",
            "    \"children_lengths\": [\"child_number_one\", \"child_number_two\"],",
            "    \"number_of_children\": 2,",
            "    \"children\": [",
            "        {",
            "            \"id\": 1,",
            "            \"children_lengths\": [],",
            "            \"number_of_children\": 0,",
            "            \"children\": []",
            "        }, {",
            "            \"id\": 2,",
            "            \"children_lengths\": [\"child_number_one\"],",
            "            \"number_of_children\": 1,",
            "            \"children\": [{",
            "                \"id\": 3,",
            "                \"children_lengths\": [],",
            "                \"number_of_children\": 0,",
            "                \"children\": []",
            "            }]",
            "        }",
            "    ]",
            "}",
        ],
    );

    // 400k lines long, at width 120
    let big_tree = make_json_tree(0, 16);

    // Print the middle of the doc, at line ~200k of ~400k
    assert_pp_region(
        &big_tree,
        120,
        // 3,1,15 means "child 15"
        &[3, 1, 15, 3, 1, 10],
        10,
        &[
"                                    ]",
"                                }",
"                            ]",
"                        }",
"                    ]",
"                }, {",
"                    \"id\": 33792,",
"                    \"children_lengths\": [",
"                        \"child_number_one\", \"child_number_two\", \"child_number_three\", \"child_number_four\",",
"                        \"child_number_five\", \"child_number_six\", \"child_number_seven\", \"child_number_eight\",",
        ]);
    bencher.iter(|| {
        print_region(&big_tree, 120, &[3, 1, 15, 3, 1, 10], 80);
    });

    #[cfg(feature = "profile")]
    {
        use no_nonsense_flamegraphs::span;

        span!("Json bench test");
        print_region(&big_tree, 120, &[3, 1, 15, 3, 1, 10], 80);
    }
}
