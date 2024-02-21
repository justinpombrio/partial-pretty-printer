use super::consolidated_notation::{
    ConsolidatedNotation, DelayedConsolidatedNotation, PrintingError,
};
use super::pretty_doc::PrettyDoc;
use crate::geometry::{str_width, Width};
use std::fmt;

const DEBUG_PRINT: bool = false;
const MAX_WIDTH: Width = 10_000;

/// A list of lines; each line has (indentation, contents)
///
/// **Invariant:** there's always at least one line
struct Layout(Vec<String>);

/// For testing!
///
/// Pretty print the document with the given width. This is meant only for testing.
/// It's slow: roughly exponential in the size of the doc.
pub fn oracular_pretty_print<'d, D: PrettyDoc<'d>>(doc: D, width: Width) -> String {
    let note = DelayedConsolidatedNotation::new(doc)
        .eval()
        .expect("Notation mismatch in oracle test (root)")
        .0;
    let layout = pp(Layout::empty(), note, 0, width).expect("Notation mismatch in oracle test");
    format!("{}", layout)
}

fn pp<'d, D: PrettyDoc<'d>>(
    prefix: Layout,
    note: ConsolidatedNotation<'d, D>,
    suffix_len: Width,
    width: Width,
) -> Result<Layout, PrintingError> {
    use ConsolidatedNotation::*;

    assert!(width < MAX_WIDTH);

    if DEBUG_PRINT {
        println!("==pp suffix_len:{:?} width:{}", suffix_len, width);
        println!("{}", prefix);
        println!("{}", note);
        println!("==");
    }

    match note {
        Empty => Ok(prefix),
        Textual(textual) => Ok(prefix.append(Layout::text(textual.str))),
        Newline(indent) => {
            let mut indent = &indent;
            let mut indent_strings = Vec::new();
            while let Some(indent_node) = indent {
                indent_strings.push(indent_node.segment.str.to_owned());
                indent = &indent_node.parent;
            }
            indent_strings.reverse();
            Ok(prefix.append(Layout::newline(indent_strings.join(""))))
        }
        Child(_, x) => pp(prefix, x.eval()?.0, suffix_len, width),
        Concat(x, y) => {
            let x = x.eval()?.0;
            let y = y.eval()?.0;
            let x_suffix_len = first_line_len(y.clone(), suffix_len)?.min(MAX_WIDTH);
            let y_prefix = pp(prefix, x, x_suffix_len, width)?;
            pp(y_prefix, y, suffix_len, width)
        }
        Choice(x, y) => {
            let x = x.eval()?.0;
            let last_len = prefix.last_line_len();
            let first_len = first_line_len(x.clone(), suffix_len)?;
            let fits = last_len + first_len <= width;
            if DEBUG_PRINT {
                println!(
                    "fits: {} + {:?} <= {} ? {}",
                    last_len, first_len, width, fits
                );
            }
            let z = if fits { x } else { y.eval()?.0 };
            pp(prefix, z, suffix_len, width)
        }
    }
}

/// Smallest possible first line length of `note`, given that its last line will have an additional
/// `suffix_len` columns after it. Assumes the rule that in (x | y), y's first line is no longer
/// than x's.
fn first_line_len<'d, D: PrettyDoc<'d>>(
    note: ConsolidatedNotation<'d, D>,
    suffix_len: Width,
) -> Result<Width, PrintingError> {
    use ConsolidatedNotation::*;

    match note {
        Empty => Ok(suffix_len),
        Textual(textual) => Ok(textual.width + suffix_len),
        Newline(_) => Ok(0),
        Child(_, x) => first_line_len(x.eval()?.0, suffix_len),
        Concat(x, y) => {
            let suffix_len = first_line_len(y.eval()?.0, suffix_len)?.min(MAX_WIDTH);
            first_line_len(x.eval()?.0, suffix_len)
        }
        Choice(_, y) => {
            // Wouldn't see a choice if we were flat, so use y.
            // Relies on the rule that in (x | y), y's first line is no longer than x's.
            first_line_len(y.eval()?.0, suffix_len)
        }
    }
}

impl Layout {
    fn empty() -> Layout {
        Layout(vec![String::new()])
    }

    fn text(s: &str) -> Layout {
        Layout(vec![s.to_string()])
    }

    fn newline(prefix: String) -> Layout {
        Layout(vec![String::new(), prefix])
    }

    fn append(self, other: Layout) -> Layout {
        // Start with self.lines
        let mut lines = self.0;

        // Then the last line of `self` extended by the first line of `other`
        let mut other_lines = other.0.into_iter();
        let suffix = other_lines.next().unwrap(); // relies on invariant
        lines.last_mut().unwrap().push_str(&suffix); // relies on invariant

        // Then the rest of the lines of `other`
        for line in other_lines {
            lines.push(line);
        }

        Layout(lines)
    }

    fn last_line_len(&self) -> Width {
        let last_line = self.0.last().unwrap(); // relies on invariant
        str_width(&last_line)
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, line) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

#[test]
fn test_layout() {
    let ab = Layout(vec![" a".to_owned(), " bb".to_owned()]);
    let cd = Layout(vec!["ccc".to_owned(), "  dddd".to_owned()]);
    let abcd = ab.append(cd);
    assert_eq!(format!("{}", abcd), " a\n bbccc\n  dddd");

    let hello = Layout::text("Hello");
    let world = Layout::text("world!");
    let newline = Layout::newline("  ".to_owned());
    let hello_world = hello.append(newline).append(world);
    assert_eq!(format!("{}", hello_world), "Hello\n  world!");
}
