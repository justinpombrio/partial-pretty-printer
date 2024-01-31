use crate::standard::pretty_testing::{assert_pp, assert_pp_region, assert_pp_seek};
use partial_pretty_printer::examples::json::{
    json_bool, json_dict, json_list, json_null, json_number, json_string, Json,
};

static NUMERALS: &[&str] = &[
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
    "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
];

#[test]
fn json_constants() {
    let doc = json_list(vec![json_bool(true), json_null(), json_bool(false)]);
    assert_pp(doc.as_ref(), 80, &["[true, null, false]"]);
}

#[test]
fn json_flow() {
    let doc = json_list(vec![json_bool(true), json_bool(false)]).with_comment(
        "Truth is much too complicated to allow anything but approximations. — John Von Neumann"
            .to_owned(),
    );
    assert_pp(
        doc.as_ref(),
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
    let doc = json_dict(vec![
        ("Name", json_string("Alice")),
        ("Age", json_number(42.0)),
    ]);
    let refn = doc.as_ref();

    assert_pp(
        refn,
        28,
        &[
            //     5   10   15   20   25   30
            r#"{"Name": "Alice", "Age": 42}"#,
        ],
    );

    assert_pp_seek(
        refn,
        27,
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
        refn,
        27,
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
        refn,
        27,
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
        refn,
        27,
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
        refn,
        27,
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
}

#[test]
#[should_panic(expected = "InvalidPath")]
fn json_invalid_path() {
    let doc = json_dict(vec![("x", json_number(1.0)), ("y", json_number(2.0))]);
    assert_pp_seek(doc.as_ref(), 80, &[0, 2], &[], &[]);
}

fn favorites_list() -> Json {
    json_list(vec![
        json_string("chocolate"),
        json_string("lemon"),
        json_string("almond"),
    ])
}

fn dictionary() -> Json {
    json_dict(vec![
        ("Name", json_string("Alice")),
        ("Age", json_number(42.0)),
        ("Favorites", favorites_list()),
    ])
}

#[test]
fn json_flow_wrapped_list() {
    assert_pp(
        favorites_list().as_ref(),
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
fn json_list_of_dicts() {
    assert_pp(
        json_list(vec![dictionary(), dictionary()]).as_ref(),
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
fn json_big_dict() {
    assert_pp(
        dictionary().as_ref(),
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
        dictionary().as_ref(),
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

fn make_json_tree(id: u32, size: usize) -> Json {
    let children = (0..size)
        .map(|n| make_json_tree(2u32.pow(n as u32) + id, n))
        .collect::<Vec<_>>();
    let children_lengths = (0..size)
        .map(|n| json_string(&format!("child_number_{}", NUMERALS[n])))
        .collect::<Vec<_>>();
    json_dict(vec![
        ("id", json_number(id as f64)),
        ("children_lengths", json_list(children_lengths)),
        ("number_of_children", json_number(size as f64)),
        ("children", json_list(children)),
    ])
}

#[test]
fn big_json_tree() {
    let little_tree = make_json_tree(0, 2);
    assert_pp(
        little_tree.as_ref(),
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
        big_tree.as_ref(),
        120,
        // 3,1,15 means "child 15"
        &[3, 1, 15, 3, 1, 10],
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
// cargo test time_json -- --include-ignored
fn time_json() {
    use crate::standard::pretty_testing::print_region;
    use std::time::Instant;

    // 480k lines long at width 120
    let big_tree = make_json_tree(0, 16);

    let start = Instant::now();
    print_region(big_tree.as_ref(), 120, &[3, 1, 15, 3, 1, 10], 80);
    println!(
        "Time to print middle 80 lines of ~480k line doc at width 120: {}μs",
        start.elapsed().as_micros()
    );
    panic!("Success!");
}
