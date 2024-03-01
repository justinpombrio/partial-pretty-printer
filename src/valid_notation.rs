use crate::notation::{CheckPos, Condition, Notation, StyleLabel};
use std::fmt;

/// A Notation that has passed validation. Obtain one by constructing a [Notation] and then calling
/// [`Notation::validate`].
#[derive(Clone, Debug)]
pub struct ValidNotation<L: StyleLabel, C: Condition>(pub(crate) Notation<L, C>);

#[derive(thiserror::Error, Debug, Clone)]
pub enum NotationError {
    #[error(
        "Notation contains a Left outside of Fold.join, but it's only meaningful inside of that."
    )]
    LeftOutsideJoin,
    #[error(
        "Notation contains a Right outside of Fold.join, but it's only meaningful inside of that."
    )]
    RightOutsideJoin,
    #[error(
        "Notation contains a CheckPos::LeftChild outside of Fold.join, but it's only meaningful inside of that."
    )]
    CheckPosLeftOutsideJoin,
    #[error(
        "Notation contains a CheckPos::RightChild outside of Fold.join, but it's only meaningful inside of that."
    )]
    CheckPosRightOutsideJoin,
    #[error("Notation contains a Fold inside a Fold, but those aren't allowed to be nested.")]
    NestedFold,
    #[error("Notation contains a Count inside a Count, but those aren't allowed to be nested.")]
    NestedCount,
    #[error("Notation contains a Child inside Count.zero, but in this case there are guaranteed to be zero children.")]
    CountZeroChild,
    #[error("Notation contains a Child with index {} inside of Count.one, but in this case there's guaranteed to be only one child.", 0)]
    CountOneChildIndex(usize),
    #[error("Notation contains a CheckPos::Child inside Count.zero, but in this case there are guaranteed to be zero children.")]
    CountZeroCheckPosChild,
    #[error("Notation contains a CheckPos::Child with index {} inside of Count.one, but in this case there's guaranteed to be only one child.", 0)]
    CountOneCheckPosChildIndex(isize),
    #[error(
        "Notation contains a Text inside a Count, but a node can't have both text and children."
    )]
    TextInsideCount,
    #[error(
        "Notation contains a Text inside a Fold, but a node can't have both text and children."
    )]
    TextInsideFold,
    #[error("Notation contains Text or Literal after an EndOfLine, which is a printing error.")]
    TextAfterEol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountContext {
    InCountZero,
    InCountOne,
    InCountMany,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FoldContext {
    InFoldFirst,
    InFoldJoin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Context {
    count: Option<CountContext>,
    fold: Option<FoldContext>,
}

impl Context {
    fn new() -> Self {
        Context {
            count: None,
            fold: None,
        }
    }

    fn count_zero(self) -> Self {
        Context {
            count: Some(CountContext::InCountZero),
            fold: self.fold,
        }
    }

    fn count_one(self) -> Self {
        Context {
            count: Some(CountContext::InCountOne),
            fold: self.fold,
        }
    }

    fn count_many(self) -> Self {
        Context {
            count: Some(CountContext::InCountMany),
            fold: self.fold,
        }
    }

    fn fold_first(self) -> Self {
        Context {
            count: self.count,
            fold: Some(FoldContext::InFoldFirst),
        }
    }

    fn fold_join(self) -> Self {
        Context {
            count: self.count,
            fold: Some(FoldContext::InFoldJoin),
        }
    }
}

impl<L: StyleLabel, C: Condition> Notation<L, C> {
    pub fn validate(mut self) -> Result<ValidNotation<L, C>, NotationError> {
        self.validate_rec(false, false, Context::new())?;
        Ok(ValidNotation(self))
    }

    #[doc(hidden)]
    pub fn cheat_validation_for_testing_only(self) -> ValidNotation<L, C> {
        ValidNotation(self)
    }

    // Returns whether any way of making Choices and Checks will end with an EOL
    fn validate_rec(
        &mut self,
        flat: bool,
        // whether this notation may have been preceded by an EOL
        eol: bool,
        ctx: Context,
    ) -> Result<bool, NotationError> {
        use CountContext::*;
        use FoldContext::*;
        use Notation::*;
        use NotationError::*;

        match self {
            Text if ctx.count.is_some() => Err(TextInsideCount),
            Text if ctx.fold.is_some() => Err(TextInsideFold),
            Empty => Ok(eol),
            Text | Literal(_) if eol => Err(TextAfterEol),
            Text | Literal(_) => Ok(false),
            Newline => Ok(false),
            EndOfLine => Ok(true),
            Flat(note) => note.validate_rec(true, eol, ctx),
            Indent(_, _, note) => note.validate_rec(flat, eol, ctx),
            Concat(note1, note2) => {
                let eol = note1.validate_rec(flat, eol, ctx)?;
                note2.validate_rec(flat, eol, ctx)
            }
            Choice(note1, note2) => {
                let eol_1 = note1.validate_rec(flat, eol, ctx)?;
                let eol_2 = note2.validate_rec(flat, eol, ctx)?;
                Ok(eol_1 || eol_2)
            }
            Check(_, pos, note1, note2) => {
                match &pos {
                    CheckPos::Here => (),
                    CheckPos::Child(_) if ctx.count == Some(InCountZero) => {
                        return Err(CountZeroCheckPosChild)
                    }
                    CheckPos::Child(i)
                        if ctx.count == Some(InCountOne) && pos.child_index(1).is_none() =>
                    {
                        return Err(CountOneCheckPosChildIndex(*i))
                    }
                    CheckPos::Child(_) => (),
                    CheckPos::LeftChild if ctx.fold != Some(InFoldJoin) => {
                        return Err(CheckPosLeftOutsideJoin)
                    }
                    CheckPos::RightChild if ctx.fold != Some(InFoldJoin) => {
                        return Err(CheckPosRightOutsideJoin)
                    }
                    CheckPos::LeftChild | CheckPos::RightChild => (),
                }
                let eol_1 = note1.validate_rec(flat, eol, ctx)?;
                let eol_2 = note2.validate_rec(flat, eol, ctx)?;
                Ok(eol_1 || eol_2)
            }
            Child(_) if ctx.count == Some(InCountZero) => Err(CountZeroChild),
            Child(n) if *n > 0 && ctx.count == Some(InCountOne) => Err(CountOneChildIndex(*n)),
            Child(_) => Ok(false),
            Style(_, note) => note.validate_rec(flat, eol, ctx),
            Count { .. } if ctx.count.is_some() => Err(NestedCount),
            Count { zero, one, many } => {
                let eol_1 = zero.validate_rec(flat, eol, ctx.count_zero())?;
                let eol_2 = one.validate_rec(flat, eol, ctx.count_one())?;
                let eol_3 = many.validate_rec(flat, eol, ctx.count_many())?;
                Ok(eol_1 || eol_2 || eol_3)
            }
            Fold { .. } if ctx.fold.is_some() => Err(NestedFold),
            Fold { first, join } => {
                // Can't easily check for EOL here
                first.validate_rec(flat, false, ctx.fold_first())?;
                join.validate_rec(flat, false, ctx.fold_join())?;
                Ok(false)
            }
            Left if ctx.fold != Some(InFoldJoin) => Err(LeftOutsideJoin),
            Right if ctx.fold != Some(InFoldJoin) => Err(RightOutsideJoin),
            Left | Right => Ok(false),
        }
    }
}

impl<L: StyleLabel, C: Condition> fmt::Display for ValidNotation<L, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
