use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::style::{Shade, Style};
use std::iter::{self, Iterator};
use std::mem;

/*******
 * API *
 *******/

pub type TextFragment<'d> = (&'d str, Style, Shade);

/// The contents of a single pretty printed line.
pub struct LineContents<'d> {
    /// The indentation of this line in spaces, and the shade of those spaces.
    pub spaces: (Width, Shade),
    /// A sequence of (string, style, shade) triples, to be displayed after `spaces`, in order from
    /// left to right, with no spacing in between.
    pub contents: Vec<TextFragment<'d>>,
}

/// Pretty print a document, focused at the node found by traversing `path` from the root.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to,
/// find a layout that fits withing that width.
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
) -> (
    impl Iterator<Item = LineContents<'d>>,
    impl Iterator<Item = LineContents<'d>>,
) {
    let notation = NotationRef::new(doc);
    let seeker = Seeker::new(notation, width);
    seeker.seek(path)
}

/// Print the entirety of the document to a single string, ignoring styles and shading.
///
/// `width` is the desired line width. The algorithm will attempt to, but is not guaranteed to,
/// find a layout that fits withing that width.
pub fn pretty_print_to_string<'d, D: PrettyDoc<'d>>(doc: D, width: Width) -> String {
    let (_, mut lines_iter) = pretty_print(doc, width, &[]);
    let mut string = lines_iter.next().unwrap().to_string();
    for line in lines_iter {
        string.push('\n');
        string.push_str(&line.to_string());
    }
    string
}

impl<'d> ToString for LineContents<'d> {
    fn to_string(&self) -> String {
        let mut string = format!("{:spaces$}", "", spaces = self.spaces.0 as usize);
        for (text, _style, _hl) in &self.contents {
            string.push_str(text);
        }
        string
    }
}

/************
 * BlockSet *
 ************/

/// (indent, is_in_cursor, notation)
type Chunk<'d, D> = (Option<Width>, Shade, NotationRef<'d, D>);

/// Means `Indent(spaces, Newline) + text + chunks`.
struct Block<'d, D: PrettyDoc<'d>> {
    spaces: (Width, Shade),
    text: Vec<TextFragment<'d>>,
    text_len: Width,
    // INVARIANT: Only contains Lit, Text, Choice, and Child. And Lit/Text cannot be leftmost.
    chunks: Vec<Chunk<'d, D>>,
}

struct BlockSet<'d, D: PrettyDoc<'d>> {
    width: Width,
    prev: Vec<Block<'d, D>>,
    block: Block<'d, D>,
    next: Vec<Block<'d, D>>,
}

#[derive(Clone, Debug)]
pub struct FirstLineLen {
    pub len: Width,
    pub has_newline: bool,
}

enum Direction {
    Up,
    Down,
}

impl<'d, D: PrettyDoc<'d>> Block<'d, D> {
    fn empty() -> Block<'d, D> {
        Block {
            spaces: (0, Shade::background()),
            text: vec![],
            text_len: 0,
            chunks: vec![],
        }
    }

    fn new(spaces: (Width, Shade)) -> Block<'d, D> {
        Block {
            spaces,
            text: vec![],
            text_len: 0,
            chunks: vec![],
        }
    }

    fn to_line_contents(self) -> LineContents<'d> {
        assert!(self.chunks.is_empty());
        LineContents {
            spaces: self.spaces,
            contents: self.text,
        }
    }

    /// Maintain the invariant by eliminating all leftmost Literal and Texts.
    fn collect_text(&mut self) {
        use NotationCase::*;

        while let Some((indent, shade, note)) = self.chunks.pop() {
            match note.case() {
                Child(_, _) | Choice(_, _) => {
                    self.chunks.push((indent, shade, note));
                    return;
                }
                Literal(lit) => {
                    self.text.push((lit.str(), lit.style(), shade));
                    self.text_len += lit.len();
                }
                Text(text, style) => {
                    self.text.push((text, style, shade));
                    self.text_len += text.chars().count() as Width;
                }
                _ => unreachable!(),
            }
        }
    }
}

impl<'d, D: PrettyDoc<'d>> BlockSet<'d, D> {
    fn new(notation: NotationRef<'d, D>, width: Width) -> BlockSet<'d, D> {
        let mut blocks = BlockSet {
            width,
            prev: vec![],
            block: Block::empty(),
            next: vec![],
        };
        blocks.insert_chunk((Some(0), Shade::background(), notation));
        blocks
    }

    /// Insert a chunk between `text` and `chunks`.
    fn insert_chunk(&mut self, chunk: Chunk<'d, D>) {
        use NotationCase::*;

        // Order: self.block.spaces  self.block.text->  stack->  <-self.block.chunks

        // Avoid recursion to make sure we don't blow the stack.
        let mut stack = vec![chunk];
        while let Some((indent, shade, note)) = stack.pop() {
            match note.case() {
                Empty => (),
                Literal(_) | Text(_, _) => {
                    self.block.chunks.push((indent, shade, note));
                }
                Newline => {
                    let mut next_block = Block::new((indent.unwrap(), shade));
                    next_block.chunks = mem::take(&mut self.block.chunks);
                    next_block.collect_text();
                    self.next.push(next_block);
                }
                Indent(j, note) => stack.push((indent.map(|i| i + j), shade, note)),
                Flat(note) => stack.push((None, shade, note)),
                Concat(left, right) => {
                    stack.push((indent, shade, left));
                    stack.push((indent, shade, right));
                }
                Child(_, _) => self.block.chunks.push((indent, shade, note)),
                Choice(_, _) => self.block.chunks.push((indent, shade, note)),
            }
        }
        self.block.collect_text();
    }

    /// Resolve the first chunk (which must be a Child or Choice) in the current block. Returns
    /// whether there was a chunk to resolve.
    fn resolve_first_chunk(&mut self) -> bool {
        use NotationCase::*;

        if let Some((indent, shade, note)) = self.block.chunks.pop() {
            match note.case() {
                Child(_, child_note) => self.insert_chunk((indent, shade, child_note)),
                Choice(opt1, opt2) => {
                    let note = resolve_choice(
                        self.width,
                        self.block.text_len,
                        indent,
                        opt1,
                        opt2,
                        &self.block.chunks,
                    );
                    self.insert_chunk((indent, shade, note));
                }
                _ => unreachable!(),
            }
            true
        } else {
            false
        }
    }

    /// Move forward to the first block that contains a notation that passes the predicate.
    fn find_block(&mut self, predicate: &dyn Fn(NotationRef<'d, D>) -> bool) {
        loop {
            for chunk in &self.block.chunks {
                if predicate(chunk.2) {
                    return;
                }
            }
            if !self.goto_next_block() {
                return;
            }
        }
    }

    /// Move forward one block. Returns whether the move was successful (i.e., false if we were
    /// already at the last block).
    fn goto_next_block(&mut self) -> bool {
        if let Some(next_block) = self.next.pop() {
            self.prev.push(mem::replace(&mut self.block, next_block));
            true
        } else {
            false
        }
    }

    /// Move forward to the very last block.
    fn goto_last_block(&mut self) {
        while self.goto_next_block() {}
    }

    /// Remove and return the current block, either advancing to the next block or receding to the
    /// previous. Returns the block, and whether this was the last remaining block.
    fn take_block(&mut self, direction: Direction) -> (Block<'d, D>, bool) {
        let source = match direction {
            Direction::Up => &mut self.prev,
            Direction::Down => &mut self.next,
        };
        let (next_block, all_out_of_blocks) = match source.pop() {
            Some(next_block) => (next_block, false),
            None => (Block::empty(), true),
        };
        let block = mem::replace(&mut self.block, next_block);
        (block, all_out_of_blocks)
    }

    fn first_chunk(&self) -> Option<&Chunk<'d, D>> {
        self.block.chunks.last()
    }

    // fn first_chunk_mut(&mut self) -> Option<&mut Chunk<'d, D>> {
    //     self.block.chunks.last_mut()
    // }
}

/// Determine which of the two options of the choice to select. Pick the first option if it fits,
/// or if the second option is invalid.
fn resolve_choice<'d, D: PrettyDoc<'d>>(
    width: Width,
    prefix_len: Width,
    indent: Option<Width>,
    opt1: NotationRef<'d, D>,
    opt2: NotationRef<'d, D>,
    suffix: &[Chunk<'d, D>],
) -> NotationRef<'d, D> {
    let flat = indent.is_none();
    let chunks: Vec<(bool, NotationRef<'d, D>)> = iter::once((flat, opt1))
        .chain(suffix.iter().map(|(i, _, n)| (i.is_none(), *n)))
        .collect();
    if fits(width.saturating_sub(prefix_len), chunks) || !is_valid(flat, opt2) {
        opt1
    } else {
        opt2
    }
}

/// Determine whether the first line of the chunks fits within the `remaining` space.
fn fits<'d, D: PrettyDoc<'d>>(
    mut remaining: Width,
    mut chunks: Vec<(bool, NotationRef<'d, D>)>,
) -> bool {
    use NotationCase::*;

    while let Some((flat, notation)) = chunks.pop() {
        match notation.case() {
            Empty => (),
            Literal(lit) => {
                let lit_len = lit.len();
                if lit_len <= remaining {
                    remaining -= lit_len;
                } else {
                    return false;
                }
            }
            Text(text, _) => {
                let text_len = text.chars().count() as Width;
                if text_len <= remaining {
                    remaining -= text_len;
                } else {
                    return false;
                }
            }
            Newline if flat => return false,
            Newline => (),
            Flat(note) => chunks.push((true, note)),
            Indent(_, note) => chunks.push((flat, note)),
            Child(_, note) => chunks.push((flat, note)),
            Concat(note1, note2) => {
                chunks.push((flat, note2));
                chunks.push((flat, note1));
            }
            Choice(opt1, opt2) => {
                // opt2 must always be strictly smaller!
                // TODO: As an optimization, pre-compute whether opt2 has an unconditional newline
                if is_valid(flat, opt2) {
                    chunks.push((flat, opt2));
                } else {
                    chunks.push((flat, opt1));
                }
            }
        }
    }
    true
}

fn is_valid<'d, D: PrettyDoc<'d>>(flat: bool, notation: NotationRef<'d, D>) -> bool {
    use NotationCase::*;

    match notation.case() {
        Empty | Literal(_) | Text(_, _) => true,
        Newline => !flat,
        Flat(note) => is_valid(true, note),
        Indent(_, note) => is_valid(flat, note),
        // TODO: As an optimization, pre-compute whether opt2 has an unconditional newline
        Choice(opt1, opt2) => is_valid(flat, opt1) || is_valid(flat, opt2),
        Concat(note1, note2) => is_valid(flat, note1) && is_valid(flat, note2),
        Child(_, child_note) => is_valid(flat, child_note),
    }
}

/***********
 * Printers *
 ***********/

struct Seeker<'d, D: PrettyDoc<'d>> {
    blocks: BlockSet<'d, D>,
}

struct DownwardPrinter<'d, D: PrettyDoc<'d>> {
    done: bool,
    blocks: BlockSet<'d, D>,
}

struct UpwardPrinter<'d, D: PrettyDoc<'d>> {
    done: bool,
    blocks: BlockSet<'d, D>,
}

impl<'d, D: PrettyDoc<'d>> Seeker<'d, D> {
    fn new(notation: NotationRef<'d, D>, width: Width) -> Seeker<'d, D> {
        Seeker {
            blocks: BlockSet::new(notation, width),
        }
    }

    fn highlight(&mut self, _shade: Shade) {
        // TODO
        // self.blocks.first_chunk_mut().unwrap().1 = shade;
    }

    fn seek(mut self, path: &[usize]) -> (UpwardPrinter<'d, D>, DownwardPrinter<'d, D>) {
        // Seek to the descendant given by `path`. Highlight the descendency chain as we go.
        let mut path = path.iter();
        self.highlight(Shade(path.len() as u8));
        while let Some(child_index) = path.next() {
            self.seek_child(*child_index);
            self.highlight(Shade(path.len() as u8));
        }

        // Split the BlockSet in two: a top half and bottom half.
        let downward_printer = DownwardPrinter {
            done: false,
            blocks: BlockSet {
                width: self.blocks.width,
                prev: vec![],
                block: self.blocks.block,
                next: self.blocks.next,
            },
        };
        let (prev_block, at_start) = match self.blocks.prev.pop() {
            Some(block) => (block, false),
            None => (Block::empty(), true),
        };
        let upward_printer = UpwardPrinter {
            done: at_start,
            blocks: BlockSet {
                width: self.blocks.width,
                prev: self.blocks.prev,
                block: prev_block,
                next: vec![],
            },
        };
        (upward_printer, downward_printer)
    }

    fn seek_child(&mut self, child_index: usize) {
        use NotationCase::*;

        let parent_doc_id = self.blocks.first_chunk().unwrap().2.doc_id();
        loop {
            // 1. Move forward to the nearest block containing a `Choice` or `Child` belonging to
            //    `parent_doc`.
            //    (NOTE: more precise would be looking for Child(child_index) or a Choice
            //     containing it, but you can't tell right now what children a choice might
            //     contain.)
            self.blocks.find_block(&|note| match note.case() {
                Choice(_, _) => note.doc_id() == parent_doc_id,
                Child(i, _) => note.doc_id() == parent_doc_id && i == child_index,
                _ => false,
            });

            // 2. Resolve the first Child or Choice in this block.
            //    - If you hit `Child(i)` belonging to `parent_doc`, success.
            //    - If you hit end of doc, panic (every child must be present).
            //    - Otherwise, go back to step 1.
            let note = self.blocks.first_chunk().expect("Missing child").2;
            match note.case() {
                Child(i, _) if note.doc_id() == parent_doc_id && i == child_index => {
                    return;
                }
                _ => (),
            }
            self.blocks.resolve_first_chunk();
        }
    }
}

impl<'d, D: PrettyDoc<'d>> Iterator for DownwardPrinter<'d, D> {
    type Item = LineContents<'d>;

    fn next(&mut self) -> Option<LineContents<'d>> {
        if self.done {
            return None;
        }

        // Finish resolving everything on the current block.
        while self.blocks.resolve_first_chunk() {}

        // Extract the block and convert it into a LineContents.
        let (block, at_end) = self.blocks.take_block(Direction::Down);
        self.done = at_end;
        Some(block.to_line_contents())
    }
}

impl<'d, D: PrettyDoc<'d>> Iterator for UpwardPrinter<'d, D> {
    type Item = LineContents<'d>;

    fn next(&mut self) -> Option<LineContents<'d>> {
        if self.done {
            return None;
        }

        // Fully resolve the last block.
        loop {
            if self.blocks.resolve_first_chunk() {
                self.blocks.goto_last_block();
            } else {
                break;
            }
        }

        // Extract the block and convert it into a LineContents.
        let (block, at_start) = self.blocks.take_block(Direction::Up);
        self.done = at_start;
        Some(block.to_line_contents())
    }
}
