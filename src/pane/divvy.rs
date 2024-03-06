use crate::pane::PaneSize;

/// A utility for fairly dividing up space (either width or height) between multiple panes.
pub struct Divvier {
    /// Each pane's size request
    pane_sizes: Vec<PaneSize>,
    /// Already allocated cookies/space, for each pane in `pane_sizes`.
    allocations: Vec<usize>,
    /// Remaining unallocated cookies/space
    cookies: usize,
}

impl Divvier {
    /// Construct a Divvier and immediately allocate space to the `Fixed` panes.
    pub fn new(available_size: usize, pane_sizes: Vec<PaneSize>) -> Divvier {
        let mut divvier = Divvier {
            allocations: vec![0; pane_sizes.len()],
            pane_sizes,
            cookies: available_size,
        };
        divvier.allocate_fixed();
        divvier
    }

    /// Get the remaining unallocated space. If called right after constructing the Divvier with
    /// `new()`, this will be the space available for the `Dynamic` and `Proportional` panes.
    pub fn remaining(&self) -> usize {
        self.cookies
    }

    /// Compute and return the final allocation for each pane. `dynamic_sizes` must contain the size
    /// of the document inside each `Dynamic` pane, given in the same order as in `pane_sizes` (but
    /// with non-`Dynamic` panes omitted).
    pub fn finish(mut self, dynamic_sizes: Vec<usize>) -> Vec<usize> {
        self.allocate_dynamic(dynamic_sizes);
        self.allocate_proportional();
        self.allocations
    }

    fn allocate_dynamic(&mut self, sizes: Vec<usize>) {
        let sum: usize = sizes.iter().sum();
        assert!(sum <= self.cookies);
        self.cookies -= sum;
        let mut sizes = sizes.into_iter();
        for (i, pane_size) in self.pane_sizes.iter().enumerate() {
            if let PaneSize::Dynamic = pane_size {
                self.allocations[i] = sizes
                    .next()
                    .expect("Divvier.allocate_dynamic(): not enough allocations");
            }
        }
        assert_eq!(
            sizes.next(),
            None,
            "Divvier.allocate_dynamic(): too many allocations"
        );
    }

    /// Divvy `cookies` up among children, where each child requires a fixed number of cookies,
    /// returning the allocation and the number of remaining cookies. If there aren't enough
    /// cookies, it's first-come first-serve. ("Children" = "fixed PaneSizes")
    fn allocate_fixed(&mut self) {
        for (i, pane_size) in self.pane_sizes.iter().enumerate() {
            if let PaneSize::Fixed(hunger) = pane_size {
                let cookies_given = self.cookies.min(*hunger);
                self.cookies -= cookies_given;
                self.allocations[i] = cookies_given;
            }
        }
    }

    /// Divvy `cookies` up among children as fairly as possible, where the `i`th child has
    /// `child_hungers[i]` hunger. Children should receive cookies in proportion to their hunger,
    /// with the difficulty that cookies cannot be split into pieces. Exact ties go to the leftmost
    /// tied child. ("Children" = "proportional PaneSizes")
    fn allocate_proportional(&mut self) {
        let mut child_hungers = Vec::new();
        for pane_size in &self.pane_sizes {
            if let PaneSize::Proportional(hunger) = pane_size {
                child_hungers.push(*hunger);
            }
        }
        let total_hunger: usize = child_hungers.iter().sum();
        // Start by allocating each child a guaranteed minimum number of cookies,
        // found as the floor of the real number of cookies they deserve.
        let mut cookie_allocation: Vec<usize> = child_hungers
            .iter()
            .map(|hunger| self.cookies * hunger / total_hunger)
            .collect();
        // Compute the number of cookies still remaining.
        let allocated_cookies: usize = cookie_allocation.iter().sum();
        let leftover: usize = self.cookies - allocated_cookies;
        // Determine what fraction of a cookie each child still deserves, found as
        // the remainder of the above division. Then hand out the remaining cookies
        // to the children with the largest remainders.
        let mut remainders: Vec<(usize, usize)> = child_hungers
            .iter()
            .map(|hunger| self.cookies * hunger % total_hunger)
            .enumerate()
            .collect();
        remainders.sort_by(|(_, r1), (_, r2)| r2.cmp(r1));
        remainders
            .into_iter()
            .take(leftover)
            .for_each(|(i, _)| cookie_allocation[i] += 1);
        // Set the maximally-fair cookie allocation.
        let mut cookie_allocation = cookie_allocation.into_iter();
        for (i, pane_size) in self.pane_sizes.iter().enumerate() {
            if let PaneSize::Proportional(_) = pane_size {
                let cookies = cookie_allocation.next().unwrap();
                self.allocations[i] = cookies;
                self.cookies -= cookies;
            }
        }
    }
}

#[test]
fn test_proportional_division() {
    fn proportionally_divide(cookies: usize, hungers: &[usize]) -> Vec<usize> {
        let pane_sizes = hungers
            .iter()
            .copied()
            .map(PaneSize::Proportional)
            .collect::<Vec<_>>();
        let divvier = Divvier::new(cookies, pane_sizes);
        divvier.finish(Vec::new())
    }

    assert_eq!(proportionally_divide(0, &[1, 1]), vec!(0, 0));
    assert_eq!(proportionally_divide(1, &[1, 1]), vec!(1, 0));
    assert_eq!(proportionally_divide(2, &[1, 1]), vec!(1, 1));
    assert_eq!(proportionally_divide(3, &[1, 1]), vec!(2, 1));
    assert_eq!(proportionally_divide(4, &[10, 11, 12]), vec!(1, 1, 2));
    assert_eq!(proportionally_divide(5, &[17]), vec!(5));
    assert_eq!(proportionally_divide(5, &[12, 10, 11]), vec!(2, 1, 2));
    assert_eq!(proportionally_divide(5, &[10, 10, 11]), vec!(2, 1, 2));
    assert_eq!(proportionally_divide(5, &[2, 0, 1]), vec!(3, 0, 2));
    assert_eq!(proportionally_divide(61, &[1, 2, 3]), vec!(10, 20, 31));
    assert_eq!(
        proportionally_divide(34583, &[55, 98, 55, 7, 12, 200]),
        vec!(4455, 7937, 4454, 567, 972, 16198)
    );
}
