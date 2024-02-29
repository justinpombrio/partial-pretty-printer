use crate::standard::generative_testing::{generate_all, generate_random, Generator, Picker};
use crate::standard::pretty_testing::{assert_pp_without_expectation, SimpleDoc};
use partial_pretty_printer::{
    notation_constructors::{empty, eol, flat, lit, nl},
    Notation,
};

struct NotationGen;

impl Generator for NotationGen {
    type Value = Notation<(), ()>;

    fn generate<P: Picker>(&self, mut size: u32, picker: &mut P) -> Notation<(), ()> {
        assert_ne!(size, 0);
        if size == 1 {
            match picker.pick_int(6) {
                0 => empty(),
                1 => nl(),
                2 => eol(),
                3 => lit("a"),
                4 => lit("bb"),
                5 => lit("cccc"),
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
    let notations = generate_all(NotationGen, 6)
        .chain(generate_random(NotationGen, 10, [0; 32]).take(1000))
        .chain(generate_random(NotationGen, 20, [0; 32]).take(1000))
        .chain(generate_random(NotationGen, 30, [0; 32]).take(1000))
        .chain(generate_random(NotationGen, 50, [0; 32]).take(200));

    let mut valid_count = 0;
    let mut invalid_count = 0;
    for notation in notations {
        if let Ok(doc) = SimpleDoc::try_new(notation) {
            for width in 1..=8 {
                assert_pp_without_expectation(&doc, width);
            }
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }
    println!(
        "Tested {} valid / {} total notations",
        valid_count,
        valid_count + invalid_count
    );
}
