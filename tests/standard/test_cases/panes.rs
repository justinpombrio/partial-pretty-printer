use crate::standard::pretty_testing::SimpleDoc;
use partial_pretty_printer::{
    doc_examples::{
        json::{json_array, json_number, json_string, Json},
        BasicStyle,
    },
    pane::{
        display_pane, DocLabel, FocusSide, PaneNotation, PaneSize, PlainText, PrintingOptions,
        WidthStrategy,
    },
    Pos, PrettyDoc, Size, Style,
};
use std::fmt::Debug;
use std::marker::PhantomData;

type NoStyle = ();

#[derive(Debug, Clone)]
struct SimpleLabel<'d, D: PrettyDoc<'d> + Clone + Debug>(
    Option<(D, PrintingOptions)>,
    PhantomData<&'d D>,
);

fn get_content<'d, D: PrettyDoc<'d> + Clone + Debug>(
    label: SimpleLabel<'d, D>,
) -> Option<(D, PrintingOptions)> {
    label.0
}

#[track_caller]
fn pane_test<'d, S: Style, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
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
fn pane_test_with_size<'d, S: Style, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    size: Size,
    notation: PaneNotation<SimpleLabel<'d, D>, S>,
    expected: &str,
) {
    let mut screen = PlainText::new(size.width, size.height);
    display_pane(&mut screen, &notation, &get_content).unwrap();
    let actual = screen.to_string();
    if actual != expected {
        eprintln!("ACTUAL:\n{}", actual);
        eprintln!("EXPECTED:\n{}", expected);
    }
    assert_eq!(actual, expected);
}

#[track_caller]
fn pane_test_with_focus<'d, S: Style, D: PrettyDoc<'d, Style = S> + Clone + Debug>(
    size: Size,
    notation: PaneNotation<SimpleLabel<'d, D>, S>,
    expected: &str,
    expected_pos: Pos,
) {
    let mut screen = PlainText::new(size.width, size.height);
    display_pane(&mut screen, &notation, &get_content).unwrap();
    let actual = screen.to_string();
    if actual != expected {
        eprintln!("ACTUAL:\n{}", actual);
        eprintln!("EXPECTED:\n{}", expected);
    }
    assert_eq!(actual, expected);
    assert_eq!(screen.focus_points(), &[expected_pos]);
}

fn fill<L: DocLabel, S: Style + Default>(ch: char) -> PaneNotation<L, S> {
    PaneNotation::Fill {
        ch,
        style: S::default(),
    }
}

#[test]
fn test_empty_pane() {
    pane_test::<NoStyle, &SimpleDoc>(PaneNotation::Empty, "");
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
        "信信信\n\
         信信信\n\
         信信信\n",
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
        focus_side: FocusSide::Start,
        set_focus: false,
    };
    let doc = json_array(vec![json_string("Hello"), json_string("world")]);
    let contents = SimpleLabel(Some((&doc, options)), PhantomData);
    pane_test(
        PaneNotation::Doc { label: contents },
        "[\n    \"He\n    \"wo\n]\n",
    );
}

#[test]
fn test_doc_pane_full_width_cutoff() {
    let options = PrintingOptions {
        focus_path: Vec::new(),
        focus_height: 0.0,
        width_strategy: WidthStrategy::Full,
        focus_side: FocusSide::Start,
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
        "\"一二三\"\n",
    );
    pane_test_with_size(
        Size {
            width: 7,
            height: 3,
        },
        note.clone(),
        "\"一二三\n",
    );
    pane_test_with_size(
        Size {
            width: 6,
            height: 3,
        },
        note.clone(),
        "\"一二\n",
    );
    pane_test_with_size(
        Size {
            width: 5,
            height: 3,
        },
        note.clone(),
        "\"一二\n",
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
            focus_side: FocusSide::Start,
            set_focus: false,
        };
        let doc = json_string("Hi");
        let contents = SimpleLabel(Some((&doc, options)), PhantomData);
        pane_test(PaneNotation::Doc { label: contents }, expected);
    }

    test_at_height(0.0, "\"Hi\"\n");
    test_at_height(0.17, "\n\"Hi\"\n");
    test_at_height(0.33, "\n\n\"Hi\"\n");
    test_at_height(0.5, "\n\n\n\"Hi\"\n");
    test_at_height(0.67, "\n\n\n\n\"Hi\"\n");
    test_at_height(0.83, "\n\n\n\n\n\"Hi\"\n");
    test_at_height(1.0, "\n\n\n\n\n\n\"Hi\"\n");
}

#[test]
fn test_pane_widths() {
    #[track_caller]
    fn test_with_width(width_strategy: WidthStrategy, expected: &str) {
        let options = PrintingOptions {
            focus_path: Vec::new(),
            focus_height: 0.0,
            width_strategy,
            focus_side: FocusSide::Start,
            set_focus: false,
        };
        let doc = json_array(vec![json_string("Hello"), json_string("world")]);
        let contents = SimpleLabel(Some((&doc, options)), PhantomData);
        pane_test(PaneNotation::Doc { label: contents }, expected);
    }

    test_with_width(WidthStrategy::Full, "[\n    \"He\n    \"wo\n]\n");
    test_with_width(WidthStrategy::Fixed(80), "[\"Hello\n");
    test_with_width(WidthStrategy::Fixed(10), "[\n    \"He\n    \"wo\n]\n");
    test_with_width(WidthStrategy::NoMoreThan(80), "[\n    \"He\n    \"wo\n]\n");
    test_with_width(WidthStrategy::NoMoreThan(5), "[\n    \"He\n    \"wo\n]\n");
}

fn make_array(start: usize, end: usize) -> Json {
    json_array(
        (start..end)
            .into_iter()
            .map(|x| json_number(x as f64))
            .collect(),
    )
}

#[test]
fn test_seek() {
    fn make_note<'a>(
        doc: &'a Json,
        path: &[usize],
        focus_side: FocusSide,
    ) -> PaneNotation<SimpleLabel<'a, &'a Json>, BasicStyle> {
        let options = PrintingOptions {
            focus_path: path.to_owned(),
            focus_height: 0.5,
            width_strategy: WidthStrategy::Full,
            focus_side,
            set_focus: false,
        };

        PaneNotation::Doc {
            label: SimpleLabel(Some((&doc, options)), PhantomData),
        }
    }
    let doc10 = make_array(0, 8);
    pane_test(
        make_note(&doc10, &[], FocusSide::Start),
        &[
            "",         // force rustfmt
            "",         // force rustfmt
            "",         // force rustfmt
            "[",        // force rustfmt
            "    0,",   // force rustfmt
            "    1,",   // force rustfmt
            "    2,\n", // force rustfmt
        ]
        .join("\n"),
    );
    pane_test(
        make_note(&doc10, &[], FocusSide::End),
        &[
            "    5,", // force rustfmt
            "    6,", // force rustfmt
            "    7",  // force rustfmt
            "]\n",    // force rustfmt
        ]
        .join("\n"),
    );
    pane_test(
        make_note(&doc10, &[1], FocusSide::Start),
        &[
            "",         // force rustfmt
            "[",        // force rustfmt
            "    0,",   // force rustfmt
            "    1,",   // force rustfmt
            "    2,",   // force rustfmt
            "    3,",   // force rustfmt
            "    4,\n", // force rustfmt
        ]
        .join("\n"),
    );
    pane_test(
        make_note(&doc10, &[1], FocusSide::End),
        &[
            "",         // force rustfmt
            "[",        // force rustfmt
            "    0,",   // force rustfmt
            "    1,",   // force rustfmt
            "    2,",   // force rustfmt
            "    3,",   // force rustfmt
            "    4,\n", // force rustfmt
        ]
        .join("\n"),
    );
}

#[test]
fn test_dynamic() {
    fn make_note<'a>(doc: &'a Json) -> PaneNotation<SimpleLabel<'a, &'a Json>, BasicStyle> {
        let options = PrintingOptions {
            focus_path: Vec::new(),
            focus_height: 0.0,
            width_strategy: WidthStrategy::Full,
            focus_side: FocusSide::Start,
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
            "[",      // force rustfmt
            "    0,", // force rustfmt
            "[",      // force rustfmt
            "    6,", // force rustfmt
            "    7,", // force rustfmt
            "    8",  // force rustfmt
            "]\n",
        ]
        .join("\n"),
    );

    pane_test(
        PaneNotation::Vert(vec![
            (PaneSize::Proportional(1), make_note(&doc5)),
            (PaneSize::Dynamic, make_note(&doc8)),
        ]),
        &[
            "[",      // force rustfmt
            "    0,", // force rustfmt
            "    1,", // force rustfmt
            "    2,", // force rustfmt
            "    3,", // force rustfmt
            "    4,", // force rustfmt
            "    5\n",
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
            "    0",   // force rustfmt
            "    1",   // force rustfmt
            "    2",   // force rustfmt
            "    3",   // force rustfmt
            "    4",   // force rustfmt
            "    5\n",
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
            "    0,",        // force rustfmt
            "    1,",        // force rustfmt
            "    2,",        // force rustfmt
            "    3,",        // force rustfmt
            "    4,",        // force rustfmt
            "    5\n",
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
            "    0",        // force rustfmt
            "    1",        // force rustfmt
            "    2",        // force rustfmt
            "    3",        // force rustfmt
            "    4",        // force rustfmt
            "    5\n",
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
        focus_side: FocusSide::End,
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
            height: 8,
        },
        PaneNotation::Vert(vec![
            (PaneSize::Fixed(3), fill('*')),
            (
                PaneSize::Proportional(1),
                PaneNotation::Horz(vec![
                    (PaneSize::Fixed(2), fill('#')),
                    (
                        PaneSize::Proportional(1),
                        PaneNotation::Doc { label: contents },
                    ),
                ]),
            ),
        ]),
        &[
            // force rustfmt
            "********************",
            "********************",
            "********************",
            "##    \"Hello\",",
            "##    \"darkness,\",",
            "##    [\"my\", \"old\"],",
            "##    \"friend\"",
            "##]",
            "",
        ]
        .join("\n"),
        Pos { col: 11, row: 5 },
    );
}
