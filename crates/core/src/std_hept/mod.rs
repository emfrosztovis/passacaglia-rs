//! Provides the standard 12-TET heptatonic pitch system used in common-practice Western music. 
//! Additionally, some convenience methods for transforming, pretty-printing and parsing are 
//! implmented for `Pitch`, `Interval`, `Scale` and `Degree`.

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
