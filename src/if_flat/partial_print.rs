use super::notation::Notation;
use super::pretty_print::choose;
use std::iter::Iterator;

type Chunk<'n> = (Option<usize>, &'n Notation);

struct DownwardPrinter<'n> {
    width: usize,
    next: Vec<Chunk<'n>>,
    spaces: usize,
    at_end: bool,
}

struct UpwardPrinter<'n> {
    width: usize,
    prev: Vec<Chunk<'n>>,
    // INVARIANT: only ever contains `Literal` and `Choice` notations.
    next: Vec<Chunk<'n>>,
    at_beginning: bool,
}

pub fn print_downward_for_testing(notation: &Notation, width: usize) -> Vec<String> {
    let printer = DownwardPrinter::new(notation, width);
    printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .collect()
}

pub fn print_upward_for_testing(notation: &Notation, width: usize) -> Vec<String> {
    let printer = UpwardPrinter::new(notation, width);
    let mut lines = printer
        .map(|(spaces, line)| format!("{:spaces$}{}", "", line, spaces = spaces))
        .collect::<Vec<_>>();
    lines.reverse();
    lines
}

impl<'n> DownwardPrinter<'n> {
    fn new(notation: &'n Notation, width: usize) -> DownwardPrinter<'n> {
        DownwardPrinter {
            width,
            next: vec![(Some(0), notation)],
            spaces: 0,
            at_end: false,
        }
    }

    fn print_first_line(&mut self) -> Option<(usize, String)> {
        use Notation::*;

        if self.at_end {
            return None;
        }

        let mut string = String::new();
        let mut prefix_len = self.spaces;
        while let Some((indent, notation)) = self.next.pop() {
            match notation {
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

impl<'n> UpwardPrinter<'n> {
    fn new(notation: &'n Notation, width: usize) -> UpwardPrinter<'n> {
        UpwardPrinter {
            width,
            prev: vec![(Some(0), notation)],
            next: vec![],
            at_beginning: false,
        }
    }

    fn print_last_line(&mut self) -> Option<(usize, String)> {
        use Notation::*;

        if self.at_beginning {
            return None;
        }

        let spaces = self.seek_start_of_last_line(false);
        let mut prefix_len = spaces.unwrap_or(0);
        while let Some((indent, notation)) = self.next.pop() {
            match notation {
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
            match notation {
                Notation::Literal(lit) => string.push_str(lit),
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
        use Notation::*;

        assert!(self.next.is_empty());
        while let Some((indent, notation)) = self.prev.pop() {
            match notation {
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

fn compute_suffix_len<'n>(next_chunks: &[Chunk<'n>]) -> usize {
    let mut len = 0;
    for (indent, notation) in next_chunks.iter().rev() {
        let flat = indent.is_none();
        let note_len = notation.min_first_line_len(flat).unwrap();
        len += note_len.len;
        if note_len.has_newline {
            break;
        }
    }
    len
}

impl<'n> Iterator for DownwardPrinter<'n> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        self.print_first_line()
    }
}

impl<'n> Iterator for UpwardPrinter<'n> {
    type Item = (usize, String);

    fn next(&mut self) -> Option<(usize, String)> {
        self.print_last_line()
    }
}
