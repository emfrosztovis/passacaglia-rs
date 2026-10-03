use std::rc::Rc;

use im::Vector;

use passacaglia_common::{rational, Rational};
use passacaglia_core::std_hept::{Interval, Pitch, PITCH_CLASSES};
use passacaglia_core::structure::{Container, Cursor, DurationalElement, TemporalElement};

/// A common-practice chord, defined by a bass pitch and a list of intervals
/// above it.
///
/// Structural equality and hashing ignore the [`Chord::label`] (which is only a
/// display hint), mirroring the TypeScript `Chord.equals`/`Chord.hash`.
#[derive(Debug, Clone)]
pub struct Chord {
    /// The lowest pitch or pitch class of the chord.
    pub bass: Pitch,
    /// The list of intervals of chord tones above the bass. These should be
    /// simple intervals, and they must be sorted from smallest to largest.
    pub intervals: Rc<[Interval]>,
    /// The list of chord pitches INCLUDING the bass. They must be sorted from
    /// lowest to highest.
    pub tones: Rc<[Pitch]>,
    /// The position number of the chord. This is equal to the index of the
    /// root pitch in an arrangement of the chord from the bass upwards. For
    /// example, a `position` of 0 means the root position, 1 means the first
    /// inversion, etc.
    pub position: usize,
    /// A text label showing the quality of the chord.
    pub label: Option<String>,
}

impl PartialEq for Chord {
    fn eq(&self, other: &Self) -> bool {
        self.bass == other.bass && self.intervals == other.intervals && self.position == other.position
    }
}

impl Eq for Chord {}

impl std::hash::Hash for Chord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bass.hash(state);
        self.intervals.hash(state);
        self.position.hash(state);
    }
}

impl Chord {
    fn new(bass: Pitch, intervals: Rc<[Interval]>, tones: Rc<[Pitch]>, position: usize) -> Chord {
        debug_assert!(position <= intervals.len());
        debug_assert!(position < tones.len());
        debug_assert_eq!(bass, tones[0]);
        Chord {
            bass,
            intervals,
            tones,
            position,
            label: None,
        }
    }

    #[must_use]
    pub fn root(&self) -> Pitch {
        self.tones[self.position]
    }

    /// Construct a chord from its tones.
    ///
    /// `ps` must be sorted from lowest to highest.
    #[must_use]
    pub fn from_pitches(ps: &[Pitch], position: usize) -> Chord {
        debug_assert!(!ps.is_empty());
        let bass = ps[0];
        let mut ints = Vec::new();
        let mut tones = vec![bass];
        for p in &ps[1..] {
            let tone = p.with_period(i32::from(p.index <= bass.index));
            tones.push(tone);
            ints.push(bass.interval_to(&tone));
        }
        Chord::new(bass, Rc::from(ints), Rc::from(tones), position)
    }

    /// Construct a chord from its intervals.
    ///
    /// `ints` must be simple intervals and sorted from smallest to largest.
    #[must_use]
    pub fn from_intervals_stacking(ints: &[Interval], position: usize, bass: Pitch) -> Chord {
        debug_assert!(!ints.is_empty());
        let mut tones = vec![bass];
        let mut tone = bass;
        for int in ints {
            tone = tone.add(int);
            tones.push(tone);
        }
        Chord::from_pitches(&tones, position)
    }

    /// Construct a chord from its intervals.
    ///
    /// `ints` must be simple intervals and sorted from smallest to largest.
    #[must_use]
    pub fn from_intervals(ints: &[Interval], position: usize, bass: Pitch) -> Chord {
        debug_assert!(!ints.is_empty());
        let tones: Vec<Pitch> = std::iter::once(bass)
            .chain(ints.iter().map(|int| bass.add(int)))
            .collect();
        Chord::new(bass, Rc::from(ints.to_vec()), Rc::from(tones), position)
    }

    #[must_use]
    pub fn contains(&self, p: &Pitch) -> bool {
        let p0 = p.with_period(0);
        self.tones.iter().any(|x| x.with_period(0) == p0)
    }

    #[must_use]
    pub fn enharmonically_contains(&self, p: &Pitch) -> bool {
        let p0 = p.with_period(0);
        self.tones
            .iter()
            .any(|x| x.with_period(0).enharmonically_equals(&p0))
    }

    #[must_use]
    pub fn with_bass(&self, b: Pitch) -> Chord {
        Chord::from_intervals(&self.intervals, self.position, b).with_label(self.label.clone())
    }

    #[must_use]
    pub fn with_root(&self, r: Pitch) -> Chord {
        self.to_position(0).with_bass(r).to_position(self.position)
    }

    #[must_use]
    pub fn with_label(&self, l: Option<String>) -> Chord {
        let mut c = self.clone();
        c.label = l;
        c
    }

    #[must_use]
    pub fn to_position(&self, n: usize) -> Chord {
        if n == self.position {
            return self.clone();
        }
        debug_assert!(n <= self.intervals.len());
        let rotated = passacaglia_common::rotate_array(&self.tones, (n as i64) - (self.position as i64));
        Chord::from_pitches(&rotated, n).with_label(self.label.clone())
    }
}

impl std::fmt::Display for Chord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.tones
                .iter()
                .map(Pitch::to_class_string)
                .collect::<Vec<_>>()
                .join("|")
        )
    }
}

/// Preset common-practice chords.
pub mod chords {
    use super::Chord;
    use passacaglia_macros::std_hept_interval as interval;

    #[must_use]
    pub fn major() -> Chord {
        Chord::from_intervals_stacking(&[interval!("M3"), interval!("m3")], 0, super::PITCH_CLASSES.c)
    }

    #[must_use]
    pub fn major6() -> Chord {
        major().to_position(1).with_label(Some("6".into()))
    }

    #[must_use]
    pub fn minor() -> Chord {
        Chord::from_intervals_stacking(&[interval!("m3"), interval!("M3")], 0, super::PITCH_CLASSES.c)
    }

    #[must_use]
    pub fn minor6() -> Chord {
        minor().to_position(1).with_label(Some("m6".into()))
    }

    #[must_use]
    pub fn dim() -> Chord {
        Chord::from_intervals_stacking(&[interval!("m3"), interval!("m3")], 0, super::PITCH_CLASSES.c)
    }

    #[must_use]
    pub fn dim6() -> Chord {
        dim().to_position(1).with_label(Some("dim6".into()))
    }

    #[must_use]
    pub fn aug() -> Chord {
        Chord::from_intervals_stacking(&[interval!("M3"), interval!("M3")], 0, super::PITCH_CLASSES.c)
    }

    #[must_use]
    pub fn dominant7() -> Chord {
        Chord::from_intervals_stacking(
            &[interval!("M3"), interval!("m3"), interval!("m3")],
            0,
            super::PITCH_CLASSES.c,
        )
    }
}

/// A duration plus an optional chord.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChordElement {
    pub duration: Rational,
    pub chord: Option<Chord>,
}

impl TemporalElement for ChordElement {}
impl DurationalElement for ChordElement {
    fn duration(&self) -> Rational {
        self.duration
    }
}

/// A chord cursor into a [`Harmony`].
pub type ChordCursor<'a> = Cursor<'a, Harmony, ()>;

/// A harmonic background: a scale plus a sequence of chord elements.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Harmony {
    pub scale: passacaglia_core::std_hept::Scale,
    pub elements: Vector<ChordElement>,
}

impl Harmony {
    #[must_use]
    pub fn new(scale: passacaglia_core::std_hept::Scale, e: Vec<ChordElement>) -> Harmony {
        Harmony {
            scale,
            elements: e.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn replace_chord(&self, i: usize, chord: Option<Chord>) -> Harmony {
        let old = self.elements[i].clone();
        let e = ChordElement {
            duration: old.duration,
            chord,
        };
        Harmony {
            scale: self.scale.clone(),
            elements: self.elements.update(i, e),
        }
    }
}

impl Container for Harmony {
    type Item = ChordElement;

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn is_sequential(&self) -> bool {
        true
    }

    fn start(&self, i: usize) -> Rational {
        self.elements
            .iter()
            .take(i)
            .fold(rational(0), |acc, e| acc + e.duration)
    }

    fn span(&self, i: usize) -> Rational {
        self.elements[i].duration
    }

    fn item(&self, i: usize) -> &ChordElement {
        &self.elements[i]
    }
}
