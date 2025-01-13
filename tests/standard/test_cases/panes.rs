use crate::standard::pretty_testing::SimpleDoc;
use partial_pretty_printer::{
    doc_examples::{
        json::{json_array, json_number, json_string, Json},
        BasicStyle,
    },
    pane::{
        display_pane, DocLabel, OverflowBehavior, PaneNotation, PaneSize, PlainText,
        PrintingOptions,
    },
    FocusTarget, Pos, PrettyDoc, Size, Style, Width,
};
use std::fmt::Debug;

type NoStyle = ();

type SimpleLabel = char;

fn clip(width: Width) -> OverflowBehavior<BasicStyle> {
    OverflowBehavior::Clip("", BasicStyle::default(), width)
}

#[track_caller]
fn pane_test_7x7<'d, S: Style + Default, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    notation: PaneNotation<SimpleLabel, S>,
    get_content: impl Fn(char, Width) -> Option<(D, PrintingOptions<S>)>,
    expected: &str,
) {
    pane_test(
        Size {
            width: 7,
            height: 7,
        },
        notation,
        get_content,
        expected,
    )
}

#[track_caller]
fn pane_test<'d, S: Style + Default, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    size: Size,
    notation: PaneNotation<SimpleLabel, S>,
    get_content: impl Fn(char, Width) -> Option<(D, PrintingOptions<S>)>,
    expected: &str,
) {
    pane_test_impl(size, notation, get_content, expected, None)
}

#[track_caller]
fn pane_test_with_focus<'d, S: Style + Default, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    size: Size,
    notation: PaneNotation<SimpleLabel, S>,
    get_content: impl Fn(char, Width) -> Option<(D, PrintingOptions<S>)>,
    expected: &str,
    expected_pos: &[Pos],
) {
    pane_test_impl(size, notation, get_content, expected, Some(expected_pos))
}

#[track_caller]
fn pane_test_impl<'d, S: Style + Default, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    size: Size,
    notation: PaneNotation<SimpleLabel, S>,
    get_content: impl Fn(char, Width) -> Option<(D, PrintingOptions<S>)>,
    expected: &str,
    expected_pos: Option<&[Pos]>,
) {
    let mut screen = PlainText::new(size.width, size.height);
    display_pane(&mut screen, &notation, &S::default(), &get_content).unwrap();
    let actual = screen.to_string();
    if actual != expected {
        eprintln!("ACTUAL:\n{}", actual);
        eprintln!("EXPECTED:\n{}", expected);
        eprintln!("END");
    }
    assert_eq!(actual, expected);
    if let Some(expected_pos) = expected_pos {
        assert_eq!(screen.focus_points(), expected_pos);
    }
}

fn fill<L: DocLabel, S: Style + Default>(ch: char) -> PaneNotation<L, S> {
    PaneNotation::Fill { ch }
}

#[test]
fn test_fill_pane() {
    pane_test_7x7::<NoStyle, &SimpleDoc>(
        fill('a'),
        |_, _| None,
        "aaaaaaa\n\
         aaaaaaa\n\
         aaaaaaa\n\
         aaaaaaa\n\
         aaaaaaa\n\
         aaaaaaa\n\
         aaaaaaa\n",
    );
}

#[test]
fn test_fill_pane_with_full_width() {
    pane_test::<NoStyle, &SimpleDoc>(
        Size {
            width: 7,
            height: 3,
        },
        fill('信'),
        |_, _| None,
        "信信信 \n\
         信信信 \n\
         信信信 \n",
    );

    pane_test::<NoStyle, &SimpleDoc>(
        Size {
            width: 6,
            height: 3,
        },
        fill('信'),
        |_, _| None,
        "信信信\n\
         信信信\n\
         信信信\n",
    );

    pane_test::<NoStyle, &SimpleDoc>(
        Size {
            width: 1,
            height: 3,
        },
        fill('信'),
        |_, _| None,
        " \n \n \n",
    );

    pane_test::<NoStyle, &SimpleDoc>(
        Size {
            width: 0,
            height: 3,
        },
        fill('信'),
        |_, _| None,
        "",
    );
}

#[test]
fn test_horz_split_pane() {
    use PaneSize::{Fixed, Proportional};

    pane_test_7x7::<NoStyle, &SimpleDoc>(
        PaneNotation::Horz(vec![
            (Proportional(2), fill('a')),
            (Proportional(3), fill('b')),
            (Fixed(1), fill('X')),
            (Proportional(2), fill('d')),
            (Proportional(3), fill('e')),
            (Fixed(1), fill('Y')),
        ]),
        |_, _| None,
        "abbXdeY\n\
         abbXdeY\n\
         abbXdeY\n\
         abbXdeY\n\
         abbXdeY\n\
         abbXdeY\n\
         abbXdeY\n",
    );
}

#[test]
fn test_vert_split_pane() {
    use PaneSize::{Fixed, Proportional};

    pane_test_7x7::<NoStyle, &SimpleDoc>(
        PaneNotation::Vert(vec![
            (Proportional(2), fill('a')),
            (Proportional(3), fill('b')),
            (Fixed(1), fill('X')),
            (Proportional(2), fill('d')),
            (Proportional(3), fill('e')),
            (Fixed(1), fill('Y')),
        ]),
        |_, _| None,
        "aaaaaaa\n\
         bbbbbbb\n\
         bbbbbbb\n\
         XXXXXXX\n\
         ddddddd\n\
         eeeeeee\n\
         YYYYYYY\n",
    );
}

#[test]
fn test_mixed_split_pane() {
    use PaneSize::{Fixed, Proportional};

    pane_test_7x7::<NoStyle, &SimpleDoc>(
        PaneNotation::Horz(vec![
            (Proportional(2), fill('|')),
            (
                Proportional(11),
                PaneNotation::Vert(vec![
                    (Proportional(2), fill('t')),
                    (
                        Proportional(5),
                        PaneNotation::Vert(vec![
                            (Fixed(1), fill('=')),
                            (
                                Fixed(3),
                                PaneNotation::Horz(vec![
                                    (Fixed(2), fill('*')),
                                    (Fixed(2), fill('@')),
                                ]),
                            ),
                            (Fixed(1), fill('=')),
                        ]),
                    ),
                ]),
            ),
            (
                Fixed(1),
                PaneNotation::Vert(vec![(Fixed(2), fill('|')), (Fixed(3), fill('!'))]),
            ),
        ]),
        |_, _| None,
        "|ttttt|\n\
         |ttttt|\n\
         |=====!\n\
         |**@@ !\n\
         |**@@ !\n\
         |**@@\n\
         |=====\n",
    );
}

#[test]
fn test_doc_pane() {
    let doc = json_array(vec![json_string("Hello"), json_string("world")]);
    let get_content = |_, width| {
        Some((
            &doc,
            PrintingOptions {
                focus_path: Vec::new(),
                focus_height: 0.0,
                focus_target: FocusTarget::Start,
                set_focus: false,
                printing_width: width,
                overflow_behavior: clip(width),
            },
        ))
    };

    pane_test_7x7(
        PaneNotation::Doc { label: 'a' },
        get_content,
        &[
            "[      ",  // force rustfmt
            "    \"He", // force rustfmt
            "    \"wo", // force rustfmt
            "]      ",  // force rustfmt
            "       ",  // force rustfmt
            "       ",  // force rustfmt
            "       ",  // force rustfmt
            "",
        ]
        .join("\n"),
    );
}

#[test]
fn test_doc_pane_full_width_cutoff() {
    let doc = json_string("一二三");
    let get_content = |_, width| {
        Some((
            &doc,
            PrintingOptions {
                focus_path: Vec::new(),
                focus_height: 0.0,
                focus_target: FocusTarget::Start,
                set_focus: false,
                printing_width: width,
                overflow_behavior: clip(width),
            },
        ))
    };
    let note = PaneNotation::Doc { label: 'a' };

    pane_test(
        Size {
            width: 8,
            height: 3,
        },
        note.clone(),
        get_content,
        "\"一二三\"\n        \n        \n",
    );
    pane_test(
        Size {
            width: 7,
            height: 3,
        },
        note.clone(),
        get_content,
        "\"一二三\n       \n       \n",
    );
    pane_test(
        Size {
            width: 6,
            height: 3,
        },
        note.clone(),
        get_content,
        "\"一二 \n      \n      \n",
    );
    pane_test(
        Size {
            width: 5,
            height: 3,
        },
        note.clone(),
        get_content,
        "\"一二\n     \n     \n",
    );
}

#[test]
fn test_pane_cursor_heights() {
    #[track_caller]
    fn test_at_height(focus_height: f32, expected: &str) {
        let doc = json_string("Hi");
        let get_content = |_, width| {
            Some((
                &doc,
                PrintingOptions {
                    focus_path: Vec::new(),
                    focus_height,
                    focus_target: FocusTarget::Start,
                    set_focus: false,
                    printing_width: width,
                    overflow_behavior: clip(width),
                },
            ))
        };

        pane_test(
            Size {
                width: 4,
                height: 7,
            },
            PaneNotation::Doc { label: 'a' },
            get_content,
            expected,
        );
    }

    test_at_height(0.0, "\"Hi\"\n    \n    \n    \n    \n    \n    \n");
    test_at_height(0.17, "    \n\"Hi\"\n    \n    \n    \n    \n    \n");
    test_at_height(0.33, "    \n    \n\"Hi\"\n    \n    \n    \n    \n");
    test_at_height(0.5, "    \n    \n    \n\"Hi\"\n    \n    \n    \n");
    test_at_height(0.67, "    \n    \n    \n    \n\"Hi\"\n    \n    \n");
    test_at_height(0.83, "    \n    \n    \n    \n    \n\"Hi\"\n    \n");
    test_at_height(1.0, "    \n    \n    \n    \n    \n    \n\"Hi\"\n");
}

#[test]
fn test_pane_widths() {
    #[track_caller]
    fn test_with_width(width_strategy: impl Fn(Width) -> Width, expected: &str) {
        let doc = json_array(vec![json_string("Hello"), json_string("world")]);
        let get_content = |_, width| {
            Some((
                &doc,
                PrintingOptions {
                    focus_path: Vec::new(),
                    focus_height: 0.0,
                    focus_target: FocusTarget::Start,
                    set_focus: false,
                    printing_width: width_strategy(width),
                    overflow_behavior: clip(width),
                },
            ))
        };
        pane_test_7x7(PaneNotation::Doc { label: 'a' }, get_content, expected);
    }

    let wide = [
        "[\"Hello", // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "",
    ]
    .join("\n");
    let narrow = [
        "[      ",  // force rustfmt
        "    \"He", // force rustfmt
        "    \"wo", // force rustfmt
        "]      ",  // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "",
    ]
    .join("\n");
    let clipped = [
        "[      ",  // force rustfmt
        "    \"  ", // force rustfmt
        "    \"  ", // force rustfmt
        "]      ",  // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "       ",  // force rustfmt
        "",
    ]
    .join("\n");

    test_with_width(|w| w, &narrow);
    test_with_width(|_| 80, &wide);
    test_with_width(|_| 10, &narrow);
    test_with_width(|w| w.min(80), &narrow);
    test_with_width(|w| w.min(5), &clipped);
}

fn make_array(start: usize, end: usize) -> Json {
    json_array((start..end).map(|x| json_number(x as f64)).collect())
}

#[test]
fn test_seek() {
    fn make_get_content<'d>(
        doc: &'d Json,
        focus_path: &[usize],
        focus_target: FocusTarget,
    ) -> impl Fn(char, Width) -> Option<(&'d Json, PrintingOptions<BasicStyle>)> + 'd {
        let focus_path = focus_path.to_owned();
        move |_, width| {
            Some((
                doc,
                PrintingOptions {
                    focus_path: focus_path.clone(),
                    focus_height: 0.5,
                    focus_target,
                    set_focus: false,
                    printing_width: width,
                    overflow_behavior: clip(width),
                },
            ))
        }
    }
    let doc10 = make_array(0, 8);
    let notation = PaneNotation::Doc { label: 'a' };

    pane_test_7x7(
        notation.clone(),
        make_get_content(&doc10, &[], FocusTarget::Start),
        &[
            "       ", // force rustfmt
            "       ", // force rustfmt
            "       ", // force rustfmt
            "[      ", // force rustfmt
            "    0, ", // force rustfmt
            "    1, ", // force rustfmt
            "    2, ", // force rustfmt
            "",
        ]
        .join("\n"),
    );
    pane_test_7x7(
        notation.clone(),
        make_get_content(&doc10, &[], FocusTarget::End),
        &[
            "    5, ", // force rustfmt
            "    6, ", // force rustfmt
            "    7  ", // force rustfmt
            "]      ", // force rustfmt
            "       ", // force rustfmt
            "       ", // force rustfmt
            "       ", // force rustfmt
            "",
        ]
        .join("\n"),
    );
    pane_test_7x7(
        notation.clone(),
        make_get_content(&doc10, &[1], FocusTarget::Start),
        &[
            "       ",   // force rustfmt
            "[      ",   // force rustfmt
            "    0, ",   // force rustfmt
            "    1, ",   // force rustfmt
            "    2, ",   // force rustfmt
            "    3, ",   // force rustfmt
            "    4, \n", // force rustfmt
        ]
        .join("\n"),
    );
    pane_test_7x7(
        notation.clone(),
        make_get_content(&doc10, &[1], FocusTarget::End),
        &[
            "       ",   // force rustfmt
            "[      ",   // force rustfmt
            "    0, ",   // force rustfmt
            "    1, ",   // force rustfmt
            "    2, ",   // force rustfmt
            "    3, ",   // force rustfmt
            "    4, \n", // force rustfmt
        ]
        .join("\n"),
    );
}

#[test]
fn test_dynamic() {
    let doc8 = make_array(0, 6);
    let doc5 = make_array(6, 9);
    let doc_num = json_number(42.0);
    let doc_unicode = json_string("一1");
    let get_content = |label, width| {
        Some((
            match label {
                '8' => &doc8,
                '5' => &doc5,
                'n' => &doc_num,
                'u' => &doc_unicode,
                _ => panic!("Bad label {}", label),
            },
            PrintingOptions {
                focus_path: Vec::new(),
                focus_height: 0.0,
                focus_target: FocusTarget::Start,
                set_focus: false,
                printing_width: width,
                overflow_behavior: clip(width),
            },
        ))
    };

    pane_test_7x7(
        PaneNotation::Vert(vec![
            (PaneSize::Proportional(1), PaneNotation::Doc { label: '8' }),
            (PaneSize::Dynamic, PaneNotation::Doc { label: '5' }),
        ]),
        get_content,
        &[
            "[      ", // force rustfmt
            "    0, ", // force rustfmt
            "[      ", // force rustfmt
            "    6, ", // force rustfmt
            "    7, ", // force rustfmt
            "    8  ", // force rustfmt
            "]      \n",
        ]
        .join("\n"),
    );

    pane_test_7x7(
        PaneNotation::Vert(vec![
            (PaneSize::Proportional(1), PaneNotation::Doc { label: '5' }),
            (PaneSize::Dynamic, PaneNotation::Doc { label: '8' }),
        ]),
        get_content,
        &[
            "[      ", // force rustfmt
            "    0, ", // force rustfmt
            "    1, ", // force rustfmt
            "    2, ", // force rustfmt
            "    3, ", // force rustfmt
            "    4, ", // force rustfmt
            "    5  \n",
        ]
        .join("\n"),
    );

    pane_test_7x7(
        PaneNotation::Horz(vec![
            (PaneSize::Proportional(1), PaneNotation::Doc { label: '8' }),
            (PaneSize::Dynamic, PaneNotation::Doc { label: 'n' }),
        ]),
        get_content,
        &[
            "[    42", // force rustfmt
            "    0  ", // force rustfmt
            "    1  ", // force rustfmt
            "    2  ", // force rustfmt
            "    3  ", // force rustfmt
            "    4  ", // force rustfmt
            "    5  \n",
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 11,
            height: 7,
        },
        PaneNotation::Horz(vec![
            (PaneSize::Proportional(1), PaneNotation::Doc { label: '8' }),
            (PaneSize::Dynamic, PaneNotation::Doc { label: 'u' }),
        ]),
        get_content,
        &[
            "[     \"一1\"", // force rustfmt
            "    0,     ",   // force rustfmt
            "    1,     ",   // force rustfmt
            "    2,     ",   // force rustfmt
            "    3,     ",   // force rustfmt
            "    4,     ",   // force rustfmt
            "    5      \n",
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 10,
            height: 7,
        },
        PaneNotation::Horz(vec![
            (PaneSize::Proportional(1), PaneNotation::Doc { label: '8' }),
            (PaneSize::Dynamic, PaneNotation::Doc { label: 'u' }),
        ]),
        get_content,
        &[
            "[    \"一1\"", // force rustfmt
            "    0     ",   // force rustfmt
            "    1     ",   // force rustfmt
            "    2     ",   // force rustfmt
            "    3     ",   // force rustfmt
            "    4     ",   // force rustfmt
            "    5     \n",
        ]
        .join("\n"),
    );
}

#[test]
fn test_focus_point() {
    let doc = json_array(vec![
        json_string("Hello"),
        json_string("darkness,"),
        json_array(vec![json_string("my"), json_string("old")]),
        json_string("friend"),
    ]);
    let get_content = |_, width| {
        Some((
            &doc,
            PrintingOptions {
                focus_path: vec![2, 0],
                focus_height: 0.5,
                focus_target: FocusTarget::End,
                set_focus: true,
                printing_width: width,
                overflow_behavior: clip(width),
            },
        ))
    };

    pane_test_with_focus(
        Size {
            width: 20,
            height: 12,
        },
        PaneNotation::Vert(vec![
            (PaneSize::Fixed(3), fill('*')),
            (
                PaneSize::Proportional(1),
                PaneNotation::Horz(vec![
                    (PaneSize::Fixed(2), fill('@')),
                    (PaneSize::Proportional(1), PaneNotation::Doc { label: 'a' }),
                ]),
            ),
        ]),
        get_content,
        &[
            // force rustfmt
            r#"********************"#,
            r#"********************"#,
            r#"********************"#,
            r#"@@                  "#,
            r#"@@[                 "#,
            r#"@@    "Hello",      "#,
            r#"@@    "darkness,",  "#,
            r#"@@    ["my", "old"],"#,
            r#"@@    "friend"      "#,
            r#"@@]                 "#,
            r#"@@                  "#,
            r#"@@                  "#,
            r#""#,
        ]
        .join("\n"),
        &[Pos { col: 11, row: 7 }],
    );
}

#[test]
fn test_line_wrapping() {
    let doc = json_array(vec![
        json_number(1111111.),
        json_number(22222222222.),
        json_array(vec![json_number(3333.), json_number(4.)]),
        json_number(55555555.),
        json_number(6666666666666666.),
    ]);
    let get_content = |_, width| {
        Some((
            &doc,
            PrintingOptions {
                focus_path: vec![2, 0],
                focus_height: 0.5,
                focus_target: FocusTarget::Start,
                set_focus: false,
                printing_width: width,
                overflow_behavior: OverflowBehavior::Wrap("/", BasicStyle::new(), width),
            },
        ))
    };
    pane_test(
        Size {
            width: 14,
            height: 20,
        },
        PaneNotation::Vert(vec![
            (PaneSize::Fixed(3), fill('*')),
            (
                PaneSize::Proportional(1),
                PaneNotation::Horz(vec![
                    (PaneSize::Fixed(2), fill('@')),
                    (PaneSize::Proportional(1), PaneNotation::Doc { label: 'a' }),
                    (PaneSize::Fixed(2), fill('@')),
                ]),
            ),
        ]),
        get_content,
        &[
            // force rustfmt
            "**************",
            "**************",
            "**************",
            "@@          @@",
            "@@          @@",
            "@@[         @@",
            "@@    111111@@",
            "@@/1,       @@",
            "@@    222222@@",
            "@@/22222,   @@",
            "@@    [     @@",
            "@@        33@@",
            "@@/33,      @@",
            "@@        4 @@",
            "@@    ],    @@",
            "@@    555555@@",
            "@@/55,      @@",
            "@@    666666@@",
            "@@/666666666@@",
            "@@/6        @@",
            "",
        ]
        .join("\n"),
    );
}

#[test]
fn test_line_wrapping_with_wide_chars() {
    let doc = json_string("一二三");
    let get_content = |_, width| {
        Some((
            &doc,
            PrintingOptions {
                focus_path: Vec::new(),
                focus_height: 0.0,
                focus_target: FocusTarget::Start,
                set_focus: false,
                printing_width: width,
                overflow_behavior: OverflowBehavior::Wrap("ooo", BasicStyle::new(), width),
            },
        ))
    };
    let note = PaneNotation::Doc { label: 'a' };

    pane_test(
        Size {
            width: 8,
            height: 3,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#""一二三""#,
            r#"        "#,
            r#"        "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 7,
            height: 3,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#""一二三"#,
            r#"ooo"   "#,
            r#"       "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 6,
            height: 3,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#""一二 "#,
            r#"ooo三""#,
            r#"      "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 5,
            height: 3,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#""一二"#,
            r#"ooo三"#,
            r#"ooo" "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 4,
            height: 3,
        },
        note.clone(),
        get_content,
        "\"一 \n    \n    \n",
    );

    pane_test(
        Size {
            width: 3,
            height: 3,
        },
        note.clone(),
        get_content,
        "\"一\n   \n   \n",
    );

    pane_test(
        Size {
            width: 2,
            height: 3,
        },
        note.clone(),
        get_content,
        "\" \n  \n  \n",
    );

    pane_test(
        Size {
            width: 1,
            height: 3,
        },
        note.clone(),
        get_content,
        "\"\n \n \n",
    );

    pane_test(
        Size {
            width: 0,
            height: 3,
        },
        note.clone(),
        get_content,
        "",
    );
}

#[test]
fn test_line_clipping_with_wide_chars() {
    let doc = json_string("一二三");
    let get_content = |_, width| {
        Some((
            &doc,
            PrintingOptions {
                focus_path: Vec::new(),
                focus_height: 0.0,
                focus_target: FocusTarget::Start,
                set_focus: false,
                printing_width: width,
                overflow_behavior: OverflowBehavior::Clip("…", BasicStyle::new(), width),
            },
        ))
    };
    let note = PaneNotation::Doc { label: 'a' };

    pane_test(
        Size {
            width: 8,
            height: 2,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#""一二三""#,
            r#"        "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 7,
            height: 2,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#""一二 …"#,
            r#"       "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 6,
            height: 2,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#""一二…"#,
            r#"      "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 3,
            height: 2,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#"" …"#, //
            r#"   "#, //
            r#""#,    //
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 2,
            height: 2,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#"" "#, //
            r#"  "#, //
            r#""#,   //
        ]
        .join("\n"),
    );

    pane_test(
        Size {
            width: 0,
            height: 2,
        },
        note.clone(),
        get_content,
        "",
    );
}

#[test]
fn test_overflow_behavior() {
    // Printing width = 10
    // Doc width      = 9
    // Pane width     = 8
    // Wrapping width = 5

    let doc_9 = make_array(0, 3);
    let note = PaneNotation::Doc { label: 'a' };

    let get_content = |_, _| {
        Some((
            &doc_9,
            PrintingOptions {
                focus_path: Vec::new(),
                focus_height: 0.0,
                focus_target: FocusTarget::Start,
                set_focus: false,
                printing_width: 10,
                overflow_behavior: OverflowBehavior::Wrap("> ", BasicStyle::new(), 5),
            },
        ))
    };
    pane_test(
        Size {
            width: 8,
            height: 3,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#"[0, 1   "#,
            r#"> , 2   "#,
            r#"> ]     "#,
            r#""#,
        ]
        .join("\n"),
    );

    let get_content = |_, _| {
        Some((
            &doc_9,
            PrintingOptions {
                focus_path: Vec::new(),
                focus_height: 0.0,
                focus_target: FocusTarget::Start,
                set_focus: false,
                printing_width: 10,
                overflow_behavior: OverflowBehavior::Clip("!", BasicStyle::new(), 5),
            },
        ))
    };
    pane_test(
        Size {
            width: 8,
            height: 3,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#"[0, !   "#,
            r#"        "#,
            r#"        "#,
            r#""#,
        ]
        .join("\n"),
    );
}

#[test]
fn test_focus_oob() {
    let doc_11 = make_array(0, 4);
    let note = PaneNotation::Doc { label: 'a' };
    let get_content = |_, _| {
        Some((
            &doc_11,
            PrintingOptions {
                focus_path: vec![2],
                focus_height: 0.0,
                focus_target: FocusTarget::Start,
                set_focus: true,
                printing_width: 15,
                overflow_behavior: clip(5),
            },
        ))
    };
    pane_test_with_focus(
        Size {
            width: 8,
            height: 3,
        },
        note.clone(),
        get_content,
        &[
            // force rustfmt
            r#"[0, 1   "#,
            r#"        "#,
            r#"        "#,
            r#""#,
        ]
        .join("\n"),
        &[],
    );
}

#[test]
fn test_segment_split_at() {
    use partial_pretty_printer::{Segment, Width};

    fn split(
        seg: &Segment<'static, &'static SimpleDoc>,
        width: Width,
    ) -> ((&'static str, Width), (&'static str, Width)) {
        let (seg_1, seg_2) = seg.clone().split_at(width);
        ((seg_1.str, seg_1.width), (seg_2.str, seg_2.width))
    }

    let seg = Segment {
        str: "x字",
        width: 3,
        style: (),
    };

    assert_eq!(split(&seg, 0), (("", 0), ("x字", 3)));
    assert_eq!(split(&seg, 1), (("x", 1), ("字", 2)));
    assert_eq!(split(&seg, 2), (("x", 1), ("字", 2)));
    assert_eq!(split(&seg, 3), (("x字", 3), ("", 0)));
}

#[test]
fn test_line_split_at() {
    use partial_pretty_printer::{Line, Segment, Width};

    type SegParts = (&'static str, Width);

    fn split(
        line: Line<'static, &'static SimpleDoc>,
        width: Width,
    ) -> (Vec<SegParts>, Vec<SegParts>) {
        let (line_1, line_2) = line.split_at(width);
        (
            line_1
                .segments
                .into_iter()
                .map(|seg| (seg.str, seg.width))
                .collect(),
            line_2
                .segments
                .into_iter()
                .map(|seg| (seg.str, seg.width))
                .collect(),
        )
    }

    fn make_line() -> Line<'static, &'static SimpleDoc> {
        Line {
            segments: vec![
                Segment {
                    str: "xx",
                    width: 2,
                    style: (),
                },
                Segment {
                    str: "字",
                    width: 2,
                    style: (),
                },
            ],
        }
    }

    assert_eq!(split(make_line(), 0), (vec![], vec![("xx", 2), ("字", 2)]));
    assert_eq!(
        split(make_line(), 1),
        (vec![("x", 1)], vec![("x", 1), ("字", 2)])
    );
    assert_eq!(split(make_line(), 2), (vec![("xx", 2)], vec![("字", 2)]));
    assert_eq!(split(make_line(), 3), (vec![("xx", 2)], vec![("字", 2)]));
    assert_eq!(split(make_line(), 4), (vec![("xx", 2), ("字", 2)], vec![]));

    fn make_line_2() -> Line<'static, &'static SimpleDoc> {
        Line {
            segments: vec![
                Segment {
                    str: "字",
                    width: 2,
                    style: (),
                },
                Segment {
                    str: "xx",
                    width: 2,
                    style: (),
                },
            ],
        }
    }

    assert_eq!(
        split(make_line_2(), 0),
        (vec![], vec![("字", 2), ("xx", 2)])
    );
    assert_eq!(
        split(make_line_2(), 1),
        (vec![], vec![("字", 2), ("xx", 2)])
    );
    assert_eq!(split(make_line_2(), 2), (vec![("字", 2)], vec![("xx", 2)]));
    assert_eq!(
        split(make_line_2(), 3),
        (vec![("字", 2), ("x", 1)], vec![("x", 1)])
    );
    assert_eq!(
        split(make_line_2(), 4),
        (vec![("字", 2), ("xx", 2)], vec![])
    );
}
