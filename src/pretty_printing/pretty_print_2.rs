use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::infra::span;
use crate::style::{Shade, Style};
use std::fmt;
use std::iter::{self, Iterator};

/// Pretty print a document, focused at the node found by traversing `path` from the root.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to,
/// find a layout that fits withing that width.
///
/// Returns a pair of iterators:
///
/// - the first prints lines above the focused node going up
/// - the second prints lines from the first line of the focused node going down
///
/// It is expected that you will take only as many lines as you need from the iterators; doing so
/// will save computation time.
pub fn pretty_print<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
) -> (
    impl Iterator<Item = LineContents<'d>>,
    impl Iterator<Item = LineContents<'d>>,
) {
    span!("Pretty Print");

    let notation = NotationRef::new(doc);
    let seeker = Seeker::new(width, notation);
    seeker.seek(path)
}

/// Print the entirety of the document to a single string, ignoring styles and shading.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to,
/// find a layout that fits withing that width.
pub fn pretty_print_to_string<'d, D: PrettyDoc<'d>>(doc: D, width: Width) -> String {
    let (_, mut lines_iter) = pretty_print(doc, width, &[]);
    let mut string = lines_iter.next().unwrap().to_string();
    for line in lines_iter {
        string.push('\n');
        string.push_str(&line.to_string());
    }
    string
}

/// (indent, is_in_cursor, notation)
type Chunk<'d, D> = (Option<Width>, Shade, NotationRef<'d, D>);

/// A cursor within a list of chunks. Each chunk is a notation, plus indentation and shading
/// information. The cursor can move forward and back through the list of chunks. It simplifies
/// most chunks (Empty, Indent, Flat, and Concat) automatically, but must be asked to resolve
/// Choice and Child chunks.
struct Printer<'d, D: PrettyDoc<'d>> {
    /// Pretty printing width
    width: Width,
    /// Chunks to the left of the cursor, in order.
    prev: Vec<Chunk<'d, D>>,
    /// Chunks to the left of the cursor, in reverse order.
    next: Vec<Chunk<'d, D>>,
}

/// The contents of a single pretty printed line.
pub struct LineContents<'d> {
    /// The indentation of this line in spaces, and the shade of those spaces.
    pub spaces: (Width, Shade),
    /// A sequence of (string, style, shade) triples, to be displayed after `spaces`, in order from
    /// left to right, with no spacing in between.
    pub contents: Vec<(&'d str, Style, Shade)>,
}

impl<'d> ToString for LineContents<'d> {
    fn to_string(&self) -> String {
        span!("LineContents::to_string");

        let mut string = format!("{:spaces$}", "", spaces = self.spaces.0 as usize);
        for (text, _style, _shade) in &self.contents {
            string.push_str(text);
        }
        string
    }
}

impl<'d, D: PrettyDoc<'d>> Printer<'d, D> {
    fn new(width: Width, notation: NotationRef<'d, D>) -> Printer<'d, D> {
        span!("Printer::new");

        let fake_nl = notation.make_fake_start_of_doc_newline();
        Printer {
            width,
            prev: vec![(Some(0), Shade::background(), fake_nl)],
            next: vec![(Some(0), Shade::background(), notation)],
        }
    }

    /// Walk forward until finding a chunk for which `predicate` returns true. Walk just past that
    /// chunk (i.e., it will be on the left).
    fn seek_forward_until(&mut self, predicate: &mut dyn FnMut(NotationRef<'d, D>) -> bool) {
        use NotationCase::*;

        span!("seek_forward");

        while let Some((indent, shade, notation)) = self.next.pop() {
            match notation.case() {
                Literal(_) | Text(_, _) | Newline | Child(_, _) | Choice(_, _) => {
                    self.prev.push((indent, shade, notation));
                }
                Empty => (),
                Indent(j, note) => self.next.push((indent.map(|i| i + j), shade, note)),
                Flat(note) => self.next.push((None, shade, note)),
                Concat(left, right) => {
                    self.next.push((indent, shade, right));
                    self.next.push((indent, shade, left));
                }
            }
            if predicate(notation) {
                return;
            }
        }
    }

    /// Walk backward until finding a chunk for which `predicate` returns true. Walk just past that
    /// chunk (i.e., it will be on the right).
    fn seek_backward_until(&mut self, predicate: &mut dyn FnMut(NotationRef<'d, D>) -> bool) {
        use NotationCase::*;

        span!("seek_backward");

        while let Some((indent, shade, notation)) = self.prev.pop() {
            match notation.case() {
                Literal(_) | Text(_, _) | Newline | Child(_, _) | Choice(_, _) => {
                    self.next.push((indent, shade, notation));
                }
                Empty => (),
                Indent(j, note) => self.prev.push((indent.map(|i| i + j), shade, note)),
                Flat(note) => self.prev.push((None, shade, note)),
                Concat(left, right) => {
                    self.prev.push((indent, shade, left));
                    self.prev.push((indent, shade, right));
                }
            }
            if predicate(notation) {
                return;
            }
        }
    }

    /// Walk forward until the next Choice or Child node on this line. Resolve it, and leave it
    /// ahead. Return the pre-resolved chunk (if any).
    ///
    /// Must be called at the start of a line (i.e., just before a Newline)!
    fn resolve_next_child_or_choice(&mut self) -> Option<Chunk<'d, D>> {
        use NotationCase::*;

        span!("resolve_next");

        // Advance past the newline and remember its indentation level.
        let nl_chunk = self.next.pop().unwrap();
        assert!(matches!(nl_chunk.2.case(), Newline));
        self.prev.push(nl_chunk);
        let mut prefix_len = nl_chunk.0.unwrap();

        while let Some(chunk) = self.next.pop() {
            let (indent, shade, notation) = chunk;
            match notation.case() {
                Empty => (),
                Literal(lit) => {
                    prefix_len += lit.len();
                    self.prev.push(chunk);
                }
                Text(text, _) => {
                    prefix_len += text.chars().count() as Width;
                    self.prev.push(chunk);
                }
                Newline => {
                    self.next.push(chunk);
                    return None;
                }
                Indent(j, note) => self.next.push((indent.map(|i| i + j), shade, note)),
                Flat(note) => self.next.push((None, shade, note)),
                Concat(left, right) => {
                    self.next.push((indent, shade, right));
                    self.next.push((indent, shade, left));
                }
                Child(_, child_note) => {
                    self.next.push((indent, shade, child_note));
                    return Some(chunk);
                }
                Choice(opt1, opt2) => {
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, &self.next);
                    self.next.push((indent, shade, choice));
                    return Some(chunk);
                }
            }
        }
        None
    }

    /// Print the next line. Must be called at the start of the line, i.e. just before a Newline.
    /// Resolves any Choice or Child nodes as required.
    fn print_line(&mut self) -> Option<LineContents<'d>> {
        use NotationCase::*;

        span!("print_line");

        // Consume the newline and remember its indentation level.
        let (spaces, spaces_shade) = match self.next.pop() {
            None => return None,
            Some((indent, shade, note)) => {
                assert!(matches!(note.case(), NotationCase::Newline));
                (indent.unwrap(), shade)
            }
        };

        let mut contents = vec![];
        let mut prefix_len = spaces;
        while let Some((indent, shade, notation)) = self.next.pop() {
            match notation.case() {
                Empty => (),
                Literal(lit) => {
                    contents.push((lit.str(), lit.style(), shade));
                    prefix_len += lit.len();
                }
                Text(text, style) => {
                    contents.push((text, style, shade));
                    prefix_len += text.chars().count() as Width;
                }
                Newline => {
                    self.next.push((indent, shade, notation));
                    break;
                }
                Indent(j, note) => self.next.push((indent.map(|i| i + j), shade, note)),
                Flat(note) => self.next.push((None, shade, note)),
                Concat(left, right) => {
                    self.next.push((indent, shade, right));
                    self.next.push((indent, shade, left));
                }
                Choice(opt1, opt2) => {
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, &self.next);
                    self.next.push((indent, shade, choice));
                }
                Child(_, child_note) => self.next.push((indent, shade, child_note)),
            }
        }
        Some(LineContents {
            spaces: (spaces, spaces_shade),
            contents,
        })
    }

    fn next_doc_id(&self) -> D::Id {
        self.next.last().unwrap().2.doc_id()
    }

    fn highlight(&mut self, shade: Shade) {
        self.next.last_mut().unwrap().1 = shade;
    }

    fn at_end(&self) -> bool {
        self.next.is_empty()
    }

    fn is_empty(&self) -> bool {
        self.prev.is_empty() && self.next.is_empty()
    }
}

impl<'d, D: PrettyDoc<'d>> fmt::Display for Printer<'d, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (_, _, notation) in &self.prev {
            write!(f, " {} ", notation)?;
        }
        write!(f, " / ")?;
        for (_, _, notation) in self.next.iter().rev() {
            write!(f, " {} ", notation)?;
        }
        Ok(())
    }
}

/// Determine which of the two options of the choice to select. Pick the first option if it fits,
/// or if the second option is invalid.
fn choose<'d, D: PrettyDoc<'d>>(
    width: Width,
    indent: Option<Width>,
    prefix_len: Width,
    opt1: NotationRef<'d, D>,
    opt2: NotationRef<'d, D>,
    suffix: &[Chunk<'d, D>],
) -> NotationRef<'d, D> {
    span!("print_line");

    let flat = indent.is_none();
    let chunks = suffix
        .iter()
        .map(|(i, _, n)| (i.is_none(), *n))
        .chain(iter::once((flat, opt1)))
        .collect();
    if fits(width.saturating_sub(prefix_len), chunks) && is_valid_entry_point(flat, opt1)
        || !is_valid_entry_point(flat, opt2)
    {
        opt1
    } else {
        opt2
    }
}

/// Determine whether the first line of the chunks fits within the `remaining` space.
fn fits<'d, D: PrettyDoc<'d>>(
    mut remaining: Width,
    mut chunks: Vec<(bool, NotationRef<'d, D>)>,
) -> bool {
    use NotationCase::*;

    span!("fits");

    while let Some((flat, notation)) = chunks.pop() {
        match notation.case() {
            Empty => (),
            Literal(lit) => {
                let lit_len = lit.len();
                if lit_len <= remaining {
                    remaining -= lit_len;
                } else {
                    return false;
                }
            }
            Text(text, _) => {
                let text_len = text.chars().count() as Width;
                if text_len <= remaining {
                    remaining -= text_len;
                } else {
                    return false;
                }
            }
            // TODO: correct?
            Newline => return !flat,
            Flat(note) => chunks.push((true, note)),
            Indent(_, note) => chunks.push((flat, note)),
            Child(_, note) => chunks.push((flat, note)),
            Concat(note1, note2) => {
                chunks.push((flat, note2));
                chunks.push((flat, note1));
            }
            Choice(opt1, opt2) => {
                // opt2 must always be strictly smaller!
                // TODO: As an optimization, pre-compute whether opt2 has an unconditional newline
                if is_valid_entry_point(flat, opt2) {
                    chunks.push((flat, opt2));
                } else {
                    chunks.push((flat, opt1));
                }
            }
        }
    }
    true
}

fn is_valid_entry_point<'d, D: PrettyDoc<'d>>(flat: bool, notation: NotationRef<'d, D>) -> bool {
    // There are a _lot_ of calls to this, and the profiler, though extremely fast, could slow it
    // down.
    span!("is_valid");
    is_valid(flat, notation)
}

fn is_valid<'d, D: PrettyDoc<'d>>(flat: bool, notation: NotationRef<'d, D>) -> bool {
    use NotationCase::*;

    match notation.case() {
        Empty | Literal(_) | Text(_, _) => true,
        Newline => !flat,
        Flat(note) => is_valid(true, note),
        Indent(_, note) => is_valid(flat, note),
        // TODO: As an optimization, pre-compute whether opt2 has an unconditional newline
        Choice(opt1, opt2) => is_valid(flat, opt1) || is_valid(flat, opt2),
        Concat(note1, note2) => is_valid(flat, note1) && is_valid(flat, note2),
        Child(_, child_note) => !flat || is_valid(flat, child_note),
    }
}

struct Seeker<'d, D: PrettyDoc<'d>> {
    printer: Printer<'d, D>,
}

struct DownwardPrinter<'d, D: PrettyDoc<'d>> {
    printer: Printer<'d, D>,
}

struct UpwardPrinter<'d, D: PrettyDoc<'d>> {
    printer: Printer<'d, D>,
}

impl<'d, D: PrettyDoc<'d>> Seeker<'d, D> {
    fn new(width: Width, notation: NotationRef<'d, D>) -> Seeker<'d, D> {
        span!("Seeker::new");

        Seeker {
            printer: Printer::new(width, notation),
        }
    }

    fn seek(mut self, path: &[usize]) -> (UpwardPrinter<'d, D>, DownwardPrinter<'d, D>) {
        use NotationCase::*;

        span!("seek");

        // Seek to the descendant given by `path`. Highlight the descendency chain as we go.
        let mut path = path.iter();
        self.printer.highlight(Shade(path.len() as u8));
        while let Some(child_index) = path.next() {
            self.seek_child(*child_index);
            self.printer.highlight(Shade(path.len() as u8));
        }

        self.printer
            .seek_backward_until(&mut |note| matches!(note.case(), Newline));

        let upward_printer = UpwardPrinter {
            printer: Printer {
                width: self.printer.width,
                prev: self.printer.prev,
                next: Vec::new(),
            },
        };
        let downward_printer = DownwardPrinter {
            printer: Printer {
                width: self.printer.width,
                prev: Vec::new(),
                next: self.printer.next,
            },
        };
        (upward_printer, downward_printer)
    }

    fn seek_child(&mut self, child_index: usize) {
        use NotationCase::*;

        span!("seek child");

        let parent_doc_id = self.printer.next_doc_id();
        loop {
            // 1. Expand forward to the nearest `Choice` or `Child` belonging to `parent_doc`.
            //    (NOTE: more precise would be looking for Child(child_index) or a Choice
            //     containing it, but you can't tell right now what children a choice might
            //     contain.)
            self.printer.seek_forward_until(&mut |note| {
                if note.doc_id() != parent_doc_id {
                    return false;
                }
                match note.case() {
                    Choice(_, _) => true,
                    Child(i, _) => i == child_index,
                    _ => false,
                }
            });

            // 2. Walk backward to the nearest Newline (or beginning of the doc).
            self.printer
                .seek_backward_until(&mut |note| matches!(note.case(), Newline));

            // 3. Walk forward to the nearest Child or Choice, and resolve it.
            //    - If you hit `Child(i)` belonging to `parent_doc`, success.
            //    - If you hit end of doc, panic (every child must be present).
            //    - Otherwise, go back to step 1.
            let chunk = self.printer.resolve_next_child_or_choice();
            let child_found = match chunk {
                None => false,
                Some((_, _, note)) => match note.case() {
                    Child(i, _) => note.doc_id() == parent_doc_id && i == child_index,
                    _ => false,
                },
            };
            if self.printer.at_end() {
                panic!("Missing child ({})", child_index);
            }
            if child_found {
                return;
            }
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Iterator for DownwardPrinter<'d, D> {
    type Item = LineContents<'d>;

    fn next(&mut self) -> Option<LineContents<'d>> {
        span!("print_line_down");

        self.printer.print_line()
    }
}

impl<'d, D: PrettyDoc<'d>> Iterator for UpwardPrinter<'d, D> {
    type Item = LineContents<'d>;

    fn next(&mut self) -> Option<LineContents<'d>> {
        use NotationCase::*;

        span!("print_line_up");

        if self.printer.is_empty() {
            return None;
        }

        // Fully resolve the last line.
        loop {
            self.printer.seek_forward_until(&mut |_| false);
            self.printer
                .seek_backward_until(&mut |note| matches!(note.case(), Newline));
            self.printer.resolve_next_child_or_choice();
            if self.printer.at_end() {
                break;
            }
        }

        // Return to the start of the (now fully resolved) last line, and print it.
        self.printer
            .seek_backward_until(&mut |note| matches!(note.case(), Newline));
        self.printer.print_line()
    }
}
