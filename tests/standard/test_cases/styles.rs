use crate::standard::pretty_testing::SimpleDoc;
use partial_pretty_printer::{
    examples::json::{json_list, json_number, Json},
    notation_constructors::lit,
    pane::{pane_print, Label, PaneNotation, PrettyWindow, RenderOptions, WidthStrategy},
    Color, Line, Pos, PrettyDoc, ShadedStyle, Size, Style, Width,
};
use std::fmt;
use std::fmt::Debug;
use std::iter;
use std::marker::PhantomData;

// It's hard to test styles directly. No one would want to read or write test cases that used
// enormous debug-printed Style objects. Instead, these tests use a textual format. The first line
// is the text of the line, and subsequent lines show the style of the character above them:
//
// - Line 2 shows the Base16 style, as a hex char
// - Line 3 shows whether the char is bold (b), underlined (u), both (!), or neither (.).
// - Line 4 shows the shading (a-x), and whether the colors are reversed (capitalized).

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
    lines: Vec<Vec<(char, ShadedStyle)>>,
    size: Size,
}

impl fmt::Display for RichText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            // Print the line
            write!(f, "|")?;
            for (ch, _) in line {
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;

            // Print the colors
            write!(f, "|")?;
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
            writeln!(f)?;

            // Print the bold & underlined styles
            write!(f, "|")?;
            for (_, style) in line {
                let emph = match (style.bold, style.underlined) {
                    (false, false) => '.',
                    (true, false) => 'b',
                    (false, true) => 'u',
                    (true, true) => '!',
                };
                write!(f, "{}", emph)?;
            }
            writeln!(f)?;

            // Print the shade & reversed styles
            write!(f, "|")?;
            for (_, style) in line {
                let shade_and_rev = match (style.shade.0, style.reversed) {
                    (0, false) => 'a',
                    (0, true) => 'A',
                    (1, false) => 'b',
                    (1, true) => 'B',
                    (2, false) => 'c',
                    (2, true) => 'C',
                    (255, false) => 'x',
                    (255, true) => 'X',
                    (_, _) => unimplemented!(),
                };
                write!(f, "{}", shade_and_rev)?;
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

    fn get_mut_line(&mut self, line_num: Line) -> &mut Vec<(char, ShadedStyle)> {
        if self.lines.len() < line_num as usize + 1 {
            self.lines.resize_with(line_num as usize + 1, Vec::new);
        }
        &mut self.lines[line_num as usize]
    }

    fn get_mut_char(&mut self, pos: Pos) -> &mut (char, ShadedStyle) {
        let line = self.get_mut_line(pos.line);
        if line.len() < pos.col as usize + 1 {
            line.resize_with(pos.col as usize + 1, || (' ', ShadedStyle::plain()));
        }
        &mut line[pos.col as usize]
    }
}

impl PrettyWindow for RichText {
    type Error = fmt::Error;

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn print(&mut self, mut pos: Pos, string: &str, style: ShadedStyle) -> Result<(), Self::Error> {
        for ch in string.chars() {
            *self.get_mut_char(pos) = (ch, style);
            pos.col += 1;
        }
        Ok(())
    }

    fn fill(
        &mut self,
        pos: Pos,
        ch: char,
        len: Width,
        style: ShadedStyle,
    ) -> Result<(), Self::Error> {
        let string: String = iter::repeat(ch).take(len as usize).collect();
        self.print(pos, &string, style)
    }
}

#[track_caller]
fn pane_test<'d>(
    doc: impl PrettyDoc<'d> + Clone + Debug + 'd,
    path: Vec<usize>,
    width: Width,
    expected: &str,
) {
    let render_options = RenderOptions {
        highlight_cursor: true,
        cursor_height: 1.0,
        width_strategy: WidthStrategy::Full,
    };
    let mut screen = RichText::new(Size { width, height: 100 });
    let label = SimpleLabel(Some((doc, path)), PhantomData);
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
        &SimpleDoc(note),
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
