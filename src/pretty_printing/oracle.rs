use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::{Line, Width};
use std::fmt;

/// (lines, flat_of_newline)
#[derive(Debug, Clone)]
struct Layout(Vec<(Width, String)>, bool);

impl Layout {
    fn text(s: &str) -> Layout {
        Layout(vec![(0, s.to_string())], false)
    }

    fn newline(flat: bool) -> Layout {
        Layout(vec![(0, String::new()), (0, String::new())], flat)
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
        self.1 |= other.1;
        self
    }

    fn pick(self, other: Layout, flat: bool, width: Width, line: Line) -> Layout {
        if flat {
            return self;
        }
        if self.1 {
            return other;
        }

        let self_line_len = {
            let (spaces, string) = &self.0[line as usize];
            spaces + string.chars().count() as Width
        };
        if self_line_len <= width {
            self
        } else {
            other
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
        Newline => cont(Layout::newline(notation.is_flat()).indent(notation.indentation())),
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
    let ab = Layout(vec![(1, "a".to_owned()), (1, "bb".to_owned())], false);
    let cd = Layout(vec![(2, "ccc".to_owned()), (2, "dddd".to_owned())], false);
    let abcd = ab.append(cd);
    assert_eq!(format!("{}", abcd), " a\n bbccc\n  dddd");

    let hello = Layout::text("Hello");
    let world = Layout::text("world!");
    let newline = Layout::newline(false);
    let hello_world = hello.append(newline).append(world);
    assert_eq!(format!("{}", hello_world), "Hello\nworld!");
    assert_eq!(format!("{}", hello_world.indent(2)), "  Hello\n  world!");
}
