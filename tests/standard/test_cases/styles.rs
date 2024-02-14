// TODO: temp
#![allow(unused)]

use crate::standard::pretty_testing::SimpleDoc;
use partial_pretty_printer::{
    examples::json::{json_list, json_number, Json},
    notation_constructors::lit,
    pane::{pane_print, Label, PaneNotation, PrettyWindow, RenderOptions, WidthStrategy},
    Pos, PrettyDoc, Row, Size, Width,
};
use std::convert::Infallible;
use std::fmt;
use std::fmt::Debug;
use std::iter;
use std::marker::PhantomData;

type Style = char;
type Mark = char;

#[derive(Debug, Clone)]
struct SimpleLabel<'d, D: PrettyDoc<'d> + Clone + Debug>(
    Option<(D, Vec<usize>)>,
    PhantomData<&'d D>,
);

impl<'d, D: PrettyDoc<'d> + Clone + Debug> Label for SimpleLabel<'d, D> {}

fn get_content<'d, D: PrettyDoc<'d> + Clone + Debug>(
    label: SimpleLabel<'d, D>,
) -> Option<(D, Vec<usize>)> {
    label.0
}

#[derive(Debug)]
struct RichText {
    lines: Vec<Vec<(char, Style, Mark)>>,
    size: Size,
}

impl fmt::Display for RichText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            // Print the line
            write!(f, "|")?;
            for (ch, _, _) in line {
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;

            // Print the styles
            write!(f, "|")?;
            for (_, style, _) in line {
                write!(f, "{}", style)?;
            }
            writeln!(f)?;

            // Print the mark
            write!(f, "|")?;
            for (_, _, mark) in line {
                write!(f, "{}", mark)?;
            }
            writeln!(f)?;

            // Print a blank line for legibility
            writeln!(f)?;
        }
        Ok(())
    }
}

impl RichText {
    fn new(size: Size) -> RichText {
        RichText {
            size,
            lines: vec![],
        }
    }

    fn get_mut_line(&mut self, line_num: Row) -> &mut Vec<(char, Style, Mark)> {
        if self.lines.len() < line_num as usize + 1 {
            self.lines.resize_with(line_num as usize + 1, Vec::new);
        }
        &mut self.lines[line_num as usize]
    }

    fn get_mut_char(&mut self, pos: Pos) -> &mut (char, Style, Mark) {
        let line = self.get_mut_line(pos.row);
        if line.len() < pos.col as usize + 1 {
            line.resize_with(pos.col as usize + 1, || (' ', ' ', ' '));
        }
        &mut line[pos.col as usize]
    }
}

impl PrettyWindow for RichText {
    type Error = Infallible;
    type Style = Style;
    type Mark = Mark;

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn print_char(
        &mut self,
        ch: char,
        pos: Pos,
        mark: Option<&Mark>,
        style: &Style,
        full_width: bool,
    ) -> Result<(), Self::Error> {
        *self.get_mut_char(pos) = (ch, *style, mark.copied().unwrap_or(' '));
        Ok(())
    }
}

#[track_caller]
fn pane_test<'d>(doc: SimpleDoc<Style>, path: Vec<usize>, width: Width, expected: &str) {
    let render_options = RenderOptions {
        cursor_height: 1.0,
        width_strategy: WidthStrategy::Full,
    };
    let mut screen = RichText::new(Size { width, height: 100 });
    let label = SimpleLabel(Some((&doc, path)), PhantomData);
    let pane_notation = PaneNotation::Doc {
        label,
        render_options,
    };
    pane_print(&mut screen, &pane_notation, &get_content).unwrap();
    let actual = screen.to_string();
    if actual != expected {
        eprintln!("ACTUAL:\n{}", actual);
        eprintln!("EXPECTED:\n{}", expected);
    }
    assert_eq!(actual, expected);
}

/*
#[test]
fn test_pane_styles() {
    let words = vec![
        lit(
            "Hello",
            Style {
                color: Color::Base09,
                bold: false,
                underlined: true,
                reversed: false,
            },
        ),
        lit(
            ",",
            Style {
                color: Color::Base0A,
                bold: true,
                underlined: false,
                reversed: false,
            },
        ),
        lit(" ", Style::plain()),
        lit(
            "world",
            Style {
                color: Color::Base0B,
                bold: false,
                underlined: false,
                reversed: false,
            },
        ),
        lit(
            "!",
            Style {
                color: Color::Base0C,
                bold: true,
                underlined: true,
                reversed: true,
            },
        ),
    ];
    let note = words.into_iter().reduce(|n1, n2| n1 + n2).unwrap();
    pane_test(
        &SimpleDoc::new(note),
        vec![],
        80,
        "|Hello, world!\n\
         |99999A5BBBBBC\n\
         |uuuuub......!\n\
         |aaaaaaaaaaaaA\n\n",
    );
}

#[test]
fn test_pane_highlighting() {
    // Expected highlighting:
    // [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]
    // cccccccccccccccccccbaaaaaabbbbbbbbbc
    fn num(n: usize) -> Json {
        json_number(n as f64)
    }
    let doc = json_list(vec![
        json_list(vec![
            json_list(vec![num(1), num(2)]),
            json_list(vec![num(3), num(4)]),
        ]),
        json_list(vec![
            json_list(vec![num(5), num(6)]),
            json_list(vec![num(7), num(8)]),
        ]),
    ]);
    pane_test(
        &doc,
        vec![1, 0],
        20,
        "|        [5, 6], [\n\
         |55555555595595555\n\
         |.................\n\
         |bbbbbbbbaaaaaabbb\n\
                           \n\
         |            7, 8\n\
         |5555555555559559\n\
         |................\n\
         |bbbbbbbbbbbbbbbb\n\
                          \n\
         |        ]\n\
         |555555555\n\
         |.........\n\
         |bbbbbbbbb\n\
                   \n\
         |    ]\n\
         |55555\n\
         |.....\n\
         |bbbbb\n\
               \n\
         |]\n\
         |5\n\
         |.\n\
         |c\n\
           \n",
    );
}
*/
