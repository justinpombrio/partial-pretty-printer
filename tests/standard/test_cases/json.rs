use crate::standard::pretty_testing::{
    assert_pp, assert_pp_focus, assert_pp_region, assert_pp_seek,
};
use partial_pretty_printer::doc_examples::json::{
    json_array, json_bool, json_comment, json_null, json_number, json_object, json_object_pair,
    json_roots, json_string, Json,
};
use partial_pretty_printer::FocusTarget;

static NUMERALS: &[&str] = &[
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
    "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
];

#[test]
fn json_constants() {
    let doc = json_array(vec![json_bool(true), json_null(), json_bool(false)]);
    assert_pp(&doc, 80, &["[true, null, false]"]);
}

#[test]
fn json_flow() {
    let doc = json_roots(vec![
        json_comment(
        "Truth is much too complicated to allow anything but approximations. — John Von Neumann"),
        json_array(vec![json_bool(true), json_bool(false)])
    ]);
    assert_pp(
        &doc,
        40,
        &[
            //   5   10   15   20   25   30   35   40
            "// Truth is much too complicated to",
            "// allow anything but approximations. —",
            "// John Von Neumann",
            "[true, false]",
        ],
    );
}

#[test]
fn json_seek() {
    let doc = json_object(vec![
        json_object_pair("Name", json_string("Alice")),
        json_object_pair("Age", json_number(42.0)),
    ]);
    let refn = &doc;

    assert_pp(
        refn,
        28,
        &[
            //     5   10   15   20   25   30
            r#"{"Name": "Alice", "Age": 42}"#,
        ],
    );
    assert_pp(
        refn,
        27,
        &[
            // force rustfmt
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42",
            "}",
        ],
    );

    assert_pp_seek(
        refn,
        27,
        &[],
        &[
            // force rustfmt
            "({",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42",
            "})",
        ],
    );
    assert_pp_seek(
        refn,
        27,
        &[0],
        &[
            // force rustfmt
            "{",
            "    (\"Name\": \"Alice\"),",
            "    \"Age\": 42",
            "}",
        ],
    );
    assert_pp_seek(
        refn,
        27,
        &[0, 0],
        &[
            // force rustfmt
            "{",
            "    (\"Name\"): \"Alice\",",
            "    \"Age\": 42",
            "}",
        ],
    );
    assert_pp_seek(
        refn,
        27,
        &[1],
        &[
            // force rustfmt
            "{",
            "    \"Name\": \"Alice\",",
            "    (\"Age\": 42)",
            "}",
        ],
    );
    assert_pp_seek(
        refn,
        27,
        &[1, 1],
        &[
            // force rustfmt
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": (42)",
            "}",
        ],
    );
}

#[test]
fn json_focus() {
    let doc = json_object(vec![
        json_object_pair("Cats", json_array(Vec::new())),
        json_object_pair("Dogs", json_array(Vec::new())),
    ]);
    let refn = &doc;

    assert_pp_focus(
        refn,
        17,
        &[1, 1],
        FocusTarget::Start,
        &[
            // force rustfmt
            "{",
            "    \"Cats\": [],",
            "    \"Dogs\": |[]",
            "}",
        ],
    );

    assert_pp_focus(
        refn,
        17,
        &[1, 1],
        FocusTarget::End,
        &[
            // force rustfmt
            "{",
            "    \"Cats\": [],",
            "    \"Dogs\": []|",
            "}",
        ],
    );

    assert_pp_focus(
        refn,
        17,
        &[1, 1],
        FocusTarget::Mark,
        &[
            // force rustfmt
            "{",
            "    \"Cats\": [],",
            "    \"Dogs\": [|]",
            "}",
        ],
    );

    assert_pp_focus(
        refn,
        17,
        &[1, 0],
        FocusTarget::Text(2),
        &[
            // force rustfmt
            "{",
            "    \"Cats\": [],",
            "    \"Do|gs\": []",
            "}",
        ],
    );

    assert_pp_focus(
        refn,
        17,
        &[1, 0],
        FocusTarget::Text(20),
        &[
            // force rustfmt
            "{",
            "    \"Cats\": [],",
            "    \"Dogs|\": []",
            "}",
        ],
    );
}

#[test]
#[should_panic(expected = "InvalidPath")]
fn json_invalid_path() {
    let doc = json_object(vec![
        json_object_pair("x", json_number(1.0)),
        json_object_pair("y", json_number(2.0)),
    ]);
    assert_pp_seek(&doc, 80, &[0, 2], &[]);
}

fn favorites_array() -> Json {
    json_array(vec![
        json_string("chocolate"),
        json_string("lemon"),
        json_string("almond"),
    ])
}

fn make_object() -> Json {
    json_object(vec![
        json_object_pair("Name", json_string("Alice")),
        json_object_pair("Age", json_number(42.0)),
        json_object_pair("Favorites", favorites_array()),
    ])
}

#[test]
fn json_flow_wrapped_array() {
    assert_pp(
        &favorites_array(),
        24,
        &[
            // force rustfmt
            "[",
            "    \"chocolate\",",
            "    \"lemon\",",
            "    \"almond\"",
            "]",
        ],
    );
}

#[test]
fn json_array_of_objects() {
    assert_pp(
        &json_array(vec![make_object(), make_object()]),
        40,
        &[
            "[",
            "    {",
            "        \"Name\": \"Alice\",",
            "        \"Age\": 42,",
            "        \"Favorites\": [",
            "            \"chocolate\",",
            "            \"lemon\",",
            "            \"almond\"",
            "        ]",
            "    },",
            "    {",
            "        \"Name\": \"Alice\",",
            "        \"Age\": 42,",
            "        \"Favorites\": [",
            "            \"chocolate\",",
            "            \"lemon\",",
            "            \"almond\"",
            "        ]",
            "    }",
            "]",
        ],
    );
}

#[test]
fn json_big_object() {
    assert_pp(
        &make_object(),
        27,
        &[
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42,",
            "    \"Favorites\": [",
            "        \"chocolate\",",
            "        \"lemon\",",
            "        \"almond\"",
            "    ]",
            "}",
        ],
    );

    assert_pp(
        &make_object(),
        60,
        &[
            "{",
            "    \"Name\": \"Alice\",",
            "    \"Age\": 42,",
            "    \"Favorites\": [\"chocolate\", \"lemon\", \"almond\"]",
            "}",
        ],
    );
}

#[test]
fn json_comment_line_breaks() {
    let array = json_array(vec![
        json_number(1.0),
        json_comment("two"),
        json_number(2.0),
        json_number(3.0),
    ]);

    assert_pp(
        &array,
        80,
        &[
            // force rustfmt
            "[",
            "    1,",
            "    // two",
            "    2,",
            "    3",
            "]",
        ],
    );
}

#[test]
fn json_comment_commas() {
    let array = json_array(vec![
        json_number(1.0),
        json_comment("two"),
        json_number(2.0),
        json_comment("three?"),
        json_comment("three."),
        json_number(3.0),
    ]);

    assert_pp(
        &array,
        10,
        &[
            // force rustfmt
            "[",
            "    1,",
            "    // two",
            "    2,",
            "    // three?",
            "    // three.",
            "    3",
            "]",
        ],
    );

    let array = json_array(vec![
        json_number(1.0),
        json_comment("two"),
        json_number(2.0),
        json_number(3.0),
        json_comment("^ three?"),
        json_comment("^ three."),
    ]);

    assert_pp(
        &array,
        20,
        &[
            // force rustfmt
            "[",
            "    1,",
            "    // two",
            "    2,",
            "    3",
            "    // ^ three?",
            "    // ^ three.",
            "]",
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
    json_object(vec![
        json_object_pair("id", json_number(id as f64)),
        json_object_pair("children_lengths", json_array(children_lengths)),
        json_object_pair("number_of_children", json_number(size as f64)),
        json_object_pair("children", json_array(children)),
    ])
}

#[test]
fn big_json_tree() {
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
            "        },",
            "        {",
            "            \"id\": 2,",
            "            \"children_lengths\": [\"child_number_one\"],",
            "            \"number_of_children\": 1,",
            "            \"children\": [",
            "                {",
            "                    \"id\": 3,",
            "                    \"children_lengths\": [],",
            "                    \"number_of_children\": 0,",
            "                    \"children\": []",
            "                }",
            "            ]",
            "        }",
            "    ]",
            "}",
        ],
    );

    // 480k lines long at width 120
    let big_tree = make_json_tree(0, 16);

    // Print the middle of the doc
    assert_pp_region(
        &big_tree,
        120,
        // 3,1,15 means "child 15"
        &[3, 1, 15, 3, 1, 10],
        FocusTarget::Start,
        10,
        &[
            "                                }",
            "                            ]",
            "                        }",
            "                    ]",
            "                },",
            "                {",
            "                    \"id\": 33792,",
            "                    \"children_lengths\": [",
            "                        \"child_number_one\",",
            "                        \"child_number_two\",",
        ],
    );
}

#[test]
#[ignore]
// cargo test --release time_json -- --include-ignored
// Currently takes ~1.1ms on Yoga
fn time_json() {
    use crate::standard::pretty_testing::print_region;
    use std::time::Instant;

    // 480k lines long at width 120
    let big_tree = make_json_tree(0, 16);

    let start = Instant::now();
    print_region(
        &big_tree,
        120,
        &[3, 1, 15, 3, 1, 10],
        FocusTarget::Start,
        80,
    );
    println!(
        "Time to print middle 80 lines of ~480k line doc at width 120: {}μs",
        start.elapsed().as_micros()
    );
    panic!("Success!");
}
