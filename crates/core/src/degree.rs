use std::hash::{Hash, Hasher};

use num_traits::Zero;
use passacaglia_common::Rational;

use crate::pitch::Pitch;
use crate::scale::Scale;
use crate::system::PitchSystem;

/// Represents a degree within a [`Scale`]: a 3-tuple (index, accidental, period). Note that it 
/// doesn't have to be an "allowed" tone in that scale; it is a generalized degree that starts from 
/// the standard pitch of a scale degree and adds an arbitrary accidental to it.
/// 
/// Borrows the scale.
#[derive(Debug)]
pub struct Degree<'a, S: PitchSystem> {
    pub scale: &'a Scale<S>,
    pub index: usize,
    pub acci: Rational,
    pub period: i32,
}

impl<S: PitchSystem> Clone for Degree<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: PitchSystem> Copy for Degree<'_, S> {}

impl<'a, S: PitchSystem> Degree<'a, S> {
    /// Create a degree.
    #[must_use]
    pub const fn new(scale: &'a Scale<S>, index: usize, acci: Rational, period: i32) -> Self {
        Degree {
            scale,
            index,
            acci,
            period,
        }
    }

    /// Returns a copy of the degree with the given accidental.
    #[must_use]
    pub fn with_acci(&self, acci: Rational) -> Self {
        Degree::new(self.scale, self.index, acci, self.period)
    }

    /// Returns a copy of the degree with the given period.
    #[must_use]
    pub fn with_period(&self, period: i32) -> Self {
        Degree::new(self.scale, self.index, self.acci, period)
    }

    /// Get the pitch of this degree.
    #[must_use]
    pub fn to_pitch(&self) -> Pitch<S> {
        self.scale.degrees[self.index]
            .pitch
            .add_accidental(self.acci)
            .add_period(self.period)
    }

    /// Get the next degree. Removes the accidental.
    #[must_use]
    pub fn next(&self) -> Self {
        let i = self.index + 1;
        if i >= self.scale.degrees.len() {
            Degree::new(
                self.scale,
                i - self.scale.degrees.len(),
                Rational::ZERO,
                self.period + 1,
            )
        } else {
            Degree::new(self.scale, i, Rational::ZERO, self.period)
        }
    }

    /// Get the previous degree. Removes the accidental.
    #[must_use]
    pub fn previous(&self) -> Self {
        if self.index == 0 {
            Degree::new(
                self.scale,
                self.scale.degrees.len() - 1,
                Rational::ZERO,
                self.period - 1,
            )
        } else {
            Degree::new(self.scale, self.index - 1, Rational::ZERO, self.period)
        }
    }

    /// Returns true if this degree is not altered (`acci` is zero), or the alteration is allowed on this degree in the scale.
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        self.acci.is_zero() || self.scale.degrees[self.index].alterations.contains(&self.acci)
    }
}

impl<S: PitchSystem> PartialEq for Degree<'_, S> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.acci == other.acci && self.period == other.period
    }
}

impl<S: PitchSystem> Eq for Degree<'_, S> {}

impl<S: PitchSystem> Hash for Degree<'_, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.index, self.acci, self.period).hash(state);
    }
}
