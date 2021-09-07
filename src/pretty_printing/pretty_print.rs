use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::infra::span;
use crate::style::{Shade, Style};
use std::iter::Iterator;

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

#[derive(Clone, Debug)]
pub struct FirstLineLen {
    pub len: Width,
    pub has_newline: bool,
}

struct Seeker<'d, D: PrettyDoc<'d>> {
    width: Width,
    prev: Vec<Chunk<'d, D>>,
    next: Vec<Chunk<'d, D>>,
}

struct DownwardPrinter<'d, D: PrettyDoc<'d>> {
    width: Width,
    next: Vec<Chunk<'d, D>>,
    spaces: (Width, Shade),
    at_end: bool,
}

struct UpwardPrinter<'d, D: PrettyDoc<'d>> {
    width: Width,
    prev: Vec<Chunk<'d, D>>,
    // INVARIANT: only ever contains `Literal`, `Text`, and `Choice` notations.
    next: Vec<Chunk<'d, D>>,
    at_beginning: bool,
}

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
    let seeker = Seeker::new(notation, width);
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

impl<'d, D: PrettyDoc<'d>> Seeker<'d, D> {
    fn new(notation: NotationRef<'d, D>, width: Width) -> Seeker<'d, D> {
        Seeker {
            width,
            prev: vec![],
            next: vec![(Some(0), Shade::background(), notation)],
        }
    }

    fn seek(mut self, path: &[usize]) -> (UpwardPrinter<'d, D>, DownwardPrinter<'d, D>) {
        use NotationCase::*;

        span!("seek");

        // Seek to the descendant given by `path`.
        // Highlight the descendency chain as we go.
        let mut path = path.iter();
        self.highlight(path.len());
        while let Some(child_index) = path.next() {
            self.seek_child(*child_index);
            self.highlight(path.len());
        }

        // Walk backward to the nearest Newline (or beginning of the doc).
        let mut spaces = (0, Shade::background());
        let mut at_beginning = true;
        while let Some((indent, hl, notation)) = self.prev.pop() {
            match notation.case() {
                Empty | Indent(_, _) | Flat(_) | Concat(_, _) | Choice(_, _) | Child(_, _) => {
                    unreachable!()
                }
                Literal(_) => self.next.push((indent, hl, notation)),
                Text(_, _) => self.next.push((indent, hl, notation)),
                Newline => {
                    // drop the newline, but take note of it by setting at_beginning=false.
                    at_beginning = false;
                    spaces = (indent.unwrap(), hl);
                    break;
                }
            }
        }

        let upward_printer = UpwardPrinter {
            width: self.width,
            prev: self.prev,
            next: vec![],
            at_beginning,
        };
        let downward_printer = DownwardPrinter {
            width: self.width,
            next: self.next,
            spaces,
            at_end: false,
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
                        let choice =
                            old_choose(self.width, indent, prefix_len, opt1, opt2, &self.next);
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

impl<'d, D: PrettyDoc<'d>> DownwardPrinter<'d, D> {
    fn print_first_line(&mut self) -> Option<LineContents<'d>> {
        use NotationCase::*;

        span!("print_first_line");

        if self.at_end {
            return None;
        }

        let mut contents = vec![];
        let mut prefix_len = self.spaces.0;
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
                    let contents = LineContents {
                        spaces: self.spaces,
                        contents,
                    };
                    self.spaces = (indent.unwrap(), hl);
                    return Some(contents);
                }
                Indent(j, note) => self.next.push((indent.map(|i| i + j), hl, note)),
                Flat(note) => self.next.push((None, hl, note)),
                Concat(left, right) => {
                    self.next.push((indent, hl, right));
                    self.next.push((indent, hl, left));
                }
                Choice(opt1, opt2) => {
                    let choice = old_choose(self.width, indent, prefix_len, opt1, opt2, &self.next);
                    self.next.push((indent, hl, choice));
                }
                Child(_, child_note) => self.next.push((indent, hl, child_note)),
            }
        }

        self.at_end = true;
        Some(LineContents {
            spaces: self.spaces,
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

impl<'d, D: PrettyDoc<'d>> UpwardPrinter<'d, D> {
    fn print_last_line(&mut self) -> Option<LineContents<'d>> {
        use NotationCase::*;

        span!("print_last_line");

        if self.at_beginning {
            return None;
        }

        // TODO: This is really a separate fn, if the arg is false?
        let newline_info = self.seek_start_of_last_line(false);
        let mut prefix_len = match newline_info {
            None => 0,
            Some((spaces, _)) => spaces,
        };
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
                    let choice = old_choose(self.width, indent, prefix_len, opt1, opt2, &self.next);
                    self.prev.push((indent, hl, choice));
                    self.seek_end();
                    let newline_info = self.seek_start_of_last_line(false);
                    prefix_len = match newline_info {
                        None => 0,
                        Some((spaces, _)) => spaces,
                    };
                }
                Empty | Newline | Indent(_, _) | Flat(_) | Concat(_, _) | Child(_, _) => {
                    unreachable!()
                }
            }
        }

        let spaces;
        if let Some((indent, hl)) = self.seek_start_of_last_line(true) {
            self.at_beginning = false;
            spaces = (indent, hl);
        } else {
            self.at_beginning = true;
            spaces = (0, Shade::background());
        }

        let mut contents = vec![];
        while let Some((_indent, hl, notation)) = self.next.pop() {
            match notation.case() {
                NotationCase::Literal(lit) => contents.push((lit.str(), lit.style(), hl)),
                NotationCase::Text(text, style) => contents.push((text, style, hl)),
                _ => panic!("display_line: expected only literals and text"),
            }
        }
        Some(LineContents { spaces, contents })
    }

    fn seek_end(&mut self) {
        span!("seek_end");

        while let Some((indent, hl, notation)) = self.next.pop() {
            self.prev.push((indent, hl, notation));
        }
    }

    /// Move the "printing cursor" from the very end to the start of the last line. If there is a
    /// preceding newline, return the indentation and shade. Otherwise -- i.e., if this is the last
    /// remaining line -- return None.
    // Maintains the invariant that `next` only ever contains `Literal` and `Choice` notations.
    fn seek_start_of_last_line(&mut self, delete_newline: bool) -> Option<(Width, Shade)> {
        use NotationCase::*;

        span!("seek_start_of_last_line");

        assert!(self.next.is_empty());
        while let Some((indent, hl, notation)) = self.prev.pop() {
            match notation.case() {
                Empty => (),
                Text(_, _) => self.next.push((indent, hl, notation)),
                Literal(_) => self.next.push((indent, hl, notation)),
                Newline => {
                    if !delete_newline {
                        self.prev.push((indent, hl, notation));
                    }
                    return Some((indent.unwrap(), hl));
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
        None
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

/////////////////////////////////////////////////////////////////////////////////////

fn min_first_line_len_entry_point<'d, D: PrettyDoc<'d>>(
    notation: NotationRef<'d, D>,
    flat: bool,
) -> Option<FirstLineLen> {
    span!("min_first_line_len");
    min_first_line_len(notation, flat)
}

// Returns None if impossible.
fn min_first_line_len<'d, D: PrettyDoc<'d>>(
    notation: NotationRef<'d, D>,
    flat: bool,
) -> Option<FirstLineLen> {
    use NotationCase::*;

    match notation.case() {
        Empty => Some(FirstLineLen {
            len: 0,
            has_newline: false,
        }),
        Literal(lit) => {
            let lit_len = lit.len();
            Some(FirstLineLen {
                len: lit_len,
                has_newline: false,
            })
        }
        Text(text, _style) => {
            let text_len = text_len(text);
            Some(FirstLineLen {
                len: text_len,
                has_newline: false,
            })
        }
        Newline => {
            if flat {
                None
            } else {
                Some(FirstLineLen {
                    len: 0,
                    has_newline: true,
                })
            }
        }
        Flat(note) => min_first_line_len(note, true),
        Indent(_, note) => min_first_line_len(note, flat),
        // Note2 must always be smaller
        Choice(note1, note2) => {
            min_first_line_len(note2, flat).or_else(|| min_first_line_len(note1, flat))
        }
        Concat(note1, note2) => min_first_line_len(note1, flat).and_then(|len1| {
            if len1.has_newline {
                Some(len1)
            } else {
                min_first_line_len(note2, flat).map(|len2| FirstLineLen {
                    len: len1.len + len2.len,
                    has_newline: len2.has_newline,
                })
            }
        }),
        Child(_, child_note) => min_first_line_len(child_note, flat),
    }
}

fn compute_suffix_len<'d, D: PrettyDoc<'d>>(next_chunks: &[Chunk<'d, D>]) -> Width {
    span!("compute_suffix_len");

    let mut len = 0;
    for (indent, _, notation) in next_chunks.iter().rev() {
        let flat = indent.is_none();
        let note_len = min_first_line_len_entry_point(*notation, flat).unwrap();
        len += note_len.len;
        if note_len.has_newline {
            break;
        }
    }
    len
}

fn old_choose<'d, D: PrettyDoc<'d>>(
    width: Width,
    indent: Option<Width>,
    prefix_len: Width,
    note1: NotationRef<'d, D>,
    note2: NotationRef<'d, D>,
    suffix: &[Chunk<'d, D>],
) -> NotationRef<'d, D> {
    // Print note1 if it fits, or if it's possible but note2 isn't.
    let suffix_len = compute_suffix_len(suffix);
    let flat = indent.is_none();
    if let Some(len1) = min_first_line_len(note1, flat) {
        let fits = if len1.has_newline {
            prefix_len + len1.len <= width
        } else {
            prefix_len + len1.len + suffix_len <= width
        };
        if fits {
            note1
        } else {
            // (impossibility logic is here)
            if min_first_line_len(note2, flat).is_none() {
                note1
            } else {
                note2
            }
        }
    } else {
        note2
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
