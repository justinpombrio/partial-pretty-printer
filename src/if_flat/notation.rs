use std::ops::{Add, BitOr, BitXor, Shr};

#[derive(Clone, Debug)]
pub enum Notation {
    /// Display nothing. Identical to `Literal("")`.
    Empty,
    /// Literal text. Cannot contain a newline.
    Literal(String),
    /// Display a newline. If this is inside an `Indent`, the new line will be indented.
    Newline,
    /// Only consider single-line options of the contained notation.
    Flat(Box<Notation>),
    /// Indent all lines of the contained notation except the first to the right by the given
    /// number of spaces.
    Indent(usize, Box<Notation>),
    /// Display both notations. The first character of the right notation immediately follows the
    /// last character of the left notation. The right notation's indentation level is not
    /// affected.
    Concat(Box<Notation>, Box<Notation>),
    /// Display the left notation if it fits on one line within the required width; otherwise the
    /// right.
    Choice(Box<Notation>, Box<Notation>),
}

pub struct FirstLineLen {
    pub len: usize,
    pub has_newline: bool,
}

impl Notation {
    // TODO: build this into the notation. This can be exponentially large!
    pub fn repeat(
        elements: Vec<Notation>,
        empty: Notation,
        lone: impl Fn(Notation) -> Notation,
        join: impl Fn(Notation, Notation) -> Notation,
        surround: impl Fn(Notation) -> Notation,
    ) -> Notation {
        let mut iter = elements.into_iter();
        match iter.len() {
            0 => empty,
            1 => lone(iter.next().unwrap()),
            _ => {
                let mut iter = iter.rev();
                let mut accumulator = iter.next().unwrap();
                for elem in iter {
                    accumulator = join(elem, accumulator);
                }
                surround(accumulator)
            }
        }
    }

    // Returns None if impossible.
    pub(super) fn min_first_line_len(&self, flat: bool) -> Option<FirstLineLen> {
        use Notation::*;

        match self {
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
            Flat(note) => note.min_first_line_len(true),
            Indent(_, note) => note.min_first_line_len(flat),
            // Note2 must always be smaller
            Choice(note1, note2) => note2
                .min_first_line_len(flat)
                .or_else(|| note1.min_first_line_len(flat)),
            Concat(note1, note2) => note1.min_first_line_len(flat).and_then(|len1| {
                if len1.has_newline {
                    Some(len1)
                } else {
                    note2.min_first_line_len(flat).and_then(|len2| {
                        Some(FirstLineLen {
                            len: len1.len + len2.len,
                            has_newline: len2.has_newline,
                        })
                    })
                }
            }),
        }
    }
}

impl Add<Notation> for Notation {
    type Output = Notation;

    /// Shorthand for `Concat`.
    fn add(self, other: Notation) -> Notation {
        Notation::Concat(Box::new(self), Box::new(other))
    }
}

impl BitOr<Notation> for Notation {
    type Output = Notation;

    /// Shorthand for `Choice`.
    fn bitor(self, other: Notation) -> Notation {
        // TODO: if_flat?!
        Notation::Choice(Box::new(self), Box::new(other))
    }
}

impl BitXor<Notation> for Notation {
    type Output = Notation;

    /// Shorthand for `X + newline() + Y`.
    fn bitxor(self, other: Notation) -> Notation {
        self + Notation::Newline + other
    }
}

impl Shr<Notation> for usize {
    type Output = Notation;
    /// Shorthand for nesting (indented newline)
    fn shr(self, notation: Notation) -> Notation {
        Notation::Indent(self, Box::new(Notation::Newline + notation))
    }
}
