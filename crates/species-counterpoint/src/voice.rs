use std::hash::{Hash, Hasher};
use std::rc::Rc;

use im::Vector;

use passacaglia_common::{rational, Rational};
use passacaglia_core::std_hept::Pitch;
use passacaglia_core::structure::{Container, Cursor, DurationalElement, TemporalElement};

use crate::basic::{empty_melodic_context, MelodicContext, MelodicSettings, NewMeasure, Step};
use crate::clef::Clef;
use crate::context::CounterpointContext;
use crate::imitation::ImitationMeasure;
use crate::score::Score;
use crate::species::{note_total, FakeMeasure, MeasureSchema, NoteSchema, SpeciesMeasure};

/// The kind of non-harmonic tone that a note can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NonHarmonicType {
    PassingTone,
    Suspension,
    Neighbor,
}

/// A note: a duration plus an optional pitch and non-harmonic type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Note {
    pub duration: Rational,
    /// `None` means either it is not filled in, or it's a rest.
    pub pitch: Option<Pitch>,
    pub non_harmonic: Option<NonHarmonicType>,
    pub debug: String,
}

impl Note {
    #[must_use]
    pub fn new(duration: Rational, pitch: Option<Pitch>, non_harmonic: Option<NonHarmonicType>) -> Note {
        Note {
            duration,
            pitch,
            non_harmonic,
            debug: String::new(),
        }
    }

    #[must_use]
    pub fn is_non_harmonic(&self) -> bool {
        self.non_harmonic.is_some()
    }

    #[must_use]
    pub fn is_tied(&self) -> bool {
        self.non_harmonic == Some(NonHarmonicType::Suspension)
    }
}

impl TemporalElement for Note {}
impl DurationalElement for Note {
    fn duration(&self) -> Rational {
        self.duration
    }
}

/// Parse a list of `(pitch, duration)` pairs into notes. A `None` pitch is a rest.
#[must_use]
pub fn parse_notes(notes: &[(Option<&str>, Rational)]) -> Vec<Note> {
    notes
        .iter()
        .map(|(p, len)| {
            let pitch = p.and_then(Pitch::parse);
            Note::new(*len, pitch, None)
        })
        .collect()
}

/// A cursor pointing at a measure inside a voice.
pub type MeasureCursor<'a> = Cursor<'a, Voice, ()>;
/// A cursor pointing at a note inside a measure (whose parent is a measure cursor).
pub type NoteCursor<'a> = Cursor<'a, Measure, MeasureCursor<'a>>;

/// A measure: a sequence of notes plus its own (possibly partial) duration and
/// a kind describing how the solver may write into it.
#[derive(Clone)]
pub struct Measure {
    pub notes: Rc<[Note]>,
    pub duration: Rational,
    pub kind: MeasureKind,
}

#[derive(Clone)]
pub enum MeasureKind {
    Blank,
    Fixed,
    Species(SpeciesMeasure),
    Fake(FakeMeasure),
    Imitation(ImitationMeasure),
}

impl Measure {
    #[must_use]
    pub fn blank(ctx: &Rc<CounterpointContext>) -> Measure {
        Measure {
            notes: Rc::from(vec![Note::new(ctx.parameters.measure_length, None, None)]),
            duration: ctx.parameters.measure_length,
            kind: MeasureKind::Blank,
        }
    }

    #[must_use]
    pub fn writable_position(&self) -> Option<Rational> {
        match &self.kind {
            MeasureKind::Blank | MeasureKind::Fake(_) => Some(rational(0)),
            MeasureKind::Fixed => None,
            MeasureKind::Species(sm) => {
                let idx = (0..self.notes.len()).find(|&i| {
                    self.notes[i].pitch.is_none()
                        && matches!(sm.note_schema.get(i), Some(NoteSchema::Tone { .. }))
                });
                idx.map_or_else(
                    || {
                        let total = note_total(&self.notes);
                        (total < sm.ctx.parameters.measure_length).then_some(total)
                    },
                    |i| Some(note_start(&self.notes, i)),
                )
            }
            MeasureKind::Imitation(im) => match im {
                ImitationMeasure::Empty { .. } => None,
                ImitationMeasure::Filled { target, filled, .. } => {
                    if filled.len() < target.len() {
                        Some(filled.iter().fold(rational(0), |acc, n| acc + n.duration))
                    } else {
                        None
                    }
                }
            },
        }
    }

    #[must_use]
    pub fn melodic_context(&self) -> MelodicContext {
        match &self.kind {
            MeasureKind::Species(sm) => sm.melodic_context,
            MeasureKind::Fake(fm) => fm.melodic_context,
            MeasureKind::Imitation(im) => match im {
                ImitationMeasure::Empty { melodic_context, .. }
                | ImitationMeasure::Filled { melodic_context, .. } => *melodic_context,
            },
            _ => empty_melodic_context(),
        }
    }

    #[must_use]
    pub fn schema_name(&self) -> Option<&str> {
        match &self.kind {
            MeasureKind::Species(sm) => Some(&sm.name),
            _ => None,
        }
    }

    #[must_use]
    pub fn index_at_time(&self, t: Rational) -> Option<usize> {
        let mut acc = rational(0);
        for (i, n) in self.notes.iter().enumerate() {
            if acc <= t && acc + n.duration > t {
                return Some(i);
            }
            acc += n.duration;
        }
        None
    }

    /// Produce the candidate successor measures for this writable measure.
    #[must_use]
    pub fn get_next_steps<'a>(&self, s: &'a Score, c: MeasureCursor<'a>) -> Vec<Step> {
        match &self.kind {
            MeasureKind::Blank => {
                let voice = c.container();
                if let Voice::Counterpoint(cp) = voice {
                    cp.make_new_measure(s, c)
                        .into_iter()
                        .map(|nm| Step {
                            measure: nm.measure,
                            advanced: rational(0),
                            cost: nm.cost,
                            debug: "from_blank".to_string(),
                            score: None,
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
            MeasureKind::Fixed => Vec::new(),
            MeasureKind::Species(sm) => crate::species::species_get_next_steps(self, sm, s, c),
            MeasureKind::Fake(fm) => crate::species::fake_get_next_steps(fm),
            MeasureKind::Imitation(im) => crate::imitation::imitation_get_next_steps(self, im, s, c),
        }
    }
}

fn note_start(notes: &[Note], i: usize) -> Rational {
    notes[..i].iter().fold(rational(0), |acc, n| acc + n.duration)
}

impl TemporalElement for Measure {}
impl DurationalElement for Measure {
    fn duration(&self) -> Rational {
        self.duration
    }
}

impl Container for Measure {
    type Item = Note;

    fn len(&self) -> usize {
        self.notes.len()
    }

    fn is_sequential(&self) -> bool {
        true
    }

    fn start(&self, i: usize) -> Rational {
        note_start(&self.notes, i)
    }

    fn span(&self, i: usize) -> Rational {
        self.notes[i].duration
    }

    fn item(&self, i: usize) -> &Note {
        &self.notes[i]
    }
}

/// A fixed voice (the cantus firmus), whose measures never change.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FixedVoice {
    pub index: usize,
    pub measures: Vector<Measure>,
    pub clef: Clef,
    pub name: String,
}

/// How a counterpoint voice generates its next measure.
#[derive(Clone)]
pub enum VoiceKind {
    Species { schemas: Rc<[MeasureSchema]> },
    Imitation { target_voice: usize, delay: i64, transform: Rc<dyn Fn(Pitch) -> Vec<Pitch>> },
}

/// A small hashable tag describing the voice kind, used for structural hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceKindTag {
    Fixed,
    Species,
    Imitation { target_voice: usize, delay: i64 },
}

/// A solver-aware voice.
#[derive(Clone)]
pub struct CounterpointVoice {
    pub index: usize,
    pub ctx: Rc<CounterpointContext>,
    pub measures: Vector<Measure>,
    pub lower_range: Pitch,
    pub higher_range: Pitch,
    pub name: String,
    pub clef: Clef,
    pub melody_settings: MelodicSettings,
    pub kind: VoiceKind,
}

impl CounterpointVoice {
    #[must_use]
    pub fn kind_tag(&self) -> VoiceKindTag {
        match &self.kind {
            VoiceKind::Species { .. } => VoiceKindTag::Species,
            VoiceKind::Imitation {
                target_voice, delay, ..
            } => VoiceKindTag::Imitation {
                target_voice: *target_voice,
                delay: *delay,
            },
        }
    }

    #[must_use]
    pub fn make_new_measure<'a>(&self, score: &'a Score, c: MeasureCursor<'a>) -> Vec<NewMeasure> {
        match &self.kind {
            VoiceKind::Species { schemas } => {
                crate::species::species_make_new_measure(self, schemas, score, c)
            }
            VoiceKind::Imitation {
                target_voice,
                delay,
                transform,
            } => crate::imitation::imitation_make_new_measure(
                self,
                *target_voice,
                *delay,
                transform.clone(),
                score,
                c,
            ),
        }
    }

    #[must_use]
    pub fn replace_measure(&self, i: usize, m: Measure) -> CounterpointVoice {
        let mut v = self.clone();
        v.measures = self.measures.update(i, m);
        v
    }
}

/// A voice: either a fixed voice or a solver-aware counterpoint voice.
#[derive(Clone)]
pub enum Voice {
    Fixed(FixedVoice),
    Counterpoint(CounterpointVoice),
}

impl Voice {
    #[must_use]
    pub fn index(&self) -> usize {
        match self {
            Voice::Fixed(f) => f.index,
            Voice::Counterpoint(cp) => cp.index,
        }
    }

    #[must_use]
    pub fn measures(&self) -> &Vector<Measure> {
        match self {
            Voice::Fixed(f) => &f.measures,
            Voice::Counterpoint(cp) => &cp.measures,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Voice::Fixed(f) => &f.name,
            Voice::Counterpoint(cp) => &cp.name,
        }
    }

    #[must_use]
    pub fn clef(&self) -> Clef {
        match self {
            Voice::Fixed(f) => f.clef,
            Voice::Counterpoint(cp) => cp.clef,
        }
    }

    #[must_use]
    pub fn ranges(&self) -> Option<(Pitch, Pitch)> {
        match self {
            Voice::Counterpoint(cp) => Some((cp.lower_range, cp.higher_range)),
            Voice::Fixed(_) => None,
        }
    }

    #[must_use]
    pub fn melody_settings(&self) -> Option<&MelodicSettings> {
        match self {
            Voice::Counterpoint(cp) => Some(&cp.melody_settings),
            Voice::Fixed(_) => None,
        }
    }

    #[must_use]
    pub fn kind_tag(&self) -> VoiceKindTag {
        match self {
            Voice::Fixed(_) => VoiceKindTag::Fixed,
            Voice::Counterpoint(cp) => cp.kind_tag(),
        }
    }

    /// The note at global time `t`, if any.
    #[must_use]
    pub fn note_at(&self, t: Rational) -> Option<NoteCursor<'_>> {
        let m = self.cursor_at_time(t)?;
        let local = t - m.time();
        let idx = m.index_at_time(local)?;
        m.child(idx)
    }

    #[must_use]
    pub fn replace_measure(&self, i: usize, m: Measure) -> Voice {
        match self {
            Voice::Fixed(f) => {
                Voice::Fixed(FixedVoice {
                    measures: f.measures.update(i, m),
                    ..f.clone()
                })
            }
            Voice::Counterpoint(cp) => Voice::Counterpoint(cp.replace_measure(i, m)),
        }
    }
}

impl TemporalElement for Voice {}
impl Container for Voice {
    type Item = Measure;

    fn len(&self) -> usize {
        self.measures().len()
    }

    fn is_sequential(&self) -> bool {
        true
    }

    fn start(&self, i: usize) -> Rational {
        self.measures()
            .iter()
            .take(i)
            .fold(rational(0), |acc, m| acc + m.duration)
    }

    fn span(&self, i: usize) -> Rational {
        self.measures()[i].duration
    }

    fn item(&self, i: usize) -> &Measure {
        &self.measures()[i]
    }
}

impl PartialEq for Voice {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index()
            && self.measures() == other.measures()
            && self.name() == other.name()
            && self.clef() == other.clef()
            && self.ranges() == other.ranges()
            && self.melody_settings() == other.melody_settings()
            && self.kind_tag() == other.kind_tag()
    }
}

impl Eq for Voice {}

impl Hash for Voice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index().hash(state);
        self.measures().hash(state);
        self.name().hash(state);
        self.clef().hash(state);
        self.ranges().hash(state);
        self.melody_settings().hash(state);
        self.kind_tag().hash(state);
    }
}

fn measure_kind_eq(a: &MeasureKind, b: &MeasureKind) -> bool {
    match (a, b) {
        (MeasureKind::Blank, MeasureKind::Blank) | (MeasureKind::Fixed, MeasureKind::Fixed) => true,
        (MeasureKind::Species(x), MeasureKind::Species(y)) => {
            x.name == y.name && x.note_schema == y.note_schema
        }
        (MeasureKind::Fake(x), MeasureKind::Fake(y)) => x.p0 == y.p0,
        (MeasureKind::Imitation(x), MeasureKind::Imitation(y)) => imitation_eq(x, y),
        _ => false,
    }
}

fn measure_kind_hash<H: Hasher>(kind: &MeasureKind, state: &mut H) {
    match kind {
        MeasureKind::Blank => 0u8.hash(state),
        MeasureKind::Fixed => 1u8.hash(state),
        MeasureKind::Species(sm) => {
            2u8.hash(state);
            sm.name.hash(state);
            sm.note_schema.hash(state);
        }
        MeasureKind::Fake(fm) => {
            3u8.hash(state);
            fm.p0.hash(state);
        }
        MeasureKind::Imitation(im) => imitation_hash(im, state),
    }
}

fn imitation_eq(a: &ImitationMeasure, b: &ImitationMeasure) -> bool {
    match (a, b) {
        (
            ImitationMeasure::Empty {
                target_voice: tv1,
                target_measure: tm1,
                ..
            },
            ImitationMeasure::Empty {
                target_voice: tv2,
                target_measure: tm2,
                ..
            },
        ) => tv1 == tv2 && tm1 == tm2,
        (
            ImitationMeasure::Filled {
                target_voice: tv1,
                target_measure: tm1,
                target: t1,
                filled: f1,
                ..
            },
            ImitationMeasure::Filled {
                target_voice: tv2,
                target_measure: tm2,
                target: t2,
                filled: f2,
                ..
            },
        ) => tv1 == tv2 && tm1 == tm2 && t1 == t2 && f1 == f2,
        _ => false,
    }
}

fn imitation_hash<H: Hasher>(im: &ImitationMeasure, state: &mut H) {
    match im {
        ImitationMeasure::Empty {
            target_voice,
            target_measure,
            ..
        } => {
            4u8.hash(state);
            target_voice.hash(state);
            target_measure.hash(state);
        }
        ImitationMeasure::Filled {
            target_voice,
            target_measure,
            target,
            filled,
            ..
        } => {
            5u8.hash(state);
            target_voice.hash(state);
            target_measure.hash(state);
            target.hash(state);
            filled.hash(state);
        }
    }
}

impl PartialEq for Measure {
    fn eq(&self, other: &Self) -> bool {
        self.notes == other.notes
            && self.duration == other.duration
            && measure_kind_eq(&self.kind, &other.kind)
    }
}

impl Eq for Measure {}

impl Hash for Measure {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.notes.hash(state);
        self.duration.hash(state);
        measure_kind_hash(&self.kind, state);
    }
}
