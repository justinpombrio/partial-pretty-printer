mod if_flat {
    use partial_pretty_printer::if_flat::{pretty_print, Notation};

    // TODO: Put these in a shared common file. Break this file into several.

    // TODO: Tests:
    // x Json
    // x text flow
    // n imports with multi-line import not sharing lines
    // - let w/ list w/ an element that doesn't want to share a line
    // - iter w/ map & closure

    fn nl() -> Notation {
        Notation::Newline
    }

    fn lit(s: &str) -> Notation {
        Notation::Literal(s.to_string())
    }

    fn indent(i: usize, n: Notation) -> Notation {
        Notation::Indent(i, Box::new(n))
    }

    fn flat(n: Notation) -> Notation {
        Notation::Flat(Box::new(n))
    }

    #[track_caller]
    fn assert_pp(notation: &Notation, width: usize, expected_lines: &[&str]) {
        let actual_lines = pretty_print(&notation, width);
        if actual_lines != expected_lines {
            eprintln!(
                "EXPECTED:\n{}\nACTUAL:\n{}\n========",
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
            let empty = lit("[]");
            let lone = |elem| lit("[") + elem + lit("]");
            let join =
                |elem: Notation, accum: Notation| elem + lit(",") + (lit(" ") | nl()) + accum;
            let surround = |accum: Notation| {
                let single = lit("[") + flat(accum.clone()) + lit("]");
                let multi = lit("[") + (4 >> accum) ^ lit("]");
                single | multi
            };
            Notation::repeat(elements, empty, lone, join, surround)
        }

        fn json_dict(entries: Vec<Notation>) -> Notation {
            let empty = lit("{}");
            let lone = |elem: Notation| {
                let single = lit("{") + elem.clone() + lit("}");
                let multi = lit("{") + (4 >> elem) ^ lit("}");
                single | multi
            };
            let join = |elem: Notation, accum: Notation| elem + lit(",") + nl() + accum;
            let surround = |accum: Notation| lit("{") + (4 >> accum) ^ lit("}");
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

    #[test]
    fn flow() {
        fn word_flow(words: &[&str]) -> Notation {
            let elements = words.iter().map(|w| lit(w)).collect::<Vec<_>>();
            let empty = lit("");
            let lone = |elem| lit("    ") + elem;
            let soft_break = || lit(" ") | nl();
            let join = |elem: Notation, accum: Notation| elem + lit(",") + soft_break() + accum;
            let surround = |accum: Notation| lit("    ") + accum;
            Notation::repeat(elements, empty, lone, join, surround)
        }

        fn mark_paragraph(notation: Notation) -> Notation {
            lit("¶") + notation + lit("□")
        }

        let n = mark_paragraph(word_flow(&[
            "Oh",
            "woe",
            "is",
            "me",
            "the",
            "turbofish",
            "remains",
            "undefeated",
        ]));

        assert_pp(
            &n,
            80,
            //0    5   10   15   20   25   30   35   40   45   50   55   60
            &["¶    Oh, woe, is, me, the, turbofish, remains, undefeated□"],
        );
        assert_pp(
            &n,
            46,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe, is, me, the, turbofish, remains,",
                "undefeated□",
            ],
        );
        assert_pp(
            &n,
            45,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe, is, me, the, turbofish,",
                "remains, undefeated□",
            ],
        );
        assert_pp(
            &n,
            20,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe, is,",
                "me, the, turbofish,",
                "remains, undefeated□",
            ],
        );
        assert_pp(
            &n,
            19,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe, is,",
                "me, the, turbofish,",
                "remains,",
                "undefeated□",
            ],
        );
        assert_pp(
            &n,
            18,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe, is,",
                "me, the,",
                "turbofish,",
                "remains,",
                "undefeated□",
            ],
        );
        assert_pp(
            &n,
            15,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe,",
                "is, me, the,",
                "turbofish,",
                "remains,",
                "undefeated□",
            ],
        );
        assert_pp(
            &n,
            0,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh,",
                "woe,",
                "is,",
                "me,",
                "the,",
                "turbofish,",
                "remains,",
                "undefeated□",
            ],
        );
    }

    #[test]
    #[ignore]
    fn let_list() {
        fn list(elements: Vec<Notation>) -> Notation {
            let empty = lit("[]");
            let lone = |elem| lit("[") + elem + lit("]");
            let join =
                |elem: Notation, accum: Notation| elem + lit(",") + (lit(" ") | nl()) + accum;
            let surround = |accum: Notation| {
                let single = lit("[") + accum.clone() + lit("]");
                let multi = lit("[") + (4 >> accum) ^ lit("]");
                single | multi
            };
            Notation::repeat(elements, empty, lone, join, surround)
        }

        fn make_let(var: &str, defn: Notation) -> Notation {
            lit("let ") + lit(var) + lit(" =") + (lit(" ") | nl()) + defn + lit(";")
        }

        // TODO: Add a way to get this to not share lines
        fn phi() -> Notation {
            lit("1 + sqrt(5)") ^ lit("-----------") ^ lit("     2")
        }

        let n = make_let(
            "best_numbers",
            list(vec![
                lit("1025"),
                lit("-58"),
                lit("33297"),
                phi(),
                lit("1.618281828"),
                lit("23"),
            ]),
        );

        assert_pp(&n, 80, &[""]);
    }

    #[test]
    fn iter_with_closure() {
        fn method(obj: Notation, method: &str, arg: Notation) -> Notation {
            let single = lit(method) + lit("(") + arg.clone() + lit(")");
            // foobaxxle.bar(arg)
            //
            // -- Eliminating this case:
            // foobaxxle.bar(
            //     arg
            // )
            //
            // foobaxxle
            //     .bar(arg)
            //
            // foobaxxle
            //     .bar(
            //         arg
            //      )

            let single = lit(".") + lit(method) + lit("(") + flat(arg.clone()) + lit(")");
            let two_lines = lit(".") + lit(method) + lit("(") + flat(arg.clone()) + lit(")");
            let multi = lit(".") + lit(method) + lit("(") + (4 >> arg) ^ lit(")");
            obj + (single | (4 >> (two_lines | multi)))
        }

        fn closure(var: &str, body: Notation) -> Notation {
            let single = lit("|") + lit(var) + lit("| { ") + body.clone() + lit(" }");
            let multi = lit("|") + lit(var) + lit("| {") + (4 >> body) ^ lit("}");
            single | multi
        }

        fn times(arg1: Notation, arg2: Notation) -> Notation {
            arg1 + lit(" * ") + arg2
        }

        let n = lit("some_vec");
        let n = method(n, "iter", lit(""));
        let n = method(n, "map", closure("elem", times(lit("elem"), lit("elem"))));
        let n = method(n, "collect", lit(""));

        assert_pp(
            &n,
            80,
            //0    5   10   15   20   25   30   35   40   45   50   55   60
            &["some_vec.iter().map(|elem| { elem * elem }).collect()"],
        );
        assert_pp(
            &n,
            50,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                // force rustfmt
                "some_vec.iter().map(|elem| { elem * elem })",
                "    .collect()",
            ],
        );
        assert_pp(
            &n,
            40,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                // force rustfmt
                "some_vec.iter()",
                "    .map(|elem| { elem * elem })",
                "    .collect()",
            ],
        );
        assert_pp(
            &n,
            30,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                // force rustfmt
                "some_vec.iter()",
                "    .map(",
                "        |elem| { elem * elem }",
                "    ).collect()",
            ],
        );

        let n = lit("some_vec");
        let n = method(n, "map", closure("elem", times(lit("elem"), lit("elem"))));

        assert_pp(
            &n,
            31,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                // force rustfmt
                "some_vec",
                "    .map(",
                "        |elem| { elem * elem }",
                "    )",
            ],
        );

        let n = lit("some_vec");
        let n = method(n, "call_the_map_method", closure("elem", lit("elem")));
        let n = method(n, "call_the_map_method", closure("elem", lit("elem")));

        assert_pp(
            &n,
            41,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                // force rustfmt
                "some_vec",
                "    .call_the_map_method(|elem| { elem })",
                "    .call_the_map_method(|elem| { elem })",
            ],
        );
        // Likewise
        assert_pp(
            &n,
            35,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                // force rustfmt
                "some_vec",
                "    .call_the_map_method(",
                "        |elem| { elem }",
                "    )",
                "    .call_the_map_method(",
                "        |elem| { elem }",
                "    )",
            ],
        );
    }

    #[test]
    fn ruby() {
        // (1..5).each do |i| puts i end
        //
        // (1..5).each do |i|
        //     puts i
        // end
        //
        // -- ELIMINATE THIS?
        // (1..5).each
        //     do |i|
        //         puts i
        //     end
        //
        // (1..5)
        //     .each do |i|
        //         puts i
        //     end
        //
        // object.method argument
        // object.method
        //     argument
        // object
        //     .method argument
        // object
        //     .method
        //         argument

        fn method(obj: Notation, method: &str, arg: Notation) -> Notation {
            let single = lit(".") + lit(method) + lit(" ") + flat(arg.clone());
            let multi = 4 >> (lit(".") + lit(method) + lit

            let single = lit(".") + lit(method) + lit("(") + flat(arg.clone()) + lit(")");
            let two_lines = lit(".") + lit(method) + lit("(") + (4 >> arg.clone()) ^ lit(")");
            let multi = lit(".") + lit(method) + lit("(") + (4 >> arg) ^ lit(")");
            obj + (single | (4 >> (two_lines | multi)))
        }

        fn closure(var: &str, body: Notation) -> Notation {
            let single = lit("|") + lit(var) + lit("| { ") + body.clone() + lit(" }");
            let multi = lit("|") + lit(var) + lit("| {") + (4 >> body) ^ lit("}");
            single | multi
        }

        fn times(arg1: Notation, arg2: Notation) -> Notation {
            arg1 + lit(" * ") + arg2
        }

        let n = lit("some_vec");
        let n = method(n, "iter", lit(""));
        let n = method(n, "map", closure("elem", times(lit("elem"), lit("elem"))));
        let n = method(n, "collect", lit(""));
    }
}
