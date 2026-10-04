use num_rational::Ratio;

use passacaglia_common::Rational;

use crate::system::{PitchSystem, ET12};

/// The standard, common-practice heptatonic pitch system (7 degrees, 12 pitch
/// classes, octave period).
///
/// |      Degree | C |   | D |   | E | F |   | G |   | A |    | B  |
/// |------------:|---|---|---|---|---|---|---|---|---|---|----|----|
/// | Pitch class | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StdHeptatonic;

impl PitchSystem for StdHeptatonic {
    const PERIOD_RATIO: Rational = Ratio::new_raw(2, 1);
    const N_PITCH_CLASSES: usize = 12;
    const N_DEGREES: usize = 7;
    const DEGREE_OFFSETS: &'static [Rational] = &[
        Ratio::new_raw(0, 1),
        Ratio::new_raw(2, 1),
        Ratio::new_raw(4, 1),
        Ratio::new_raw(5, 1),
        Ratio::new_raw(7, 1),
        Ratio::new_raw(9, 1),
        Ratio::new_raw(11, 1),
    ];
}

impl ET12 for StdHeptatonic {}
