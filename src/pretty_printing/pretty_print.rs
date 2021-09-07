use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::infra::span;
use crate::style::{Shade, Style};
use std::iter::Iterator;

// TODO: "hl" -> "shade"

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
        for (text, _style, _hl) in &self.contents {
            string.push_str(text);
        }
        string
    }
}

////////////////////////////////////////

/// Can seek to an arbitrary position within the document, while resolving as few choices as
/// possible.
struct Seeker<'d, D: PrettyDoc<'d>> {
    width: Width,
    prev: Vec<Chunk<'d, D>>,
    next: Vec<Chunk<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> Seeker<'d, D> {
    fn new(width: Width, notation: NotationRef<'d, D>) -> Seeker<'d, D> {
        let fake_nl = notation.make_fake_start_of_doc_newline();
        Seeker {
            width,
            prev: vec![(Some(0), Shade::background(), fake_nl)],
            next: vec![(Some(0), Shade::background(), notation)],
        }
    }

    fn seek(mut self, path: &[usize]) -> (UpwardPrinter<'d, D>, DownwardPrinter<'d, D>) {
        use NotationCase::*;

        span!("seek");

        // Seek to the descendant given by `path`. Highlight the descendency chain as we go.
        let mut path = path.iter();
        self.highlight(path.len());
        while let Some(child_index) = path.next() {
            self.seek_child(*child_index);
            self.highlight(path.len());
        }

        // Walk backward to the nearest Newline.
        while let Some((indent, hl, notation)) = self.prev.pop() {
            match notation.case() {
                Empty | Indent(_, _) | Flat(_) | Concat(_, _) | Choice(_, _) | Child(_, _) => {
                    unreachable!()
                }
                Literal(_) => self.next.push((indent, hl, notation)),
                Text(_, _) => self.next.push((indent, hl, notation)),
                Newline => {
                    self.next.push((indent, hl, notation));
                    break;
                }
            }
        }

        let upward_printer = UpwardPrinter {
            width: self.width,
            prev: self.prev,
            next: vec![],
        };
        let downward_printer = DownwardPrinter {
            width: self.width,
            next: self.next,
        };
        (upward_printer, downward_printer)
    }

    fn seek_child(&mut self, child_index: usize) {
        use NotationCase::*;

        span!("seek_child");

        let parent_doc_id = self.next.last().unwrap().2.doc_id();
        'find_child: loop {
            // 1. Expand forward to the nearest `Choice` or `Child` belonging to `parent_doc`.
            //    (NOTE: more precise would be looking for Child(child_index) or a Choice
            //     containing it, but you can't tell right now what children a choice might
            //     contain.)
            while let Some((indent, hl, notation)) = self.next.pop() {
                match notation.case() {
                    Empty => (),
                    Literal(_) => self.prev.push((indent, hl, notation)),
                    Newline => self.prev.push((indent, hl, notation)),
                    Text(_, _) => self.prev.push((indent, hl, notation)),
                    Indent(j, note) => self.next.push((indent.map(|i| i + j), hl, note)),
                    Flat(note) => self.next.push((None, hl, note)),
                    Concat(left, right) => {
                        self.next.push((indent, hl, right));
                        self.next.push((indent, hl, left));
                    }
                    Choice(_, _) if notation.doc_id() == parent_doc_id => {
                        self.next.push((indent, hl, notation));
                        break;
                    }
                    Choice(_, _) => self.prev.push((indent, hl, notation)),
                    Child(i, _) if notation.doc_id() == parent_doc_id && i == child_index => {
                        self.next.push((indent, hl, notation));
                        break;
                    }
                    Child(_, _) => self.prev.push((indent, hl, notation)),
                }
            }

            // 2. Walk backward to the nearest Newline (or beginning of the doc).
            let mut prefix_len = 0;
            while let Some((indent, hl, notation)) = self.prev.pop() {
                match notation.case() {
                    Empty | Indent(_, _) | Flat(_) | Concat(_, _) => unreachable!(),
                    Literal(_) | Text(_, _) | Choice(_, _) | Child(_, _) => {
                        self.next.push((indent, hl, notation))
                    }
                    Newline => {
                        prefix_len = indent.unwrap();
                        self.prev.push((indent, hl, notation));
                        break;
                    }
                }
            }

            // 3. Walk forward to the nearest Child or Choice, and resolve it. Go back to 1.
            //    If you hit `Child(i)` belonging to `parent_doc`, success.
            //    If you hit end of doc, panic (every child must be present).
            while let Some((indent, hl, notation)) = self.next.pop() {
                match notation.case() {
                    Empty | Indent(_, _) | Flat(_) | Concat(_, _) | Newline => unreachable!(),
                    Literal(lit) => {
                        prefix_len += lit.len();
                        self.prev.push((indent, hl, notation))
                    }
                    Text(text, _style) => {
                        prefix_len += text.chars().count() as Width;
                        self.prev.push((indent, hl, notation));
                    }
                    Child(i, child) if notation.doc_id() == parent_doc_id && i == child_index => {
                        // Found!
                        self.next.push((indent, hl, child));
                        return;
                    }
                    Child(_, child) => {
                        self.next.push((indent, hl, child));
                        continue 'find_child;
                    }
                    Choice(opt1, opt2) => {
                        let choice = choose(self.width, indent, prefix_len, opt1, opt2, &self.next);
                        self.next.push((indent, hl, choice));
                        continue 'find_child;
                    }
                }
            }

            panic!("Missing child ({})", child_index);
        }
    }

    fn highlight(&mut self, shade: usize) {
        // TODO: longer paths?
        self.next.last_mut().unwrap().1 = Shade(shade as u8);
    }

    #[allow(unused)]
    fn display(&self) {
        for (_, _, notation) in &self.prev {
            print!("{} ", notation);
        }
        print!(" / ");
        for (_, _, notation) in self.next.iter().rev() {
            print!("{} ", notation);
        }
        println!();
    }

    #[allow(unused)]
    fn display_shades(&self) {
        for (_, shade, _) in &self.prev {
            print!("{} ", shade.0);
        }
        print!(" / ");
        for (_, shade, _) in self.next.iter().rev() {
            print!("{} ", shade.0);
        }
        println!();
    }
}

/// Constructed at an arbitrary position within the document. Prints lines from there one at a
/// time, going down.
struct DownwardPrinter<'d, D: PrettyDoc<'d>> {
    width: Width,
    next: Vec<Chunk<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> DownwardPrinter<'d, D> {
    fn print_first_line(&mut self) -> Option<LineContents<'d>> {
        use NotationCase::*;

        span!("print_first_line");

        // We should be at the start of a line (in which case we look a the Newline's indentation
        // level to see how many spaces are at the start of this line), or at the very end of the
        // document (in which case our iteration is done).
        let (spaces, spaces_hl) = if let Some((indent, hl, notation)) = self.next.pop() {
            assert!(matches!(notation.case(), Newline));
            (indent.unwrap(), hl)
        } else {
            return None;
        };

        let mut contents = vec![];
        let mut prefix_len = spaces;
        while let Some((indent, hl, notation)) = self.next.pop() {
            match notation.case() {
                Empty => (),
                Literal(lit) => {
                    contents.push((lit.str(), lit.style(), hl));
                    prefix_len += lit.len();
                }
                Text(text, style) => {
                    contents.push((text, style, hl));
                    prefix_len += text_len(text);
                }
                Newline => {
                    self.next.push((indent, hl, notation));
                    return Some(LineContents {
                        spaces: (spaces, spaces_hl),
                        contents,
                    });
                }
                Indent(j, note) => self.next.push((indent.map(|i| i + j), hl, note)),
                Flat(note) => self.next.push((None, hl, note)),
                Concat(left, right) => {
                    self.next.push((indent, hl, right));
                    self.next.push((indent, hl, left));
                }
                Choice(opt1, opt2) => {
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, &self.next);
                    self.next.push((indent, hl, choice));
                }
                Child(_, child_note) => self.next.push((indent, hl, child_note)),
            }
        }

        Some(LineContents {
            spaces: (spaces, spaces_hl),
            contents,
        })
    }

    #[allow(unused)]
    fn display(&self) {
        for (_, _, notation) in self.next.iter().rev() {
            print!("{} ", notation);
        }
        println!();
    }
}

/// Constructed at an arbitrary position within the document. Prints lines from there one at a
/// time, going up.
struct UpwardPrinter<'d, D: PrettyDoc<'d>> {
    width: Width,
    prev: Vec<Chunk<'d, D>>,
    // INVARIANT: only ever contains `Literal`, `Text`, and `Choice` notations.
    next: Vec<Chunk<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> UpwardPrinter<'d, D> {
    fn print_last_line(&mut self) -> Option<LineContents<'d>> {
        use NotationCase::*;

        span!("print_last_line");

        // 1. Go to the start of the "last line", and remember its indentation. However, the "last
        //    line" might not be fully expanded, and could contain hidden newlines in it.
        self.seek_start_of_line();
        let mut prefix_len = if let Some((indent, hl, notation)) = self.next.pop() {
            self.prev.push((indent, hl, notation));
            indent.unwrap()
        } else {
            return None;
        };

        // 2. Start expanding the "last line". If we encounter a choice, resolve it, but then seek
        //    back to the "start of the last line" again, as where that is might have changed if
        //    the choice contained a newline.
        while let Some((indent, hl, notation)) = self.next.pop() {
            match notation.case() {
                Literal(lit) => {
                    prefix_len += lit.len();
                    self.prev.push((indent, hl, notation));
                }
                Text(text, _style) => {
                    prefix_len += text_len(text);
                    self.prev.push((indent, hl, notation));
                }
                Choice(opt1, opt2) => {
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, &self.next);
                    self.prev.push((indent, hl, choice));

                    // Reset everything. This is equivalent to a recursive call.
                    self.seek_end();
                    self.seek_start_of_line();
                    prefix_len = if let Some((indent, _hl, notation)) = self.next.pop() {
                        self.prev.push((indent, hl, notation));
                        indent.unwrap()
                    } else {
                        return None;
                    };
                }
                Empty | Newline | Indent(_, _) | Flat(_) | Concat(_, _) | Child(_, _) => {
                    unreachable!()
                }
            }
        }

        self.seek_start_of_line();
        let (indent, hl, _notation) = self.next.pop().unwrap();
        let (spaces, spaces_hl) = (indent.unwrap(), hl);

        let mut contents = vec![];
        while let Some((_indent, hl, notation)) = self.next.pop() {
            match notation.case() {
                NotationCase::Literal(lit) => contents.push((lit.str(), lit.style(), hl)),
                NotationCase::Text(text, style) => contents.push((text, style, hl)),
                _ => panic!("display_line: expected only literals and text"),
            }
        }
        Some(LineContents {
            spaces: (spaces, spaces_hl),
            contents,
        })
    }

    fn seek_end(&mut self) {
        span!("seek_end");

        while let Some((indent, hl, notation)) = self.next.pop() {
            self.prev.push((indent, hl, notation));
        }
    }

    /// Move the "printing cursor" to just before the previous newline. (Or do nothing if there is
    /// no such newline.)
    // Maintains the invariant that `next` only ever contains `Literal` and `Choice` notations.
    fn seek_start_of_line(&mut self) {
        use NotationCase::*;

        span!("seek_start_of_line");

        while let Some((indent, hl, notation)) = self.prev.pop() {
            match notation.case() {
                Empty => (),
                Text(_, _) => self.next.push((indent, hl, notation)),
                Literal(_) => self.next.push((indent, hl, notation)),
                Newline => {
                    self.next.push((indent, hl, notation));
                    return;
                }
                Indent(j, note) => self.prev.push((indent.map(|i| i + j), hl, note)),
                Flat(note) => self.prev.push((None, hl, note)),
                Concat(left, right) => {
                    self.prev.push((indent, hl, left));
                    self.prev.push((indent, hl, right));
                }
                Choice(_, _) => self.next.push((indent, hl, notation)),
                Child(_, note) => self.prev.push((indent, hl, note)),
            }
        }
    }

    #[allow(unused)]
    fn display(&self) {
        for (_, _, notation) in &self.prev {
            print!("{} ", notation);
        }
        print!(" / ");
        for (_, _, notation) in self.next.iter().rev() {
            print!("{} ", notation);
        }
        println!();
    }
}

/////////////////////////////////////////////////////////////////////////////////////

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
    span!("choose");

    use std::iter;
    let flat = indent.is_none();
    let chunks = suffix
        .iter()
        .map(|(i, _, n)| (i.is_none(), *n))
        .chain(iter::once((flat, opt1)))
        .collect();
    if fits(width.saturating_sub(prefix_len), chunks) && is_valid(flat, opt1)
        || !is_valid(flat, opt2)
    {
        opt1
    } else {
        opt2
    }
}

/// Determine whether the first line of the chunks fits within the `remaining` space.
fn fits<'d, D: PrettyDoc<'d>>(width: Width, chunks: Vec<(bool, NotationRef<'d, D>)>) -> bool {
    use NotationCase::*;

    span!("fits");

    let mut remaining = width;
    let mut chunks = chunks;

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
    // There are a _lot_ of calls to `is_valid`, and the profiler, though extremely fast, could
    // slow it down. So only profile the initial call.
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
        // TODO: As an optimization, pre-compute whether opt2 has an unconditional newline.
        Choice(opt1, opt2) => is_valid(flat, opt1) || is_valid(flat, opt2),
        Concat(note1, note2) => is_valid(flat, note1) && is_valid(flat, note2),
        Child(_, child_note) => !flat || is_valid(flat, child_note),
    }
}

fn text_len(text: &str) -> Width {
    text.chars().count() as Width
}

impl<'d, D: PrettyDoc<'d>> Iterator for DownwardPrinter<'d, D> {
    type Item = LineContents<'d>;

    fn next(&mut self) -> Option<LineContents<'d>> {
        self.print_first_line()
    }
}

impl<'d, D: PrettyDoc<'d>> Iterator for UpwardPrinter<'d, D> {
    type Item = LineContents<'d>;

    fn next(&mut self) -> Option<LineContents<'d>> {
        self.print_last_line()
    }
}
