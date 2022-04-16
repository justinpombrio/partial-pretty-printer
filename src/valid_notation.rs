use crate::notation::{Notation, RepeatInner};
use std::fmt;

// TODO:
// - Check over this validation. It probably misses some subtle points.

/// A Notation that has passed validation. Obtain one by constructing a [Notation] and then calling
/// [Notation::validate].
#[derive(Clone, Debug)]
pub struct ValidNotation(pub(crate) Notation);

#[derive(Debug, Clone)]
pub enum NotationError {
    LeftOutsideOfJoin,
    RightOutsideOfJoin,
    SurroundedOutsideOfSurround,
    NestedRepeat,
}

impl fmt::Display for NotationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use NotationError::*;

        match self {
            LeftOutsideOfJoin => {
                write!(f, "The Notation contains a `Left` used outside of `RepeatInner.join`, but `Left` is only meaningful inside `join`.")
            }
            RightOutsideOfJoin => {
                write!(f, "The Notation contains a `Right` used outside of `RepeatInner.join`, but `Right` is only meaningful inside `join`.")
            }
            SurroundedOutsideOfSurround => {
                write!(f, "The Notation contains a `Surrounded` used outside of `RepeatInner.surround`, but `Surrounded` is only meaningful inside `surround`.")
            }
            NestedRepeat => {
                write!(f, "The Notation contains a nested `Repeat`, but one Repeat is not allowed to occur inside another.")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    Notation,
    InRepeat,
    Join,
    Surround,
}

impl Notation {
    pub fn validate(mut self) -> Result<ValidNotation, NotationError> {
        self.validate_rec(Context::Notation)?;
        Ok(ValidNotation(self))
    }

    fn validate_rec(&mut self, ctx: Context) -> Result<(), NotationError> {
        use crate::Notation::*;
        use Context::*;
        use NotationError::*;

        match self {
            Empty => Ok(()),
            Text(_) => Ok(()),
            Literal(_) => Ok(()),
            Newline => Ok(()),
            Flat(note) => note.validate_rec(ctx),
            Indent(_, note) => note.validate_rec(ctx),
            Concat(note1, note2) => {
                note1.validate_rec(ctx)?;
                note2.validate_rec(ctx)
            }
            Group(note) => note.validate_rec(ctx),
            Choice(note1, note2) => {
                note1.validate_rec(ctx)?;
                note2.validate_rec(ctx)
            }
            IfFlat(note1, note2) => {
                note1.validate_rec(ctx)?;
                note2.validate_rec(ctx)
            }
            IfEmptyText(note1, note2) => {
                note1.validate_rec(ctx)?;
                note2.validate_rec(ctx)
            }
            Child(_) => Ok(()),
            Repeat(repeat) if ctx == Notation => repeat.validate_rec(),
            Repeat(_) => Err(NestedRepeat),
            Left if ctx == Join => Ok(()),
            Right if ctx == Join => Ok(()),
            Left => Err(LeftOutsideOfJoin),
            Right => Err(RightOutsideOfJoin),
            Surrounded if ctx == Surround => Ok(()),
            Surrounded => Err(SurroundedOutsideOfSurround),
        }
    }
}

impl RepeatInner {
    fn validate_rec(&mut self) -> Result<(), NotationError> {
        use Context::*;

        self.empty.validate_rec(InRepeat)?;
        self.lone.validate_rec(InRepeat)?;
        self.join.validate_rec(Join)?;
        self.surround.validate_rec(Surround)?;
        Ok(())
    }
}
