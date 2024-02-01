use super::consolidated_notation::{
    ConsolidatedNotation, DelayedConsolidatedNotation, PrintingError,
};
use super::pretty_doc::PrettyDoc;
use crate::geometry::{str_width, Width};
use crate::infra::span;
use std::collections::HashSet;
use std::iter::Iterator;

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

/// Can seek to an arbitrary position within the document, while resolving as few choices as
/// possible.
struct Seeker<'d, D: PrettyDoc<'d>> {
    width: Width,
    prev: Vec<Chunk<'d, D>>,
    next: Vec<Chunk<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> Seeker<'d, D> {
    fn new(width: Width, doc: D) -> Result<Seeker<'d, D>, PrintingError> {
        let notation = DelayedConsolidatedNotation::new(doc);
        Ok(Seeker {
            width,
            prev: vec![Chunk {
                id: D::Id::default(),
                mark: None,
                notation: ConsolidatedNotation::Newline(0),
            }],
            next: vec![Chunk::new(notation)?],
        })
    }

    fn seek(
        mut self,
        path: &[usize],
    ) -> Result<(UpwardPrinter<'d, D>, DownwardPrinter<'d, D>), PrintingError> {
        use ConsolidatedNotation::*;

        span!("seek");

        // Seek to the descendant given by `path`.
        let mut path = path.iter();
        while let Some(child_index) = path.next() {
            self.seek_child(*child_index)?;
        }

        // Walk backward to the nearest Newline.
        self.seek_start_of_line();

        let upward_printer = UpwardPrinter {
            width: self.width,
            prev: self.prev,
            next: vec![],
        };
        let downward_printer = DownwardPrinter {
            width: self.width,
            next: self.next,
        };
        Ok((upward_printer, downward_printer))
    }

    fn seek_start_of_line(&mut self) {
        use ConsolidatedNotation::*;

        span!("seek_start_of_line");

        while let Some(chunk) = self.prev.pop() {
            match chunk.notation {
                Empty | Concat(_, _) | Choice(_, _) | Child(_, _) => {
                    unreachable!()
                }
                Literal(_) => self.next.push(chunk),
                Text(_, _) => self.next.push(chunk),
                Newline(_) => {
                    self.next.push(chunk);
                    break;
                }
            }
        }
    }

    fn seek_child(&mut self, child_index: usize) -> Result<(), PrintingError> {
        use ConsolidatedNotation::*;

        span!("seek_child");

        let parent_doc_id = self.next.last().unwrap().id;
        loop {
            // 1. Expand forward to the nearest `Choice` or `Child` belonging to `parent_doc`.
            //    (It would be more precise to look for Child(child_index) or a Choice
            //     containing it, but we can't tell right now what children a choice might
            //     contain.)
            while let Some(chunk) = self.next.pop() {
                match chunk.notation {
                    Empty => (),
                    Literal(_) | Newline(_) | Text(_, _) => self.prev.push(chunk),
                    Concat(left, right) => {
                        self.next.push(chunk.sub_chunk(right)?);
                        self.next.push(chunk.sub_chunk(left)?);
                    }
                    Choice(_, _) if chunk.id == parent_doc_id => {
                        self.next.push(chunk);
                        break;
                    }
                    // TODO: impossible case?
                    Choice(_, _) => self.prev.push(chunk),
                    Child(i, _) if chunk.id == parent_doc_id && i == child_index => {
                        self.next.push(chunk);
                        // TODO: can this just return, or do we need to expand any choices before
                        // it on the same line?
                        break;
                    }
                    Child(_, _) => self.prev.push(chunk),
                }
            }

            // 2. Walk backward to the nearest Newline (or beginning of the doc).
            let mut prefix_len = 0;
            while let Some(chunk) = self.prev.pop() {
                match chunk.notation {
                    Empty | Concat(_, _) => unreachable!(),
                    Literal(_) | Text(_, _) | Choice(_, _) | Child(_, _) => self.next.push(chunk),
                    Newline(indent) => {
                        prefix_len = indent;
                        self.prev.push(chunk);
                        break;
                    }
                }
            }

            // 3. Walk forward to the nearest Child or Choice, and resolve it. Go back to 1.
            //    If we hit `Child(i)` belonging to `parent_doc`, success.
            //    If we hit end of doc, panic (every child must be present).
            while let Some(chunk) = self.next.pop() {
                match chunk.notation {
                    Empty | Concat(_, _) | Newline(_) => unreachable!(),
                    Literal(lit) => {
                        prefix_len += lit.width();
                        self.prev.push(chunk);
                    }
                    Text(text, _style) => {
                        prefix_len += str_width(text);
                        self.prev.push(chunk);
                    }
                    Child(i, child) if chunk.id == parent_doc_id && i == child_index => {
                        // Found!
                        self.next.push(chunk.sub_chunk(child)?);

                        return Ok(());
                    }
                    Child(i, child) => {
                        self.next.push(chunk.sub_chunk(child)?);
                        break;
                    }
                    Choice(opt1, opt2) => {
                        let choice = choose(self.width, prefix_len, opt1, opt2, &self.next)?;
                        self.next.push(chunk.sub_chunk(choice)?);
                        break;
                    }
                }
            }
            if self.next.is_empty() {
                return Err(PrintingError::InvalidPath(child_index));
            }
        }
    }

    #[allow(unused)]
    fn display(&self) {
        for chunk in &self.prev {
            print!("{} ", chunk.notation);
        }
        print!(" / ");
        for chunk in self.next.iter().rev() {
            print!("{} ", chunk.notation);
        }
        println!();
    }
}

/// Constructed at an arbitrary position within the document. Prints lines from there one at a
/// time, going down.
struct DownwardPrinter<'d, D: PrettyDoc<'d>> {
    width: Width,
    next: Vec<Chunk<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> DownwardPrinter<'d, D> {
    fn print_first_line(&mut self) -> Result<Option<LineContents<'d, D>>, PrintingError> {
        use ConsolidatedNotation::*;

        span!("print_first_line");

        // We should be at the start of a line (in which case we look at the Newline's indentation
        // level to see how many spaces are at the start of this line), or at the very end of the
        // document (in which case our iteration is done).
        let indentation = if let Some(chunk) = self.next.pop() {
            match chunk.notation {
                Newline(num_spaces) => Indentation {
                    num_spaces,
                    doc_id: chunk.id,
                    mark: chunk.mark,
                },
                _ => panic!("Bug: print_first_line not invoked after newline"),
            }
        } else {
            return Ok(None);
        };

        let mut pieces = vec![];
        let mut prefix_len = indentation.num_spaces;
        while let Some(chunk) = self.next.pop() {
            match chunk.notation {
                Empty => (),
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
                Newline(indent) => {
                    self.next.push(chunk);
                    return Ok(Some(LineContents {
                        indentation,
                        pieces,
                    }));
                }
                Concat(left, right) => {
                    self.next.push(chunk.sub_chunk(right)?);
                    self.next.push(chunk.sub_chunk(left)?);
                }
                Choice(opt1, opt2) => {
                    let choice = choose(self.width, prefix_len, opt1, opt2, &self.next)?;
                    self.next.push(chunk.sub_chunk(choice)?);
                }
                Child(_, child_note) => self.next.push(chunk.sub_chunk(child_note)?),
            }
        }

        Ok(Some(LineContents {
            indentation,
            pieces,
        }))
    }

    #[allow(unused)]
    fn display(&self) {
        for chunk in self.next.iter().rev() {
            print!("{} ", chunk.notation);
        }
        println!();
    }
}

/// Constructed at an arbitrary position within the document. Prints lines from there one at a
/// time, going up.
// INVARIANT: `next` only ever contains `Literal`, `Text`, and `Choice` notations.
struct UpwardPrinter<'d, D: PrettyDoc<'d>> {
    width: Width,
    prev: Vec<Chunk<'d, D>>,
    next: Vec<Chunk<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> UpwardPrinter<'d, D> {
    fn print_last_line(&mut self) -> Result<Option<LineContents<'d, D>>, PrintingError> {
        use ConsolidatedNotation::*;

        span!("print_last_line");

        // 1. Go to the start of the "last line", and remember its indentation. However, the "last
        //    line" might not be fully expanded, and could contain hidden newlines in it.
        let mut prefix_len = match self.seek_start_of_line()? {
            None => return Ok(None),
            Some(prefix_len) => prefix_len,
        };

        // 2. Start expanding the "last line". If we encounter a choice, resolve it, but then seek
        //    back to the "start of the last line" again, as where that is might have changed if
        //    the choice contained a newline.
        while let Some(chunk) = self.next.pop() {
            match chunk.notation {
                Literal(lit) => {
                    prefix_len += lit.width();
                    self.prev.push(chunk);
                }
                Text(text, _style) => {
                    prefix_len += str_width(text);
                    self.prev.push(chunk);
                }
                Choice(opt1, opt2) => {
                    let choice = choose(self.width, prefix_len, opt1, opt2, &self.next)?;
                    self.prev.push(chunk.sub_chunk(choice)?);

                    // Reset everything. This is equivalent to a recursive call.
                    self.seek_end();
                    prefix_len = match self.seek_start_of_line()? {
                        None => return Ok(None),
                        Some(prefix_len) => prefix_len,
                    };
                }
                Empty | Newline(_) | Concat(_, _) | Child(_, _) => {
                    unreachable!()
                }
            }
        }

        let num_spaces = match self.seek_start_of_line()? {
            None => return Ok(None),
            Some(num_spaces) => num_spaces,
        };
        let newline_chunk = self.prev.pop().unwrap();
        let indentation = Indentation {
            num_spaces,
            doc_id: newline_chunk.id,
            mark: newline_chunk.mark,
        };

        let mut pieces = vec![];
        while let Some(chunk) = self.next.pop() {
            match chunk.notation {
                Literal(lit) => pieces.push(Piece {
                    str: lit.str(),
                    style: lit.style(),
                    doc_id: chunk.id,
                    mark: chunk.mark,
                }),
                Text(text, style) => pieces.push(Piece {
                    str: text,
                    style,
                    doc_id: chunk.id,
                    mark: chunk.mark,
                }),
                _ => panic!("Bug (display_line): expected only literals and text"),
            }
        }
        Ok(Some(LineContents {
            indentation,
            pieces,
        }))
    }

    fn seek_end(&mut self) {
        span!("seek_end");

        while let Some(chunk) = self.next.pop() {
            self.prev.push(chunk);
        }
    }

    /// Move the "printing cursor" to just after the previous newline. Returns None if there is
    /// no such newline, or Some of the newline's indentation if there is.
    // Maintains the invariant that `next` only ever contains `Literal`, `Text`, and `Choice` notations.
    fn seek_start_of_line(&mut self) -> Result<Option<Width>, PrintingError> {
        use ConsolidatedNotation::*;

        span!("seek_start_of_line");

        while let Some(chunk) = self.prev.pop() {
            match chunk.notation {
                Empty => (),
                Text(_, _) | Literal(_) => self.next.push(chunk),
                Newline(indent) => {
                    self.prev.push(chunk);
                    return Ok(Some(indent));
                }
                Concat(left, right) => {
                    self.prev.push(chunk.sub_chunk(left)?);
                    self.prev.push(chunk.sub_chunk(right)?);
                }
                Choice(_, _) => self.next.push(chunk),
                Child(_, note) => self.prev.push(chunk.sub_chunk(note)?),
            }
        }
        Ok(None)
    }

    #[allow(unused)]
    fn display(&self) {
        for chunk in &self.prev {
            print!("{} ", chunk.notation);
        }
        print!(" / ");
        for chunk in self.next.iter().rev() {
            print!("{} ", chunk.notation);
        }
        println!();
    }
}

/// Determine which of the two options of the choice to select. Pick the first option if it fits.
/// (We also want to pick the first option if we're inside a `Flat`, but ConsolidatedNotation already
/// took care of that.)
fn choose<'d, D: PrettyDoc<'d>>(
    width: Width,
    prefix_len: Width,
    opt1: DelayedConsolidatedNotation<'d, D>,
    opt2: DelayedConsolidatedNotation<'d, D>,
    next_chunks: &[Chunk<'d, D>],
) -> Result<DelayedConsolidatedNotation<'d, D>, PrintingError> {
    span!("choose");

    let opt1_evaled = opt1.eval()?.0;

    if width >= prefix_len && fits(width - prefix_len, opt1_evaled, next_chunks)? {
        Ok(opt1)
    } else {
        Ok(opt2)
    }
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
            Choice(opt1, opt2) => {
                // This assumes that:
                //     For every layout a in opt1 and b in opt2,
                //     first_line_len(a) >= first_line_len(b)
                // And also that ConsolidatedNotation would have removed this choice if we're in a Flat
                notations.push(opt2.eval()?.0);
            }
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Iterator for DownwardPrinter<'d, D> {
    type Item = Result<LineContents<'d, D>, PrintingError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.print_first_line().transpose()
    }
}

impl<'d, D: PrettyDoc<'d>> Iterator for UpwardPrinter<'d, D> {
    type Item = Result<LineContents<'d, D>, PrintingError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.print_last_line().transpose()
    }
}
