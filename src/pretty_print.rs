use crate::{
    consolidated_notation::{
        ConsolidatedNotation, DelayedConsolidatedNotation, IndentNode, Textual,
    },
    infra::span,
    PrettyDoc, PrintingError, Segment, Width,
};
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
/// Returns a tuple with three things:
///
/// - an iterator that prints lines above the focused line, going up
/// - the line containing the focus point
/// - an iterator that prints lines below the focused line, going down
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

    let mut printer = Printer::new(width)?;
    printer.seek(doc, path, seek_end)?;

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

/// Print the entirety of the document to a single string, ignoring styles.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to, find
/// a layout that fits withing that width.
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
}

impl<'d, D: PrettyDoc<'d>> Clone for Chunk<'d, D> {
    fn clone(&self) -> Self {
        Chunk {
            id: self.id,
            notation: self.notation.clone(),
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Chunk<'d, D> {
    fn new(notation: DelayedConsolidatedNotation<'d, D>) -> Result<Self, PrintingError> {
        Ok(Chunk {
            id: notation.doc().id(),
            notation: notation.eval()?,
        })
    }
}

/// The contents of a single pretty-printed line.
pub struct Line<'d, D: PrettyDoc<'d>> {
    /// A sequence of pieces of text to be displayed in order from left to right, with no spacing in
    /// between.
    pub segments: Vec<Segment<'d, D>>,
}

/// The contents of the pretty-printed line that contains the focus point.
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

/// A `Block` stores a partially-resolved piece of the document, separated from other `Block`s by
/// newlines. We start with notations in `chunks`, and then resolve them either by pushing text into
/// `segments` or by splitting the `Block` into two when we encounter a `Newline`.
///
/// Structure:
///     | segments ->| at_eol? |<- chunks |
///     ^^^^^^^^^^^^^^
///     prefix_len
struct Block<'d, D: PrettyDoc<'d>> {
    /// Stack of resolved text. The last element is the _rightmost_ text.
    segments: Vec<Segment<'d, D>>,
    /// The sum of the segment string widths.
    prefix_len: Width,
    /// Whether there is an `EndOfLine` between the `segments` and the `chunks`.
    at_eol: bool,
    /// Stack of unresolved notations. The last element is the _leftmost_ chunk.
    /// INVARIANT: These are "expanded chunks", meaning that their top-level notation may only be
    /// `Textual`, `Choice`, `Child`, or `EndOfLine`.
    chunks: Vec<Chunk<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> Block<'d, D> {
    fn new(indentation: Option<Rc<IndentNode<'d, D>>>, chunks: Vec<Chunk<'d, D>>) -> Block<'d, D> {
        let mut remaining_indentation = &indentation;
        let mut indent_segments = Vec::new();
        while let Some(indent_node) = remaining_indentation {
            indent_segments.push(indent_node.segment.clone());
            remaining_indentation = &indent_node.parent;
        }
        indent_segments.reverse();

        Block {
            prefix_len: indent_segments.iter().map(|seg| seg.width).sum(),
            segments: indent_segments,
            at_eol: false,
            chunks,
        }
    }

    fn push_text(&mut self, textual: Textual<'d, D>) -> Result<(), PrintingError> {
        if self.at_eol {
            return Err(PrintingError::TextAfterEndOfLine);
        }
        self.segments.push(Segment {
            str: textual.str,
            width: textual.width,
            style: textual.style,
        });
        self.prefix_len += textual.width;
        Ok(())
    }

    fn print(self) -> Line<'d, D> {
        assert!(self.chunks.is_empty());

        Line {
            segments: self.segments,
        }
    }
}

/// While seeking, the Printer has a "focus" at some position in the text. This focus is defined as
/// the boundary between `segments` and `chunks` of the top Block in `next_blocks`. The focus is
/// only defined while seeking, not during calls to `print_prev_line` and `print_next_line`.
///
/// | prev_blocks ->| (focus) |<- next_blocks|
struct Printer<'d, D: PrettyDoc<'d>> {
    /// Printing width
    width: Width,
    /// Stack of blocks before the focus. The last element is the previous line.
    prev_blocks: Vec<Block<'d, D>>,
    /// Stack of blocks after the focus. The last element is the next line.
    next_blocks: Vec<Block<'d, D>>,
}

impl<'d, D: PrettyDoc<'d>> Printer<'d, D> {
    fn new(width: Width) -> Result<Printer<'d, D>, PrintingError> {
        let empty_block = Block::new(None, Vec::new());
        Ok(Printer {
            width,
            prev_blocks: Vec::new(),
            next_blocks: vec![empty_block],
        })
    }

    /// Returns `None` if it already reached the bottom of the document.
    fn print_next_line(&mut self) -> Result<Option<Line<'d, D>>, PrintingError> {
        use ConsolidatedNotation::*;
        span!("print_next_line");

        let mut block = match self.next_blocks.pop() {
            None => return Ok(None),
            Some(block) => block,
        };
        while let Some(chunk) = block.chunks.pop() {
            match chunk.notation {
                Empty | Newline(_) | Concat(_, _) => {
                    panic!("bug in print_next_line: unexpanded chunk")
                }
                EndOfLine => block.at_eol = true,
                Textual(textual) => block.push_text(textual)?,
                Child(_, note) => {
                    self.expand_focusing_first_block(&mut block, Chunk::new(note)?)?
                }
                Choice(opt1, opt2) => {
                    let choice = self.choose(&block, opt1, opt2)?;
                    self.expand_focusing_first_block(&mut block, choice)?;
                }
            }
        }
        Ok(Some(block.print()))
    }

    /// Returns `None` if it already reached the top of the document.
    fn print_prev_line(&mut self) -> Result<Option<Line<'d, D>>, PrintingError> {
        use ConsolidatedNotation::*;
        span!("print_prev_line");

        let mut block = match self.prev_blocks.pop() {
            None => return Ok(None),
            Some(block) => block,
        };
        while let Some(chunk) = block.chunks.pop() {
            match chunk.notation {
                Empty | Newline(_) | Concat(_, _) => {
                    panic!("bug in print_prev_line: unexpanded chunk")
                }
                EndOfLine => block.at_eol = true,
                Textual(textual) => block.push_text(textual)?,
                Child(_, note) => self.expand_focusing_last_block(&mut block, Chunk::new(note)?)?,
                Choice(opt1, opt2) => {
                    let choice = self.choose(&block, opt1, opt2)?;
                    self.expand_focusing_last_block(&mut block, choice)?;
                }
            }
        }
        Ok(Some(block.print()))
    }

    /// Focus on the left or right of the node at the given path.
    /// (You don't want to seek twice.)
    fn seek(&mut self, doc: D, path: &[usize], seek_end: bool) -> Result<(), PrintingError> {
        span!("seek");

        let mut chunk = Chunk::new(DelayedConsolidatedNotation::new(doc))?;
        for child_index in path {
            chunk = self.seek_child(chunk, *child_index)?;
        }
        if seek_end {
            self.seek_end(chunk)
        } else {
            self.seek_start(chunk)
        }
    }

    /// Given an _unexpanded_ chunk that belongs at the focus, move the focus to just past the end
    /// of it.
    fn seek_end(&mut self, chunk: Chunk<'d, D>) -> Result<(), PrintingError> {
        use ConsolidatedNotation::*;
        span!("seek_end");

        // | segments ->| chunk |<- chunks |

        let mut block = self.next_blocks.pop().unwrap();
        let num_chunks_after = block.chunks.len();
        self.expand_focusing_last_block(&mut block, chunk)?;

        while block.chunks.len() > num_chunks_after {
            let chunk = block.chunks.pop().unwrap();
            match chunk.notation {
                Empty | Newline(_) | Concat(_, _) => panic!("bug in seek: unexpanded chunk"),
                EndOfLine => block.at_eol = true,
                Textual(textual) => block.push_text(textual)?,
                Child(_, note) => self.expand_focusing_last_block(&mut block, Chunk::new(note)?)?,
                Choice(opt1, opt2) => {
                    let choice = self.choose(&block, opt1, opt2)?;
                    self.expand_focusing_last_block(&mut block, choice)?;
                }
            }
        }
        self.next_blocks.push(block);
        Ok(())
    }

    /// Given an _unexpanded_ chunk that belongs at the focus, move the focus to just before the
    /// start of it.
    fn seek_start(&mut self, chunk: Chunk<'d, D>) -> Result<(), PrintingError> {
        span!("seek_start");

        let mut block = self.next_blocks.pop().unwrap();
        self.expand_focusing_first_block(&mut block, chunk)?;
        self.next_blocks.push(block);
        Ok(())
    }

    /// Given an _unexpanded_ chunk that belongs at the focus, find its i'th `Child` and return the
    /// notation that the child contains as another _unexpanded_ chunk. The focus will be around the
    /// location of that notation.
    fn seek_child(
        &mut self,
        parent: Chunk<'d, D>,
        child_index: usize,
    ) -> Result<Chunk<'d, D>, PrintingError> {
        use ConsolidatedNotation::*;
        span!("seek_child");

        let parent_id = parent.id;
        let mut block = self.next_blocks.pop().unwrap();
        self.expand_focusing_first_block(&mut block, parent)?;
        self.next_blocks.push(block);
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
                // We reached the end of the document without finding the target child.
                None => return Err(PrintingError::InvalidPath(child_index)),
            };

            // 2. Resolve the first Child or Choice. If we find `Child(child_index)`
            // belonging to `parent_doc`, success. Otherwise, go back to 1.
            while let Some(chunk) = block.chunks.pop() {
                match chunk.notation {
                    Empty | Newline(_) | Concat(_, _) => {
                        panic!("bug in seek_child: unexpanded chunk")
                    }
                    EndOfLine => block.at_eol = true,
                    Textual(textual) => block.push_text(textual)?,
                    Child(i, child) if chunk.id == parent_id && i == child_index => {
                        // Found!
                        self.next_blocks.push(block);
                        return Chunk::new(child);
                    }
                    Child(_i, child) => {
                        self.expand_focusing_first_block(&mut block, Chunk::new(child)?)?;
                        self.next_blocks.push(block);
                        break;
                    }
                    Choice(opt1, opt2) => {
                        let choice = self.choose(&block, opt1, opt2)?;
                        self.expand_focusing_first_block(&mut block, choice)?;
                        self.next_blocks.push(block);
                        break;
                    }
                }
            }
        }
    }

    /// Expand out all the `Empty`, `Newline`, and `Concat` notations in `chunk`. Whenever the block
    /// is split by a `Newline`, keep the focus on the _first_ block.
    ///
    /// Illustration:
    ///     aaaaa       aaaaa      aaaaa
    ///     a|*|B   ->  a|***  or  a|**B
    ///     BBBBB       ***BB      BBBBB
    ///                 BBBBB
    /// where `a`s are `block.segments`, `B`s are `block.chunks`, `*`s are
    /// `chunk`, and `|` is the focus.
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
                Textual(_) | Choice(_, _) | Child(_, _) | EndOfLine => block.chunks.push(chunk),
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

    /// Expand out all the `Empty`, `Newline`, and `Concat` notations in `chunk`. Whenever the block
    /// is split by a `Newline`, keep the focus on the _last_ block.
    ///
    /// Illustration:
    ///     aaaaa      aaaaa      aaaaa
    ///     a|*|B  ->  a****  or  a|**B
    ///     BBBBB      |***B      BBBBB
    ///                BBBBB
    /// where `a`s are `block.segments`, `B`s are `block.chunks`, `*`s are
    /// `chunk`, and `|` is the focus.
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
                Textual(_) | Choice(_, _) | Child(_, _) | EndOfLine => chunks.push(chunk),
                Newline(indentation) => {
                    chunks.reverse();
                    let prev_block = Block {
                        segments: mem::take(&mut block.segments),
                        prefix_len: block.prefix_len,
                        at_eol: block.at_eol,
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

    /// Determine which of the two options of the choice to select. Pick the first option if it
    /// fits. (We also want to pick the first option if we're inside a `Flat`, but
    /// ConsolidatedNotation already took care of that.)
    fn choose(
        &self,
        block: &Block<'d, D>,
        opt1: DelayedConsolidatedNotation<'d, D>,
        opt2: DelayedConsolidatedNotation<'d, D>,
    ) -> Result<Chunk<'d, D>, PrintingError> {
        span!("choose");

        let chunk1 = Chunk::new(opt1)?;

        if self.width >= block.prefix_len
            && fits(
                self.width - block.prefix_len,
                block.at_eol,
                chunk1.notation.clone(),
                &block.chunks,
            )?
        {
            Ok(chunk1)
        } else {
            Chunk::new(opt2)
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
/// the available `width`, and does not cause there to be an EOL followed by text.
fn fits<'d, D: PrettyDoc<'d>>(
    width: Width,
    at_eol: bool,
    notation: ConsolidatedNotation<'d, D>,
    next_chunks: &[Chunk<'d, D>],
) -> Result<bool, PrintingError> {
    use ConsolidatedNotation::*;
    span!("fits");

    let mut next_chunks = next_chunks;
    let mut remaining = width;
    let mut notations = vec![notation];
    let mut at_eol = at_eol;

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
                if at_eol {
                    return Ok(false);
                }
                if textual.width <= remaining {
                    remaining -= textual.width;
                } else {
                    return Ok(false);
                }
            }
            EndOfLine => at_eol = true,
            Newline(_) => return Ok(true),
            Child(_, note) => notations.push(note.eval()?),
            Concat(note1, note2) => {
                notations.push(note2.eval()?);
                notations.push(note1.eval()?);
            }
            Choice(_opt1, opt2) => {
                // This assumes that for every layout A in opt1 and layout B in opt2:
                //     - first_line_len(A) >= first_line_len(B)
                //     - if first_line_len(A) is not None, then first_line_len(B) is not None
                // And also assumes that ConsolidatedNotation would have already removed this choice
                // by picking opt1 if we're in a Flat
                notations.push(opt2.eval()?);
            }
        }
    }
}

/// An iterator for printing lines above the focused line.
struct UpwardPrinter<'d, D: PrettyDoc<'d>>(Printer<'d, D>);

impl<'d, D: PrettyDoc<'d>> Iterator for UpwardPrinter<'d, D> {
    type Item = Result<Line<'d, D>, PrintingError>;

    fn next(&mut self) -> Option<Result<Line<'d, D>, PrintingError>> {
        self.0.print_prev_line().transpose()
    }
}

/// An iterator for printing lines below the focused line.
struct DownwardPrinter<'d, D: PrettyDoc<'d>>(Printer<'d, D>);

impl<'d, D: PrettyDoc<'d>> Iterator for DownwardPrinter<'d, D> {
    type Item = Result<Line<'d, D>, PrintingError>;

    fn next(&mut self) -> Option<Result<Line<'d, D>, PrintingError>> {
        self.0.print_next_line().transpose()
    }
}
