use super::consolidated_notation::{
    ConsolidatedNotation, DelayedConsolidatedNotation, PrintingError,
};
use super::pretty_doc::PrettyDoc;
use crate::geometry::{str_width, Width};
use std::fmt;

const DEBUG_PRINT: bool = false;
const MAX_WIDTH: Width = 10_000;

// TODO: docs
/// A list of lines; each line has (indentation, contents)
///
/// **Invariant:** there's always at least one line
struct Layout {
    indent_stack: Vec<String>,
    lines: Vec<String>,
}

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
        Textual(textual) => Ok(prefix.append_text(textual.str)),
        Newline => Ok(prefix.append_newline()),
        Child(_, x) => pp(prefix, x.eval()?.0, suffix_len, width),
        Concat(x, y) => {
            let x = x.eval()?.0;
            let y = y.eval()?.0;
            let x_suffix_len = first_line_len(y, suffix_len)?.min(MAX_WIDTH);
            let y_prefix = pp(prefix, x, x_suffix_len, width)?;
            pp(y_prefix, y, suffix_len, width)
        }
        Choice(x, y) => {
            let x = x.eval()?.0;
            let last_len = prefix.last_line_len();
            let first_len = first_line_len(x, suffix_len)?;
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
        PushIndent(textual) => Ok(prefix.push_indent(textual.str)),
        PopIndent(_) => Ok(prefix.pop_indent()),
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
        Empty | PushIndent(_) | PopIndent(_) => Ok(suffix_len),
        Textual(textual) => Ok(textual.width + suffix_len),
        Newline => Ok(0),
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
        Layout {
            lines: vec![String::new()],
            indent_stack: Vec::new(),
        }
    }

    fn append_text(mut self, text: &str) -> Layout {
        self.lines.last_mut().unwrap().push_str(&text); // relies on invariant
        self
    }

    fn append_newline(mut self) -> Layout {
        let new_line = self.indent_stack.join("");
        self.lines.push(new_line);
        self
    }

    fn push_indent(mut self, indent: &str) -> Layout {
        self.indent_stack.push(indent.to_owned());
        self
    }

    fn pop_indent(mut self) -> Layout {
        self.indent_stack.pop();
        self
    }

    fn last_line_len(&self) -> Width {
        let last_line = self.lines.last().unwrap(); // relies on invariant
        str_width(last_line)
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

/* TODO
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
*/
