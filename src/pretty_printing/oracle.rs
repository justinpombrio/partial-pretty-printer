use super::notation_walker::{DelayedNotationWalker, NotationWalker};
use super::pretty_doc::PrettyDoc;
use crate::geometry::{str_width, Width};
use std::fmt;

const DEBUG_PRINT: bool = false;

/// A list of lines; each line has (indentation, contents)
///
/// **Invariant:** there's always at least one line
struct Layout(Vec<(Width, String)>);

/// For testing!
///
/// Pretty print the document with the given width. This is meant only for testing.
/// It's slow: roughly exponential in the size of the doc.
pub fn oracular_pretty_print<'d, D: PrettyDoc<'d>>(doc: D, width: Width) -> String {
    let walker = DelayedNotationWalker::new(doc);
    let layout = pp(Layout::empty(), walker, 0, width);
    format!("{}", layout)
}

fn pp<'d, D: PrettyDoc<'d>>(
    prefix: Layout,
    walker: DelayedNotationWalker<'d, D>,
    suffix_len: Width,
    width: Width,
) -> Layout {
    use NotationWalker::*;

    if DEBUG_PRINT {
        println!("==pp suffix_len:{:?} width:{}", suffix_len, width);
        println!("{}", prefix);
        println!("{}", walker);
        println!("==");
    }

    match walker.force() {
        Empty => prefix,
        Literal(lit) => prefix.append(Layout::text(lit.str())),
        Newline(indent) => prefix.append(Layout::newline(indent)),
        Text(txt, _) => prefix.append(Layout::text(txt)),
        Child(_, x) => pp(prefix, x, suffix_len, width),
        Concat(x, y) => {
            let x_suffix_len = first_line_len(y, suffix_len);
            let y_prefix = pp(prefix, x, x_suffix_len, width);
            pp(y_prefix, y, suffix_len, width)
        }
        Choice(x, y) => {
            let last_len = prefix.last_line_len();
            let first_len = first_line_len(x, suffix_len);
            let fits = last_len + first_len <= width;
            if DEBUG_PRINT {
                println!(
                    "fits: {} + {:?} <= {} ? {}",
                    last_len, first_len, width, fits
                );
            }
            let z = if fits { x } else { y };
            pp(prefix, z, suffix_len, width)
        }
    }
}

fn first_line_len<'d, D: PrettyDoc<'d>>(
    walker: DelayedNotationWalker<'d, D>,
    suffix_len: Width,
) -> Width {
    use NotationWalker::*;

    match walker.force() {
        Empty => suffix_len,
        Literal(lit) => lit.width() + suffix_len,
        Newline(_) => 0,
        Text(txt, _) => str_width(txt) + suffix_len,
        Child(_, x) => first_line_len(x, suffix_len),
        Concat(x, y) => {
            let suffix_len = first_line_len(y, suffix_len);
            first_line_len(x, suffix_len)
        }
        Choice(_, y) => {
            // Wouldn't see a choice if we were flat, so use y
            first_line_len(y, suffix_len)
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
        let first_line = (0, String::new());
        let second_line = (indent, String::new());
        Layout(vec![first_line, second_line])
    }

    fn append(self, other: Layout) -> Layout {
        // Start with self.lines
        let mut lines = self.0;

        // Then the last line of `self` extended by the first line of `other`
        let mut other_lines = other.0.into_iter();
        let suffix = other_lines.next().unwrap().1; // relies on invariant
        lines.last_mut().unwrap().1.push_str(&suffix); // relies on invariant

        // Then the rest of the lines of `other`
        for line in other_lines {
            lines.push(line);
        }

        Layout(lines)
    }

    fn last_line_len(&self) -> Width {
        let last_line = self.0.last().unwrap(); // relies on invariant
        last_line.0 + str_width(&last_line.1)
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
