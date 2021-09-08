use crate::standard::pretty_testing::{assert_pp_without_expectation, SimpleDoc};
use crate::standard::random_testing;
use partial_pretty_printer::{
    notation_constructors::{empty, flat, lit, nl},
    Notation, Style,
};

impl random_testing::Arbitrary for Notation {
    // TODO: construct error notations
    fn make(mut size: u32, mut gen: random_testing::Gen) -> Notation {
        assert_ne!(size, 0);
        if size == 1 {
            match gen.pick(5) {
                0 => empty(),
                1 => nl(),
                2 => lit("a", Style::default()),
                3 => lit("bb", Style::default()),
                4 => lit("cccc", Style::default()),
                _ => unreachable!(),
            }
        } else if size == 2 {
            match gen.pick(2) {
                0 => flat(Notation::make(1, gen)),
                1 => 2 >> Notation::make(1, gen),
                _ => unreachable!(),
            }
        } else {
            size -= 1;
            match gen.pick(4) {
                0 => {
                    let left_size = gen.pick(size - 1) + 1;
                    let right_size = size - left_size;
                    let left = Notation::make(left_size, gen.reborrow());
                    let right = Notation::make(right_size, gen);
                    left + right
                }
                1 => {
                    let left_size = gen.pick(size - 1) + 1;
                    let right_size = size - left_size;
                    let left = Notation::make(left_size, gen.reborrow());
                    let right = Notation::make(right_size, gen);
                    left | right
                }
                2 => flat(Notation::make(size, gen)),
                3 => 2 >> Notation::make(size, gen),
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn oracle_tests() {
    // TODO: random notations too
    for notation in random_testing::all::<Notation>(5) {
        println!("{}", notation);
        let doc = SimpleDoc(notation);
        for width in 1..=8 {
            assert_pp_without_expectation(&doc, width);
        }
    }
}
