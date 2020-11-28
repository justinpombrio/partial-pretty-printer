mod if_flat {
    use partial_pretty_printer::if_flat::{pretty_print, Notation};

    // TODO: min_width -> fits_sl_width

    // TODO: Put these in a shared common file. Break this file into several.

    // TODO: Tests:
    // x Json
    // - text flow
    // - imports with multi-line import not sharing lines
    // - let w/ list
    // - iter w/ map & closure

    fn nl() -> Notation {
        Notation::Newline
    }

    fn lit(s: &str) -> Notation {
        Notation::Literal(s.to_string())
    }

    fn assert_pp(notation: &Notation, width: usize, expected_lines: &[&str]) {
        let actual_lines = pretty_print(&notation, width);
        if actual_lines != expected_lines {
            eprintln!(
                "EXPECTED:\n{}\nACTUAL:\n{}",
                expected_lines.join("\n"),
                actual_lines.join("\n"),
            );
            assert_eq!(actual_lines, expected_lines);
        }
    }

    #[test]
    fn basics() {
        // Empty
        assert_pp(&Notation::Empty, 80, &[""]);
        // Literal
        assert_pp(&lit("Hello world!"), 80, &["Hello world!"]);
        // Concat
        assert_pp(&(lit("Hello") + lit(" world!")), 80, &["Hello world!"]);
        // Newline
        assert_pp(&(lit("Hello") ^ lit("world!")), 80, &["Hello", "world!"]);
        // Indent
        assert_pp(
            &(lit("Hello") + (2 >> lit("world!"))),
            80,
            &["Hello", "  world!"],
        );
        // Choice
        let notation = lit("Hello world!") | lit("Hello") ^ lit("world!");
        assert_pp(&notation, 12, &["Hello world!"]);
        assert_pp(&notation, 11, &["Hello", "world!"]);
    }

    #[test]
    fn json() {
        fn json_string(s: &str) -> Notation {
            // Using single quote instead of double quote to avoid inconvenient
            // escaping
            lit("'") + lit(s) + lit("'")
        }

        // TODO: allow newline?
        fn json_entry(key: &str, value: Notation) -> Notation {
            json_string(key) + lit(": ") + value
        }

        fn json_list(elements: Vec<Notation>) -> Notation {
            let tab = 4;
            let empty = lit("[]");
            let lone = |elem| lit("[") + elem + lit("]");
            let join =
                |elem: Notation, accum: Notation| elem + lit(",") + (lit(" ") | nl()) + accum;
            let surround = |accum: Notation| {
                let single = lit("[") + accum.clone() + lit("]");
                let multi = lit("[") + (tab >> accum) ^ lit("]");
                single | multi
            };
            Notation::repeat(elements, empty, lone, join, surround)
        }

        fn json_dict(entries: Vec<Notation>) -> Notation {
            let tab = 4;
            let empty = lit("{}");
            let lone = |elem: Notation| {
                let single = lit("{") + elem.clone() + lit("}");
                let multi = lit("{") + (tab >> elem) ^ lit("}");
                single | multi
            };
            let join = |elem: Notation, accum: Notation| elem + lit(",") + nl() + accum;
            let surround = |accum: Notation| {
                // This single case is never used, because `accum` is never flat!
                let single = lit("{") + accum.clone() + lit("}");
                let multi = lit("{") + (tab >> accum) ^ lit("}");
                single | multi
            };
            Notation::repeat(entries, empty, lone, join, surround)
        }

        let e1 = json_entry("Name", json_string("Alice"));
        let e2 = json_entry("Age", lit("42"));
        let favorites_list = json_list(vec![
            json_string("chocolate"),
            json_string("lemon"),
            json_string("almond"),
        ]);
        let e3 = json_entry("Favorites", favorites_list.clone());
        let dict = json_dict(vec![e1.clone(), e2.clone(), e3.clone()]);

        assert_pp(
            &json_dict(vec![e1.clone(), e2.clone()]),
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
            &favorites_list,
            24,
            &[
                // force rustfmt
                "[",
                "    'chocolate',",
                "    'lemon', 'almond'",
                "]",
            ],
        );

        assert_pp(
            &e3,
            27,
            &[
                "'Favorites': [",
                "    'chocolate', 'lemon',",
                "    'almond'",
                "]",
            ],
        );

        assert_pp(
            &dict,
            27,
            &[
                "{",
                "    'Name': 'Alice',",
                "    'Age': 42,",
                "    'Favorites': [",
                "        'chocolate',",
                "        'lemon', 'almond'",
                "    ]",
                "}",
            ],
        );

        assert_pp(
            &dict,
            60,
            &[
                "{",
                "    'Name': 'Alice',",
                "    'Age': 42,",
                "    'Favorites': ['chocolate', 'lemon', 'almond']",
                "}",
            ],
        );

        assert_pp(
            &dict,
            40,
            &[
                "{",
                "    'Name': 'Alice',",
                "    'Age': 42,",
                "    'Favorites': [",
                "        'chocolate', 'lemon', 'almond'",
                "    ]",
                "}",
            ],
        );
    }
}
