#![allow(unused)]

use super::consolidated_notation::{
    ConsolidatedNotation, DelayedConsolidatedNotation, PrintingError,
};
use super::pretty_doc::PrettyDoc;
use crate::geometry::{str_width, Width};
use crate::infra::span;
use std::iter::Iterator;
use std::mem;

/// Pretty print a document, focused at the node found by traversing `path` from the root.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to,
/// find a layout that fits within that width.
///
/// `marks` is a set of document node ids to mark. Each chunk of text in the output will say which,
/// if any, marked id it is part of.
///
/// Returns a pair of iterators:
///
/// - the first prints lines above the focused node going up
/// - the second prints lines from the first line of the focused node going down
///
/// It is expected that you will take only as many lines as you need from the iterators; doing so
/// will save computation time.
/*
pub fn pretty_print<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
) -> Result<
    (
        impl Iterator<Item = Result<LineContents<'d, D>, PrintingError>>,
        impl Iterator<Item = Result<LineContents<'d, D>, PrintingError>>,
    ),
    PrintingError,
> {
    span!("Pretty Print");

    let seeker = Seeker::new(width, doc)?;
    seeker.seek(path)
}

/// Print the entirety of the document to a single string, ignoring styles and shading.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to,
/// find a layout that fits withing that width.
pub fn pretty_print_to_string<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
) -> Result<String, PrintingError> {
    let (_, mut lines_iter) = pretty_print(doc, width, &[])?;
    let mut string = lines_iter.next().unwrap()?.to_string();
    for line in lines_iter {
        string.push('\n');
        string.push_str(&line?.to_string());
    }
    Ok(string)
}
*/

struct Chunk<'d, D: PrettyDoc<'d>> {
    id: D::Id,
    mark: Option<&'d D::Mark>,
    notation: ConsolidatedNotation<'d, D>,
}

impl<'d, D: PrettyDoc<'d>> Clone for Chunk<'d, D> {
    fn clone(&self) -> Self {
        Chunk {
            id: self.id,
            mark: self.mark,
            notation: self.notation,
        }
    }
}
impl<'d, D: PrettyDoc<'d>> Copy for Chunk<'d, D> {}

impl<'d, D: PrettyDoc<'d>> Chunk<'d, D> {
    fn new(notation: DelayedConsolidatedNotation<'d, D>) -> Result<Self, PrintingError> {
        let id = notation.doc().id();
        let (notation, mark) = notation.eval()?;
        Ok(Chunk { id, mark, notation })
    }

    fn sub_chunk(
        &self,
        notation: DelayedConsolidatedNotation<'d, D>,
    ) -> Result<Self, PrintingError> {
        let id = notation.doc().id();
        let (notation, mark) = notation.eval()?;
        let mark = mark.or(self.mark);
        Ok(Chunk { id, mark, notation })
    }
}

// TODO: Remove D::Id from these? Seems redundant with marks.
/// The contents of a single pretty printed line.
pub struct LineContents<'d, D: PrettyDoc<'d>> {
    /// The indentation of this line in spaces.
    pub indentation: Indentation<'d, D>,
    /// A sequence of pieces of text to be displayed after `spaces`, in order from left to right,
    /// with no spacing in between.
    pub pieces: Vec<Piece<'d, D>>,
}

pub struct Indentation<'d, D: PrettyDoc<'d>> {
    pub num_spaces: Width,
    pub doc_id: D::Id,
    pub mark: Option<&'d D::Mark>,
}

pub struct Piece<'d, D: PrettyDoc<'d>> {
    pub str: &'d str,
    pub style: &'d D::Style,
    pub doc_id: D::Id,
    pub mark: Option<&'d D::Mark>,
}

impl<'d, D: PrettyDoc<'d>> LineContents<'d, D> {
    pub fn width(&self) -> Width {
        let mut width = self.indentation.num_spaces;
        for piece in &self.pieces {
            width += str_width(piece.str);
        }
        width
    }
}

impl<'d, D: PrettyDoc<'d>> ToString for LineContents<'d, D> {
    fn to_string(&self) -> String {
        span!("LineContents::to_string");

        let mut string = format!(
            "{:spaces$}",
            "",
            spaces = self.indentation.num_spaces as usize
        );
        for piece in &self.pieces {
            string.push_str(piece.str);
        }
        string
    }
}

struct Block<'d, D: PrettyDoc<'d>> {
    indentation: Indentation<'d, D>,
    chunks: Vec<Chunk<'d, D>>,
}

struct Printer<'d, D: PrettyDoc<'d>> {
    width: Width,
    prev_blocks: Vec<Block<'d, D>>,
    indentation: Indentation<'d, D>,
    prev_chunks: Vec<Chunk<'d, D>>,
    next_chunks: Vec<Chunk<'d, D>>,
    next_blocks: Vec<Block<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> Printer<'d, D> {
    // aaaaa      aaaaa      aaaaa
    // a|*|B  ->  a****  or  a|**B
    // BBBBB      |***B      BBBBB
    //            BBBBB
    fn simplify_last_line(&mut self, chunk: Chunk<'d, D>) -> Result<(), PrintingError> {
        use ConsolidatedNotation::*;
        span!("printer.push_prev");

        let mut chunks = Vec::new();
        let mut stack = vec![chunk];
        while let Some(chunk) = stack.pop() {
            match chunk.notation {
                Empty => (),
                Literal(_) | Text(_, _) | Choice(_, _) | Child(_, _) => chunks.push(chunk),
                Newline(num_spaces) => {
                    let mut prev_chunks = mem::take(&mut self.prev_chunks);
                    prev_chunks.extend(mem::take(&mut chunks));
                    self.prev_blocks.push(Block {
                        indentation: Indentation {
                            num_spaces,
                            doc_id: chunk.id,
                            mark: chunk.mark,
                        },
                        chunks: prev_chunks,
                    });
                }
                Concat(left, right) => {
                    stack.push(chunk.sub_chunk(right)?);
                    stack.push(chunk.sub_chunk(left)?);
                }
            }
        }
        chunks.extend(mem::take(&mut self.next_chunks));
        self.next_chunks = chunks;
        Ok(())
    }

    // aaaaa       aaaaa      aaaaa
    // a|*|B   ->  a|***  or  a|**B
    // BBBBB       ***BB      BBBBB
    //             BBBBB
    fn simplify_first_line(&mut self, chunk: Chunk<'d, D>) -> Result<(), PrintingError> {
        use ConsolidatedNotation::*;
        span!("printer.push_next");

        match chunk.notation {
            Empty => (),
            Literal(_) | Text(_, _) | Choice(_, _) | Child(_, _) => self.next_chunks.push(chunk),
            Newline(num_spaces) => {
                self.next_blocks.push(Block {
                    indentation: Indentation {
                        num_spaces,
                        doc_id: chunk.id,
                        mark: chunk.mark,
                    },
                    chunks: mem::take(&mut self.next_chunks),
                });
            }
            Concat(left, right) => {
                self.simplify_first_line(chunk.sub_chunk(right)?)?;
                self.simplify_first_line(chunk.sub_chunk(left)?)?;
            }
        }
        Ok(())
    }
}

fn print_expanded_block<'d, D: PrettyDoc<'d>>(mut block: Block<'d, D>) -> LineContents<'d, D> {
    use ConsolidatedNotation::*;

    let mut pieces = vec![];
    let mut prefix_len = block.indentation.num_spaces;
    while let Some(chunk) = block.chunks.pop() {
        match chunk.notation {
            Literal(lit) => {
                pieces.push(Piece {
                    str: lit.str(),
                    style: lit.style(),
                    doc_id: chunk.id,
                    mark: chunk.mark,
                });
                prefix_len += lit.width();
            }
            Text(text, style) => {
                pieces.push(Piece {
                    str: text,
                    style,
                    doc_id: chunk.id,
                    mark: chunk.mark,
                });
                prefix_len += str_width(text);
            }
            Empty | Newline(_) | Concat(_, _) | Choice(_, _) | Child(_, _) => {
                panic!("Bug (print_expanded_block): expected only Literal,Text")
            }
        }
    }
    LineContents {
        indentation: block.indentation,
        pieces,
    }
}

/// Determine which of the two options of the choice to select. Pick the first option if it fits.
/// (We also want to pick the first option if we're inside a `Flat`, but ConsolidatedNotation already
/// took care of that.)
fn choose<'d, D: PrettyDoc<'d>>(
    _width: Width,
    _prefix_len: Width,
    _opt1: DelayedConsolidatedNotation<'d, D>,
    _opt2: DelayedConsolidatedNotation<'d, D>,
    _next_chunks: &[Chunk<'d, D>],
    _next_blocks: &[Block<'d, D>],
) -> Result<DelayedConsolidatedNotation<'d, D>, PrintingError> {
    todo!()
}

/// Determine whether the first line of the notations (`notation` followed by `chunks`) fits within
/// `width`.
fn fits<'d, D: PrettyDoc<'d>>(
    width: Width,
    notation: ConsolidatedNotation<'d, D>,
    next_chunks: &[Chunk<'d, D>],
) -> Result<bool, PrintingError> {
    use ConsolidatedNotation::*;

    span!("fits");

    let mut next_chunks = next_chunks;
    let mut remaining = width;
    let mut notations = vec![notation];

    loop {
        let notation = match notations.pop() {
            Some(notation) => notation,
            None => match next_chunks.split_last() {
                None => return Ok(true),
                Some((chunk, more)) => {
                    next_chunks = more;
                    chunk.notation
                }
            },
        };

        match notation {
            Empty => (),
            Literal(lit) => {
                let lit_len = lit.width();
                if lit_len <= remaining {
                    remaining -= lit_len;
                } else {
                    return Ok(false);
                }
            }
            Text(text, _) => {
                let text_len = str_width(text);
                if text_len <= remaining {
                    remaining -= text_len;
                } else {
                    return Ok(false);
                }
            }
            Newline(_) => return Ok(true),
            Child(_, note) => notations.push(note.eval()?.0),
            Concat(note1, note2) => {
                notations.push(note2.eval()?.0);
                notations.push(note1.eval()?.0);
            }
            Choice(_opt1, opt2) => {
                // This assumes that:
                //     For every layout a in opt1 and b in opt2,
                //     first_line_len(a) >= first_line_len(b)
                // And also that ConsolidatedNotation would have removed this choice if we're in a Flat
                notations.push(opt2.eval()?.0);
            }
        }
    }
}
