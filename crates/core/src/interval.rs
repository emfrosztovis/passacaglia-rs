use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use num_traits::Signed;
use passacaglia_common::{rational, Rational};

use crate::system::PitchSystem;

/// Represents a signed musical interval in a scale system: a 3-tuple (steps, distance, sign).
#[derive(Debug)]
pub struct Interval<S: PitchSystem> {
    /// A nonnegative integer representing the step count between the two pitches. E.g. 0 means 
    /// they share the same degree, 1 means the higher pitch is the next degree.
    pub steps: usize,
    /// Number of subdivisions between the two pitches (nonnegative).
    pub distance: Rational,
    /// Sign of the interval, either `1` or `-1`.
    pub sign: i8,
    _system: PhantomData<S>,
}

impl<S: PitchSystem> Clone for Interval<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: PitchSystem> Copy for Interval<S> {}

impl<S: PitchSystem> PartialEq for Interval<S> {
    fn eq(&self, other: &Self) -> bool {
        self.steps == other.steps && self.distance == other.distance && self.sign == other.sign
    }
}

impl<S: PitchSystem> Eq for Interval<S> {}

impl<S: PitchSystem> Hash for Interval<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.steps, self.distance, self.sign).hash(state);
    }
}

impl<S: PitchSystem> Interval<S> {
    /// Create an interval.
    #[must_use]
    pub const fn new(steps: usize, distance: Rational, sign: i8) -> Self {
        assert!(sign == 1 || sign == -1);
        Interval {
            steps,
            distance,
            sign,
            _system: PhantomData,
        }
    }

    /// True if both have the same sign and distance, ignoring steps.
    #[must_use]
    pub fn equals_enharmonically(&self, other: &Interval<S>) -> bool {
        self.sign == other.sign && self.distance == other.distance
    }

    /// Add another interval to the interval. The result's `steps` and `distance` are independently 
    /// combined from the two intervals' respective fields.
    #[must_use]
    pub fn add(&self, other: &Interval<S>) -> Self {
        let d = self.distance * i64::from(self.sign) + other.distance * i64::from(other.sign);
        let s =
            self.steps as i64 * i64::from(self.sign) + other.steps as i64 * i64::from(other.sign);
        let most_signful = if *d.numer() == 0 { s } else { *d.numer() };
        Interval::new(
            s.unsigned_abs() as usize,
            d.abs(),
            if most_signful < 0 { -1 } else { 1 },
        )
    }

    /// Add a given number of periods to the interval.
    #[must_use]
    pub fn add_period(&self, n: i32) -> Self {
        if n == 0 {
            return *self;
        }
        let abs_n = n.unsigned_abs() as usize;
        let offset = Interval::<S>::new(
            S::N_DEGREES * abs_n,
            rational(S::N_PITCH_CLASSES as i64 * abs_n as i64),
            if n < 0 { -1 } else { 1 },
        );
        self.add(&offset)
    }

    /// Reduce compound intervals (i.e. spanning more than one period in the system) to simple 
    /// intervals.
    #[must_use]
    pub fn to_simple(&self, preserve_up_to_steps: Option<usize>) -> Self {
        if self.steps < S::N_DEGREES {
            return *self;
        }
        if let Some(p) = preserve_up_to_steps
            && self.steps <= p
        {
            return *self;
        }

        let preserve_periods = preserve_up_to_steps.map_or(0, |p| p / S::N_DEGREES);
        let periods = (self.steps / S::N_DEGREES)
            .min((self.distance / rational(S::N_PITCH_CLASSES as i64)).to_integer() as usize)
            .saturating_sub(preserve_periods);
        let mut periods = periods;
        let mut new_steps = self.steps - periods * S::N_DEGREES;

        if let Some(p) = preserve_up_to_steps
            && new_steps > p
        {
            periods += 1;
            new_steps -= S::N_DEGREES;
        }

        let new_distance = self.distance - rational(periods as i64 * S::N_PITCH_CLASSES as i64);
        Interval::new(new_steps, new_distance, self.sign)
    }

    /// Returns true if `other` equals `this`, or `other` is larger but reduces to the same simple 
    /// interval as `this`.
    #[must_use]
    pub fn matches(&self, other: &Interval<S>) -> bool {
        other.to_simple(None) == self.to_simple(None) && other.distance >= self.distance
    }

    /// Returns true if `other` equals `this` enharmonically, or `other` is larger but reduces to 
    /// a simple interval enharmonically equivalent to `this`.
    #[must_use]
    pub fn matches_enharmonically(&self, other: &Interval<S>) -> bool {
        other
            .to_simple(None)
            .equals_enharmonically(&self.to_simple(None))
            && other.distance >= self.distance
    }

    /// Returns a copy of the interval with the given sign.
    #[must_use]
    pub fn with_sign(&self, sign: i8) -> Self {
        Interval::new(self.steps, self.distance, sign)
    }

    /// Returns the negation of the interval.
    #[must_use]
    pub fn negate(&self) -> Self {
        Interval::new(
            self.steps,
            self.distance,
            if self.sign == 1 { -1 } else { 1 },
        )
    }

    /// Returns a copy of the interval with a positive sign.
    #[must_use]
    pub fn abs(&self) -> Self {
        Interval::new(self.steps, self.distance, 1)
    }
}
