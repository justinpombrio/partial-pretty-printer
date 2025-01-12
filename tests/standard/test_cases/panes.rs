use crate::standard::pretty_testing::SimpleDoc;
use partial_pretty_printer::{
    doc_examples::{
        json::{json_array, json_number, json_string, Json},
        BasicStyle,
    },
    pane::{
        display_pane, DocLabel, LineWrapping, PaneNotation, PaneSize, PlainText, PrintingOptions,
        WidthStrategy,
    },
    FocusTarget, Pos, PrettyDoc, Size, Style,
};
use std::fmt::Debug;
use std::marker::PhantomData;

type NoStyle = ();

#[derive(Debug, Clone)]
struct SimpleLabel<'d, D: PrettyDoc<'d> + Clone + Debug>(
    Option<(D, PrintingOptions<D::Style>)>,
    PhantomData<&'d D>,
);

fn get_content<'d, D: PrettyDoc<'d> + Clone + Debug>(
    label: SimpleLabel<'d, D>,
) -> Option<(D, PrintingOptions<D::Style>)> {
    label.0
}

#[track_caller]
fn pane_test<'d, S: Style + Default, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    notation: PaneNotation<SimpleLabel<'d, D>, S>,
    expected: &str,
) {
    pane_test_with_size(
        Size {
            width: 7,
            height: 7,
        },
        notation,
        expected,
    )
}

#[track_caller]
fn pane_test_with_size<'d, S: Style + Default, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    size: Size,
    notation: PaneNotation<SimpleLabel<'d, D>, S>,
    expected: &str,
) {
    let mut screen = PlainText::new(size.width, size.height);
    display_pane(&mut screen, &notation, &S::default(), &get_content).unwrap();
    let actual = screen.to_string();
    if actual != expected {
        eprintln!("ACTUAL:\n{}", actual);
        eprintln!("EXPECTED:\n{}", expected);
    }
    assert_eq!(actual, expected);
}

#[track_caller]
fn pane_test_with_focus<'d, S: Style + Default, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    size: Size,
    notation: PaneNotation<SimpleLabel<'d, D>, S>,
    expected: &str,
    expected_pos: Pos,
) {
    let mut screen = PlainText::new(size.width, size.height);
    display_pane(&mut screen, &notation, &S::default(), &get_content).unwrap();
    let actual = screen.to_string();
    if actual != expected {
        eprintln!("ACTUAL:\n{}", actual);
        eprintln!("EXPECTED:\n{}", expected);
    }
    assert_eq!(actual, expected);
    assert_eq!(screen.focus_points(), &[expected_pos]);
}

fn fill<L: DocLabel, S: Style + Default>(ch: char) -> PaneNotation<L, S> {
    PaneNotation::Fill { ch }
}

#[test]
fn test_fill_pane() {
    pane_test::<NoStyle, &SimpleDoc>(
        fill('a'),
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
    pane_test_with_size::<NoStyle, &SimpleDoc>(
        Size {
            width: 7,
            height: 3,
        },
        fill('信'),
        "信信信 \n\
         信信信 \n\
         信信信 \n",
    );

    pane_test_with_size::<NoStyle, &SimpleDoc>(
        Size {
            width: 6,
            height: 3,
        },
        fill('信'),
        "信信信\n\
         信信信\n\
         信信信\n",
    );

    pane_test_with_size::<NoStyle, &SimpleDoc>(
        Size {
            width: 1,
            height: 3,
        },
        fill('信'),
        " \n \n \n",
    );

    pane_test_with_size::<NoStyle, &SimpleDoc>(
        Size {
            width: 0,
            height: 3,
        },
        fill('信'),
        "",
    );
}

#[test]
fn test_horz_split_pane() {
    use PaneSize::{Fixed, Proportional};

    pane_test::<NoStyle, &SimpleDoc>(
        PaneNotation::Horz(vec![
            (Proportional(2), fill('a')),
            (Proportional(3), fill('b')),
            (Fixed(1), fill('X')),
            (Proportional(2), fill('d')),
            (Proportional(3), fill('e')),
            (Fixed(1), fill('Y')),
        ]),
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

    pane_test::<NoStyle, &SimpleDoc>(
        PaneNotation::Vert(vec![
            (Proportional(2), fill('a')),
            (Proportional(3), fill('b')),
            (Fixed(1), fill('X')),
            (Proportional(2), fill('d')),
            (Proportional(3), fill('e')),
            (Fixed(1), fill('Y')),
        ]),
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

    pane_test::<NoStyle, &SimpleDoc>(
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
    let options = PrintingOptions {
        focus_path: Vec::new(),
        focus_height: 0.0,
        width_strategy: WidthStrategy::Full,
        focus_target: FocusTarget::Start,
        line_wrapping: LineWrapping::Clip,
        set_focus: false,
    };
    let doc = json_array(vec![json_string("Hello"), json_string("world")]);
    let contents = SimpleLabel(Some((&doc, options)), PhantomData);
    pane_test(
        PaneNotation::Doc { label: contents },
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
    let options = PrintingOptions {
        focus_path: Vec::new(),
        focus_height: 0.0,
        width_strategy: WidthStrategy::Full,
        focus_target: FocusTarget::Start,
        line_wrapping: LineWrapping::Clip,
        set_focus: false,
    };
    let doc = json_string("一二三");
    let contents = SimpleLabel(Some((&doc, options)), PhantomData);
    let note = PaneNotation::Doc { label: contents };

    pane_test_with_size(
        Size {
            width: 8,
            height: 3,
        },
        note.clone(),
        "\"一二三\"\n        \n        \n",
    );
    pane_test_with_size(
        Size {
            width: 7,
            height: 3,
        },
        note.clone(),
        "\"一二三\n       \n       \n",
    );
    pane_test_with_size(
        Size {
            width: 6,
            height: 3,
        },
        note.clone(),
        "\"一二 \n      \n      \n",
    );
    pane_test_with_size(
        Size {
            width: 5,
            height: 3,
        },
        note.clone(),
        "\"一二\n     \n     \n",
    );
}

#[test]
fn test_pane_cursor_heights() {
    #[track_caller]
    fn test_at_height(focus_height: f32, expected: &str) {
        let options = PrintingOptions {
            focus_path: Vec::new(),
            focus_height,
            width_strategy: WidthStrategy::Full,
            focus_target: FocusTarget::Start,
            line_wrapping: LineWrapping::Clip,
            set_focus: false,
        };
        let doc = json_string("Hi");
        let contents = SimpleLabel(Some((&doc, options)), PhantomData);
        pane_test_with_size(
            Size {
                width: 4,
                height: 7,
            },
            PaneNotation::Doc { label: contents },
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
    fn test_with_width(width_strategy: WidthStrategy, expected: &str) {
        let options = PrintingOptions {
            focus_path: Vec::new(),
            focus_height: 0.0,
            width_strategy,
            focus_target: FocusTarget::Start,
            line_wrapping: LineWrapping::Clip,
            set_focus: false,
        };
        let doc = json_array(vec![json_string("Hello"), json_string("world")]);
        let contents = SimpleLabel(Some((&doc, options)), PhantomData);
        pane_test(PaneNotation::Doc { label: contents }, expected);
    }

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

    test_with_width(WidthStrategy::Full, &narrow);
    test_with_width(WidthStrategy::Fixed(80), &wide);
    test_with_width(WidthStrategy::Fixed(10), &narrow);
    test_with_width(WidthStrategy::NoMoreThan(80), &narrow);
    test_with_width(WidthStrategy::NoMoreThan(5), &clipped);
}

fn make_array(start: usize, end: usize) -> Json {
    json_array((start..end).map(|x| json_number(x as f64)).collect())
}

#[test]
fn test_seek() {
    fn make_note<'a>(
        doc: &'a Json,
        path: &[usize],
        focus_target: FocusTarget,
    ) -> PaneNotation<SimpleLabel<'a, &'a Json>, BasicStyle> {
        let options = PrintingOptions {
            focus_path: path.to_owned(),
            focus_height: 0.5,
            width_strategy: WidthStrategy::Full,
            line_wrapping: LineWrapping::Clip,
            focus_target,
            set_focus: false,
        };

        PaneNotation::Doc {
            label: SimpleLabel(Some((&doc, options)), PhantomData),
        }
    }
    let doc10 = make_array(0, 8);
    pane_test(
        make_note(&doc10, &[], FocusTarget::Start),
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
    pane_test(
        make_note(&doc10, &[], FocusTarget::End),
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
    pane_test(
        make_note(&doc10, &[1], FocusTarget::Start),
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
    pane_test(
        make_note(&doc10, &[1], FocusTarget::End),
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
    fn make_note(doc: &Json) -> PaneNotation<SimpleLabel<&Json>, BasicStyle> {
        let options = PrintingOptions {
            focus_path: Vec::new(),
            focus_height: 0.0,
            width_strategy: WidthStrategy::Full,
            line_wrapping: LineWrapping::Clip,
            focus_target: FocusTarget::Start,
            set_focus: false,
        };

        PaneNotation::Doc {
            label: SimpleLabel(Some((&doc, options)), PhantomData),
        }
    }

    let doc8 = make_array(0, 6);
    let doc5 = make_array(6, 9);
    let doc_num = json_number(42.0);
    let doc_unicode = json_string("一1");

    pane_test(
        PaneNotation::Vert(vec![
            (PaneSize::Proportional(1), make_note(&doc8)),
            (PaneSize::Dynamic, make_note(&doc5)),
        ]),
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

    pane_test(
        PaneNotation::Vert(vec![
            (PaneSize::Proportional(1), make_note(&doc5)),
            (PaneSize::Dynamic, make_note(&doc8)),
        ]),
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

    pane_test(
        PaneNotation::Horz(vec![
            (PaneSize::Proportional(1), make_note(&doc8)),
            (PaneSize::Dynamic, make_note(&doc_num)),
        ]),
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

    pane_test_with_size(
        Size {
            width: 11,
            height: 7,
        },
        PaneNotation::Horz(vec![
            (PaneSize::Proportional(1), make_note(&doc8)),
            (PaneSize::Dynamic, make_note(&doc_unicode)),
        ]),
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

    pane_test_with_size(
        Size {
            width: 10,
            height: 7,
        },
        PaneNotation::Horz(vec![
            (PaneSize::Proportional(1), make_note(&doc8)),
            (PaneSize::Dynamic, make_note(&doc_unicode)),
        ]),
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
    let options = PrintingOptions {
        focus_path: vec![2, 0],
        focus_height: 0.5,
        width_strategy: WidthStrategy::Full,
        line_wrapping: LineWrapping::Clip,
        focus_target: FocusTarget::End,
        set_focus: true,
    };
    let doc = json_array(vec![
        json_string("Hello"),
        json_string("darkness,"),
        json_array(vec![json_string("my"), json_string("old")]),
        json_string("friend"),
    ]);
    let contents = SimpleLabel(Some((&doc, options)), PhantomData);
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
                    (
                        PaneSize::Proportional(1),
                        PaneNotation::Doc { label: contents },
                    ),
                ]),
            ),
        ]),
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
        Pos { col: 11, row: 7 },
    );
}

#[test]
fn test_line_wrapping() {
    let options = PrintingOptions {
        focus_path: vec![2, 0],
        focus_height: 0.5,
        width_strategy: WidthStrategy::Full,
        line_wrapping: LineWrapping::Wrap("/", BasicStyle::new()),
        focus_target: FocusTarget::Start,
        set_focus: false,
    };
    let doc = json_array(vec![
        json_number(1111111.),
        json_number(22222222222.),
        json_array(vec![json_number(3333.), json_number(4.)]),
        json_number(55555555.),
        json_number(6666666666666666.),
    ]);
    let contents = SimpleLabel(Some((&doc, options)), PhantomData);
    pane_test_with_size(
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
                    (
                        PaneSize::Proportional(1),
                        PaneNotation::Doc { label: contents },
                    ),
                    (PaneSize::Fixed(2), fill('@')),
                ]),
            ),
        ]),
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
    let options = PrintingOptions {
        focus_path: Vec::new(),
        focus_height: 0.0,
        width_strategy: WidthStrategy::Full,
        line_wrapping: LineWrapping::Wrap("ooo", BasicStyle::new()),
        focus_target: FocusTarget::Start,
        set_focus: false,
    };
    let doc = json_string("一二三");
    let contents = SimpleLabel(Some((&doc, options)), PhantomData);
    let note = PaneNotation::Doc { label: contents };

    pane_test_with_size(
        Size {
            width: 8,
            height: 3,
        },
        note.clone(),
        &[
            // force rustfmt
            r#""一二三""#,
            r#"        "#,
            r#"        "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test_with_size(
        Size {
            width: 7,
            height: 3,
        },
        note.clone(),
        &[
            // force rustfmt
            r#""一二三"#,
            r#"ooo"   "#,
            r#"       "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test_with_size(
        Size {
            width: 6,
            height: 3,
        },
        note.clone(),
        &[
            // force rustfmt
            r#""一二 "#,
            r#"ooo三""#,
            r#"      "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test_with_size(
        Size {
            width: 5,
            height: 3,
        },
        note.clone(),
        &[
            // force rustfmt
            r#""一二"#,
            r#"ooo三"#,
            r#"ooo" "#,
            r#""#,
        ]
        .join("\n"),
    );

    pane_test_with_size(
        Size {
            width: 4,
            height: 3,
        },
        note.clone(),
        "\"一 \n    \n    \n",
    );

    pane_test_with_size(
        Size {
            width: 3,
            height: 3,
        },
        note.clone(),
        "\"一\n   \n   \n",
    );

    pane_test_with_size(
        Size {
            width: 2,
            height: 3,
        },
        note.clone(),
        "\" \n  \n  \n",
    );

    pane_test_with_size(
        Size {
            width: 1,
            height: 3,
        },
        note.clone(),
        "\"\n \n \n",
    );

    pane_test_with_size(
        Size {
            width: 0,
            height: 3,
        },
        note.clone(),
        "",
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
