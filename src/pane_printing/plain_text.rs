use super::pretty_window::PrettyWindow;
use crate::geometry::{Col, Line, Size};
use crate::style::Shade;
use crate::LineContents;
use std::fmt;
use std::mem;

/// Render a document in plain text.
pub struct PlainText {
    lines: Vec<String>,
    size: Size,
}

impl fmt::Display for PlainText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl PlainText {
    /// Construct a screen with the given width and height.
    pub fn new(size: Size) -> PlainText {
        PlainText {
            lines: vec![],
            size,
        }
    }

    /// Construct a screen with the given width and unbounded height.
    pub fn new_unbounded_height(width: Col) -> PlainText {
        let size = Size {
            height: Line::max_value(),
            width,
        };
        PlainText::new(size)
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

    fn print_line(
        &mut self,
        line_num: Line,
        line_contents: LineContents,
    ) -> Result<(), Self::Error> {
        if line_num >= self.size.height {
            return Ok(());
        }
        let line_mut = self.get_mut_line(line_num as usize);
        let _ = mem::replace(line_mut, line_contents.to_string());
        Ok(())
    }

    fn highlight(
        &mut self,
        _line: Line,
        _cols: (Col, Col),
        _shade: Option<Shade>,
        _reverse: bool,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
