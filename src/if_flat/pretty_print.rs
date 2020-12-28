use super::notation::Notation;

pub fn pretty_print(notation: &Notation, width: usize) -> Vec<String> {
    let mut printer = PrettyPrinter::new(width);
    printer.print(notation, Some(0), 0);
    printer.lines
}

struct PrettyPrinter {
    width: usize,
    lines: Vec<String>,
}

impl PrettyPrinter {
    fn new(width: usize) -> PrettyPrinter {
        PrettyPrinter {
            width,
            lines: vec![String::new()],
        }
    }

    // indent = None means flat
    fn print(&mut self, notation: &Notation, indent: Option<usize>, suffix_len: usize) {
        use Notation::*;

        match notation {
            Empty => (),
            Literal(text) => self.lines.last_mut().unwrap().push_str(text),
            Newline => {
                if let Some(indent) = indent {
                    let new_line = format!("{:indent$}", "", indent = indent);
                    self.lines.push(new_line);
                } else {
                    panic!("Newline inside flat");
                }
            }
            Flat(note) => self.print(note, None, suffix_len),
            Indent(j, note) => self.print(note, indent.map(|i| i + j), suffix_len),
            Choice(note1, note2) => {
                let prefix_len = self.lines.last().unwrap().chars().count();
                let choice = choose(self.width, indent, prefix_len, note1, note2, suffix_len);
                self.print(choice, indent, suffix_len);
            }
            Concat(note1, note2) => {
                let flat = indent.is_none();
                let new_suffix_len = if let Some(len2) = note2.min_first_line_len(flat) {
                    if len2.has_newline {
                        len2.len
                    } else {
                        len2.len + suffix_len
                    }
                } else {
                    panic!("Newline inside flat");
                };
                self.print(note1, indent, new_suffix_len);
                self.print(note2, indent, suffix_len);
            }
        }
    }
}

pub(crate) fn choose<'n>(
    width: usize,
    indent: Option<usize>,
    prefix_len: usize,
    note1: &'n Notation,
    note2: &'n Notation,
    suffix_len: usize,
) -> &'n Notation {
    // Print note1 if it fits, or if it's possible but note2 isn't.
    let flat = indent.is_none();
    if let Some(len1) = note1.min_first_line_len(flat) {
        let fits = if len1.has_newline {
            prefix_len + len1.len <= width
        } else {
            prefix_len + len1.len + suffix_len <= width
        };
        if fits {
            note1
        } else {
            // (impossibility logic is here)
            if note2.min_first_line_len(flat).is_none() {
                note1
            } else {
                note2
            }
        }
    } else {
        note2
    }
}
