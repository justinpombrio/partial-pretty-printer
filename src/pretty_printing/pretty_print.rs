use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::style::{Shade, Style};
use std::iter::Iterator;

/// (indent, is_in_cursor, notation)
type Chunk<'d, D> = (Option<usize>, Shade, NotationRef<'d, D>);

pub struct LineContents<'d> {
    pub spaces: usize,
    pub spaces_shade: Shade,
    /// (string, style, highlighting)
    pub contents: Vec<(&'d str, Style, Shade)>,
}

#[derive(Clone, Debug)]
pub struct FirstLineLen {
    pub len: usize,
    pub has_newline: bool,
}

struct Seeker<'d, D: PrettyDoc> {
    width: usize,
    prev: Vec<Chunk<'d, D>>,
    next: Vec<Chunk<'d, D>>,
}

struct DownwardPrinter<'d, D: PrettyDoc> {
    width: usize,
    next: Vec<Chunk<'d, D>>,
    spaces: usize,
    spaces_shade: Shade,
    at_end: bool,
}

struct UpwardPrinter<'d, D: PrettyDoc> {
    width: usize,
    prev: Vec<Chunk<'d, D>>,
    // INVARIANT: only ever contains `Literal`, `Text`, and `Choice` notations.
    next: Vec<Chunk<'d, D>>,
    at_beginning: bool,
}

pub fn pretty_print<'d, D: PrettyDoc>(
    doc: &'d D,
    width: usize,
    path: &[usize],
) -> (
    impl Iterator<Item = LineContents<'d>> + 'd,
    impl Iterator<Item = LineContents<'d>> + 'd,
) {
    let notation = NotationRef::new(doc);
    let seeker = Seeker::new(notation, width);
    seeker.seek(path)
}

pub fn pretty_print_to_string<'d, D: PrettyDoc>(doc: &'d D, width: usize) -> String {
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
        let mut string = format!("{:spaces$}", "", spaces = self.spaces);
        for (text, _style, _hl) in &self.contents {
            string.push_str(text);
        }
        string
    }
}

impl<'d, D: PrettyDoc> Seeker<'d, D> {
    fn new(notation: NotationRef<'d, D>, width: usize) -> Seeker<'d, D> {
        Seeker {
            width,
            prev: vec![],
            next: vec![(Some(0), Shade::background(), notation)],
        }
    }

    fn seek(mut self, path: &[usize]) -> (UpwardPrinter<'d, D>, DownwardPrinter<'d, D>) {
        use NotationCase::*;

        // Seek to the descendant given by `path`.
        // Highlight the descendency chain as we go.
        let mut path = path.into_iter();
        self.highlight(path.len());
        while let Some(child_index) = path.next() {
            self.seek_child(*child_index);
            self.highlight(path.len());
        }

        // Walk backward to the nearest Newline (or beginning of the doc).
        let mut spaces = 0;
        let mut spaces_shade = Shade::background();
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
                    spaces = indent.unwrap();
                    spaces_shade = hl;
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
            spaces_shade,
            at_end: false,
        };
        (upward_printer, downward_printer)
    }

    fn seek_child(&mut self, child_index: usize) {
        use NotationCase::*;

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
                        prefix_len += text.chars().count();
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
                        let suffix_len = compute_suffix_len(&self.next);
                        let choice = choose(self.width, indent, prefix_len, opt1, opt2, suffix_len);
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

impl<'d, D: PrettyDoc> DownwardPrinter<'d, D> {
    fn print_first_line(&mut self) -> Option<LineContents<'d>> {
        use NotationCase::*;

        if self.at_end {
            return None;
        }

        let mut contents = vec![];
        let mut prefix_len = self.spaces;
        while let Some((indent, hl, notation)) = self.next.pop() {
            match notation.case() {
                Empty => (),
                Literal(lit) => {
                    contents.push((lit.str(), lit.style(), hl));
                    prefix_len += lit.len();
                }
                Text(text, style) => {
                    contents.push((text.as_ref(), style, hl));
                    prefix_len += text_len(text);
                }
                Newline => {
                    let contents = LineContents {
                        spaces: self.spaces,
                        spaces_shade: self.spaces_shade,
                        contents: contents,
                    };
                    self.spaces = indent.unwrap();
                    self.spaces_shade = hl;
                    return Some(contents);
                }
                Indent(j, note) => self.next.push((indent.map(|i| i + j), hl, note)),
                Flat(note) => self.next.push((None, hl, note)),
                Concat(left, right) => {
                    self.next.push((indent, hl, right));
                    self.next.push((indent, hl, left));
                }
                Choice(opt1, opt2) => {
                    let suffix_len = compute_suffix_len(&self.next);
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, suffix_len);
                    self.next.push((indent, hl, choice));
                }
                Child(_, child_note) => self.next.push((indent, hl, child_note)),
            }
        }

        self.at_end = true;
        Some(LineContents {
            spaces: self.spaces,
            spaces_shade: self.spaces_shade,
            contents: contents,
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

impl<'d, D: PrettyDoc> UpwardPrinter<'d, D> {
    fn print_last_line(&mut self) -> Option<LineContents<'d>> {
        use NotationCase::*;

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
                    let suffix_len = compute_suffix_len(&self.next);
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, suffix_len);
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
        let spaces_shade;
        if let Some((indent, hl)) = self.seek_start_of_last_line(true) {
            self.at_beginning = false;
            spaces = indent;
            spaces_shade = hl;
        } else {
            self.at_beginning = true;
            spaces = 0;
            spaces_shade = Shade::background();
        }

        let mut contents = vec![];
        while let Some((_indent, hl, notation)) = self.next.pop() {
            match notation.case() {
                NotationCase::Literal(lit) => contents.push((lit.str(), lit.style(), hl)),
                NotationCase::Text(text, style) => contents.push((text.as_ref(), style, hl)),
                _ => panic!("display_line: expected only literals and text"),
            }
        }
        Some(LineContents {
            spaces,
            spaces_shade,
            contents: contents,
        })
    }

    fn seek_end(&mut self) {
        while let Some((indent, hl, notation)) = self.next.pop() {
            self.prev.push((indent, hl, notation));
        }
    }

    /// Move the "printing cursor" from the very end to the start of the last line. If there is a
    /// preceding newline, return the indentation and shade. Otherwise -- i.e., if this is the last
    /// remaining line -- return None.
    // Maintains the invariant that `next` only ever contains `Literal` and `Choice` notations.
    fn seek_start_of_last_line(&mut self, delete_newline: bool) -> Option<(usize, Shade)> {
        use NotationCase::*;

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

// Returns None if impossible.
fn min_first_line_len<'d, D: PrettyDoc>(
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
                min_first_line_len(note2, flat).and_then(|len2| {
                    Some(FirstLineLen {
                        len: len1.len + len2.len,
                        has_newline: len2.has_newline,
                    })
                })
            }
        }),
        Child(_, child_note) => min_first_line_len(child_note, flat),
    }
}

fn compute_suffix_len<'d, D: PrettyDoc>(next_chunks: &[Chunk<'d, D>]) -> usize {
    let mut len = 0;
    for (indent, _, notation) in next_chunks.iter().rev() {
        let flat = indent.is_none();
        let note_len = min_first_line_len(*notation, flat).unwrap();
        len += note_len.len;
        if note_len.has_newline {
            break;
        }
    }
    len
}

fn choose<'d, D: PrettyDoc>(
    width: usize,
    indent: Option<usize>,
    prefix_len: usize,
    note1: NotationRef<'d, D>,
    note2: NotationRef<'d, D>,
    suffix_len: usize,
) -> NotationRef<'d, D> {
    // Print note1 if it fits, or if it's possible but note2 isn't.
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

fn text_len(text: &str) -> usize {
    text.chars().count()
}

impl<'d, D: PrettyDoc> Iterator for DownwardPrinter<'d, D> {
    type Item = LineContents<'d>;

    fn next(&mut self) -> Option<LineContents<'d>> {
        self.print_first_line()
    }
}

impl<'d, D: PrettyDoc> Iterator for UpwardPrinter<'d, D> {
    type Item = LineContents<'d>;

    fn next(&mut self) -> Option<LineContents<'d>> {
        self.print_last_line()
    }
}
