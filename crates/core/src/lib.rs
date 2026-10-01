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
pub use scale::Scale;
pub use system::{PitchSystem, ET12};
pub use tuning::{EqualTemperament, Tuning};

pub use passacaglia_common::Rational;
