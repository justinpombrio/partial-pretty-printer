use super::notation::Notation;

pub fn pretty_print(notation: &Notation, width: usize) -> Vec<String> {
    let mut printer = PrettyPrinter::new(width);
    printer.print(notation, Some(0), 0, 0);
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
    fn print(
        &mut self,
        notation: &Notation,
        indent: Option<usize>,
        prefix_len: usize,
        suffix_len: usize,
    ) {
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
            Flat(note) => self.print(note, None, prefix_len, suffix_len),
            Indent(i, note) => self.print(note, indent.map(|j| j + i), prefix_len, suffix_len),
            Choice(note1, note2) => {
                println!(
                    "Choice (indent={:?}, prefix={}, suffix={})",
                    indent, prefix_len, suffix_len
                );
                // Print note1 if it fits, or if it's possible but note2 isn't.
                let flat = indent.is_none();
                let choice = if let Some(len1) = note1.min_first_line_len(flat) {
                    let fits = if len1.has_newline {
                        println!("(len1={}, has newline)", len1.len);
                        prefix_len + len1.len <= self.width
                    } else {
                        println!("(len1={}, no newline)", len1.len);
                        prefix_len + len1.len + suffix_len <= self.width
                    };
                    if fits {
                        println!("Note1 fits");
                        note1
                    } else {
                        // (impossibility logic is here)
                        if note2.min_first_line_len(flat).is_none() {
                            println!("Note1 possible, Note2 impossible");
                            note1
                        } else {
                            println!("Note1 possible, Note2 possible");
                            note2
                        }
                    }
                } else {
                    println!("Note1 impossible");
                    note2
                };
                self.print(choice, indent, prefix_len, suffix_len);
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
                /*
                println!(
                    "Suffix len: {}(from {})\n{:#?}",
                    new_suffix_len, suffix_len, note2
                );
                */
                self.print(note1, indent, prefix_len, new_suffix_len);
                let new_prefix_len = self.lines.last().unwrap().chars().count();
                self.print(note2, indent, new_prefix_len, suffix_len);
            }
        }
    }

    /*
    fn fits(&mut self, notation: &Notation, flat: bool, prefix_len: usize, suffix_len: usize) {
        use Notation::*;

        match notation {
            Empty => Some(FirstLineLen {
                len: 0,
                has_newline: false,
            }),
            Literal(text) => {
                let text_len = text.chars().count();
                let available_len = self.width.saturating_sub(prefix_len + suffix_len);
                if text_len <= available_len {
                    Some(FirstLineLen {
                        len: text_len,
                        has_newline: false,
                    })
                } else {
                    None
                }
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
            Flat(note) => self.fits(note, true, prefix_len, suffix_len),
            Indent(_, note) => self.fits(note, flat, prefix_len, suffix_len),
            // Note2 must always be smaller
            Choice(note1, note2) => self
                .fits(note2, flat, prefix_len, suffix_len)
                .or_else(|| self.fits(note1, flat, prefix_len, suffix_len)),
            Concat(note1, note2) => {
                if let Some(len1) = self.fits(note1, flat, prefix_len, suffix_len) {
                    if len1.has_newline {
                        Some(len1)
                    } else {
                        if let Some(len2) = note2.fits(len - len1.len, flat) {
                            Some(FirstLineLen {
                                len: len1.len + len2.len,
                                has_newline: len2.has_newline,
                            })
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            }
        }
    }
    */
}
