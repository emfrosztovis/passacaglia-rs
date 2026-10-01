//! `MusicXML` serialization (Western staff notation) for the passacaglia music
//! engine.
//!
//! This crate contains the domain-independent pieces of the serializer (the
//! [`pitch`] and [`note`] element writers) and the [`ToMxl`] adapter trait,
//! implemented for the `species-counterpoint` domain types. The clef types are
//! re-exported from `species-counterpoint`.

mod adapters;
mod pitch;

pub use adapters::{note, ToMxl};
pub use passacaglia_species_counterpoint::{Clef, ClefType};
pub use pitch::pitch;
