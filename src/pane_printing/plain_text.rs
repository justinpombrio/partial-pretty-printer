use super::pretty_window::PrettyWindow;
use crate::geometry::{Height, Pos, Size, Width};
use std::fmt;
use std::iter;

/// Render a document in plain text.
#[derive(Debug)]
pub struct PlainText {
    lines: Vec<Vec<char>>,
    size: Size,
}

const SENTINEL: char = '\0';

impl fmt::Display for PlainText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            for ch in line {
                if *ch != SENTINEL {
                    write!(f, "{}", ch)?;
                }
            }
            writeln!(f);
        }
        Ok(())
    }
}

impl PlainText {
    /// Construct a screen with the given width and height.
    pub fn new(width: Width, height: Height) -> PlainText {
        PlainText {
            lines: vec![],
            size: Size { width, height },
        }
    }

    /// Construct a screen with the given width and unbounded height.
    pub fn new_unbounded_height(width: Width) -> PlainText {
        PlainText::new(width, Height::max_value())
    }

    fn get_mut_line(&mut self, line_num: usize) -> &mut Vec<char> {
        if self.lines.len() < line_num + 1 {
            self.lines.resize_with(line_num + 1, Vec::new);
        }
        &mut self.lines[line_num as usize]
    }
}

impl PrettyWindow for PlainText {
    type Error = fmt::Error;
    type Style = ();
    type Mark = ();

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn print_char(
        &mut self,
        ch: char,
        pos: Pos,
        mark: &Self::Mark,
        style: &Self::Style,
        full_width: bool,
    ) -> Result<(), Self::Error> {
        let row = pos.row as usize;
        let col = pos.col as usize;
        if self.lines.len() < row + 1 {
            self.lines.resize_with(row + 1, Vec::new);
        }
        let line = &mut self.lines[row];

        let width = if full_width { 2 } else { 1 };
        if line.len() < col + width {
            line.resize(col + width, ' ');
        }
        line[col] = ch;
        if full_width {
            line[col + 1] = SENTINEL;
        }
        Ok(())
    }
}
