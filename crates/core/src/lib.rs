// Musical values here are tiny integers, so the `as` casts are in range and
// lossless in practice. These pedantic cast lints are pinned to `allow` here
// (rather than `try_from().unwrap()` everywhere) so the IDE and `cargo clippy`
// agree. `missing_*_doc` and `wildcard_imports` are also allowed for the same
// consistency reason.
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::wildcard_imports
)]

//! Core musical abstractions for the passacaglia music engine.
//!
//! This crate models pitch/interval/scale systems. A pitch is a
//! `(degree, accidental, period)` triple that keeps musical spelling distinct
//! from sounding frequency; a [`PitchSystem`] describes the abstract structure
//! (period ratio, pitch classes, degrees) of a system.

pub mod degree;
pub mod interval;
pub mod pitch;
pub mod scale;
pub mod structure;
pub mod system;
pub mod tuning;

pub mod std_hept;

pub use degree::Degree;
pub use interval::Interval;
pub use pitch::Pitch;
pub use scale::{DegreeDefinition, Scale};
pub use system::{PitchSystem, ET12};
pub use tuning::{EqualTemperament, Tuning};

pub use passacaglia_common::Rational;
