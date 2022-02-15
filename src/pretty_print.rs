use crate::geometry::Width;
use crate::measure::Measure;
use crate::notation::{MeasuredNotation, Notation};

pub fn pretty_print_to_string(notation: &MeasuredNotation, width: Width) -> String {
    use Notation::*;

    let mut output = String::new();
    let mut prefix_len: Width = 0;
    let mut stack: Vec<(Option<Width>, &MeasuredNotation, Measure)> =
        vec![(Some(0), notation, Measure::flat(0))];
    while let Some((indent, note, suffix_len)) = stack.pop() {
        match &*note.notation {
            Empty => (),
            Newline => {
                output.push('\n');
                for _ in 0..indent.unwrap() {
                    output.push(' ');
                }
                prefix_len = indent.unwrap();
            }
            Literal(lit, _) => {
                output.push_str(lit);
                prefix_len += note.measure.flat_len.unwrap();
            }
            Flat(note) => stack.push((None, note, suffix_len)),
            Indent(j, note) => stack.push((indent.map(|i| i + j), note, suffix_len)),
            Concat(l_note, r_note) => {
                stack.push((indent, r_note, suffix_len));
                stack.push((indent, l_note, r_note.measure.concat(suffix_len)));
            }
            Choice(l_note, r_note) => {
                let l_fits = Measure::flat(prefix_len)
                    .concat(l_note.measure)
                    .concat(suffix_len)
                    .fits_in_width(width);
                if l_fits || !r_note.measure.is_valid() {
                    stack.push((indent, l_note, suffix_len));
                } else {
                    stack.push((indent, r_note, suffix_len));
                }
            }
        }
    }
    output
}
