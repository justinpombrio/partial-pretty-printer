mod common;

use common::SimpleDoc;
use partial_pretty_printer::{
    examples::json::{json_list, json_string},
    pane::{pane_print, Label, PaneNotation, PaneSize, PlainText, RenderOptions, WidthStrategy},
    PrettyDoc, Style,
};
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct SimpleLabel<D: PrettyDoc + Clone + Debug>(Option<(D, Vec<usize>)>);
impl<D: PrettyDoc + Clone + Debug> Label for SimpleLabel<D> {}

fn get_content<D: PrettyDoc + Clone + Debug>(label: SimpleLabel<D>) -> Option<(D, Vec<usize>)> {
    label.0
}

#[track_caller]
fn pane_test<D: PrettyDoc + Clone + Debug>(notation: PaneNotation<SimpleLabel<D>>, expected: &str) {
    let mut screen = PlainText::new(7, 7);
    pane_print(&mut screen, &notation, &get_content).unwrap();
    let actual = screen.to_string();
    if actual != expected {
        eprintln!("ACTUAL:\n{}", actual);
        eprintln!("EXPECTED:\n{}", expected);
    }
    assert_eq!(actual, expected);
}

fn fill<L: Label>(ch: char) -> PaneNotation<L> {
    PaneNotation::Fill {
        ch,
        style: Style::default(),
    }
}

#[test]
fn test_empty_pane() {
    pane_test::<SimpleDoc>(PaneNotation::Empty, "");
}

#[test]
fn test_fill_pane() {
    pane_test::<SimpleDoc>(
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
fn test_horz_split_pane() {
    use PaneSize::{Fixed, Proportional};

    pane_test::<SimpleDoc>(
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

    pane_test::<SimpleDoc>(
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

    pane_test::<SimpleDoc>(
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
    let render_options = RenderOptions {
        highlight_cursor: false,
        cursor_height: 1.0,
        width_strategy: WidthStrategy::Full,
    };
    let doc = json_list(vec![json_string("Hello"), json_string("world")]);
    let contents = SimpleLabel(Some((doc, vec![])));
    pane_test(
        PaneNotation::Doc {
            label: contents,
            render_options,
        },
        "[\n    \"He\n    \"wo\n]\n",
    );
}

#[test]
fn test_pane_cursor_heights() {
    #[track_caller]
    fn test_at_height(cursor_height: f32, expected: &str) {
        let render_options = RenderOptions {
            highlight_cursor: false,
            cursor_height,
            width_strategy: WidthStrategy::Full,
        };
        let doc = json_string("Hi");
        let contents = SimpleLabel(Some((doc, vec![])));
        pane_test(
            PaneNotation::Doc {
                label: contents,
                render_options,
            },
            expected,
        );
    }

    test_at_height(1.0, "\"Hi\"\n");
    test_at_height(0.83, "\n\"Hi\"\n");
    test_at_height(0.67, "\n\n\"Hi\"\n");
    test_at_height(0.5, "\n\n\n\"Hi\"\n");
    test_at_height(0.33, "\n\n\n\n\"Hi\"\n");
    test_at_height(0.17, "\n\n\n\n\n\"Hi\"\n");
    test_at_height(0.0, "\n\n\n\n\n\n\"Hi\"\n");
}

#[test]
fn test_pane_widths() {
    #[track_caller]
    fn test_with_width(width_strategy: WidthStrategy, expected: &str) {
        let render_options = RenderOptions {
            highlight_cursor: false,
            cursor_height: 1.0,
            width_strategy,
        };
        let doc = json_list(vec![json_string("Hello"), json_string("world")]);
        let contents = SimpleLabel(Some((doc, vec![])));
        pane_test(
            PaneNotation::Doc {
                label: contents,
                render_options,
            },
            expected,
        );
    }

    test_with_width(WidthStrategy::Full, "[\n    \"He\n    \"wo\n]\n");
    test_with_width(WidthStrategy::Fixed(80), "[\"Hello\n");
    test_with_width(WidthStrategy::Fixed(10), "[\n    \"He\n    \"wo\n]\n");
    test_with_width(WidthStrategy::NoMoreThan(80), "[\n    \"He\n    \"wo\n]\n");
    test_with_width(WidthStrategy::NoMoreThan(5), "[\n    \"He\n    \"wo\n]\n");
}
