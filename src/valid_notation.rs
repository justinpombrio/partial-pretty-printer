use crate::{notation::normalize_child_index, CheckPos, Condition, Notation, StyleLabel};
use std::fmt;

/// A notation that has passed validation and does not have any detectable flaws. Obtain one by
/// constructing a [`Notation`] and then calling [`Notation::validate()`].
#[derive(Clone, Debug)]
pub struct ValidNotation<L: StyleLabel, C: Condition>(pub(crate) Notation<L, C>);

/// Flaws that can be detected when validating a [`Notation`] to produce a [`ValidNotation`].
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
    CountOneChildIndex(isize),
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
    #[error(
        "Notation contains Text or Literal after an EndOfLine, which would cause a printing error."
    )]
    TextAfterEol,
    #[error("Notation contains multiple FocusMarks, but only the first one would ever be used.")]
    MultipleFocusMarks,
}

/// Tracks what notations we are inside of during the validation process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Context {
    count: Option<CountContext>,
    fold: Option<FoldContext>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountContext {
    Zero,
    One,
    Many,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FoldContext {
    InFoldFirst,
    InFoldJoin,
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
            count: Some(CountContext::Zero),
            fold: self.fold,
        }
    }

    fn count_one(self) -> Self {
        Context {
            count: Some(CountContext::One),
            fold: self.fold,
        }
    }

    fn count_many(self) -> Self {
        Context {
            count: Some(CountContext::Many),
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

/// Record whether certain notations were encountered earlier in the notation.
#[derive(Debug, Clone, Copy, Default)]
struct History {
    /// Did the last notation visisted _potentially_ contain a Notation::EndOfLine?
    ///
    /// More precisely, is there any way of picking [`Choice`](Notation::Choice)s and
    /// [`Check`](Notation::Check)s such that the notation will end with an
    /// [`EndOfLine`](Notation::EndOfLine). We assume here that no [`Child`](Notation::Child) will
    /// end with [`EndOfLine`](Notation::EndOfLine).
    eol: bool,
    /// Did the previously visited notations _definitely_ contain a
    /// [`FocusMark`](Notation::FocusMark)?
    ///
    /// (It's not a disaster for a Notation to contain multiple `FocusMark`s, so it's ok if _some_
    /// possible combinations of choices result in multiple `FocusMark`s, so long as not all of
    /// them do.)
    mark: bool,
}

impl History {
    fn uncertain() -> History {
        History {
            eol: false,
            mark: false,
        }
    }

    fn choice(self, other: History) -> History {
        History {
            eol: self.eol || other.eol,
            mark: self.mark && other.mark,
        }
    }

    fn with_mark(mut self) -> History {
        self.mark = true;
        self
    }

    fn with_eol(mut self) -> History {
        self.eol = true;
        self
    }

    fn without_eol(mut self) -> History {
        self.eol = false;
        self
    }
}

impl<L: StyleLabel, C: Condition> Notation<L, C> {
    /// If no flaws are found in this [`Notation`], convert it into a [`ValidNotation`].
    pub fn validate(self) -> Result<ValidNotation<L, C>, NotationError> {
        self.validate_rec(History::default(), Context::new())?;
        Ok(ValidNotation(self))
    }

    #[doc(hidden)]
    // For testing how the pretty printer handles a notation that would have failed validation.
    pub fn cheat_validation_for_testing_only(self) -> ValidNotation<L, C> {
        ValidNotation(self)
    }

    /// Returns `Err` if a flaw is detected.
    ///
    fn validate_rec(&self, history: History, ctx: Context) -> Result<History, NotationError> {
        use FoldContext::*;
        use Notation::*;
        use NotationError::*;

        match self {
            Text if ctx.count.is_some() => Err(TextInsideCount),
            Text if ctx.fold.is_some() => Err(TextInsideFold),
            Empty => Ok(history),
            Text | Literal(_) if history.eol => Err(TextAfterEol),
            Text | Literal(_) => Ok(history.without_eol()),
            Newline => Ok(history.without_eol()),
            EndOfLine => Ok(history.with_eol()),
            Flat(note) => note.validate_rec(history, ctx),
            Indent(_, _, note) => note.validate_rec(history, ctx),
            Concat(note1, note2) => {
                let history = note1.validate_rec(history, ctx)?;
                note2.validate_rec(history, ctx)
            }
            Choice(note1, note2) => {
                let history_1 = note1.validate_rec(history, ctx)?;
                let history_2 = note2.validate_rec(history, ctx)?;
                Ok(history_1.choice(history_2))
            }
            Check(_, pos, note1, note2) => {
                match &pos {
                    CheckPos::Here => (),
                    CheckPos::Child(_) if ctx.count == Some(CountContext::Zero) => {
                        return Err(CountZeroCheckPosChild)
                    }
                    CheckPos::Child(i)
                        if ctx.count == Some(CountContext::One)
                            && normalize_child_index(*i, 1).is_none() =>
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
                let history_1 = note1.validate_rec(history, ctx)?;
                let history_2 = note2.validate_rec(history, ctx)?;
                Ok(history_1.choice(history_2))
            }
            Child(_) if ctx.count == Some(CountContext::Zero) => Err(CountZeroChild),
            Child(i)
                if ctx.count == Some(CountContext::One)
                    && normalize_child_index(*i, 1).is_none() =>
            {
                Err(CountOneChildIndex(*i))
            }
            Child(_) => Ok(history.without_eol()),
            Style(_, note) => note.validate_rec(history, ctx),
            FocusMark if history.mark => Err(MultipleFocusMarks),
            FocusMark => Ok(history.with_mark()),
            Count { .. } if ctx.count.is_some() => Err(NestedCount),
            Count { zero, one, many } => {
                let history_1 = zero.validate_rec(history, ctx.count_zero())?;
                let history_2 = one.validate_rec(history, ctx.count_one())?;
                let history_3 = many.validate_rec(history, ctx.count_many())?;
                Ok(history_1.choice(history_2).choice(history_3))
            }
            Fold { .. } if ctx.fold.is_some() => Err(NestedFold),
            Fold { first, join } => {
                // Can't easily check for EOL/mark here
                first.validate_rec(History::uncertain(), ctx.fold_first())?;
                join.validate_rec(History::uncertain(), ctx.fold_join())?;
                Ok(History::uncertain())
            }
            Left if ctx.fold != Some(InFoldJoin) => Err(LeftOutsideJoin),
            Right if ctx.fold != Some(InFoldJoin) => Err(RightOutsideJoin),
            // Can't easily check for EOL/mark here
            Left | Right => Ok(History::uncertain()),
        }
    }
}

impl<L: StyleLabel, C: Condition> fmt::Display for ValidNotation<L, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
