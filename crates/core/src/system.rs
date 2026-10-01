use passacaglia_common::Rational;

/// A **pitch system**: the abstract structure of a musical space.
///
/// * `PERIOD_RATIO` — the period (equave) as a frequency ratio, larger than 1.
/// * `N_PITCH_CLASSES` — subdivisions of a period.
/// * `N_DEGREES` — named degrees (fewer than the pitch classes).
/// * `DEGREE_OFFSETS` — pitch-class ordinal of each degree, strictly increasing.
///
/// All of these are compile-time constants, so a system is a zero-sized marker
/// and every derived computation is monomorphized to immediates.
pub trait PitchSystem {
    const PERIOD_RATIO: Rational;
    const N_PITCH_CLASSES: usize;
    const N_DEGREES: usize;
    const DEGREE_OFFSETS: &'static [Rational];
}

/// Marker for 12-tone equal-temperament systems (octave period, 12 pitch classes).
pub trait ET12: PitchSystem {}
