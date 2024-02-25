use super::consolidated_notation::{
    ConsolidatedNotation, DelayedConsolidatedNotation, IndentNode, PrintingError, Segment, Textual,
};
use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::infra::span;
use std::convert::From;
use std::iter::Iterator;
use std::mem;
use std::rc::Rc;

/// Pretty print a document, focused to the left or right of the node found by traversing `path`
/// from the root. If `seek_end` is true, focus will be to the right of the node, otherwise to the
/// left.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to,
/// find a layout that fits within that width.
///
/// `marks` is a set of document node ids to mark. Each chunk of text in the output will say which,
/// if any, marked id it is part of.
///
/// Returns a tuple with three things:
///
/// - an iterator that prints lines above the focused line going up
/// - the line containing the focus point
/// - an iterator that prints lines below the focused line going down
///
/// It is expected that you will take only as many lines as you need from the iterators; doing so
/// will save computation time.
pub fn pretty_print<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
    path: &[usize],
    seek_end: bool,
) -> Result<
    (
        impl Iterator<Item = Result<Line<'d, D>, PrintingError>>,
        FocusedLine<'d, D>,
        impl Iterator<Item = Result<Line<'d, D>, PrintingError>>,
    ),
    PrintingError,
> {
    span!("Pretty Print");

    let mut printer = Printer::new(doc, width)?;
    printer.seek(path, seek_end)?;

    let num_left_segs = printer.next_blocks.last().unwrap().segments.len();
    let mut line = printer.print_next_line()?.unwrap();
    let focused_line = FocusedLine {
        right_segments: line.segments.split_off(num_left_segs),
        left_segments: line.segments,
    };

    let upward_printer = UpwardPrinter(Printer {
        width,
        prev_blocks: printer.prev_blocks,
        next_blocks: Vec::new(),
    });

    let downward_printer = DownwardPrinter(Printer {
        width,
        prev_blocks: Vec::new(),
        next_blocks: printer.next_blocks,
    });

    Ok((upward_printer, focused_line, downward_printer))
}

/// Print the entirety of the document to a single string, ignoring styles and shading.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to,
/// find a layout that fits withing that width.
pub fn pretty_print_to_string<'d, D: PrettyDoc<'d>>(
    doc: D,
    width: Width,
) -> Result<String, PrintingError> {
    let (_, focused_line, lines_iter) = pretty_print(doc, width, &[], false)?;
    let mut string = focused_line.to_string();
    for line in lines_iter {
        string.push('\n');
        string.push_str(&line?.to_string());
    }
    Ok(string)
}

struct Chunk<'d, D: PrettyDoc<'d>> {
    notation: ConsolidatedNotation<'d, D>,
    id: D::Id,
    mark: Option<&'d D::Mark>,
}

impl<'d, D: PrettyDoc<'d>> Clone for Chunk<'d, D> {
    fn clone(&self) -> Self {
        Chunk {
            id: self.id,
            mark: self.mark,
            notation: self.notation.clone(),
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Chunk<'d, D> {
    fn new(notation: DelayedConsolidatedNotation<'d, D>) -> Result<Self, PrintingError> {
        let id = notation.doc().id();
        let (notation, mark) = notation.eval()?;
        Ok(Chunk { id, mark, notation })
    }
}

// TODO: Make this struct Line(Vec<Segment>)?
/// The contents of a single pretty printed line.
pub struct Line<'d, D: PrettyDoc<'d>> {
    /// A sequence of pieces of text to be displayed in order from left to
    /// right, with no spacing in between.
    pub segments: Vec<Segment<'d, D>>,
}

/// The contents of the line containing the focus point.
pub struct FocusedLine<'d, D: PrettyDoc<'d>> {
    /// Pieces of text that appear before the focus point.
    pub left_segments: Vec<Segment<'d, D>>,
    /// Pieces of text that appear after the focus point.
    pub right_segments: Vec<Segment<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> Line<'d, D> {
    pub fn width(&self) -> Width {
        self.segments.iter().map(|seg| seg.width).sum()
    }
}

impl<'d, D: PrettyDoc<'d>> FocusedLine<'d, D> {
    pub fn width(&self) -> Width {
        let segs = self.left_segments.iter().chain(self.right_segments.iter());
        segs.map(|seg| seg.width).sum()
    }

    pub fn to_left_string(&self) -> String {
        let mut string = String::new();
        for segment in &self.left_segments {
            string.push_str(segment.str);
        }
        string
    }

    pub fn to_right_string(&self) -> String {
        let mut string = String::new();
        for segment in &self.right_segments {
            string.push_str(segment.str);
        }
        string
    }
}

impl<'d, D: PrettyDoc<'d>> From<FocusedLine<'d, D>> for Line<'d, D> {
    fn from(focused_line: FocusedLine<'d, D>) -> Line<'d, D> {
        let mut segments = focused_line.left_segments;
        segments.extend(focused_line.right_segments);
        Line { segments }
    }
}

impl<'d, D: PrettyDoc<'d>> ToString for Line<'d, D> {
    fn to_string(&self) -> String {
        span!("Line::to_string");

        let mut string = String::new();
        for segment in &self.segments {
            string.push_str(segment.str);
        }
        string
    }
}

impl<'d, D: PrettyDoc<'d>> ToString for FocusedLine<'d, D> {
    fn to_string(&self) -> String {
        span!("FocusedLine::to_string");

        let mut string = String::new();
        for segment in self.left_segments.iter().chain(self.right_segments.iter()) {
            string.push_str(segment.str);
        }
        string
    }
}

/// INVARIANTS:
/// - prefix_len is always sum(segment width)
/// - chunks only contains Textual, Choice, Child
///
/// | segments ->|<- chunks |
/// ^^^^^^^^^^^^^^
/// prefix_len
struct Block<'d, D: PrettyDoc<'d>> {
    /// Resolved text. Last element is the rightmost text.
    segments: Vec<Segment<'d, D>>,
    prefix_len: Width,
    /// Unresolved notations. Last element is the _leftmost_ chunk.
    chunks: Vec<Chunk<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> Block<'d, D> {
    // TODO: Make this &IndentNode for efficiency
    fn new(indentation: Option<Rc<IndentNode<'d, D>>>, chunks: Vec<Chunk<'d, D>>) -> Block<'d, D> {
        let mut remaining_indentation = &indentation;
        let mut indent_segments = Vec::new();
        while let Some(indent_node) = remaining_indentation {
            indent_segments.push(indent_node.segment);
            remaining_indentation = &indent_node.parent;
        }
        indent_segments.reverse();

        Block {
            prefix_len: indent_segments.iter().map(|seg| seg.width).sum(),
            segments: indent_segments,
            chunks,
        }
    }

    fn push_text(&mut self, doc_id: D::Id, mark: Option<&'d D::Mark>, textual: Textual<'d, D>) {
        self.segments.push(Segment {
            str: textual.str,
            width: textual.width,
            style: textual.style,
            doc_id,
            mark,
        });
        self.prefix_len += textual.width;
    }

    fn print(self) -> Line<'d, D> {
        assert!(self.chunks.is_empty());

        Line {
            segments: self.segments,
        }
    }
}

/// While seeking, the Printer has a "focus" at some position in the text.
/// This focus is defined as the boundary between `segments` and `chunks`
/// of the top Block in `next_blocks`. The focus is only defined while
/// seeking, not during calls to `print_prev_line` and `print_next_line`.
struct Printer<'d, D: PrettyDoc<'d>> {
    /// Printing width
    width: Width,
    /// Stack of blocks before the focus. The last element is the previous line.
    prev_blocks: Vec<Block<'d, D>>,
    /// Stack of blocks after the focus. The last element is the next line.
    next_blocks: Vec<Block<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> Printer<'d, D> {
    fn new(doc: D, width: Width) -> Result<Printer<'d, D>, PrintingError> {
        let chunk = Chunk::new(DelayedConsolidatedNotation::new(doc))?;
        let mut block = Block::new(None, Vec::new());
        let mut printer = Printer {
            width,
            prev_blocks: Vec::new(),
            next_blocks: Vec::new(),
        };
        printer.expand_focusing_first_block(&mut block, chunk)?;
        printer.next_blocks.push(block);
        Ok(printer)
    }

    fn print_next_line(&mut self) -> Result<Option<Line<'d, D>>, PrintingError> {
        use ConsolidatedNotation::*;
        span!("print_next_line");

        let mut block = match self.next_blocks.pop() {
            None => return Ok(None),
            Some(block) => block,
        };
        while let Some(chunk) = block.chunks.pop() {
            match chunk.notation {
                Empty | Newline(_) | Concat(_, _) => panic!("bug in print_next_line"),
                Textual(textual) => block.push_text(chunk.id, chunk.mark, textual),
                Child(_, note) => {
                    self.expand_focusing_first_block(&mut block, Chunk::new(note)?)?
                }
                Choice(opt1, opt2) => {
                    let choice = self.choose(&block, opt1, opt2)?;
                    self.expand_focusing_first_block(&mut block, Chunk::new(choice)?)?;
                }
            }
        }
        Ok(Some(block.print()))
    }

    fn print_prev_line(&mut self) -> Result<Option<Line<'d, D>>, PrintingError> {
        use ConsolidatedNotation::*;
        span!("print_next_line");

        let mut block = match self.prev_blocks.pop() {
            None => return Ok(None),
            Some(block) => block,
        };
        while let Some(chunk) = block.chunks.pop() {
            match chunk.notation {
                Empty | Newline(_) | Concat(_, _) => panic!("bug in print_prev_line"),
                Textual(textual) => block.push_text(chunk.id, chunk.mark, textual),
                Child(_, note) => self.expand_focusing_last_block(&mut block, Chunk::new(note)?)?,
                Choice(opt1, opt2) => {
                    let choice = self.choose(&block, opt1, opt2)?;
                    self.expand_focusing_last_block(&mut block, Chunk::new(choice)?)?;
                }
            }
        }
        Ok(Some(block.print()))
    }

    /// Moves the focus to the left or right of the node at the given path,
    /// relative to the node to the right of the current focus.
    /// (You don't want to seek twice.)
    fn seek(&mut self, path: &[usize], seek_end: bool) -> Result<(), PrintingError> {
        use ConsolidatedNotation::*;
        span!("seek");

        for (i, child_index) in path.iter().enumerate() {
            self.seek_child(*child_index, i + 1 < path.len())?;
        }
        if seek_end && path.is_empty() {
            while self.next_blocks.len() > 1 {
                let block = self.next_blocks.pop().unwrap();
                self.prev_blocks.push(block);
            }
        }
        if seek_end {
            let mut block = self.next_blocks.pop().unwrap();
            let num_chunks_after = if path.is_empty() {
                0
            } else {
                block.chunks.len() - 1
            };
            while block.chunks.len() > num_chunks_after {
                let chunk = block.chunks.pop().unwrap();
                match chunk.notation {
                    Empty | Newline(_) | Concat(_, _) => panic!("bug in seek"),
                    Textual(textual) => block.push_text(chunk.id, chunk.mark, textual),
                    Child(_, note) => {
                        self.expand_focusing_last_block(&mut block, Chunk::new(note)?)?
                    }
                    Choice(opt1, opt2) => {
                        let choice = self.choose(&block, opt1, opt2)?;
                        self.expand_focusing_last_block(&mut block, Chunk::new(choice)?)?;
                    }
                }
            }
            self.next_blocks.push(block);
        }
        Ok(())
    }

    // 1. A B C

    /// Moves the focus to the left of the i'th child of the chunk that is immediately to the right
    /// of the current focus.
    fn seek_child(&mut self, child_index: usize, expand_child: bool) -> Result<(), PrintingError> {
        use ConsolidatedNotation::*;
        span!("seek_child");

        // Get the doc node id of the chunk that is immediately to the right of the current focus
        // point
        let parent_id = self.next_blocks.last().unwrap().chunks.last().unwrap().id;
        loop {
            // 1. Walk forward to the block containing the nearest `Choice` or `Child`
            // belonging to `parent_doc`.
            // (It would be more precise to look for Child(child_index) or a Choice
            // containing it, but we can't tell right now what children a choice might
            // contain.)
            let contains_relevant_chunk = |block: &Block<'d, D>| -> bool {
                block.chunks.iter().any(|chunk| match chunk.notation {
                    Child(i, _) => chunk.id == parent_id && i == child_index,
                    Choice(_, _) => chunk.id == parent_id,
                    _ => false,
                })
            };
            while let Some(block) = self.next_blocks.pop() {
                if contains_relevant_chunk(&block) {
                    self.next_blocks.push(block);
                    break;
                } else {
                    self.prev_blocks.push(block);
                }
            }
            let mut block = match self.next_blocks.pop() {
                Some(block) => block,
                None => return Err(PrintingError::InvalidPath(child_index)),
            };

            // 2. Resolve the first Child or Choice. Go back to 1.
            // If we hit `Child(i)` belonging to `parent_doc`, success.
            // If we hit end of doc, panic (every child must be present).
            while let Some(chunk) = block.chunks.pop() {
                match chunk.notation {
                    Empty | Newline(_) | Concat(_, _) => panic!("bug in seek_child"),
                    Textual(textual) => block.push_text(chunk.id, chunk.mark, textual),
                    Child(i, child) if chunk.id == parent_id && i == child_index => {
                        // Found!
                        if expand_child {
                            self.expand_focusing_first_block(&mut block, Chunk::new(child)?)?;
                        } else {
                            // Put the chunk back. Can't clone directly b.c. partly moved.
                            block.chunks.push(Chunk {
                                notation: Child(i, child),
                                id: chunk.id,
                                mark: chunk.mark,
                            });
                        }
                        self.next_blocks.push(block);
                        return Ok(());
                    }
                    Child(_i, child) => {
                        self.expand_focusing_first_block(&mut block, Chunk::new(child)?)?;
                        self.next_blocks.push(block);
                        break;
                    }
                    Choice(opt1, opt2) => {
                        let choice = self.choose(&block, opt1, opt2)?;
                        self.expand_focusing_first_block(&mut block, Chunk::new(choice)?)?;
                        self.next_blocks.push(block);
                        break;
                    }
                }
            }
        }
    }

    // aaaaa       aaaaa      aaaaa
    // a|*|B   ->  a|***  or  a|**B
    // BBBBB       ***BB      BBBBB
    //             BBBBB
    /// Expand all Empty, Newline, and Concats in `chunk`, keeping the focus on the
    /// first block of the expanded stuff.
    fn expand_focusing_first_block(
        &mut self,
        block: &mut Block<'d, D>,
        chunk: Chunk<'d, D>,
    ) -> Result<(), PrintingError> {
        use ConsolidatedNotation::*;
        span!("expand_first");

        // | block.segments ->| stack ->|<- block.chunks |
        let mut stack = vec![chunk];
        while let Some(chunk) = stack.pop() {
            match chunk.notation {
                Empty => (),
                Textual(_) | Choice(_, _) | Child(_, _) => block.chunks.push(chunk),
                Newline(indentation) => {
                    let chunks = mem::take(&mut block.chunks);
                    self.next_blocks.push(Block::new(indentation, chunks));
                }
                Concat(left, right) => {
                    stack.push(Chunk::new(left)?);
                    stack.push(Chunk::new(right)?);
                }
            }
        }
        Ok(())
    }

    // aaaaa      aaaaa      aaaaa
    // a|*|B  ->  a****  or  a|**B
    // BBBBB      |***B      BBBBB
    //            BBBBB
    /// Expand all Empty, Newline, and Concats in `chunk`, keeping the focus on the
    /// last block of the expanded stuff.
    fn expand_focusing_last_block(
        &mut self,
        block: &mut Block<'d, D>,
        chunk: Chunk<'d, D>,
    ) -> Result<(), PrintingError> {
        use ConsolidatedNotation::*;
        span!("expand_last");

        // | block.segments ->| chunks ->|<- stack |<- block.chunks |
        let mut chunks = Vec::new();
        let mut stack = vec![chunk];
        while let Some(chunk) = stack.pop() {
            match chunk.notation {
                Empty => (),
                Textual(_) | Choice(_, _) | Child(_, _) => chunks.push(chunk),
                Newline(indentation) => {
                    chunks.reverse();
                    let prev_block = Block {
                        segments: mem::take(&mut block.segments),
                        prefix_len: block.prefix_len,
                        chunks: mem::take(&mut chunks),
                    };
                    self.prev_blocks.push(prev_block);
                    *block = Block::new(indentation, mem::take(&mut block.chunks));
                }
                Concat(left, right) => {
                    stack.push(Chunk::new(right)?);
                    stack.push(Chunk::new(left)?);
                }
            }
        }
        chunks.reverse();
        block.chunks.extend(mem::take(&mut chunks));
        Ok(())
    }

    /// Determine which of the two options of the choice to select. Pick the first option if it fits.
    /// (We also want to pick the first option if we're inside a `Flat`, but ConsolidatedNotation already
    /// took care of that.)
    fn choose(
        &self,
        block: &Block<'d, D>,
        opt1: DelayedConsolidatedNotation<'d, D>,
        opt2: DelayedConsolidatedNotation<'d, D>,
    ) -> Result<DelayedConsolidatedNotation<'d, D>, PrintingError> {
        span!("choose");

        let opt1_evaled = opt1.clone().eval()?.0;

        if self.width >= block.prefix_len
            && fits(self.width - block.prefix_len, opt1_evaled, &block.chunks)?
        {
            Ok(opt1)
        } else {
            Ok(opt2)
        }
    }

    #[allow(unused)]
    fn debug_long(&self) {
        for block in &self.prev_blocks {
            for segment in &block.segments {
                print!("'{}' ", segment.str);
            }
            for chunk in &block.chunks {
                print!("{} ", chunk.notation);
            }
        }
        print!(" / ");
        for block in self.next_blocks.iter().rev() {
            for segment in &block.segments {
                print!("'{}' ", segment.str);
            }
            for chunk in &block.chunks {
                print!("{} ", chunk.notation);
            }
        }
        println!();
    }

    #[allow(unused)]
    pub fn debug_short(&self) {
        fn print_block<'d, D: PrettyDoc<'d>>(block: &Block<'d, D>) {
            println!(
                "{}|{}",
                block
                    .segments
                    .iter()
                    .map(|seg| seg.str)
                    .fold(String::new(), |a, b| a + b),
                block.chunks.len(),
            );
        }

        for block in &self.prev_blocks {
            print_block(block);
        }
        println!("---focus---");
        if let Some(focused_block) = self.next_blocks.last() {
            print_block(focused_block);
        }
        println!("-----------");
        for block in self.next_blocks.iter().rev().skip(1) {
            print_block(block);
        }
        println!();
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
                    chunk.notation.clone()
                }
            },
        };

        match notation {
            Empty => (),
            Textual(textual) => {
                if textual.width <= remaining {
                    remaining -= textual.width;
                } else {
                    return Ok(false);
                }
            }
            Newline(_) => return Ok(true),
            Child(_, note) => notations.push(note.clone().eval()?.0),
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

struct UpwardPrinter<'d, D: PrettyDoc<'d>>(Printer<'d, D>);

impl<'d, D: PrettyDoc<'d>> Iterator for UpwardPrinter<'d, D> {
    type Item = Result<Line<'d, D>, PrintingError>;

    fn next(&mut self) -> Option<Result<Line<'d, D>, PrintingError>> {
        self.0.print_prev_line().transpose()
    }
}

struct DownwardPrinter<'d, D: PrettyDoc<'d>>(Printer<'d, D>);

impl<'d, D: PrettyDoc<'d>> Iterator for DownwardPrinter<'d, D> {
    type Item = Result<Line<'d, D>, PrintingError>;

    fn next(&mut self) -> Option<Result<Line<'d, D>, PrintingError>> {
        self.0.print_next_line().transpose()
    }
}
