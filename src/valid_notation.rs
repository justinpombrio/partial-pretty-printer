use crate::notation::{CheckPos, Condition, Notation, StyleLabel};
use std::fmt;

/// A Notation that has passed validation. Obtain one by constructing a [Notation] and then calling
/// [`Notation::validate`].
#[derive(Clone, Debug)]
pub struct ValidNotation<L: StyleLabel, C: Condition>(pub(crate) Notation<L, C>);

#[derive(thiserror::Error, Debug, Clone)]
pub enum NotationError {
    #[error(
        "Notation contains a Fold outside of Count.many or Count.one, but it's only meaningful inside of those."
    )]
    FoldOutsideCount,
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
    CountOneCheckPosChildIndex(usize),
    #[error(
        "Notation contains a Text inside a Count, but a node can't have both text and children."
    )]
    TextInsideCount,
    #[error("Notation contains Text or Literal after an EndOfLine, which is a printing error.")]
    TextAfterEol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    InNothing,
    // in Count
    InCountZero,
    InCountOne,
    InCountMany,
    // in Count _and_ Fold
    InFoldFirst,
    InFoldJoin,
}

impl Context {
    fn in_count(&self) -> bool {
        use Context::*;

        match self {
            InNothing => false,
            InCountZero | InCountOne | InCountMany | InFoldFirst | InFoldJoin => true,
        }
    }

    fn in_fold(&self) -> bool {
        use Context::*;

        match self {
            InNothing | InCountZero | InCountOne | InCountMany => false,
            InFoldFirst | InFoldJoin => true,
        }
    }
}

impl<L: StyleLabel, C: Condition> Notation<L, C> {
    pub fn validate(mut self) -> Result<ValidNotation<L, C>, NotationError> {
        self.validate_rec(false, false, Context::InNothing)?;
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
        use Context::*;
        use Notation::*;
        use NotationError::*;

        match self {
            Text if ctx.in_count() => Err(TextInsideCount),
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
                match pos {
                    CheckPos::Here => (),
                    CheckPos::Child(_) if ctx == InCountZero => return Err(CountZeroCheckPosChild),
                    CheckPos::Child(n) if *n > 0 && ctx == InCountOne => {
                        return Err(CountOneCheckPosChildIndex(*n))
                    }
                    CheckPos::Child(_) => (),
                    CheckPos::LeftChild if ctx != InFoldJoin => {
                        return Err(CheckPosLeftOutsideJoin)
                    }
                    CheckPos::RightChild if ctx != InFoldJoin => {
                        return Err(CheckPosRightOutsideJoin)
                    }
                    CheckPos::LeftChild | CheckPos::RightChild => (),
                }
                let eol_1 = note1.validate_rec(flat, eol, ctx)?;
                let eol_2 = note2.validate_rec(flat, eol, ctx)?;
                Ok(eol_1 || eol_2)
            }
            Child(_) if ctx == InCountZero => Err(CountZeroChild),
            Child(n) if *n > 0 && ctx == InCountOne => Err(CountOneChildIndex(*n)),
            Child(_) => Ok(false),
            Style(_, note) => note.validate_rec(flat, eol, ctx),
            Count { .. } if ctx.in_count() => Err(NestedCount),
            Count { zero, one, many } => {
                let eol_1 = zero.validate_rec(flat, eol, InCountZero)?;
                let eol_2 = one.validate_rec(flat, eol, InCountOne)?;
                let eol_3 = many.validate_rec(flat, eol, InCountMany)?;
                Ok(eol_1 || eol_2 || eol_3)
            }
            Fold { .. } if ctx.in_fold() => Err(NestedFold),
            Fold { .. } if !matches!(ctx, InCountMany | InCountOne) => Err(FoldOutsideCount),
            Fold { first, join } => {
                // Can't easily check for EOL here
                first.validate_rec(flat, false, InFoldFirst)?;
                join.validate_rec(flat, false, InFoldJoin)?;
                Ok(false)
            }
            Left if ctx != InFoldJoin => Err(LeftOutsideJoin),
            Right if ctx != InFoldJoin => Err(RightOutsideJoin),
            Left | Right => Ok(false),
        }
    }
}

impl<L: StyleLabel, C: Condition> fmt::Display for ValidNotation<L, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
