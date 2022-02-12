use crate::notation::{Notation, RepeatInner};
use std::fmt;

// TODO:
// - Check over this validation. It probably misses some subtle points.

/// A Notation that has passed validation. Obtain one with `Notation.validate()`.
#[derive(Clone, Debug)]
pub struct ValidNotation(pub(crate) Notation);

#[derive(Debug, Clone)]
pub enum NotationError {
    FlattenedNewline,
    FlattenedChild,
    LeftOutsideOfJoin,
    RightOutsideOfJoin,
    SurroundedOutsideOfSurround,
    NestedRepeat,
}

impl fmt::Display for NotationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use NotationError::*;

        match self {
            FlattenedNewline => {
                write!(
                    f,
                    "The Notation contained a Flat around a Newline. There is no reason to
            have this, since the purpose of Flat is to disallow Newlines. To fix this, either
            remove the Flat or the Newline."
                )
            }
            FlattenedChild => {
                write!(
                    f,
                    "The Notation contains Children, but lacks any choice in which none
            of the Children are Flattened. This is required in case all of the children contain
            unconditional Newlines."
                )
            }
            LeftOutsideOfJoin => {
                write!(f, "The Notations contains a `Left` used outside of `RepeatInner.join`, but `Left` is only meaningful inside `join`.")
            }
            RightOutsideOfJoin => {
                write!(f, "The Notations contains a `Right` used outside of `RepeatInner.join`, but `Right` is only meaningful inside `join`.")
            }
            SurroundedOutsideOfSurround => {
                write!(f, "The Notations contains a `Surrounded` used outside of `RepeatInner.surround`, but `Surrounded` is only meaningful inside `surround`.")
            }
            NestedRepeat => {
                write!(f, "The Notation contains a nested `Repeat`, but one Repeat is not allowed to occur inside another.")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FlatThings {
    /// Does this notation contain an _unconditional_ newline?
    newline: bool,
    /// Does this notation contain an _unconditional_ flattened child?
    flat_child: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    Notation,
    InRepeat,
    Join,
    Surround,
}

impl FlatThings {
    fn none() -> FlatThings {
        FlatThings {
            newline: false,
            flat_child: false,
        }
    }

    fn newline() -> FlatThings {
        FlatThings {
            newline: true,
            flat_child: false,
        }
    }

    fn flat_child() -> FlatThings {
        FlatThings {
            newline: false,
            flat_child: true,
        }
    }

    fn concat(self: FlatThings, other: FlatThings) -> FlatThings {
        FlatThings {
            newline: self.newline || other.newline,
            flat_child: self.flat_child || other.flat_child,
        }
    }

    fn choice(self: FlatThings, other: FlatThings) -> FlatThings {
        FlatThings {
            newline: self.newline && other.newline,
            flat_child: self.flat_child && other.flat_child,
        }
    }
}

impl Notation {
    pub fn validate(mut self) -> Result<ValidNotation, NotationError> {
        let validation_info = self.validate_rec(Context::Notation, false)?;
        if validation_info.flat_child {
            return Err(NotationError::FlattenedChild);
        }
        Ok(ValidNotation(self))
    }

    fn validate_rec(&mut self, ctx: Context, flat: bool) -> Result<FlatThings, NotationError> {
        use crate::Notation::*;
        use Context::*;
        use NotationError::*;

        match self {
            Empty => Ok(FlatThings::none()),
            Text(_) => Ok(FlatThings::none()),
            Literal(_) => Ok(FlatThings::none()),
            Newline => Ok(FlatThings::newline()),
            Flat(note) => {
                let flattened = note.validate_rec(ctx, true)?;
                if flattened.newline {
                    Err(FlattenedNewline)
                } else {
                    Ok(flattened)
                }
            }
            Indent(_, note) => note.validate_rec(ctx, flat),
            Concat(note1, note2) => {
                let flattened1 = note1.validate_rec(ctx, flat)?;
                let flattened2 = note2.validate_rec(ctx, flat)?;
                Ok(flattened1.concat(flattened2))
            }
            Choice((note1, nl1), (note2, nl2)) => {
                let flattened1 = note1.validate_rec(ctx, flat)?;
                *nl1 = flattened1.newline;
                let flattened2 = note2.validate_rec(ctx, flat)?;
                *nl2 = flattened2.newline;
                Ok(flattened1.choice(flattened2))
            }
            IfEmptyText(note1, note2) => {
                let flattened1 = note1.validate_rec(ctx, flat)?;
                let flattened2 = note2.validate_rec(ctx, flat)?;
                Ok(flattened1.choice(flattened2))
            }
            Child(_) if flat => Ok(FlatThings::flat_child()),
            Child(_) => Ok(FlatThings::none()),
            Repeat(repeat) if ctx == Notation => repeat.validate_rec(flat),
            Repeat(_) => Err(NestedRepeat),
            Left if ctx == Join && flat => Ok(FlatThings::flat_child()),
            Right if ctx == Join && flat => Ok(FlatThings::flat_child()),
            Left if ctx == Join => Ok(FlatThings::none()),
            Right if ctx == Join => Ok(FlatThings::none()),
            Left => Err(LeftOutsideOfJoin),
            Right => Err(RightOutsideOfJoin),
            Surrounded if ctx == Surround && flat => Ok(FlatThings::flat_child()),
            Surrounded if ctx == Surround => Ok(FlatThings::none()),
            Surrounded => Err(SurroundedOutsideOfSurround),
        }
    }
}

impl RepeatInner {
    fn validate_rec(&mut self, flat: bool) -> Result<FlatThings, NotationError> {
        use Context::*;

        let f_empty = self.empty.validate_rec(InRepeat, flat)?;
        let f_lone = self.lone.validate_rec(InRepeat, flat)?;
        let f_join = self.join.validate_rec(Join, flat)?;
        let f_surround = self.surround.validate_rec(Surround, flat)?;
        Ok(f_empty.concat(f_lone.concat(f_join.concat(f_surround))))
    }
}
