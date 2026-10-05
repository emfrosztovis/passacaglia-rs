use std::hash::{Hash, Hasher};

use num_traits::Zero;
use passacaglia_common::{rational, rotate_array, Rational};

use crate::degree::Degree;
use crate::interval::Interval;
use crate::pitch::Pitch;
use crate::system::PitchSystem;

/// Definition of a degree of a [`Scale`].
#[derive(Debug)]
pub struct DegreeDefinition<S: PitchSystem> {
    /// The standard pitch class of this degree.
    pub pitch: Pitch<S>,
    /// Permitted alterations on the degree, represented by accidental offsets. 
    /// The standard pitch (offset 0) is always available and need not be listed here.
    pub alterations: Vec<Rational>,
}

impl<S: PitchSystem> Clone for DegreeDefinition<S> {
    fn clone(&self) -> Self {
        DegreeDefinition {
            pitch: self.pitch,
            alterations: self.alterations.clone(),
        }
    }
}

impl<S: PitchSystem> PartialEq for DegreeDefinition<S> {
    fn eq(&self, other: &Self) -> bool {
        self.pitch == other.pitch && self.alterations == other.alterations
    }
}

impl<S: PitchSystem> Eq for DegreeDefinition<S> {}

impl<S: PitchSystem> Hash for DegreeDefinition<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pitch.hash(state);
        self.alterations.hash(state);
    }
}

/// Represents a scale in a pitch system, starting from a given pitch class (the root) and 
/// consisting of several degrees. Each degree can allow a number of altered tones to base on it.
/// 
/// Although both pitch systems and scales have "degrees", they're different things and you should
/// note this when reading.
#[derive(Debug, Clone)]
pub struct Scale<S: PitchSystem> {
    /// List of degrees. Always non-decreasing and spans less than the system's
    /// period. The first degree is the root.
    pub degrees: Vec<DegreeDefinition<S>>,
    /// List of intervals. Always nonnegative.
    pub intervals: Vec<Interval<S>>,
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
    #[must_use]
    fn new(degrees: Vec<DegreeDefinition<S>>, intervals: Vec<Interval<S>>) -> Self {
        assert!(!intervals.is_empty());
        debug_assert_eq!(intervals.len(), degrees.len());
        Scale { degrees, intervals }
    }

    /// Build a scale from a root and its intervals (the last wraps back to the root).
    #[must_use]
    pub fn from_intervals(
        root: Pitch<S>,
        intervals: &[Interval<S>],
    ) -> Self {
        let mut pitches = vec![root];
        let mut current = root;
        for (i, int) in intervals.iter().enumerate() {
            debug_assert_eq!(int.sign, 1);
            current = current.add(int);
            if i == intervals.len() - 1 {
                debug_assert_eq!(
                    root.distance_to(&current),
                    rational(S::N_PITCH_CLASSES as i64)
                );
            } else {
                pitches.push(current);
            }
        }
        let degrees = pitches
            .into_iter()
            .map(|pitch| DegreeDefinition { pitch, alterations: Vec::new() })
            .collect();
        Scale::new(degrees, intervals.to_vec())
    }

    /// Build a scale from its degrees.
    #[must_use]
    pub fn from_pitches(degrees: &[Pitch<S>]) -> Self {
        let mut intervals = Vec::with_capacity(degrees.len());
        for i in 1..degrees.len() {
            let int = degrees[i - 1].interval_to(&degrees[i]);
            assert!(int.sign > 0);
            intervals.push(int);
        }
        let wrap = degrees
            .last()
            .expect("non-empty degrees")
            .interval_to(&degrees[0].add_period(1));
        assert!(wrap.sign > 0);
        assert!(wrap.distance < rational(S::N_PITCH_CLASSES as i64));
        intervals.push(wrap);
        let degrees = degrees.iter().copied()
            .map(|pitch| DegreeDefinition { pitch, alterations: Vec::new() })
            .collect();
        Scale::new(degrees, intervals)
    }

    /// Build a scale from its degrees.
    #[must_use]
    pub fn from_definitions(degrees: &[DegreeDefinition<S>]) -> Self {
        let mut intervals = Vec::with_capacity(degrees.len());
        for i in 1..degrees.len() {
            let int = degrees[i - 1].pitch.interval_to(&degrees[i].pitch);
            assert!(int.sign > 0);
            intervals.push(int);
        }
        let wrap = degrees
            .last()
            .expect("non-empty degrees")
            .pitch.interval_to(&degrees[0].pitch.add_period(1));
        assert!(wrap.sign > 0);
        assert!(wrap.distance < rational(S::N_PITCH_CLASSES as i64));
        intervals.push(wrap);
        Scale::new(degrees.to_vec(), intervals)
    }

    /// Get the definition of the root degree.
    #[must_use]
    pub fn root(&self) -> DegreeDefinition<S> {
        self.degrees[0].clone()
    }

    /// Get the degree at an index.
    #[must_use]
    pub fn at(&self, i: usize) -> Degree<'_, S> {
        Degree::new(self, i, Rational::ZERO, 0)
    }

    /// Match a pitch to a degree in the scale. Returns `None` if no allowed tone matches the pitch.
    #[must_use]
    pub fn get_degree(&self, p: &Pitch<S>) -> Option<Degree<'_, S>> {
        for (i, deg) in self.degrees.iter().enumerate() {
            if deg.pitch.index != p.index { continue; }
            let diff = p.acci - deg.pitch.acci;
            if diff.is_zero() || deg.alterations.contains(&diff) {
                return Some(Degree::new(self, i, diff,
                    p.period - deg.pitch.period));
            }
        }
        None
    }

    /// Returns all possible (allowed) degrees within a period. This includes all standard degrees 
    /// and all alterations of them.
    #[must_use]
    pub fn all_degrees(&self) -> Vec<Degree<'_, S>> {
        let mut result = Vec::new();
        for (i, deg) in self.degrees.iter().enumerate() {
            result.push(Degree::new(self, i, Rational::ZERO, 0));
            for &alt in &deg.alterations {
                result.push(Degree::new(self, i, alt, 0));
            }
        }
        result
    }

    /// Returns all possible (allowed) degrees within a given pitch range. This includes all 
    /// standard degrees and all alterations of them.
    #[must_use]
    pub fn get_degrees_in_range(&self, l: &Pitch<S>, h: &Pitch<S>) -> Vec<Degree<'_, S>> {
        let mut result = Vec::new();
        let mut current = self.at(0).with_period(l.period - 1);

        let lord = l.ord();
        let hord = h.ord();
        loop {
            let mut all_past = true;

            let ord = current.to_pitch().ord();
            if ord <= hord {
                all_past = false;
            }
            if ord >= lord && ord <= hord {
                result.push(current);
            }

            for &alt in &self.degrees[current.index].alterations {
                let deg = current.with_acci(alt);
                let ord = deg.to_pitch().ord();
                if ord <= hord {
                    all_past = false;
                }
                if ord >= lord && ord <= hord {
                    result.push(deg);
                }
            }
            if all_past { break; }
            current = current.next();
        }
        
        result
    }

    /// Rotate the scale. Positive `n` shifts left. The result is transposed back so that the root pitch remains the same, *unless* `move_root` is set to `false`.
    #[must_use]
    pub fn rotate(&self, n: i64, move_root: bool) -> Scale<S> {
        let new_intervals = rotate_array(&self.intervals, n);
        let degrees = rotate_array(&self.degrees, n);
        let scale = Scale::new(degrees, new_intervals);
        if move_root {
            scale
        } else {
            scale.transpose_to(&self.root().pitch)
        }
    }

    /// Transpose the scale by an interval.
    #[must_use]
    pub fn transpose(&self, int: &Interval<S>) -> Scale<S> {
        let mut int = *int;
        let new_root = self.root().pitch.add(&int);
        if new_root.period != 0 {
            int = int.add_period(-new_root.period);
        }
        let degrees = self
            .degrees
            .iter()
            .map(|d| DegreeDefinition {
                pitch: d.pitch.add(&int),
                alterations: d.alterations.clone(),
            })
            .collect();
        Scale::new(degrees, self.intervals.clone())
    }

    /// Transpose the scale so that its root becomes `new_root`.
    #[must_use]
    pub fn transpose_to(&self, new_root: &Pitch<S>) -> Scale<S> {
        let int = self.root().pitch.interval_to(&new_root.with_period(0));
        self.transpose(&int)
    }

    /// Compare only the interval structure and allowed alterations, ignoring the root.
    #[must_use]
    pub fn shape_equals(&self, other: &Scale<S>) -> bool {
        self.intervals.len() == other.intervals.len()
            && &self.transpose_to(&other.root().pitch) == other
    }
}
