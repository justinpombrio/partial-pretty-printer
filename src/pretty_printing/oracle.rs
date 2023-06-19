use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use std::fmt;

const DEBUG_PRINT: bool = false;

struct Layout(Vec<(Width, String)>);

/// For testing!
///
/// Pretty print the document with the given width. This is meant only for testing.
/// It's slow: roughly exponential in the size of the doc.
pub fn oracular_pretty_print<'d, D: PrettyDoc<'d>>(doc: D, width: Width) -> String {
    let notation = NotationRef::new(doc);
    let layout = pp(Layout::empty(), notation, false, 0, Some(0), width);
    format!("{}", layout)
}

fn pp<'d, D: PrettyDoc<'d>>(
    prefix: Layout,
    notation: NotationRef<'d, D>,
    is_flat: bool,
    indent: Width,
    suffix_len: Option<Width>,
    width: Width,
) -> Layout {
    use NotationCase::*;

    if DEBUG_PRINT {
        println!(
            "==pp flat:{} indent:{} suffix_len:{:?} width:{}",
            is_flat, indent, suffix_len, width
        );
        println!("{}", prefix);
        println!("{}", notation);
        println!("==");
    }

    match notation.case() {
        Empty => prefix,
        Literal(lit) => prefix.append(Layout::text(lit.str())),
        Newline => prefix.append(Layout::newline(indent)),
        Text(txt, _) => prefix.append(Layout::text(txt)),
        Indent(i, x) => pp(prefix, x, is_flat, indent + i, suffix_len, width),
        Flat(x) => pp(prefix, x, true, indent, suffix_len, width),
        Child(_, x) => pp(prefix, x, is_flat, indent, suffix_len, width),
        Concat(x, y) => {
            let x_suffix_len = first_line_len(y, is_flat, suffix_len);
            let y_prefix = pp(prefix, x, is_flat, indent, x_suffix_len, width);
            pp(y_prefix, y, is_flat, indent, suffix_len, width)
        }
        Choice(x, y) => {
            let last_len = prefix.last_line_len();
            let first_len = first_line_len(x, is_flat, suffix_len);
            let fits = if let Some(first_len) = first_len {
                last_len + first_len <= width
            } else {
                false
            };
            if DEBUG_PRINT {
                println!(
                    "fits: {} + {:?} <= {} ? {} (flat: {})",
                    last_len, first_len, width, fits, is_flat
                );
            }
            let z = if is_flat || fits { x } else { y };
            pp(prefix, z, is_flat, indent, suffix_len, width)
        }
    }
}

fn first_line_len<'d, D: PrettyDoc<'d>>(
    notation: NotationRef<'d, D>,
    is_flat: bool,
    suffix_len: Option<Width>,
) -> Option<Width> {
    use NotationCase::*;

    match notation.case() {
        Empty => suffix_len,
        Literal(lit) => suffix_len.map(|len| lit.len() + len),
        Newline if is_flat => None,
        Newline => Some(0),
        Text(txt, _) => suffix_len.map(|len| txt.chars().count() as Width + len),
        Indent(_, x) => first_line_len(x, is_flat, suffix_len),
        Flat(x) => first_line_len(x, true, suffix_len),
        Child(_, x) => first_line_len(x, is_flat, suffix_len),
        Concat(x, y) => {
            let suffix_len = first_line_len(y, is_flat, suffix_len);
            first_line_len(x, is_flat, suffix_len)
        }
        Choice(x, y) => {
            let z = if is_flat { x } else { y };
            first_line_len(z, is_flat, suffix_len)
        }
    }
}

impl Layout {
    fn empty() -> Layout {
        Layout(vec![(0, String::new())])
    }

    fn text(s: &str) -> Layout {
        Layout(vec![(0, s.to_string())])
    }

    fn newline(indent: Width) -> Layout {
        Layout(vec![(0, String::new()), (indent, String::new())])
    }

    fn append(self, other: Layout) -> Layout {
        // Start with self.lines
        let mut lines = self.0;

        // Then the last line of `self` extended by the first line of `other`
        let mut other_lines = other.0.into_iter();
        let suffix = other_lines.next().unwrap().1;
        lines.last_mut().unwrap().1.push_str(&suffix);

        // Then the rest of the lines of `other`
        for line in other_lines {
            lines.push(line);
        }

        Layout(lines)
    }

    fn last_line_len(&self) -> Width {
        let last_line = self.0.last().unwrap();
        last_line.0 + last_line.1.chars().count() as Width
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, (spaces, line)) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{:spaces$}{}", "", line, spaces = *spaces as usize)?;
        }
        Ok(())
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
    let newline = Layout::newline(2);
    let hello_world = hello.append(newline).append(world);
    assert_eq!(format!("{}", hello_world), "Hello\n  world!");
}
