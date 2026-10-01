//! `MusicXML` serialization (Western staff notation) for the passacaglia music
//! engine.
//!
//! This crate currently contains only the domain-independent pieces of the
//! serializer: the [`Clef`] data type and the [`pitch`] element writer. The
//! score-level adapters (`note`, `measure`, `part`, `score`) depend on the
//! `tonal` domain types and are deferred until those are disentangled from the
//! counterpoint playground; see `agent-documents/porting-musicxml.md`.

mod clef;
mod pitch;

pub use clef::{Clef, ClefType};
pub use pitch::pitch;
