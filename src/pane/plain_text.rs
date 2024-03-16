use crate::Style;
use crate::{pane::PrettyWindow, Height, Pos, Size, Width};
use std::convert::Infallible;
use std::fmt;
use std::marker::PhantomData;

/// A simple [`PrettyWindow`] that outputs the contents of the "window" as plain text. Use
/// [`fmt::Display`] to view the text.
#[derive(Debug)]
pub struct PlainText<S: Style> {
    /// A line is stored as a vector of characters. Each element represents one column position, so
    /// a full-width unicode character will be followed by a `SENTINEL` value to indicate that it
    /// takes up the next column as well.
    lines: Vec<Vec<char>>,
    /// The size of the window.
    size: Size,
    /// The style is ignored.
    phantom_style: PhantomData<S>,
    /// The focus points of the documents.
    focus_points: Vec<Pos>,
}

// Follows each full-width char.
const SENTINEL: char = '\0';

impl<S: Style> fmt::Display for PlainText<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            for ch in line {
                if *ch != SENTINEL {
                    write!(f, "{}", ch)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<S: Style> PlainText<S> {
    /// Construct a window with the given width and height.
    pub fn new(width: Width, height: Height) -> PlainText<S> {
        PlainText::<S> {
            lines: vec![],
            size: Size { width, height },
            focus_points: Vec::new(),
            phantom_style: PhantomData,
        }
    }

    /// Construct a window with the given width and unbounded height.
    pub fn new_unbounded_height(width: Width) -> PlainText<S> {
        PlainText::<S>::new(width, Height::max_value())
    }

    pub fn focus_points(&self) -> &[Pos] {
        &self.focus_points
    }
}

impl<S: Style> PrettyWindow for PlainText<S> {
    type Error = Infallible;
    type Style = S;

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn display_char(
        &mut self,
        ch: char,
        pos: Pos,
        _style: &Self::Style,
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

    fn set_focus(&mut self, pos: Pos) -> Result<(), Self::Error> {
        self.focus_points.push(pos);
        Ok(())
    }
}
