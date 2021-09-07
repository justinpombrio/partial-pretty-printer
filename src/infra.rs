#[cfg(feature = "profile")]
pub use no_nonsense_flamegraphs::span;

#[cfg(not(feature = "profile"))]
#[macro_export]
macro_rules! __span {
    ($name:expr) => {};
}

// Workaround for the fact that `macro_export` puts the macro at the crate root.
#[cfg(not(feature = "profile"))]
pub use crate::__span as span;
