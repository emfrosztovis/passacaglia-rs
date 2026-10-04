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
//! The core structures defined in this crate aim to be as general and unbiased as possible while 
//! also being practically useful. They are generic over [`PitchSystem`]s and don't refer to a
//! single notation system or musical culture. On the other hand, the heptatonic pitch system used
//! in common-practice Western music is also provided along with a set of convenience methods.

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
