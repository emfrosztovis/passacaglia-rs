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
pub use system::StdHeptatonic;

/// A pitch in the standard heptatonic system. The `period` corresponds to the
/// octave number in scientific notation.
pub type Pitch = crate::pitch::Pitch<StdHeptatonic>;

/// A signed interval in the standard heptatonic system.
pub type Interval = crate::interval::Interval<StdHeptatonic>;

pub type Scale = crate::scale::Scale<StdHeptatonic>;
pub type Degree<'a> = crate::degree::Degree<'a, StdHeptatonic>;
