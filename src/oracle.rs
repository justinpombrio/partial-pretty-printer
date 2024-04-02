use crate::{
    consolidated_notation::{ConsolidatedNotation, DelayedConsolidatedNotation, PrintingError},
    geometry::str_width,
    PrettyDoc, Width,
};
use std::fmt;

const DEBUG_PRINT: bool = false;
const MAX_WIDTH: Width = 10_000;

/// A list of lines.
///
/// **Invariant:** there's always at least one line.
struct Layout {
    lines: Vec<String>,
    /// Whether there's an `EndOfLine` at the end of the last line of `lines`.
    ends_with_eol: bool,
}

/// Print the document using a simple but slow algorithm; the time is roughly exponential in the
/// size of the doc. This function should always produce the same output as
/// [`pretty_print_to_string`](crate::pretty_print_to_string), so it can be used for automated
/// testing of the more efficient but complex partial-pretty-printing algorithm.
pub fn oracular_pretty_print<'d, D: PrettyDoc<'d>>(doc: D, width: Width) -> String {
    let note = DelayedConsolidatedNotation::new(doc)
        .unwrap()
        .eval()
        .expect("Notation mismatch in oracle test (root)");
    let layout =
        pp(Layout::empty(), note, Some(0), width).expect("Notation mismatch in oracle test");
    format!("{}", layout)
}

fn pp<'d, D: PrettyDoc<'d>>(
    // Everything that's been printed so far
    mut prefix: Layout,
    // The next notation to print
    note: ConsolidatedNotation<'d, D>,
    // The number of columns known to follow `note`, or `None` if it's followed by `EndOfLine +
    // Text`.
    suffix_len: Option<Width>,
    // The printing width
    width: Width,
) -> Result<Layout, PrintingError<D::Error>> {
    use ConsolidatedNotation::*;

    assert!(width < MAX_WIDTH);

    if DEBUG_PRINT {
        println!("==pp suffix_len:{:?} width:{}", suffix_len, width);
        println!("{}", prefix);
        println!("{}", note);
        println!("==");
    }

    match note {
        Empty | FocusMark => Ok(prefix),
        Textual(textual) => Ok(prefix.append_text(textual.str)),
        EndOfLine => {
            prefix.ends_with_eol = true;
            Ok(prefix)
        }
        Newline(indentation) => {
            let mut remaining_indentation = &indentation;
            let mut indent_strings = Vec::new();
            while let Some(indent_node) = remaining_indentation {
                indent_strings.push(indent_node.segment.str.to_owned());
                remaining_indentation = &indent_node.parent;
            }
            indent_strings.reverse();
            Ok(prefix.append_newline(indent_strings.join("")))
        }
        Child(_, x) => pp(prefix, x.eval()?, suffix_len, width),
        Concat(x, y) => {
            let x = x.eval()?;
            let y = y.eval()?;
            let x_suffix_len = first_line_len(y.clone(), suffix_len)?.map(|w| w.min(MAX_WIDTH));
            let y_prefix = pp(prefix, x, x_suffix_len, width)?;
            pp(y_prefix, y, suffix_len, width)
        }
        Choice(x, y) => {
            let x = x.eval()?;
            let last_len = prefix.last_line_len();
            let fits = match first_line_len(x.clone(), suffix_len)? {
                None => false,
                Some(first_len) => {
                    if prefix.ends_with_eol && first_len > 0 {
                        false
                    } else {
                        last_len + first_len <= width
                    }
                }
            };
            if DEBUG_PRINT {
                println!("fits: {:?} + ? <= {} ? {}", last_len, width, fits);
            }
            let z = if fits { x } else { y.eval()? };
            pp(prefix, z, suffix_len, width)
        }
    }
}

/// Compute the smallest possible first line length of `note`, given that its last line will have an
/// additional `suffix_len` columns after it (where `None` means the suffix contains `EndOfLine +
/// Textual`). Assumes the rule that "in (x | y), if x fits then y fits", and so always chooses y.
/// Returns `None` if the first line contains `EndOfLine + Textual`.
fn first_line_len<'d, D: PrettyDoc<'d>>(
    note: ConsolidatedNotation<'d, D>,
    suffix_len: Option<Width>,
) -> Result<Option<Width>, PrintingError<D::Error>> {
    use ConsolidatedNotation::*;

    match note {
        Empty | FocusMark => Ok(suffix_len),
        Textual(textual) => Ok(suffix_len.map(|w| textual.width + w)),
        EndOfLine => match suffix_len {
            None => Ok(None),
            Some(0) => Ok(Some(0)), // Followed by a newline, good
            Some(_) => Ok(None),    // Followed by text, bad
        },
        Newline(_) => Ok(Some(0)),
        Child(_, x) => first_line_len(x.eval()?, suffix_len),
        Concat(x, y) => {
            let suffix_len = first_line_len(y.eval()?, suffix_len)?.map(|w| w.min(MAX_WIDTH));
            first_line_len(x.eval()?, suffix_len)
        }
        Choice(_, y) => {
            // Wouldn't see a choice if we were flat, so use y.
            // Relies on the rule that in (x | y), y's first line is no longer than x's.
            first_line_len(y.eval()?, suffix_len)
        }
    }
}

impl Layout {
    fn empty() -> Layout {
        Layout {
            lines: vec![String::new()],
            ends_with_eol: false,
        }
    }

    fn append_newline(mut self, indentation: String) -> Layout {
        self.lines.push(indentation);
        self.ends_with_eol = false;
        self
    }

    fn append_text(mut self, text: &str) -> Layout {
        if self.ends_with_eol {
            panic!("Oracle: encountered EOL + text");
        }
        self.lines.last_mut().unwrap().push_str(text); // relies on invariant
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
