use super::notation_ref::{NotationCase, NotationRef};
use super::pretty_doc::PrettyDoc;
use crate::geometry::Width;
use crate::infra::span;
use crate::style::{Shade, Style};
use std::iter::Iterator;

/// Determine which of the two options of the choice to select. Pick the first option if it fits,
/// or if the second option is invalid.
fn choose<'d, D: PrettyDoc<'d>>(
    width: Width,
    indent: Option<Width>,
    prefix_len: Width,
    opt1: NotationRef<'d, D>,
    opt2: NotationRef<'d, D>,
    suffix: &[Chunk<'d, D>],
) -> NotationRef<'d, D> {
    span!("choose");

    let flat = indent.is_none();
    let chunks = suffix
        .iter()
        .map(|(i, _, n)| (i.is_none(), *n))
        .chain(std::iter::once((flat, opt1)))
        .collect::<Vec<_>>()
        .into_iter()
        .rev();

    if width < prefix_len {
        return opt2;
    }
    let width = width - prefix_len;

    if fits_all(width, chunks) && is_valid(flat, opt1) || !is_valid(flat, opt2) {
        opt1
    } else {
        opt2
    }
}

/// The amount of space remaining on a line, for use in `fits` computation.
enum RemainingWidth {
    /// There's this much space left.
    SingleLine(Width),
    /// Does not fit. Either the line width was exceeded, or encountered an error (flat of
    /// newline).
    NoFit,
    /// Definitely fits because we encountered a newline.
    MultiLine,
}

impl RemainingWidth {
    fn and_then(self, f: impl FnOnce(Width) -> RemainingWidth) -> RemainingWidth {
        use RemainingWidth::*;

        match self {
            NoFit => NoFit,
            MultiLine => MultiLine,
            SingleLine(remaining_width) => f(remaining_width),
        }
    }

    fn or_else(self, f: impl FnOnce() -> RemainingWidth) -> RemainingWidth {
        use RemainingWidth::*;

        match self {
            NoFit => f(),
            MultiLine => MultiLine,
            SingleLine(w1) => match f() {
                NoFit => SingleLine(w1),
                MultiLine => MultiLine,
                SingleLine(w2) => SingleLine(w1.max(w2)),
            },
        }
    }
}

/// Determine whether the first line of the chunks fits within the `remaining` space.
fn fits_all<'d, D: PrettyDoc<'d>>(
    mut width: Width,
    chunks: impl Iterator<Item = (bool, NotationRef<'d, D>)>,
) -> bool {
    use RemainingWidth::*;

    span!("fits_all");

    for (flat, note) in chunks {
        match fits(width, flat, note) {
            NoFit => return false,
            MultiLine => return true,
            SingleLine(remaining_width) => {
                width = remaining_width;
            }
        }
    }
    true
}

// A wrapper around the recursive function, so that the profiler doesn't get invoked on each
// recursion.
fn fits<'d, D: PrettyDoc<'d>>(
    width: Width,
    flat: bool,
    notation: NotationRef<'d, D>,
) -> RemainingWidth {
    span!("fits");

    fits_rec(width, flat, notation)
}

/// Determine whether the first line of the notation fits within the `remaining` space.
fn fits_rec<'d, D: PrettyDoc<'d>>(
    width: Width,
    flat: bool,
    notation: NotationRef<'d, D>,
) -> RemainingWidth {
    use NotationCase::*;
    use RemainingWidth::*;

    match notation.case() {
        Empty => SingleLine(width),
        Literal(lit) => {
            let lit_len = lit.len();
            if lit_len <= width {
                SingleLine(width - lit_len)
            } else {
                NoFit
            }
        }
        Text(text, _) => {
            let text_len = text.chars().count() as Width;
            if text_len <= width {
                SingleLine(width - text_len)
            } else {
                NoFit
            }
        }
        Newline if flat => NoFit,
        Newline => MultiLine,
        Flat(note) => fits_rec(width, true, note),
        Indent(_, note) => fits_rec(width, flat, note),
        Child(_, note) => fits_rec(width, flat, note),
        Concat(note1, note2) => fits_rec(width, flat, note1).and_then(|w| fits_rec(w, flat, note2)),
        // NOTE: This is linear time in the size of the `NotationRef`, which could be exponential
        // in the size of the `Notation`, because of `Repeat`s. If we assume that the first line of
        // `opt2` is no longer than the first line of `opt1`, then this can only ever traverse one
        // branch of the Choice by saying:
        // ```
        // if is_valid(opt2) { fits(opt2) } else { fits(opt1) }
        // ```
        // The downside is that this makes the implementation behave differently than the oracle.
        //
        // (Of course, this only helps if we also make sure that `is_valid` only ever inspects one
        //  option of a Choice.)
        Choice(opt1, opt2) => fits_rec(width, flat, opt1).or_else(|| fits_rec(width, flat, opt2)),
    }
}

// A wrapper around the recursive function, so that the profiler doesn't get invoked on each
// recursion.
fn is_valid<'d, D: PrettyDoc<'d>>(flat: bool, notation: NotationRef<'d, D>) -> bool {
    span!("is_valid");
    is_valid_rec(flat, notation)
}

fn is_valid_rec<'d, D: PrettyDoc<'d>>(flat: bool, notation: NotationRef<'d, D>) -> bool {
    use NotationCase::*;

    match notation.case() {
        Empty | Literal(_) | Text(_, _) => true,
        Newline => !flat,
        Flat(note) => is_valid_rec(true, note),
        Indent(_, note) => is_valid_rec(flat, note),
        // TODO: As an optimization, pre-compute whether opt2 has an unconditional newline.
        Choice(opt1, opt2) => is_valid_rec(flat, opt1) || is_valid_rec(flat, opt2),
        Concat(note1, note2) => is_valid_rec(flat, note1) && is_valid_rec(flat, note2),
        Child(_, child_note) => !flat || is_valid_rec(flat, child_note),
    }
}
