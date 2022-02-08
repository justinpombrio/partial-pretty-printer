use crate::notation::Notation;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    Texty,
    Listy,
    Fixed(usize),
}

pub struct ValidNotation {
    notation: Notation,
    arity: Arity,
}

#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("Notation validation error: newline inside a Flat")]
    FlattenedNewline,
    #[error("Notation validation error: There is no possible layout that does not Flat-ten any children. It is required that there be one, in case all children contain unconditional newlines.")]
    FlattenedChildren,
    #[error(
        "Notation validation error: Found a Repeat {0}. Repeats are not allowed to be nested."
    )]
    NestedRepeat(Context),
    #[error("Notation validation error: IfEmptyText may only appear in texty nodes, but found it in a {0}.")]
    ArityMismatchIfEmptyText(Arity),
    #[error(
        "Notation validation error: Text may only appear in texty nodes, but found it in a {0}."
    )]
    ArityMismatchText(Arity),
    #[error("Notation validation error: Child may only appear in listy nodes(*), but found it in a {0}. (*) With one exception: Child(0) can appear in the `lone` case of a Repeat to refer to the sole child of the node.")]
    ArityMismatchChild(Arity),
    #[error("Notation validation error: Child index out of bounds. Found index {0} in a {1}. (Indexing is zero based.)")]
    ArityMismatchChildIndex(usize, Arity),
    #[error(
        "Notation validation error: Repeats may only occur in listy nodes, but found one in a {0}."
    )]
    ArityMismatchRepeat(Arity),
    #[error("Notation validation error: Left may only occur inside the `join` case of a Repeat, but found one {0}.")]
    LeftOutsideJoin(Context),
    #[error("Notation validation error: Right may only occur inside the `join` case of a Repeat, but found one {0}.")]
    RightOutsideJoin(Context),
    #[error("Notation validation error: Surround may only occur inside the `surround` case of a Repeat, but found one {0}.")]
    SurroundedOutsideSurround(Context),
}

impl fmt::Display for Arity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Arity::*;

        match self {
            Texty => write!(f, "texty node"),
            Listy => write!(f, "listy node"),
            Fixed(num_children) => write!(f, "fixed node with {} children", num_children),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    /// Not inside a `Repeat`.
    None,
    /// Inside the `empty` of a `Repeat`.
    Empty,
    /// Inside the `lone` of a `Repeat`.
    Lone,
    /// Inside the `surround` of a `Repeat`.
    Surround,
    /// Inside the `join` of a `Repeat`.
    Join,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Context::None;
        use Context::*;

        match self {
            None => write!(f, "outside of Repeat"),
            Empty => write!(f, "in the `empty` part of a Repeat"),
            Lone => write!(f, "in the `lone` part of a Repeat"),
            Surround => write!(f, "in the `surround` part of a Repeat"),
            Join => write!(f, "in the `join` part of a Repeat"),
        }
    }
}

impl Notation {
    pub fn validate(self, arity: Arity) -> Result<ValidNotation, ValidationError> {
        let free_range_children = self.validate_rec(arity, Context::None, false)?;
        if free_range_children {
            Ok(ValidNotation {
                notation: self,
                arity,
            })
        } else {
            Err(ValidationError::FlattenedChildren)
        }
    }

    /// Recursive calls of `validate`:
    /// - `is_flat` says whether this node in the notation is inside a Flat.
    /// - The result bool is true if there is at least one layout in which no children are
    ///   flattened. This is a requirement in case all children contain an unconditional newline.
    fn validate_rec(
        &self,
        arity: Arity,
        ctx: Context,
        is_flat: bool,
    ) -> Result<bool, ValidationError> {
        use Notation::*;
        use ValidationError::*;

        match self {
            Empty | Literal(_) => Ok(true),
            Newline => {
                if is_flat {
                    Err(FlattenedNewline)
                } else {
                    Ok(true)
                }
            }
            Flat(note) => note.validate_rec(arity, ctx, true),
            Indent(_, note) => note.validate_rec(arity, ctx, is_flat),
            Left => {
                if ctx != Context::Join {
                    Err(LeftOutsideJoin(ctx))
                } else {
                    // TODO: too strict. Likewise the Surrounded case (but not the Right case,
                    // which refers to a child!).
                    Ok(!is_flat)
                }
            }
            Right => {
                if ctx != Context::Join {
                    Err(LeftOutsideJoin(ctx))
                } else {
                    Ok(!is_flat)
                }
            }
            Surrounded => {
                if ctx != Context::Surround {
                    Err(SurroundedOutsideSurround(ctx))
                } else {
                    Ok(!is_flat)
                }
            }
            Text(_) => {
                if arity != Arity::Texty {
                    Err(ArityMismatchText(arity))
                } else {
                    Ok(true)
                }
            }
            Child(i) => match arity {
                Arity::Texty => Err(ArityMismatchChild(arity)),
                Arity::Listy if !(*i == 0 && ctx == Context::Lone) => {
                    Err(ArityMismatchChild(arity))
                }
                Arity::Fixed(n) if *i >= n => Err(ArityMismatchChildIndex(*i, arity)),
                _ => Ok(!is_flat),
            },
            IfEmptyText(note1, note2) => {
                if arity != Arity::Texty {
                    return Err(ArityMismatchIfEmptyText(arity));
                }
                let r1 = note1.validate_rec(arity, ctx, is_flat)?;
                let r2 = note2.validate_rec(arity, ctx, is_flat)?;
                Ok(r1 || r2)
            }
            Concat(note1, note2) => {
                let r1 = note1.validate_rec(arity, ctx, is_flat)?;
                let r2 = note2.validate_rec(arity, ctx, is_flat)?;
                Ok(r1 && r2)
            }
            Choice(note1, note2) => {
                let r1 = note1.validate_rec(arity, ctx, is_flat)?;
                let r2 = note2.validate_rec(arity, ctx, is_flat)?;
                Ok(r1 || r2)
            }
            Repeat(repeat) => {
                if ctx != Context::None {
                    return Err(NestedRepeat(ctx));
                }
                if arity != Arity::Listy {
                    return Err(ArityMismatchRepeat(arity));
                }

                let empty = repeat.empty.validate_rec(arity, ctx, is_flat)?;
                let lone = repeat.lone.validate_rec(arity, ctx, is_flat)?;
                let join = repeat.join.validate_rec(arity, ctx, is_flat)?;
                let surround = repeat.surround.validate_rec(arity, ctx, is_flat)?;

                Ok(empty && lone && join && surround)
            }
        }
    }
}
