//! Sample document types, with notations.
//!
//! - [`Json`](json::Json)

mod json_notation;

pub mod json {
    pub use super::json_notation::*;
}
