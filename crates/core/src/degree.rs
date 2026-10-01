use std::hash::{Hash, Hasher};

use passacaglia_common::Rational;

use crate::pitch::Pitch;
use crate::scale::Scale;
use crate::system::PitchSystem;

/// A degree within a [`Scale`]: an index, accidental, and period.
///
/// Borrows its scale; it is `Copy` and structurally equal by `(index, acci, period)`,
/// matching the original `hash()` (which ignored the scale).
#[derive(Debug)]
pub struct Degree<'a, S: PitchSystem> {
    pub scale: &'a Scale<S>,
    pub index: usize,
    pub acci: Rational,
    pub period: i32,
}

impl<'a, S: PitchSystem> Clone for Degree<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, S: PitchSystem> Copy for Degree<'a, S> {}

impl<'a, S: PitchSystem> Degree<'a, S> {
    pub const fn new(scale: &'a Scale<S>, index: usize, acci: Rational, period: i32) -> Self {
        Degree {
            scale,
            index,
            acci,
            period,
        }
    }

    pub fn with_period(&self, p: i32) -> Self {
        Degree::new(self.scale, self.index, self.acci, p)
    }

    pub fn to_pitch(&self) -> Pitch<S> {
        self.scale.degrees[self.index]
            .add_accidental(self.acci)
            .add_period(self.period)
    }

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

impl<'a, S: PitchSystem> PartialEq for Degree<'a, S> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.acci == other.acci && self.period == other.period
    }
}

impl<'a, S: PitchSystem> Eq for Degree<'a, S> {}

impl<'a, S: PitchSystem> Hash for Degree<'a, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.index, self.acci, self.period).hash(state);
    }
}
