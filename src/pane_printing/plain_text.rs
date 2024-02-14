use super::pretty_window::PrettyWindow;
use crate::geometry::{Height, Pos, Size, Width};
use std::convert::Infallible;
use std::fmt;
use std::marker::PhantomData;

/// Render a document in plain text.
#[derive(Debug)]
pub struct PlainText<S: fmt::Debug + Default, M: fmt::Debug> {
    lines: Vec<Vec<char>>,
    size: Size,
    phantom: PhantomData<(S, M)>,
}

const SENTINEL: char = '\0';

impl<S: fmt::Debug + Default, M: fmt::Debug> fmt::Display for PlainText<S, M> {
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

impl<S: fmt::Debug + Default, M: fmt::Debug> PlainText<S, M> {
    /// Construct a screen with the given width and height.
    pub fn new(width: Width, height: Height) -> PlainText<S, M> {
        PlainText::<S, M> {
            lines: vec![],
            size: Size { width, height },
            phantom: PhantomData,
        }
    }

    /// Construct a screen with the given width and unbounded height.
    pub fn new_unbounded_height(width: Width) -> PlainText<S, M> {
        PlainText::<S, M>::new(width, Height::max_value())
    }
}

impl<S: fmt::Debug + Default, M: fmt::Debug> PrettyWindow for PlainText<S, M> {
    type Error = Infallible;
    type Style = S;
    type Mark = M;

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn print_char(
        &mut self,
        ch: char,
        pos: Pos,
        _mark: Option<&Self::Mark>,
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
}
