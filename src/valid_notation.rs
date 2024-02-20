use crate::notation::Notation;

/// A Notation that has passed validation. Obtain one by constructing a [Notation] and then calling
/// [`Notation::validate`].
#[derive(Clone, Debug)]
pub struct ValidNotation<S>(pub(crate) Notation<S>);

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
    #[error("Notation contains a Fold inside a Fold, but those aren't allowed to be nested.")]
    NestedFold,
    #[error("Notation contains a Count inside a Count, but those aren't allowed to be nested.")]
    NestedCount,
    #[error("Notation contains a Child inside Count.zero, but in this case there are guaranteed to be zero children.")]
    CountZeroChild,
    #[error("Notation contains a Child with index {} inside of Count.one, but in this case there's guaranteed to be only one child.", 0)]
    CountOneChildIndex(usize),
    #[error("Notation contains a Child inside of IfEmptyText, but a node can't have both text and children.")]
    ChildInsideIfEmptyText,
    #[error("Notation contains an IfEmptyText inside a Count, but a node can't have both text and children.")]
    IfEmptyTextInsideCount,
    #[error(
        "Notation contains a Text inside a Count, but a node can't have both text and children."
    )]
    TextInsideCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    InNothing,
    // in IfEmptyText
    InIfEmptyText,
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
            InNothing | InIfEmptyText => false,
            InCountZero | InCountOne | InCountMany | InFoldFirst | InFoldJoin => true,
        }
    }

    fn in_fold(&self) -> bool {
        use Context::*;

        match self {
            InNothing | InIfEmptyText | InCountZero | InCountOne | InCountMany => false,
            InFoldFirst | InFoldJoin => true,
        }
    }
}

impl<S> Notation<S> {
    pub fn validate(mut self) -> Result<ValidNotation<S>, NotationError> {
        self.validate_rec(false, Context::InNothing)?;
        Ok(ValidNotation(self))
    }

    fn validate_rec(&mut self, flat: bool, ctx: Context) -> Result<(), NotationError> {
        use Context::*;
        use Notation::*;
        use NotationError::*;

        match self {
            Text(_) if ctx.in_count() => return Err(TextInsideCount),
            Empty | Text(_) | Literal(_) | Newline => (),
            Flat(note) => note.validate_rec(true, ctx)?,
            Indent(_, note) => note.validate_rec(flat, ctx)?,
            Concat(note1, note2) => {
                note1.validate_rec(flat, ctx)?;
                note2.validate_rec(flat, ctx)?;
            }
            Choice(note1, note2) => {
                note1.validate_rec(flat, ctx)?;
                note2.validate_rec(flat, ctx)?;
            }
            IfEmptyText(_, _) if ctx.in_count() => return Err(IfEmptyTextInsideCount),
            IfEmptyText(note1, note2) => {
                note1.validate_rec(flat, InIfEmptyText)?;
                note2.validate_rec(flat, InIfEmptyText)?;
            }
            Child(_) if ctx == InIfEmptyText => return Err(ChildInsideIfEmptyText),
            Child(_) if ctx == InCountZero => return Err(CountZeroChild),
            Child(n) if *n > 0 && ctx == InCountOne => return Err(CountOneChildIndex(*n)),
            Child(_) => (),
            Mark(_, note) => note.validate_rec(flat, ctx)?,
            Count { .. } if ctx.in_count() => return Err(NestedCount),
            Count { zero, one, many } => {
                zero.validate_rec(flat, InCountZero)?;
                one.validate_rec(flat, InCountOne)?;
                many.validate_rec(flat, InCountMany)?;
            }
            Fold { .. } if ctx.in_fold() => return Err(NestedFold),
            Fold { .. } if !matches!(ctx, InCountMany | InCountOne) => {
                return Err(FoldOutsideCount)
            }
            Fold { first, join } => {
                first.validate_rec(flat, InFoldFirst)?;
                join.validate_rec(flat, InFoldJoin)?;
            }
            Left if ctx != InFoldJoin => return Err(LeftOutsideJoin),
            Right if ctx != InFoldJoin => return Err(RightOutsideJoin),
            Left | Right => (),
        }

        Ok(())
    }
}
