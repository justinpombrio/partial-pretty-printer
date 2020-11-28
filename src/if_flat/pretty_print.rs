use super::notation::Notation;

pub fn pretty_print(notation: &Notation, width: usize) -> Vec<String> {
    let mut printer = PrettyPrinter::new(width);
    printer.print(notation, 0, 0, 0);
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

    fn print(&mut self, notation: &Notation, indent: usize, prefix_len: usize, suffix_len: usize) {
        use Notation::*;

        match notation {
            Empty => (),
            Literal(text) => self.lines.last_mut().unwrap().push_str(text),
            Newline => {
                let new_line = format!("{:indent$}", "", indent = indent);
                self.lines.push(new_line);
            }
            Indent(i, note) => self.print(note, indent + i, prefix_len, suffix_len),
            Choice(note1, note2) => {
                let available_len = self.width.saturating_sub(prefix_len + suffix_len);
                let fits = note1.fits_when_flat(available_len).is_some();
                let choice = if fits { note1 } else { note2 };
                self.print(choice, indent, prefix_len, suffix_len);
            }
            Concat(note1, note2) => {
                let note2_len = note2.min_first_line_len();
                let new_suffix_len = if note2_len.has_newline {
                    note2_len.len
                } else {
                    note2_len.len + suffix_len
                };
                self.print(note1, indent, prefix_len, new_suffix_len);
                let new_prefix_len = self.lines.last().unwrap().chars().count();
                self.print(note2, indent, new_prefix_len, suffix_len);
            }
        }
    }
}
