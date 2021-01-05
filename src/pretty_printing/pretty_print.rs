use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use std::iter::Iterator;

type Chunk<'d, D> = (Option<usize>, NotationRef<'d, D>);

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
    at_end: bool,
}

struct UpwardPrinter<'d, D: PrettyDoc> {
    width: usize,
    prev: Vec<Chunk<'d, D>>,
    // INVARIANT: only ever contains `Literal` and `Choice` notations.
    next: Vec<Chunk<'d, D>>,
    at_beginning: bool,
}

pub fn pretty_print<'d, D: PrettyDoc>(
    doc: &'d D,
    width: usize,
    path: impl IntoIterator<Item = usize>,
) -> (
    impl Iterator<Item = (usize, String)> + 'd,
    impl Iterator<Item = (usize, String)> + 'd,
) {
    let notation = NotationRef::new(doc);
    let seeker = Seeker::new(notation, width);
    seeker.seek(path)
}

impl<'d, D: PrettyDoc> Seeker<'d, D> {
    fn new(notation: NotationRef<'d, D>, width: usize) -> Seeker<'d, D> {
        Seeker {
            width,
            prev: vec![],
            next: vec![(Some(0), notation)],
        }
    }

    fn seek(
        mut self,
        path: impl IntoIterator<Item = usize>,
    ) -> (UpwardPrinter<'d, D>, DownwardPrinter<'d, D>) {
        use NotationCase::*;

        let path = path.into_iter().collect::<Vec<_>>();
        let mut path = path.into_iter();
        while let Some(child_index) = path.next() {
            self.seek_child(child_index);
        }

        // Walk backward to the nearest Newline (or beginning of the doc).
        let mut spaces = 0;
        let mut at_beginning = true;
        while let Some((indent, notation)) = self.prev.pop() {
            match notation.case() {
                Empty | Indent(_, _) | Flat(_) | Concat(_, _) | Choice(_, _) | Child(_, _) => {
                    unreachable!()
                }
                Literal(_) => self.next.push((indent, notation)),
                Text(_) => self.next.push((indent, notation)),
                Newline => {
                    // drop the newline, but take note of it by setting at_beginning=false.
                    at_beginning = false;
                    spaces = indent.unwrap();
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

        let parent_doc_id = self.next.last().unwrap().1.doc_id();
        'find_child: loop {
            // 1. Expand forward to the nearest `Choice` or `Child` belonging to `parent_doc`.
            //    (NOTE: more precise would be looking for Child(child_index) or a Choice
            //     containing it, but you can't tell right now what children a choice might
            //     contain.)
            while let Some((indent, notation)) = self.next.pop() {
                match notation.case() {
                    Empty => (),
                    Literal(_) => self.prev.push((indent, notation)),
                    Newline => self.prev.push((indent, notation)),
                    Text(_) => self.prev.push((indent, notation)),
                    Indent(j, note) => self.next.push((indent.map(|i| i + j), note)),
                    Flat(note) => self.next.push((None, note)),
                    Concat(left, right) => {
                        self.next.push((indent, right));
                        self.next.push((indent, left));
                    }
                    Choice(_, _) if notation.doc_id() == parent_doc_id => {
                        self.next.push((indent, notation));
                        break;
                    }
                    Choice(_, _) => self.prev.push((indent, notation)),
                    Child(i, _) if notation.doc_id() == parent_doc_id && i == child_index => {
                        self.next.push((indent, notation));
                        break;
                    }
                    Child(_, _) => self.prev.push((indent, notation)),
                }
            }

            // 2. Walk backward to the nearest Newline (or beginning of the doc).
            let mut prefix_len = 0;
            while let Some((indent, notation)) = self.prev.pop() {
                match notation.case() {
                    Empty | Indent(_, _) | Flat(_) | Concat(_, _) => unreachable!(),
                    Literal(_) | Text(_) | Choice(_, _) | Child(_, _) => {
                        self.next.push((indent, notation))
                    }
                    Newline => {
                        prefix_len = indent.unwrap();
                        self.prev.push((indent, notation));
                        break;
                    }
                }
            }

            // 3. Walk forward to the nearest Child or Choice, and resolve it. Go back to 1.
            //    If you hit `Child(i)` belonging to `parent_doc`, success.
            //    If you hit end of doc, panic (every child must be present).
            while let Some((indent, notation)) = self.next.pop() {
                match notation.case() {
                    Empty | Indent(_, _) | Flat(_) | Concat(_, _) | Newline => unreachable!(),
                    Literal(lit) => {
                        prefix_len += lit.chars().count();
                        self.prev.push((indent, notation))
                    }
                    Text(text) => {
                        prefix_len += text.chars().count();
                        self.prev.push((indent, notation));
                    }
                    Child(i, child) if notation.doc_id() == parent_doc_id && i == child_index => {
                        self.next.push((indent, child));
                        return;
                    }
                    Child(_, child) => {
                        self.next.push((indent, child));
                        continue 'find_child;
                    }
                    Choice(opt1, opt2) => {
                        let suffix_len = compute_suffix_len(&self.next);
                        let choice = choose(self.width, indent, prefix_len, opt1, opt2, suffix_len);
                        self.next.push((indent, choice));
                        continue 'find_child;
                    }
                }
            }

            panic!("Missing child ({})", child_index);
        }
    }

    #[allow(unused)]
    fn display(&self) {
        for (_, notation) in &self.prev {
            print!("{} ", notation);
        }
        print!(" / ");
        for (_, notation) in self.next.iter().rev() {
            print!("{} ", notation);
        }
        println!();
    }
}

impl<'d, D: PrettyDoc> DownwardPrinter<'d, D> {
    fn print_first_line(&mut self) -> Option<(usize, String)> {
        use NotationCase::*;

        if self.at_end {
            return None;
        }

        let mut string = String::new();
        let mut prefix_len = self.spaces;
        while let Some((indent, notation)) = self.next.pop() {
            match notation.case() {
                Empty => (),
                Literal(lit) => {
                    string.push_str(lit);
                    prefix_len += lit.chars().count();
                }
                Text(text) => {
                    let text = text.as_ref();
                    string.push_str(text);
                    prefix_len += text.chars().count();
                }
                Newline => {
                    let line = (self.spaces, string);
                    self.spaces = indent.unwrap();
                    return Some(line);
                }
                Indent(j, note) => self.next.push((indent.map(|i| i + j), note)),
                Flat(note) => self.next.push((None, note)),
                Concat(left, right) => {
                    self.next.push((indent, right));
                    self.next.push((indent, left));
                }
                Choice(opt1, opt2) => {
                    let suffix_len = compute_suffix_len(&self.next);
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, suffix_len);
                    self.next.push((indent, choice));
                }
                Child(_, child_note) => self.next.push((indent, child_note)),
            }
        }

        self.at_end = true;
        Some((self.spaces, string))
    }

    #[allow(unused)]
    fn display(&self) {
        for (_, notation) in self.next.iter().rev() {
            print!("{} ", notation);
        }
        println!();
    }
}

impl<'d, D: PrettyDoc> UpwardPrinter<'d, D> {
    fn print_last_line(&mut self) -> Option<(usize, String)> {
        use NotationCase::*;

        if self.at_beginning {
            return None;
        }

        let spaces = self.seek_start_of_last_line(false);
        let mut prefix_len = spaces.unwrap_or(0);
        while let Some((indent, notation)) = self.next.pop() {
            match notation.case() {
                Literal(lit) => {
                    prefix_len += lit.chars().count();
                    self.prev.push((indent, notation));
                }
                Text(text) => {
                    prefix_len += text.chars().count();
                    self.prev.push((indent, notation));
                }
                Choice(opt1, opt2) => {
                    let suffix_len = compute_suffix_len(&self.next);
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, suffix_len);
                    self.prev.push((indent, choice));
                    self.seek_end();
                    let spaces = self.seek_start_of_last_line(false);
                    prefix_len = spaces.unwrap_or(0);
                }
                Empty | Newline | Indent(_, _) | Flat(_) | Concat(_, _) | Child(_, _) => {
                    unreachable!()
                }
            }
        }

        let spaces;
        if let Some(indent) = self.seek_start_of_last_line(true) {
            self.at_beginning = false;
            spaces = indent;
        } else {
            self.at_beginning = true;
            spaces = 0;
        }

        let mut string = String::new();
        while let Some((_, notation)) = self.next.pop() {
            match notation.case() {
                NotationCase::Literal(lit) => string.push_str(lit),
                NotationCase::Text(text) => string.push_str(text),
                _ => panic!("display_line: expected only literals and text"),
            }
        }
        Some((spaces, string))
    }

    fn seek_end(&mut self) {
        while let Some((indent, notation)) = self.next.pop() {
            self.prev.push((indent, notation));
        }
    }

    /// Move the "printing cursor" from the very end to the start of the last line. If there is a
    /// preceding newline, return the indentation. Otherwise -- i.e., if this is the last remaining
    /// line -- return None.
    // Maintains the invariant that `next` only ever contains `Literal` and `Choice` notations.
    fn seek_start_of_last_line(&mut self, delete_newline: bool) -> Option<usize> {
        use NotationCase::*;

        assert!(self.next.is_empty());
        while let Some((indent, notation)) = self.prev.pop() {
            match notation.case() {
                Empty => (),
                Text(_) => self.next.push((indent, notation)),
                Literal(_) => self.next.push((indent, notation)),
                Newline => {
                    if !delete_newline {
                        self.prev.push((indent, notation));
                    }
                    return Some(indent.unwrap());
                }
                Indent(j, note) => self.prev.push((indent.map(|i| i + j), note)),
                Flat(note) => self.prev.push((None, note)),
                Concat(left, right) => {
                    self.prev.push((indent, left));
                    self.prev.push((indent, right));
                }
                Choice(_, _) => self.next.push((indent, notation)),
                Child(_, note) => self.prev.push((indent, note)),
            }
        }
        None
    }

    #[allow(unused)]
    fn display(&self) {
        for (_, notation) in &self.prev {
            print!("{} ", notation);
        }
        print!(" / ");
        for (_, notation) in self.next.iter().rev() {
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
        Literal(text) => {
            let text_len = text.chars().count();
            Some(FirstLineLen {
                len: text_len,
                has_newline: false,
            })
        }
        Text(text) => {
            let text_len = text.chars().count();
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
    for (indent, notation) in next_chunks.iter().rev() {
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

impl<'d, D: PrettyDoc> Iterator for DownwardPrinter<'d, D> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        self.print_first_line()
    }
}

impl<'d, D: PrettyDoc> Iterator for UpwardPrinter<'d, D> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        self.print_last_line()
    }
}
