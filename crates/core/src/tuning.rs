use std::marker::PhantomData;

use passacaglia_common::{rational_value, Rational};

use crate::pitch::Pitch;
use crate::system::PitchSystem;

/// Maps a pitch to a sounding frequency.
pub trait Tuning<S: PitchSystem> {
    fn frequency_of(&self, p: &Pitch<S>) -> f64;

    fn cent_between(&self, p1: &Pitch<S>, p2: &Pitch<S>) -> f64 {
        1200.0 * (self.frequency_of(p2) / self.frequency_of(p1)).log2()
    }

    fn ratio_between(&self, p1: &Pitch<S>, p2: &Pitch<S>) -> f64 {
        self.frequency_of(p2) / self.frequency_of(p1)
    }
}

/// N-tone equal temperament, referenced to a given pitch and frequency.
pub struct EqualTemperament<S: PitchSystem> {
    reference_freq: f64,
    reference_ord: Rational,
    _system: PhantomData<S>,
}

impl<S: PitchSystem> EqualTemperament<S> {
    pub fn new(reference_freq: f64, reference_pitch: &Pitch<S>) -> Self {
        EqualTemperament {
            reference_freq,
            reference_ord: reference_pitch.ord(),
            _system: PhantomData,
        }
    }
}

impl<S: PitchSystem> Tuning<S> for EqualTemperament<S> {
    fn frequency_of(&self, p: &Pitch<S>) -> f64 {
        let steps = rational_value(p.ord() - self.reference_ord) / S::N_PITCH_CLASSES as f64;
        self.reference_freq * 2f64.powf(steps)
    }
}
