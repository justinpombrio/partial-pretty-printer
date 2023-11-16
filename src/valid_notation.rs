use crate::notation::Notation;
use std::fmt;

/// A Notation that has passed validation. Obtain one by constructing a [Notation] and then calling
/// [Notation::validate].
#[derive(Clone, Debug)]
pub struct ValidNotation(pub(crate) Notation);

#[derive(thiserror::Error, Debug, Clone)]
pub enum NotationError {
    #[error("Notation contains a Choice inside a Flat, but that is guaranteed to have no effect.")]
    ChoiceInsideFlat,
    #[error(
        "Notation contains a Fold outside of Count.many, but it's only meaningful inside of that."
    )]
    FoldOutsideOfCount,
    #[error(
        "Notation contains a Left outside of Fold.join, but it's only meaningful inside of that."
    )]
    LeftOutsideOfJoin,
    #[error(
        "Notation contains a Right outside of Fold.join, but it's only meaningful inside of that."
    )]
    RightOutsideOfJoin,
    #[error("Notation contains a Fold inside a Fold, but those aren't allowed to be nested.")]
    NestedFold,
    #[error("Notation contains a Count inside a Count, but those aren't allowed to be nested.")]
    NestedCount,
    #[error("Notation contains a child inside Count.zero, but in this case there are guaranteed to be zero children.")]
    CountZeroChild,
    #[error("Notation contains a child with index {} inside of Count.one, which is the case that there's only one child.", 0)]
    CountOneChildIndex(usize),
    #[error("Notation contains a child with index {} inside of Count.many, which only guarantees that there are at least two children.", 0)]
    CountManyChildIndex(usize),
    #[error("Notation contains a child inside of IfEmptyText, but a node can't have both text and children.")]
    ChildInText,
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
    // in IfEmptyText
    InText,
}

impl Notation {
    pub fn validate(mut self) -> Result<ValidNotation, NotationError> {
        self.validate_rec(false, Context::InNothing)?;
        Ok(ValidNotation(self))
    }

    fn validate_rec(&mut self, flat: bool, ctx: Context) -> Result<(), NotationError> {
        use Context::*;
        use Notation::*;
        use NotationError::*;

        match self {
            Empty | Text(_) | Literal(_) | Newline => (),
            Flat(note) => note.validate_rec(true, ctx)?,
            Indent(_, note) => note.validate_rec(flat, ctx)?,
            Concat(note1, note2) => {
                note1.validate_rec(flat, ctx)?;
                note2.validate_rec(flat, ctx)?;
            }
            Choice(_, _) if flat => return Err(ChoiceInsideFlat),
            Choice(note1, note2) => {
                note1.validate_rec(flat, ctx)?;
                note2.validate_rec(flat, ctx)?;
            }
            IfEmptyText(note1, note2) => {
                note1.validate_rec(flat, InText)?;
                note2.validate_rec(flat, InText)?;
            }
            Child(_) if ctx == InText => return Err(ChildInText),
            Child(_) if ctx == InCountZero => return Err(CountZeroChild),
            Child(n) if *n > 0 && ctx == InCountOne => return Err(CountOneChildIndex(*n)),
            Child(n) if *n > 1 && matches!(ctx, InCountMany | InFoldFirst | InFoldJoin) => {
                return Err(CountManyChildIndex(*n))
            }
            Child(_) => (),
            Count { .. }
                if matches!(
                    ctx,
                    InCountZero | InCountOne | InCountMany | InFoldFirst | InFoldJoin
                ) =>
            {
                return Err(NestedCount)
            }
            Count { zero, one, many } => {
                zero.validate_rec(flat, InCountZero)?;
                one.validate_rec(flat, InCountOne)?;
                many.validate_rec(flat, InCountMany)?;
            }
            Fold { .. } if matches!(ctx, InFoldFirst | InFoldJoin) => return Err(NestedFold),
            Fold { .. } if !matches!(ctx, InCountMany) => return Err(FoldOutsideOfCount),
            Fold { last, join } => {
                last.validate_rec(flat, InFoldFirst)?;
                join.validate_rec(flat, InFoldJoin)?;
            }
            Left if !matches!(ctx, InFoldJoin) => return Err(LeftOutsideOfJoin),
            Right if !matches!(ctx, InFoldJoin) => return Err(RightOutsideOfJoin),
            Left | Right => (),
        }

        Ok(())
    }
}
