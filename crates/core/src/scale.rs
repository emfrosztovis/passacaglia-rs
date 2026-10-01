use std::hash::{Hash, Hasher};

use passacaglia_common::{rational, rotate_array, Rational};

use crate::degree::Degree;
use crate::interval::Interval;
use crate::pitch::Pitch;
use crate::system::PitchSystem;

/// A scale: an ordered list of degrees and the intervals between them.
///
/// Degrees are non-decreasing and span less than the system's period; the first
/// degree is the root. Intervals are non-negative and include the wrapping
/// interval from the last degree back to the root.
#[derive(Debug)]
pub struct Scale<S: PitchSystem> {
    pub degrees: Vec<Pitch<S>>,
    pub intervals: Vec<Interval<S>>,
}

impl<S: PitchSystem> Clone for Scale<S> {
    fn clone(&self) -> Self {
        Scale {
            degrees: self.degrees.clone(),
            intervals: self.intervals.clone(),
        }
    }
}

impl<S: PitchSystem> PartialEq for Scale<S> {
    fn eq(&self, other: &Self) -> bool {
        self.degrees == other.degrees && self.intervals == other.intervals
    }
}

impl<S: PitchSystem> Eq for Scale<S> {}

impl<S: PitchSystem> Hash for Scale<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.degrees.hash(state);
        self.intervals.hash(state);
    }
}

impl<S: PitchSystem> Scale<S> {
    pub fn new(degrees: Vec<Pitch<S>>, intervals: Vec<Interval<S>>) -> Self {
        debug_assert!(!intervals.is_empty());
        debug_assert_eq!(intervals.len(), degrees.len());
        Scale { degrees, intervals }
    }

    pub fn root(&self) -> Pitch<S> {
        self.degrees[0]
    }

    /// The degree at `i`, optionally with an accidental.
    pub fn at(&self, i: usize, acci: Rational) -> Degree<'_, S> {
        Degree::new(self, i, acci, 0)
    }

    pub fn get_exact_degree(&self, p: &Pitch<S>, allow_enharmonic: bool) -> Option<Degree<'_, S>> {
        let p0 = p.with_period(0);
        let i = self.degrees.iter().position(|x| {
            let x0 = x.with_period(0);
            if allow_enharmonic {
                x0.enharmonically_equals(&p0)
            } else {
                x0 == p0
            }
        })?;
        Some(self.at(i, rational(0)))
    }

    pub fn get_degrees_in_range(&self, l: &Pitch<S>, h: &Pitch<S>) -> Vec<Degree<'_, S>> {
        let mut result = Vec::new();
        let mut current = self.at(0, rational(0)).with_period(l.period - 1);
        while current.to_pitch().ord() < l.ord() {
            current = current.next();
        }
        while current.to_pitch().ord() <= h.ord() {
            result.push(current);
            current = current.next();
        }
        result
    }

    /// Rotate the scale. Positive `n` shifts left; the intervals rotate, and the
    /// root moves only when `move_root` is set.
    pub fn rotate(&self, n: i64, move_root: bool) -> Scale<S> {
        let new_intervals = rotate_array(&self.intervals, n);

        let mut current = self.root();
        if move_root {
            let len = self.degrees.len() as i64;
            let signed = (n % len).abs() * if n < 0 { -1 } else { 1 };
            let idx = if signed < 0 { len + signed } else { signed } as usize;
            current = self.degrees[idx].with_period(0);
        }

        let mut new_degs = vec![current];
        for int in &new_intervals[..new_intervals.len() - 1] {
            current = current.add(int);
            new_degs.push(current);
        }
        Scale::new(new_degs, new_intervals)
    }

    /// Transpose the scale by an interval. The intervals themselves do not change.
    pub fn transpose(&self, int: &Interval<S>) -> Scale<S> {
        let mut int = *int;
        let new_root = self.root().add(&int);
        if new_root.period != 0 {
            int = int.add_period(-new_root.period);
        }
        let degrees = self.degrees.iter().map(|x| x.add(&int)).collect();
        Scale::new(degrees, self.intervals.clone())
    }

    /// Transpose the scale so that its root becomes `new_root`.
    pub fn transpose_to(&self, new_root: &Pitch<S>) -> Scale<S> {
        let int = self.root().interval_to(&new_root.with_period(0));
        self.transpose(&int)
    }

    /// Compare only the interval structure, ignoring the root.
    pub fn interval_equals(&self, other: &Scale<S>) -> bool {
        self.intervals.len() == other.intervals.len()
            && self
                .intervals
                .iter()
                .zip(&other.intervals)
                .all(|(a, b)| a == b)
    }
}
