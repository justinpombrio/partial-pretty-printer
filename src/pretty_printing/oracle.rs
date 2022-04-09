use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::{Line, Width};
use std::fmt;

#[derive(Debug, Clone)]
enum Layout {
    Lines(Vec<(Width, String)>),
    Error,
}

impl Layout {
    fn empty() -> Layout {
        Layout::Lines(vec![(0, String::new())])
    }

    fn text(s: &str) -> Layout {
        Layout::Lines(vec![(0, s.to_string())])
    }

    fn newline() -> Layout {
        Layout::Lines(vec![(0, String::new()), (0, String::new())])
    }

    fn indent(self, ind: Width) -> Layout {
        match self {
            Layout::Error => Layout::Error,
            Layout::Lines(mut lines) => {
                for i in 1..lines.len() {
                    lines[i].0 += ind;
                }
                Layout::Lines(lines)
            }
        }
    }

    fn flatten(self) -> Layout {
        match self {
            Layout::Error => Layout::Error,
            Layout::Lines(lines) if lines.len() == 1 => Layout::Lines(lines),
            Layout::Lines(_) => Layout::Error,
        }
    }

    fn append(self, other: Layout) -> Layout {
        match (self, other) {
            (Layout::Error, _) => Layout::Error,
            (_, Layout::Error) => Layout::Error,
            (Layout::Lines(x), Layout::Lines(y)) => {
                let mut y_iter = y.into_iter();
                let suffix = y_iter.next().unwrap().1;
                let mut lines = x;
                lines.last_mut().unwrap().1.push_str(&suffix);
                for line in y_iter {
                    lines.push(line);
                }
                Layout::Lines(lines)
            }
        }
    }

    fn pick(self, other: Layout, width: Width, line: Line) -> Layout {
        match (self, other) {
            (Layout::Error, lay) => lay,
            (lay, Layout::Error) => lay,
            (Layout::Lines(x_lines), Layout::Lines(y_lines)) => {
                let x_line_len = {
                    let (spaces, string) = &x_lines[line as usize];
                    spaces + string.chars().count() as Width
                };
                if x_line_len <= width {
                    Layout::Lines(x_lines)
                } else {
                    Layout::Lines(y_lines)
                }
            }
        }
    }

    fn num_newlines(&self) -> Line {
        match self {
            Layout::Error => 0,
            Layout::Lines(lines) => (lines.len() - 1) as Line,
        }
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Layout::Error => write!(f, "[error]"),
            Layout::Lines(lines) => {
                let len = lines.len();
                for (i, (spaces, line)) in lines.into_iter().enumerate() {
                    write!(f, "{:spaces$}{}", "", line, spaces = *spaces as usize)?;
                    if i + 1 != len {
                        write!(f, "\n")?;
                    }
                }
                Ok(())
            }
        }
    }
}

/// For testing!
///
/// Pretty print the document with the given width. Prints "\\[error\\]" in case of error (Flat of
/// Newline). This is only meant for testing. It's slow: roughly exponential in the size of the doc
/// / doubly exponential in the size of the output.
pub fn oracular_pretty_print<'d, D: PrettyDoc<'d>>(doc: D, width: Width) -> String {
    let notation = NotationRef::new(doc);
    let layout = pp(width, 0, notation, &|s: Layout| s);
    format!("{}", layout)
}

fn pp<'d, D: PrettyDoc<'d>>(
    width: Width,
    line: Line,
    notation: NotationRef<'d, D>,
    cont: &dyn Fn(Layout) -> Layout,
) -> Layout {
    use NotationCase::*;

    match notation.case() {
        Empty => cont(Layout::empty()),
        Literal(s) => cont(Layout::text(s.str())),
        Newline => cont(Layout::newline()),
        Text(s, _) => cont(Layout::text(s)),
        Indent(i, x) => pp(width, line, x, &|lay| cont(lay.indent(i))),
        Flat(x) => pp(width, line, x, &|lay| cont(lay.flatten())),
        Child(_, x) => pp(width, line, x, cont),
        Concat(x, y) => pp(width, line, x, &|x_lay| {
            pp(width, line + x_lay.num_newlines(), y, &|y_lay| {
                cont(x_lay.clone().append(y_lay))
            })
        }),
        Choice(x, y) => {
            let x_lay = pp(width, line, x, cont);
            let y_lay = pp(width, line, y, cont);
            x_lay.pick(y_lay, width, line)
        }
    }
}

#[test]
fn test_layout() {
    let ab = Layout::Lines(vec![(1, "a".to_owned()), (1, "bb".to_owned())]);
    let cd = Layout::Lines(vec![(2, "ccc".to_owned()), (2, "dddd".to_owned())]);
    let abcd = ab.append(cd);
    assert_eq!(format!("{}", abcd), " a\n bbccc\n  dddd");

    let hello = Layout::text("Hello");
    let world = Layout::text("world!");
    let newline = Layout::newline();
    let hello_world = hello.append(newline).append(world);
    assert_eq!(format!("{}", hello_world), "Hello\nworld!");
    assert_eq!(format!("{}", hello_world.indent(2)), "Hello\n  world!");
}
