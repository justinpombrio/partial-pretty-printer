use crate::doc::{Doc, NotationCase, NotationRef};
use std::iter::Iterator;

// Seeking:
//
// 1. Expand forward to sought Child, or Choice containing sought Child.
//    If instead reach end of doc, panic (every hole must be present).
// 2. Walk backwards to nearest Newline (or beginning of doc).
// 3. Walk forwards to nearest Child or Choice, and resolve it. Go to step 1.
//    If instead reach sought Child or Choice containing sought Child,
//    continue seeking the inner child, or you're done seeking.
//    If instead reach end of doc, panic (every hole must be present).

// Expansion: eliminate Flat, Indent, Concat, Empty.
// Walk: assume that expansion has already happened.
// Resolution: eliminate Choice, Child.

type Chunk<'d, D> = (Option<usize>, NotationRef<'d, D>);

#[derive(Clone, Debug)]
pub struct FirstLineLen {
    pub len: usize,
    pub has_newline: bool,
}

struct DownwardPrinter<'d, D: Doc> {
    width: usize,
    next: Vec<Chunk<'d, D>>,
    spaces: usize,
    at_end: bool,
}

struct UpwardPrinter<'d, D: Doc> {
    width: usize,
    prev: Vec<Chunk<'d, D>>,
    // INVARIANT: only ever contains `Literal` and `Choice` notations.
    next: Vec<Chunk<'d, D>>,
    at_beginning: bool,
}

pub fn print_downward_for_testing<D: Doc>(doc: &D, width: usize) -> Vec<String> {
    let notation = NotationRef {
        doc,
        notation: doc.notation(),
    };
    let printer = DownwardPrinter::new(notation, width);
    printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .collect()
}

pub fn print_upward_for_testing<D: Doc>(doc: &D, width: usize) -> Vec<String> {
    let notation = NotationRef {
        doc,
        notation: doc.notation(),
    };
    let printer = UpwardPrinter::new(notation, width);
    let mut lines = printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .collect::<Vec<_>>();
    lines.reverse();
    lines
}

impl<'d, D: Doc> DownwardPrinter<'d, D> {
    fn new(notation: NotationRef<'d, D>, width: usize) -> DownwardPrinter<'d, D> {
        DownwardPrinter {
            width,
            next: vec![(Some(0), notation)],
            spaces: 0,
            at_end: false,
        }
    }

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

impl<'d, D: Doc> UpwardPrinter<'d, D> {
    fn new(notation: NotationRef<'d, D>, width: usize) -> UpwardPrinter<'d, D> {
        UpwardPrinter {
            width,
            prev: vec![(Some(0), notation)],
            next: vec![],
            at_beginning: false,
        }
    }

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
                Choice(opt1, opt2) => {
                    let suffix_len = compute_suffix_len(&self.next);
                    let choice = choose(self.width, indent, prefix_len, opt1, opt2, suffix_len);
                    self.prev.push((indent, choice));
                    self.seek_end();
                    let spaces = self.seek_start_of_last_line(false);
                    prefix_len = spaces.unwrap_or(0);
                }
                _ => unreachable!(),
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
                _ => panic!("display_line: expected only literals"),
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
fn min_first_line_len<'d, D: Doc>(
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

fn compute_suffix_len<'d, D: Doc>(next_chunks: &[Chunk<'d, D>]) -> usize {
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

fn choose<'d, D: Doc>(
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

impl<'d, D: Doc> Iterator for DownwardPrinter<'d, D> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        self.print_first_line()
    }
}

impl<'d, D: Doc> Iterator for UpwardPrinter<'d, D> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        self.print_last_line()
    }
}
