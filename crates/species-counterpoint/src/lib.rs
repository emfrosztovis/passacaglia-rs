//! Species counterpoint: domain model, rules, and a best-first search solver.
//!
//! This crate mirrors the TypeScript `species-counterpoint` package: it keeps
//! the `Score`/`Measure`/`Note`/`Voice` domain model local (standard-heptatonic
//! specific) and layers the rule registry and solver on top.
//!
//! Like the source it ports, the solver is single-threaded and its rule
//! closures capture plain owned data. The `Rc`s shared across measures/voices
//! are therefore not `Send`/`Sync`; the `too_many_arguments`,
//! `type_complexity`, `too_many_lines`, and `similar_names` allowances mirror
//! the source's construction signatures, higher-ranked cursor types, and
//! verbatim rule bodies.
#![warn(clippy::pedantic)]
#![allow(
    clippy::arc_with_non_send_sync,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::similar_names,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::wildcard_imports
)]

pub mod basic;
pub mod chord;
pub mod clef;
pub mod context;
pub mod imitation;
pub mod rules;
pub mod score;
pub mod solver;
pub mod species;
pub mod voice;

/// Collect rule references into an `Rc`-wrapped `Vec`, avoiding the repetitive
/// `Rc::new(&…)` boilerplate when populating a [`CounterpointContext`].
///
/// ```ignore
/// ctx.harmony_rules = import_rules![
///     rules::enforce_functional_progression_major,
///     rules::enforce_valid_chords,
/// ];
/// ```
///
/// The type-annotated form produces a `Vec` whose element type is already the
/// concrete rule trait object, so it can be used where no coercion site exists
/// (e.g. inside a `HashMap::from([…])` array):
///
/// ```ignore
/// ctx.non_harmonic_tone_rules = HashMap::from([
///     (NonHarmonicType::Neighbor, import_rules!(CandidateRule; rules::make_neighbor_tone)),
/// ]);
/// ```
#[macro_export]
macro_rules! import_rules {
    ($ty:ty; $($rule:path),+ $(,)?) => {{
        let rules: Vec<$ty> = vec![$(::std::rc::Rc::new(&$rule)),+];
        rules
    }};
    ($($rule:path),+ $(,)?) => {
        vec![$(::std::rc::Rc::new(&$rule)),+]
    };
}

pub use basic::{CounterpointScoreBuilder, MelodicContext, MelodicSettings, Step, VoiceConstructor};
pub use chord::{Chord, ChordElement, Harmony};
pub use clef::{Clef, ClefType};
pub use context::{
    CandidateRule, Candidates, CounterpointContext, GlobalRule, HarmonyRule, LocalRule,
};
pub use imitation::define_imitation;
pub use score::{Parameters, Score};
pub use solver::{CounterpointSolver, CounterpointSolverProgress, CounterpointSolverRewardStrategy};
pub use species::{species1, species2, species3, species4, species5};
pub use voice::{parse_notes, Measure, NonHarmonicType, Note, Voice};
