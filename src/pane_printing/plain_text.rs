use super::pretty_window::PrettyWindow;
use crate::geometry::{Height, Pos, Size, Width};
use crate::style::ShadedStyle;
use std::fmt;
use std::iter;

/// Render a document in plain text.
#[derive(Debug)]
pub struct PlainText {
    lines: Vec<String>,
    size: Size,
}

impl fmt::Display for PlainText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            write!(f, "{}\n", line)?;
        }
        Ok(())
    }
}

impl PlainText {
    /// Construct a screen with the given width and height.
    pub fn new(width: u16, height: u32) -> PlainText {
        PlainText {
            lines: vec![],
            size: Size {
                width: Width(width),
                height: Height(height),
            },
        }
    }

    /// Construct a screen with the given width and unbounded height.
    pub fn new_unbounded_height(width: u16) -> PlainText {
        PlainText::new(width, u32::max_value())
    }

    fn get_mut_line(&mut self, line_num: usize) -> &mut String {
        if self.lines.len() < line_num + 1 {
            self.lines.resize_with(line_num + 1, || String::new());
        }
        &mut self.lines[line_num as usize]
    }
}

impl PrettyWindow for PlainText {
    type Error = fmt::Error;

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn print(&mut self, pos: Pos, string: &str, _style: ShadedStyle) -> Result<(), Self::Error> {
        if pos.line.0 >= self.size.height.0 {
            return Ok(());
        }
        let line_mut = self.get_mut_line(pos.line.0 as usize);
        let mut old_chars = line_mut.chars();
        let mut new_line = String::new();

        // Print out the old contents that are the to left of the start column.
        for _ in 0..pos.col.0 {
            new_line.push(old_chars.next().unwrap_or(' '));
        }

        // Print out the new contents.
        new_line.push_str(string);

        // Print out the old contents that are to the right of the end column.
        let old_chars = old_chars.skip(string.chars().count());
        for ch in old_chars {
            new_line.push(ch);
        }

        *line_mut = new_line;
        Ok(())
    }

    fn fill(
        &mut self,
        pos: Pos,
        ch: char,
        len: usize,
        style: ShadedStyle,
    ) -> Result<(), Self::Error> {
        let string: String = iter::repeat(ch).take(len).collect();
        self.print(pos, &string, style)
    }
}
