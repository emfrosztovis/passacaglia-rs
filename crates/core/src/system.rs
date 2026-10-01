use passacaglia_common::Rational;

/// Our model for a **pitch system**. To make it abstract and general enough but
/// also useful, we must make the following assumptions for the system:
///
/// * It has a **period** expressible as a frequency ratio.
/// * It subdivides a period into N **pitch classes** (think semitones for
///   common-practice heptatonic scales). The subdivision does *not* have to be
///   even, but we assume it has a kind of meaning as to make transposition based
///   on it somewhat meaningful.
/// * It contains M (M < N) **degrees** (think named tones like C, D, E...) among
///   the pitch classes in a period.
/// * Accidentals are treated like pitch class index offsets (but we allow
///   rationals for flexibility). Therefore, combining accidentals must be linear
///   arithmetics.
///
/// All of these are compile-time constants, so a system is a zero-sized marker.
pub trait PitchSystem {
    /// The period of the system, also known as the equave, expressed as frequency
    /// ratio. For example, most systems use the octave as the period, which is
    /// 2:1 = 2. Must be larger than 1.
    const PERIOD_RATIO: Rational;

    /// The number of pitch classes in a period.
    const N_PITCH_CLASSES: usize;

    /// The number of degrees in a period.
    const N_DEGREES: usize;

    /// The pitch class index for each of the degrees in a period. Must be a
    /// strictly increasing array with length equal to `N_PITCH_CLASSES`, and the 
    /// first element must be 0.
    const DEGREE_OFFSETS: &'static [Rational];
}

/// Marker for all 12-TET pitch systems with the octave as period.
pub trait ET12: PitchSystem {}
