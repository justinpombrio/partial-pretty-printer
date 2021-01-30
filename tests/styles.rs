#![feature(iterator_fold_self)]

mod common;

use common::SimpleDoc;
use partial_pretty_printer::notation_constructors::{empty, lit};
use partial_pretty_printer::{
    pane_print, Color, Emph, Height, Label, Line, Notation, PaneNotation, Pos, PrettyDoc,
    PrettyWindow, RenderOptions, ShadedStyle, Size, Style, Width, WidthStrategy,
};
use std::fmt;
use std::fmt::Debug;
use std::iter;

#[derive(Debug, Clone)]
struct SimpleLabel<D: PrettyDoc + Clone + Debug>(Option<(D, Vec<usize>)>);
impl<D: PrettyDoc + Clone + Debug> Label for SimpleLabel<D> {}

fn get_content<D: PrettyDoc + Clone + Debug>(label: SimpleLabel<D>) -> Option<(D, Vec<usize>)> {
    label.0
}

#[derive(Debug)]
struct RichText {
    lines: Vec<Vec<(char, ShadedStyle)>>,
    size: Size,
}

impl fmt::Display for RichText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            for (ch, _) in line {
                write!(f, "{}", ch)?;
            }
            write!(f, "\n")?;
            for (_, style) in line {
                let color = match style.color {
                    Color::Base00 => '0',
                    Color::Base01 => '1',
                    Color::Base02 => '2',
                    Color::Base03 => '3',
                    Color::Base04 => '4',
                    Color::Base05 => '5',
                    Color::Base06 => '6',
                    Color::Base07 => '7',
                    Color::Base08 => '8',
                    Color::Base09 => '9',
                    Color::Base0A => 'A',
                    Color::Base0B => 'B',
                    Color::Base0C => 'C',
                    Color::Base0D => 'D',
                    Color::Base0E => 'E',
                    Color::Base0F => 'F',
                };
                write!(f, "{}", color)?;
            }
            write!(f, "\n")?;
            for (_, style) in line {
                let emph = match (style.emph.bold, style.emph.underlined) {
                    (false, false) => ' ',
                    (true, false) => 'b',
                    (false, true) => 'u',
                    (true, true) => '!',
                };
                write!(f, "{}", emph)?;
            }
            write!(f, "\n")?;
            for (_, style) in line {
                let shade_and_rev = match (style.shade.0, style.reversed) {
                    (0, false) => 'a',
                    (0, true) => 'A',
                    (1, false) => 'b',
                    (1, true) => 'B',
                    (2, false) => 'c',
                    (2, true) => 'C',
                    (_, false) => 'd',
                    (_, true) => 'D',
                };
                write!(f, "{}", shade_and_rev)?;
            }
            write!(f, "\n\n")?;
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

    fn get_mut_line(&mut self, line_num: Line) -> &mut Vec<(char, ShadedStyle)> {
        if self.lines.len() < line_num.0 as usize + 1 {
            self.lines
                .resize_with(line_num.0 as usize + 1, || Vec::new());
        }
        &mut self.lines[line_num.0 as usize]
    }

    fn get_mut_char(&mut self, pos: Pos) -> &mut (char, ShadedStyle) {
        let line = self.get_mut_line(pos.line);
        if line.len() < pos.col.0 as usize + 1 {
            line.resize_with(pos.col.0 as usize + 1, || (' ', ShadedStyle::plain()));
        }
        &mut line[pos.col.0 as usize]
    }
}

impl PrettyWindow for RichText {
    type Error = fmt::Error;

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn print(&mut self, pos: Pos, string: &str, style: ShadedStyle) -> Result<(), Self::Error> {
        for (i, ch) in string.chars().enumerate() {
            *self.get_mut_char(pos + Width(i as u16)) = (ch, style);
        }
        Ok(())
    }

    fn fill(
        &mut self,
        pos: Pos,
        ch: char,
        len: usize,
        style: ShadedStyle,
    ) -> Result<(), Self::Error> {
        let string: String = iter::repeat(ch).take(len).collect();
        self.print(pos, &string, style)
    }
}

#[track_caller]
fn pane_test(notation: Notation, expected: &str) {
    let render_options = RenderOptions {
        highlight_cursor: true,
        cursor_height: 1.0,
        width_strategy: WidthStrategy::Full,
    };
    let mut screen = RichText::new(Size {
        width: Width(80),
        height: Height(10),
    });
    let label = SimpleLabel(Some((SimpleDoc(notation), vec![])));
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

#[test]
fn test_pane_styles() {
    pane_test(empty(), "");

    let words = vec![
        lit(
            "Hello",
            Style {
                color: Color::Base09,
                emph: Emph::underlined(),
                reversed: false,
            },
        ),
        lit(
            ",",
            Style {
                color: Color::Base0A,
                emph: Emph::bold(),
                reversed: false,
            },
        ),
        lit(" ", Style::plain()),
        lit(
            "world",
            Style {
                color: Color::Base0B,
                emph: Emph::plain(),
                reversed: false,
            },
        ),
        lit(
            "!",
            Style {
                color: Color::Base0C,
                emph: Emph {
                    bold: true,
                    underlined: true,
                },
                reversed: true,
            },
        ),
    ];
    let note = words.into_iter().fold_first(|n1, n2| n1 + n2).unwrap();
    pane_test(
        note,
        "Hello, world!\n\
         99999A5BBBBBC\n\
         uuuuub      !\n\
         ddddddddddddD\n\n",
    );
}
