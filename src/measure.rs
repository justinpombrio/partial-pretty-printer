use crate::geometry::Width;

#[derive(Debug, Clone, Copy)]
pub struct Measure {
    /// The minimum possible length of this subdocument if it is rendered flat (without newlines),
    /// or `None` if it cannot be rendered flat.
    pub flat_len: Option<Width>,
    /// The minimum possible length of the first line of this subdocument up until the first
    /// newline, or `None` if it cannot contain a newline.
    pub nonflat_len: Option<Width>,
}

impl Measure {
    pub fn flat(flat_len: Width) -> Measure {
        Measure {
            flat_len: Some(flat_len),
            nonflat_len: None,
        }
    }

    pub fn nonflat(nonflat_len: Width) -> Measure {
        Measure {
            flat_len: None,
            nonflat_len: Some(nonflat_len),
        }
    }

    pub fn flattened(self) -> Measure {
        Measure {
            flat_len: self.flat_len,
            nonflat_len: None,
        }
    }

    pub fn concat(self, other: Measure) -> Measure {
        Measure {
            flat_len: match (self.flat_len, other.flat_len) {
                (Some(x), Some(y)) => Some(x + y),
                (_, _) => None,
            },
            nonflat_len: match (self.nonflat_len, self.flat_len, other.nonflat_len) {
                (Some(lfirst), Some(lflat), Some(rfirst)) => Some(lfirst.min(lflat + rfirst)),
                (Some(lfirst), _, _) => Some(lfirst),
                (None, Some(lflat), Some(rfirst)) => Some(lflat + rfirst),
                (_, _, _) => None,
            },
        }
    }

    pub fn choice(self, other: Measure) -> Measure {
        Measure {
            flat_len: match (self.flat_len, other.flat_len) {
                (Some(x), Some(y)) => Some(x.min(y)),
                (Some(x), None) | (None, Some(x)) => Some(x),
                (None, None) => None,
            },
            nonflat_len: match (self.nonflat_len, other.nonflat_len) {
                (Some(x), Some(y)) => Some(x.min(y)),
                (Some(x), None) | (None, Some(x)) => Some(x),
                (None, None) => None,
            },
        }
    }

    pub fn is_valid(self) -> bool {
        match (self.nonflat_len, self.flat_len) {
            (None, None) => false,
            (_, _) => true,
        }
    }

    pub fn fits_in_width(self, width: Width) -> bool {
        match (self.nonflat_len, self.flat_len) {
            (Some(x), Some(y)) => x <= width || y <= width,
            (Some(x), None) | (None, Some(x)) => x <= width,
            (None, None) => false,
        }
    }
}
