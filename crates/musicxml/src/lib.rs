//! `MusicXML` serialization (Western staff notation) for the passacaglia music
//! engine.
//!
//! This crate contains the domain-independent pieces of the serializer (the
//! [`pitch`] element writer) and re-exports the clef types from the
//! `species-counterpoint` domain crate. The score-level adapters (`note`,
//! `measure`, `part`, `score`) are added against `species-counterpoint` types;
//! see `agent-documents/porting-musicxml.md`.

mod pitch;

pub use passacaglia_species_counterpoint::{Clef, ClefType};
pub use pitch::pitch;
