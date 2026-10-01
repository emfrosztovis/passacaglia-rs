pub mod utils;

mod voice_motion;
mod voice_overlapping;
mod melody;
mod leaps;
mod scales;
mod vertical_consonance;
mod nearby_perfects;
mod similar_motion;
mod passing_tone;
mod neighbor_tone;
mod suspension;
mod valid_chords;
mod functional_harmony;
mod root_progression;

pub use voice_motion::prioritize_voice_motion;
pub use voice_overlapping::forbid_voice_overlapping2;
pub use melody::{avoid_repeat2, enforce_melody_intervals, enforce_stepwise_around_short_notes};
pub use leaps::{enforce_leap_preparation, enforce_leap_resolution, limit_consecutive_leaps};
pub use scales::{
    degree_matrix_preset_major, enforce_directional_degree_matrix, enforce_minor,
    enforce_scale_tones, DegreeMatrix, DegreeTransition,
};
pub use vertical_consonance::enforce_vertical_consonance_with_moving_local;
pub use nearby_perfects::forbid_nearby_perfects;
pub use similar_motion::forbid_perfects_by_similar_motion;
pub use passing_tone::{enforce_passing_tones, make_passing_tone};
pub use neighbor_tone::{enforce_neighbor_tones, make_neighbor_tone};
pub use suspension::{enforce_suspension, make_suspension};
pub use valid_chords::{enforce_chord_tone, enforce_fixed_progression, enforce_valid_chords};
pub use functional_harmony::enforce_functional_progression_major;
pub use root_progression::enforce_root_progression;
