use crate::standard::generative_testing::{generate_all, generate_random, Generator, Picker};
use crate::standard::pretty_testing::{assert_pp_without_expectation, SimpleDoc};
use partial_pretty_printer::{
    notation_constructors::{empty, flat, lit, nl},
    Arity, Notation, Style,
};

struct SimpleNotationGenerator;

impl Generator for SimpleNotationGenerator {
    type Value = Notation;

    fn generate<P: Picker>(&self, mut size: u32, picker: &mut P) -> Notation {
        assert_ne!(size, 0);
        if size == 1 {
            match picker.pick_int(5) {
                0 => empty(),
                1 => nl(),
                2 => lit("a", Style::default()),
                3 => lit("bb", Style::default()),
                4 => lit("cccc", Style::default()),
                _ => unreachable!(),
            }
        } else if size == 2 {
            match picker.pick_int(2) {
                0 => flat(self.generate(1, picker)),
                1 => 2 >> self.generate(1, picker),
                _ => unreachable!(),
            }
        } else {
            size -= 1;
            match picker.pick_int(4) {
                0 => {
                    let left_size = picker.pick_int(size - 1) + 1;
                    let right_size = size - left_size;
                    let left = self.generate(left_size, picker);
                    let right = self.generate(right_size, picker);
                    left + right
                }
                1 => {
                    let left_size = picker.pick_int(size - 1) + 1;
                    let right_size = size - left_size;
                    let left = self.generate(left_size, picker);
                    let right = self.generate(right_size, picker);
                    left | right
                }
                2 => flat(self.generate(size, picker)),
                3 => 2 >> self.generate(size, picker),
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn oracle_tests() {
    let seed = [0; 32];

    let small_notations = (1..7).flat_map(|size| generate_all(SimpleNotationGenerator, size));
    let random_notations =
        (1..100).flat_map(|size| generate_random(SimpleNotationGenerator, size, seed).take(1000));
    let notations = small_notations.chain(random_notations);

    for notation in notations {
        if notation.validate(Arity::Fixed(0)).is_ok() {
            let doc = SimpleDoc(notation);
            for width in 1..=8 {
                assert_pp_without_expectation(&doc, width);
            }
        } else {
            // TODO: don't print
            println!("INVALID {}", notation);
        }
    }
}
