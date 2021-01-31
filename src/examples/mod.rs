//! [FILL]

mod doc;
mod json_notation;

pub use doc::{Doc, Sort};

pub mod json {
    pub use super::json_notation::*;
}
