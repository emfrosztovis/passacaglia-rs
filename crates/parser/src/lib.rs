//! Pure string parsers and shared naming tables for passacaglia literals.
//!
//! This crate has no dependencies so that both `passacaglia-core` (for its
//! runtime `FromStr` impls) and `passacaglia-macros` (for compile-time literal
//! expansion) can share the exact same grammar without duplicating it.

mod error;
mod parse;

pub use error::ParseError;
pub use parse::{
    interval_data, parse_accidental, parse_interval, parse_pitch, parse_rational, IntervalParts,
    PitchParts, Quality,
};
