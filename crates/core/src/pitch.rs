use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use num_traits::Signed;
use passacaglia_common::{rational, Rational};

use crate::interval::Interval;
use crate::system::{PitchSystem, ET12};

/// Represents a musical pitch in a scale system: a 3-tuple
/// (degree index, accidental, period index). It can also represent a pitch class
/// in some contexts, where the period number is ignored.
///
/// Keeping the degree and accidental distinct from the sounding ordinal lets a
/// spelling like `C#` stay distinct from `Db`. `S` is a zero-sized marker for the
/// pitch system.
#[derive(Debug)]
pub struct Pitch<S: PitchSystem> {
    /// A nonnegative integer representing the degree index.
    pub index: usize,
    /// The accidental attached to the pitch.
    pub acci: Rational,
    /// An integer representing the period index. Ignored in contexts where this
    /// doesn't exist (pitch classes).
    pub period: i32,
    _system: PhantomData<S>,
}

impl<S: PitchSystem> Clone for Pitch<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: PitchSystem> Copy for Pitch<S> {}

impl<S: PitchSystem> PartialEq for Pitch<S> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.acci == other.acci && self.period == other.period
    }
}

impl<S: PitchSystem> Eq for Pitch<S> {}

impl<S: PitchSystem> Hash for Pitch<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.index, self.acci, self.period).hash(state);
    }
}

impl<S: PitchSystem> Pitch<S> {
    /// Create a pitch.
    #[must_use]
    pub const fn new(index: usize, acci: Rational, period: i32) -> Self {
        assert!(index < S::N_DEGREES);
        Pitch {
            index,
            acci,
            period,
            _system: PhantomData,
        }
    }

    /// Get the ordinal number of this pitch.
    #[must_use]
    pub fn ord(&self) -> Rational {
        self.acci
            + rational(i64::from(self.period) * S::N_PITCH_CLASSES as i64)
            + S::DEGREE_OFFSETS[self.index]
    }

    /// Calculates the difference between pitches in pitch class units.
    /// If `this` is higher than `other`, a negative number will be returned.
    #[must_use]
    pub fn distance_to(&self, other: &Pitch<S>) -> Rational {
        other.ord() - self.ord()
    }

    /// Calculates the difference between pitches in steps, disregarding
    /// accidental marks. If `this` is higher than `other`, a negative number
    /// will be returned.
    #[must_use]
    pub fn steps_to(&self, other: &Pitch<S>) -> i64 {
        (i64::from(other.period) * S::N_DEGREES as i64 + other.index as i64)
            - (i64::from(self.period) * S::N_DEGREES as i64 + self.index as i64)
    }

    /// Get the interval from `this` to `other`. It will be negative if `this` is
    /// higher than `other`.
    #[must_use]
    pub fn interval_to(&self, other: &Pitch<S>) -> Interval<S> {
        let steps = self.steps_to(other);
        let distance = self.distance_to(other);
        let sign = if *distance.numer() < 0 { -1 } else { 1 };
        Interval::new(steps.unsigned_abs() as usize, distance.abs(), sign)
    }

    /// Add an interval. The accidental is recomputed so the result's ordinal
    /// equals exactly `ord + distance*sign`.
    #[must_use]
    pub fn add(&self, i: &Interval<S>) -> Self {
        let total_steps = i64::from(self.period) * S::N_DEGREES as i64
            + self.index as i64
            + i.steps as i64 * i64::from(i.sign);
        let period = total_steps.div_euclid(S::N_DEGREES as i64) as i32;
        let index = total_steps.rem_euclid(S::N_DEGREES as i64) as usize;

        let without_acci_ord = Pitch::<S>::new(index, rational(0), period).ord();
        let target_ord = self.ord() + i.distance * i64::from(i.sign);
        let acci = target_ord - without_acci_ord;
        Pitch::<S>::new(index, acci, period)
    }

    #[must_use]
    pub fn add_accidental(&self, n: Rational) -> Self {
        Pitch::<S>::new(self.index, self.acci + n, self.period)
    }

    #[must_use]
    pub fn with_period(&self, p: i32) -> Self {
        Pitch::<S>::new(self.index, self.acci, p)
    }

    #[must_use]
    pub fn add_period(&self, p: i32) -> Self {
        Pitch::<S>::new(self.index, self.acci, self.period + p)
    }

    #[must_use]
    pub fn enharmonically_equals(&self, other: &Pitch<S>) -> bool {
        self.ord() == other.ord()
    }
}

impl<S: ET12> Pitch<S> {
    /// The MIDI note number of this pitch (ordinal + 12).
    #[must_use]
    pub fn to_midi(&self) -> Rational {
        self.ord() + rational(12)
    }
}
