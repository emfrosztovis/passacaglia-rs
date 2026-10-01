//! The standard heptatonic (12-TET) pitch system and its concrete pitch,
//! interval, scale, and degree types, plus parsing and printing.

mod accidental;
mod constants;
mod interval;
mod parse;
mod pitch;
mod scale;
mod system;

pub use constants::{scales, PitchClasses, PITCH_CLASSES};
pub use parse::ParseError;
pub use system::StandardHeptatonic;

pub type Pitch = crate::pitch::Pitch<StandardHeptatonic>;
pub type Interval = crate::interval::Interval<StandardHeptatonic>;
pub type Scale = crate::scale::Scale<StandardHeptatonic>;
pub type Degree<'a> = crate::degree::Degree<'a, StandardHeptatonic>;
