use std::hash::{Hash, Hasher};

use passacaglia_common::Rational;

use crate::pitch::Pitch;
use crate::scale::Scale;
use crate::system::PitchSystem;

/// A degree within a [`Scale`]: an index, accidental, and period.
///
/// Borrows its scale; it is `Copy` and structurally equal by `(index, acci, period)`.
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
    #[must_use]
    pub const fn new(scale: &'a Scale<S>, index: usize, acci: Rational, period: i32) -> Self {
        Degree {
            scale,
            index,
            acci,
            period,
        }
    }

    #[must_use]
    pub fn with_period(&self, p: i32) -> Self {
        Degree::new(self.scale, self.index, self.acci, p)
    }

    #[must_use]
    pub fn to_pitch(&self) -> Pitch<S> {
        self.scale.degrees[self.index]
            .add_accidental(self.acci)
            .add_period(self.period)
    }

    #[must_use]
    pub fn next(&self) -> Self {
        let i = self.index + 1;
        if i >= self.scale.degrees.len() {
            Degree::new(
                self.scale,
                i - self.scale.degrees.len(),
                self.acci,
                self.period + 1,
            )
        } else {
            Degree::new(self.scale, i, self.acci, self.period)
        }
    }

    #[must_use]
    pub fn previous(&self) -> Self {
        if self.index == 0 {
            Degree::new(
                self.scale,
                self.scale.degrees.len() - 1,
                self.acci,
                self.period - 1,
            )
        } else {
            Degree::new(self.scale, self.index - 1, self.acci, self.period)
        }
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
