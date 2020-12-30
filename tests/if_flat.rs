mod tests {
    use partial_pretty_printer::{
        print_downward_for_testing, print_upward_for_testing, Doc, Notation,
    };

    // TODO: Put these in a shared common file. Break this file into several.
    // TODO: Use real tree-shaped Docs for testing

    // TESTS:
    // x Json
    // x text flow
    // n imports with multi-line import not sharing lines
    // - let w/ list w/ an element that doesn't want to share a line
    // x iter w/ map & closure

    #[derive(Debug, Clone)]
    struct Tree {
        notation: Notation,
        children: Vec<Tree>,
    }

    impl Tree {
        fn new_branch(notation: Notation, children: Vec<Tree>) -> Tree {
            Tree { notation, children }
        }

        fn new_leaf(notation: Notation) -> Tree {
            Tree {
                notation,
                children: vec![],
            }
        }
    }

    impl Doc for Tree {
        fn notation(&self) -> &Notation {
            &self.notation
        }

        fn child(&self, i: usize) -> &Tree {
            &self.children[i]
        }
    }

    fn nl() -> Notation {
        Notation::Newline
    }

    fn child(i: usize) -> Notation {
        Notation::Child(i)
    }

    fn lit(s: &str) -> Notation {
        Notation::Literal(s.to_string())
    }

    fn flat(n: Notation) -> Notation {
        Notation::Flat(Box::new(n))
    }

    fn compare_lines(message: &str, actual: &[String], expected: &[&str]) {
        if actual != expected {
            eprintln!(
                "{}\nEXPECTED:\n{}\nACTUAL:\n{}\n=========",
                message,
                expected.join("\n"),
                actual.join("\n"),
            );
            assert_eq!(actual, expected);
        }
    }

    #[track_caller]
    fn assert_pp<D: Doc>(doc: &D, width: usize, expected_lines: &[&str]) {
        let downward_lines = print_downward_for_testing(doc, width);
        compare_lines("IN DOWNWARD PRINTING", &downward_lines, expected_lines);
        let upward_lines = print_upward_for_testing(doc, width);
        compare_lines("IN UPWARD PRINTING", &upward_lines, expected_lines);
    }

    #[test]
    fn basics() {
        // Empty
        let notation = Notation::Empty;
        assert_pp(&Tree::new_leaf(notation), 80, &[""]);
        // Literal
        let notation = lit("Hello world!");
        assert_pp(&Tree::new_leaf(notation), 80, &["Hello world!"]);
        // Concat
        let notation = lit("Hello") + lit(" world!");
        assert_pp(&Tree::new_leaf(notation), 80, &["Hello world!"]);
        // Newline
        let notation = lit("Hello") ^ lit("world!");
        assert_pp(&Tree::new_leaf(notation), 80, &["Hello", "world!"]);
        // Indent
        let notation = lit("Hello") + (2 >> lit("world!"));
        assert_pp(&Tree::new_leaf(notation), 80, &["Hello", "  world!"]);
        // Choice
        let notation = lit("Hello world!") | lit("Hello") ^ lit("world!");
        assert_pp(&Tree::new_leaf(notation.clone()), 12, &["Hello world!"]);
        assert_pp(&Tree::new_leaf(notation), 11, &["Hello", "world!"]);
    }

    #[test]
    fn json() {
        fn json_string(s: &str) -> Tree {
            // Using single quote instead of double quote to avoid inconvenient
            // escaping
            Tree::new_leaf(lit("'") + lit(s) + lit("'"))
        }

        fn json_number(n: &str) -> Tree {
            Tree::new_leaf(lit(n))
        }

        // TODO: allow newline?
        fn json_entry(key: &str, value: Tree) -> Tree {
            let notation = lit("'") + lit(key) + lit("': ") + child(0);
            Tree::new_branch(notation, vec![value])
        }

        fn json_list(elements: Vec<Tree>) -> Tree {
            let empty = lit("[]");
            let lone = |elem| lit("[") + elem + lit("]");
            let join =
                |elem: Notation, accum: Notation| elem + lit(",") + (lit(" ") | nl()) + accum;
            let surround = |accum: Notation| {
                let single = lit("[") + flat(accum.clone()) + lit("]");
                let multi = lit("[") + (4 >> accum) ^ lit("]");
                single | multi
            };
            let notation = Notation::repeat(elements.len(), empty, lone, join, surround);
            Tree::new_branch(notation, elements)
        }

        fn json_dict(entries: Vec<Tree>) -> Tree {
            let empty = lit("{}");
            let lone = |elem: Notation| {
                let single = lit("{") + elem.clone() + lit("}");
                let multi = lit("{") + (4 >> elem) ^ lit("}");
                single | multi
            };
            let join = |elem: Notation, accum: Notation| elem + lit(",") + nl() + accum;
            let surround = |accum: Notation| lit("{") + (4 >> accum) ^ lit("}");
            let notation = Notation::repeat(entries.len(), empty, lone, join, surround);
            Tree::new_branch(notation, entries)
        }

        let e1 = json_entry("Name", json_string("Alice"));
        let e2 = json_entry("Age", json_number("42"));
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
        fn word_flow(words: &[&str]) -> Tree {
            let elements = words
                .iter()
                .map(|w| Tree::new_leaf(lit(w)))
                .collect::<Vec<_>>();
            let empty = lit("");
            let lone = |elem| lit("    ") + elem;
            let soft_break = || lit(" ") | nl();
            let join = |elem: Notation, accum: Notation| elem + lit(",") + soft_break() + accum;
            let surround = |accum: Notation| lit("    ") + accum;
            let notation = Notation::repeat(elements.len(), empty, lone, join, surround);
            Tree::new_branch(notation, elements)
        }

        fn mark_paragraph(paragraph: Tree) -> Tree {
            let notation = lit("¶") + child(0) + lit("□");
            Tree::new_branch(notation, vec![paragraph])
        }

        let doc = mark_paragraph(word_flow(&[
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
            &doc,
            80,
            //0    5   10   15   20   25   30   35   40   45   50   55   60
            &["¶    Oh, woe, is, me, the, turbofish, remains, undefeated□"],
        );
        assert_pp(
            &doc,
            46,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe, is, me, the, turbofish, remains,",
                "undefeated□",
            ],
        );
        assert_pp(
            &doc,
            45,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe, is, me, the, turbofish,",
                "remains, undefeated□",
            ],
        );
        assert_pp(
            &doc,
            20,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                "¶    Oh, woe, is,",
                "me, the, turbofish,",
                "remains, undefeated□",
            ],
        );
        assert_pp(
            &doc,
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
            &doc,
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
            &doc,
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
            &doc,
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
        fn list(elements: Vec<Tree>) -> Tree {
            let empty = lit("[]");
            let lone = |elem| lit("[") + elem + lit("]");
            let join =
                |elem: Notation, accum: Notation| elem + lit(",") + (lit(" ") | nl()) + accum;
            let surround = |accum: Notation| {
                let single = lit("[") + accum.clone() + lit("]");
                let multi = lit("[") + (4 >> accum) ^ lit("]");
                single | multi
            };
            let notation = Notation::repeat(elements.len(), empty, lone, join, surround);
            Tree::new_branch(notation, elements)
        }

        fn make_let(var: &str, defn: Tree) -> Tree {
            let notation =
                lit("let ") + child(0) + lit(" =") + (lit(" ") | nl()) + child(1) + lit(";");
            Tree::new_branch(notation, vec![Tree::new_leaf(lit(var)), defn])
        }

        // TODO: Add a way to get this to not share lines
        fn phi() -> Tree {
            Tree::new_leaf(lit("1 + sqrt(5)") ^ lit("-----------") ^ lit("     2"))
        }

        fn num(n: &str) -> Tree {
            Tree::new_leaf(lit(n))
        }

        let doc = make_let(
            "best_numbers",
            list(vec![
                num("1025"),
                num("-58"),
                num("33297"),
                phi(),
                num("1.618281828"),
                num("23"),
            ]),
        );

        assert_pp(&doc, 80, &[""]);
    }

    #[test]
    fn iter_with_closure() {
        fn method(obj: Tree, method: &str, arg: Tree) -> Tree {
            // foobaxxle.bar(arg)
            //
            // -- Disallowing this layout:
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

            let single = lit(".") + lit(method) + lit("(") + flat(child(1).clone()) + lit(")");
            let two_lines = lit(".") + lit(method) + lit("(") + flat(child(1).clone()) + lit(")");
            let multi = lit(".") + lit(method) + lit("(") + (4 >> child(1)) ^ lit(")");
            let notation = child(0) + (single | (4 >> (two_lines | multi)));
            Tree::new_branch(notation, vec![obj, arg])
        }

        fn closure(var: &str, body: Tree) -> Tree {
            let single = lit("|") + lit(var) + lit("| { ") + child(0) + lit(" }");
            let multi = lit("|") + lit(var) + lit("| {") + (4 >> child(0)) ^ lit("}");
            let notation = single | multi;
            Tree::new_branch(notation, vec![body])
        }

        fn times(arg1: Tree, arg2: Tree) -> Tree {
            let notation = child(0) + lit(" * ") + child(1);
            Tree::new_branch(notation, vec![arg1, arg2])
        }

        fn var(var: &str) -> Tree {
            Tree::new_leaf(lit(var))
        }

        let doc = var("some_vec");
        let doc = method(doc, "iter", var(""));
        let doc = method(doc, "map", closure("elem", times(var("elem"), var("elem"))));
        let doc = method(doc, "collect", var(""));

        assert_pp(
            &doc,
            80,
            //0    5   10   15   20   25   30   35   40   45   50   55   60
            &["some_vec.iter().map(|elem| { elem * elem }).collect()"],
        );
        assert_pp(
            &doc,
            50,
            //  0    5   10   15   20   25   30   35   40   45   50   55   60
            &[
                // force rustfmt
                "some_vec.iter().map(|elem| { elem * elem })",
                "    .collect()",
            ],
        );
        assert_pp(
            &doc,
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
            &doc,
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

        let doc = var("some_vec");
        let doc = method(doc, "map", closure("elem", times(var("elem"), var("elem"))));

        assert_pp(
            &doc,
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

        let doc = var("some_vec");
        let doc = method(doc, "call_the_map_method", closure("elem", var("elem")));
        let doc = method(doc, "call_the_map_method", closure("elem", var("elem")));

        assert_pp(
            &doc,
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
            &doc,
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
    fn ruby_loop() {
        // (1..5).each do |i| puts i end
        //
        // (1..5).each do |i|
        //     puts i
        // end
        //
        // -- Dissalow this layout?
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

        fn method(obj: Tree, method: &str, arg: Tree) -> Tree {
            let single = lit(".") + child(1) + lit(" ") + child(2);
            let two_lines = lit(".") + child(1) + lit(" ") + child(2);
            let multi = lit(".") + child(1) + (4 >> child(2));
            let notation = child(0) + (single | (4 >> (two_lines | multi)));
            Tree::new_branch(notation, vec![obj, var(method), arg])
        }

        fn ruby_do(var_name: &str, body: Tree) -> Tree {
            let single = lit("do |") + child(0) + lit("| ") + flat(child(1)) + lit(" end");
            let multi = lit("do |") + child(0) + lit("|") + (4 >> child(1)) ^ lit("end");
            let notation = single | multi;
            Tree::new_branch(notation, vec![var(var_name), body])
        }

        fn var(var_name: &str) -> Tree {
            Tree::new_leaf(lit(var_name))
        }

        let doc = method(var("(1..5)"), "each", ruby_do("i", var("puts i")));
        assert_pp(
            &doc,
            30,
            //  0    5   10   15   20   25   30   35   40
            &[
                // force rustfmt
                "(1..5).each do |i| puts i end",
            ],
        );
        assert_pp(
            &doc,
            20,
            //  0    5   10   15   20   25   30   35   40
            &[
                // force rustfmt
                "(1..5).each do |i|",
                "    puts i",
                "end",
            ],
        );
        assert_pp(
            &doc,
            15,
            //  0    5   10   15   20   25   30   35   40
            &[
                // force rustfmt
                "(1..5)",
                "    .each",
                "        do |i|",
                "            puts i",
                "        end",
            ],
        );
    }
}
