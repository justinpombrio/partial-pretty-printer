use rand::{rngs::StdRng, Rng, SeedableRng};
use std::marker::PhantomData;

/// An interface for types that can be randomly (or deterministically) generated, for use as test
/// inputs. Implementors must obey these requirements:
///
/// - `make` must be a pure function. That is, it must behave deterministically (given the output
///   of `gen`), and it cannot have side effects.
/// - For any given `size`, `make` must produce only finitely many possible values.
pub trait Arbitrary: Sized {
    fn make<'g>(size: u32, gen: Gen<'g>) -> Self;
}

// Wrapper to make the enum variants private.
/// A source of randomness (or determinism) for constructing [Arbitrary] values.
pub struct Gen<'g>(GenEnum<'g>);

enum GenEnum<'g> {
    Random(&'g mut StdRng),
    All(&'g mut GenAll),
}

/// Construct an infinite stream of random values of the given size.
pub fn random<A: Arbitrary>(size: u32, seed: [u8; 32]) -> impl Iterator<Item = A> {
    GenRandomIter::new(size, seed)
}

/// Construct a finite stream of all values of type `A` of the given size.
pub fn all<A: Arbitrary>(size: u32) -> impl Iterator<Item = A> {
    GenAllIter::new(size)
}

impl<'g> Gen<'g> {
    /// Generate an integer between 0 and `max`, excluding `max`.
    pub fn pick(&mut self, max: u32) -> u32 {
        self.0.pick(max)
    }

    pub fn reborrow<'a>(&'a mut self) -> Gen<'a> {
        Gen(self.0.reborrow())
    }
}

impl<'g> GenEnum<'g> {
    fn pick(&mut self, max: u32) -> u32 {
        assert_ne!(max, 0);

        match self {
            GenEnum::Random(rng) => rng.gen_range(0..max),
            GenEnum::All(iter) => iter.pick(max),
        }
    }

    fn reborrow<'a>(&'a mut self) -> GenEnum<'a> {
        match self {
            GenEnum::Random(rng) => GenEnum::Random(rng),
            GenEnum::All(all) => GenEnum::All(all),
        }
    }
}

struct GenAll {
    index: usize,
    stack: Vec<(u32, u32)>,
    done: bool,
}

impl GenAll {
    fn new() -> GenAll {
        GenAll {
            index: 0,
            stack: vec![],
            done: false,
        }
    }

    fn advance(&mut self) {
        self.index = 0;
        while let Some((n, max)) = self.stack.pop() {
            if n + 1 < max {
                self.stack.push((n + 1, max));
                return;
            }
        }
        if self.stack.is_empty() {
            self.done = true;
        }
    }

    fn pick(&mut self, max: u32) -> u32 {
        if let Some((n, _)) = self.stack.get(self.index) {
            self.index += 1;
            *n
        } else {
            assert_eq!(self.index, self.stack.len());
            self.stack.push((0, max));
            self.index += 1;
            0
        }
    }
}

struct GenAllIter<A: Arbitrary> {
    size: u32,
    gen: GenAll,
    phantom: PhantomData<A>,
}

impl<A: Arbitrary> GenAllIter<A> {
    fn new(size: u32) -> GenAllIter<A> {
        GenAllIter {
            size,
            gen: GenAll::new(),
            phantom: PhantomData,
        }
    }
}

impl<A: Arbitrary> Iterator for GenAllIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<A> {
        if self.gen.done {
            None
        } else {
            let item = A::make(self.size, Gen(GenEnum::All(&mut self.gen)));
            self.gen.advance();
            Some(item)
        }
    }
}

struct GenRandomIter<A: Arbitrary> {
    size: u32,
    rng: StdRng,
    phantom: PhantomData<A>,
}

impl<A: Arbitrary> GenRandomIter<A> {
    fn new(size: u32, seed: [u8; 32]) -> GenRandomIter<A> {
        GenRandomIter {
            size,
            rng: StdRng::from_seed(seed),
            phantom: PhantomData,
        }
    }
}

impl<A: Arbitrary> Iterator for GenRandomIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<A> {
        let gen = Gen(GenEnum::Random(&mut self.rng));
        Some(A::make(self.size, gen))
    }
}

#[test]
fn test_random_testing() {
    use std::fmt;

    #[derive(Debug)]
    struct Tree(Vec<Tree>);

    impl fmt::Display for Tree {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "(")?;
            let len = self.0.len();
            for (i, child) in self.0.iter().enumerate() {
                write!(f, "{}", child)?;
                if i + 1 != len {
                    write!(f, " ")?;
                }
            }
            write!(f, ")")
        }
    }

    impl Arbitrary for Tree {
        /// Panics if size is zero, as there are no trees of size 0!
        fn make(mut size: u32, mut gen: Gen) -> Tree {
            assert_ne!(size, 0);
            // Account for this node
            size -= 1;

            // Divvy `size` out to any number of children.
            let mut children = vec![];
            while size > 0 {
                let mut gen = gen.reborrow();
                let child_size = gen.pick(size) + 1;
                size -= child_size;
                children.push(Tree::make(child_size, gen));
            }
            Tree(children)
        }
    }

    let trees = all::<Tree>(5).collect::<Vec<_>>();
    // for tree in &trees {
    //     println!("{}", tree);
    // }
    assert_eq!(trees.len(), 14);
}
