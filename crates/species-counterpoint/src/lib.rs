//! Species counterpoint: domain model, rules, and a best-first search solver.
//!
#![warn(clippy::pedantic)]
#![allow(
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

pub use basic::{CounterpointScoreBuilder, MelodicContext, MelodicSettings, Step, VoiceConstructor};
pub use chord::{Chord, ChordElement, Harmony};
pub use clef::{Clef, ClefType};
pub use context::{
    CandidateRule, Candidates, CounterpointContext, GlobalRule, HarmonyRule, LocalRule,
};
pub use imitation::define_imitation;
pub use score::{Parameters, Score};
pub use solver::{
    CounterpointSolver, CounterpointSolverProgress, CounterpointSolverRewardStrategy, NodeKind,
    SearchNode,
};
pub use species::{species1, species2, species3, species4, species5};
pub use voice::{parse_notes, Measure, NonHarmonicType, Note, Voice};
