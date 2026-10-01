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

//! Shared primitives for the passacaglia music engine.
//!
//! This crate mirrors the small slice of the original TypeScript `common`
//! package that `core` depends on: an exact rational-number type (a type alias
//! over [`num_rational::Ratio`]) and a handful of utility helpers. It does not
//! re-implement hashing, `HashMap`, or a bespoke `Rational` — those concerns are
//! handled by the standard library and `num-rational`.

mod rational;
mod utils;

pub use rational::{rational, rational_to_string, rational_value, Rational};
pub use utils::rotate_array;
