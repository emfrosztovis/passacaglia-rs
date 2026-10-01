// Musical values here are tiny integers, so the `as` casts are in range and
// lossless in practice. These pedantic cast lints are pinned to `allow` here
// (rather than `try_from().unwrap()` everywhere) so the IDE and `cargo clippy`
// agree.
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

//! Shared primitives for the passacaglia music engine: an exact rational-number
//! type and a handful of utility helpers.

mod rational;
mod utils;

pub use rational::{rational, rational_to_string, rational_value, Rational};
pub use utils::rotate_array;
