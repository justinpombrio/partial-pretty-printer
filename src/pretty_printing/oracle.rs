use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::{Line, Width};
use std::fmt;

#[derive(Debug, Clone)]
struct Layout(Vec<(Width, String)>);

impl Layout {
    fn text(s: &str) -> Layout {
        Layout(vec![(0, s.to_string())])
    }

    fn newline() -> Layout {
        Layout(vec![(0, String::new()), (0, String::new())])
    }

    fn indent(mut self, ind: Width) -> Layout {
        for line in &mut self.0 {
            line.0 += ind;
        }
        self
    }

    fn append(mut self, other: Layout) -> Layout {
        let mut other_iter = other.0.into_iter();
        let suffix = other_iter.next().unwrap().1;
        let lines = &mut self.0;
        lines.last_mut().unwrap().1.push_str(&suffix);
        for line in other_iter {
            lines.push(line);
        }
        self
    }

    fn pick(self, other: Layout, flat: bool, width: Width, line: Line) -> Layout {
        let x_lines = self.0;
        let y_lines = other.0;

        if flat {
            return Layout(x_lines);
        }

        let x_line_len = {
            let (spaces, string) = &x_lines[line as usize];
            spaces + string.chars().count() as Width
        };
        if x_line_len <= width {
            Layout(x_lines)
        } else {
            Layout(y_lines)
        }
    }

    fn num_newlines(&self) -> Line {
        (self.0.len() - 1) as Line
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lines = &self.0;
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
        Literal(s) => cont(Layout::text(s.str())),
        Newline => cont(Layout::newline().indent(notation.indentation())),
        Text(s, _) => cont(Layout::text(s)),
        Child(_, x) => pp(width, line, x, cont),
        Concat(x, y) => pp(width, line, x, &|x_lay| {
            pp(width, line + x_lay.num_newlines(), y, &|y_lay| {
                cont(x_lay.clone().append(y_lay))
            })
        }),
        Choice(x, y) => {
            let x_lay = pp(width, line, x, cont);
            let y_lay = pp(width, line, y, cont);
            x_lay.pick(y_lay, notation.is_flat(), width, line)
        }
    }
}

#[test]
fn test_layout() {
    let ab = Layout(vec![(1, "a".to_owned()), (1, "bb".to_owned())]);
    let cd = Layout(vec![(2, "ccc".to_owned()), (2, "dddd".to_owned())]);
    let abcd = ab.append(cd);
    assert_eq!(format!("{}", abcd), " a\n bbccc\n  dddd");

    let hello = Layout::text("Hello");
    let world = Layout::text("world!");
    let newline = Layout::newline();
    let hello_world = hello.append(newline).append(world);
    assert_eq!(format!("{}", hello_world), "Hello\nworld!");
    assert_eq!(format!("{}", hello_world.indent(2)), "  Hello\n  world!");
}
