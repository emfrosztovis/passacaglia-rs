use std::collections::HashMap;
use std::rc::Rc;

use passacaglia_core::std_hept::{Interval, Pitch};

use crate::basic::Step;
use crate::chord::{Chord, ChordCursor};
use crate::score::{Parameters, Score};
use crate::voice::{Measure, Note, NoteCursor, NonHarmonicType};

/// A cost map keyed by candidate (pitch or chord), stored as an ordered list.
///
/// Candidate sets are small (scale tones in a voice range), so a linear-scan
/// `Vec` is both faster and — unlike `HashMap` — deterministic: insertion order
/// is preserved, mirroring the TypeScript `HashMap` (a `Map` with insertion
/// order). `set` updates the value in place without changing position.
#[derive(Debug, Clone)]
pub struct Candidates<T: Clone + Eq>(Vec<(T, f64)>);

impl<T: Clone + Eq> Default for Candidates<T> {
    fn default() -> Self {
        Candidates(Vec::new())
    }
}

impl<T: Clone + Eq> Candidates<T> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_pairs(iter: impl IntoIterator<Item = (T, f64)>) -> Self {
        let mut c = Candidates::new();
        for (k, v) in iter {
            c.set(k, v);
        }
        c
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn get(&self, k: &T) -> Option<f64> {
        self.0.iter().find(|(ek, _)| ek == k).map(|(_, v)| *v)
    }

    #[must_use]
    pub fn contains(&self, k: &T) -> bool {
        self.0.iter().any(|(ek, _)| ek == k)
    }

    pub fn set(&mut self, k: T, v: f64) {
        if let Some(entry) = self.0.iter_mut().find(|(ek, _)| *ek == k) {
            entry.1 = v;
        } else {
            self.0.push((k, v));
        }
    }

    pub fn remove(&mut self, k: &T) {
        self.0.retain(|(ek, _)| ek != k);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&T, &f64)> {
        self.0.iter().map(|(k, v)| (k, v))
    }

    pub fn filter(&mut self, pred: impl Fn(&T, f64) -> bool) {
        self.0.retain(|(k, v)| pred(k, *v));
    }

    pub fn filter_map(&mut self, pred: impl Fn(&T, f64) -> Option<f64>) {
        let mut i = 0;
        while i < self.0.len() {
            let r = pred(&self.0[i].0, self.0[i].1);
            match r {
                Some(nv) => {
                    self.0[i].1 = nv;
                    i += 1;
                }
                None => {
                    self.0.remove(i);
                }
            }
        }
    }

    pub fn intersect(&mut self, other: &Candidates<T>) {
        self.0.retain(|(k, _)| other.contains(k));
    }

    pub fn intersect_with(&mut self, other: &Candidates<T>, combine: impl Fn(f64, f64) -> f64) {
        let mut i = 0;
        while i < self.0.len() {
            let ov = other.get(&self.0[i].0);
            match ov {
                Some(ov) => {
                    self.0[i].1 = combine(self.0[i].1, ov);
                    i += 1;
                }
                None => {
                    self.0.remove(i);
                }
            }
        }
    }
}

/// A local rule evaluating the cost of a just-written note.
pub type LocalRule =
    Rc<dyn for<'a> Fn(&CounterpointContext, &'a Score, NoteCursor<'a>) -> f64>;

/// A global rule returning a description of a violation, or `None` if valid.
pub type GlobalRule =
    Rc<dyn for<'a> Fn(&CounterpointContext, &'a Score) -> Option<String>>;

/// A rule refining the candidate pitches for a note.
pub type CandidateRule = Rc<
    dyn for<'a> Fn(
        &CounterpointContext,
        &'a Score,
        NoteCursor<'a>,
        Option<Candidates<Pitch>>,
        Option<NonHarmonicType>,
    ) -> Candidates<Pitch>,
>;

/// A rule refining the candidate chords for a harmony slot.
pub type HarmonyRule = Rc<HarmonyRuleFn>;

pub type HarmonyRuleFn = dyn for<'a> Fn(
    &CounterpointContext,
    &'a Score,
    ChordCursor<'a>,
    Option<Candidates<Chord>>,
) -> Candidates<Chord>;

/// Rule registry, cost settings, and candidate-generation helpers.
pub struct CounterpointContext {
    pub target_measures: usize,
    pub parameters: Parameters,

    pub local_rules: Vec<LocalRule>,
    pub global_rules: Vec<GlobalRule>,
    pub candidate_rules_before: Vec<CandidateRule>,
    pub candidate_rules_after: Vec<CandidateRule>,
    pub harmony_rules: Vec<HarmonyRule>,
    pub non_harmonic_tone_rules: HashMap<NonHarmonicType, Vec<CandidateRule>>,
    pub harmonic_tone_rules: Vec<CandidateRule>,

    pub similar_motion_cost: f64,
    pub oblique_motion_cost: f64,
    pub contrary_motion_cost: f64,
    pub melodic_intervals: HashMap<Interval, f64>,
    pub allow_unison: bool,
}

/// Parse a list of `(abbreviation, cost)` interval preferences.
#[must_use]
pub fn parse_preferred(ps: &[(&str, f64)]) -> HashMap<Interval, f64> {
    ps.iter()
        .map(|(ex, cost)| (Interval::parse(ex).expect("valid interval literal"), *cost))
        .collect()
}

impl CounterpointContext {
    #[must_use]
    pub fn new(target_measures: usize, parameters: Parameters) -> Self {
        CounterpointContext {
            target_measures,
            parameters,
            local_rules: Vec::new(),
            global_rules: Vec::new(),
            candidate_rules_before: Vec::new(),
            candidate_rules_after: Vec::new(),
            harmony_rules: Vec::new(),
            non_harmonic_tone_rules: HashMap::new(),
            harmonic_tone_rules: Vec::new(),
            similar_motion_cost: 40.0,
            oblique_motion_cost: 20.0,
            contrary_motion_cost: 0.0,
            melodic_intervals: parse_preferred(&[
                ("m2", 0.0),
                ("M2", 0.0),
                ("-m2", 60.0),
                ("-M2", 60.0),
                ("m3", 75.0),
                ("M3", 75.0),
                ("-m3", 75.0),
                ("-M3", 75.0),
                ("P4", 90.0),
                ("-P4", 90.0),
                ("P5", 90.0),
                ("-P5", 90.0),
                ("m6", 90.0),
                ("M6", 90.0),
                ("-m6", 90.0),
                ("-M6", 90.0),
                ("P8", 100.0),
                ("-P8", 100.0),
                ("P1", 200.0),
            ]),
            allow_unison: false,
        }
    }

    pub fn get_candidates<'a>(
        &self,
        rules: &[CandidateRule],
        s: &'a Score,
        current: NoteCursor<'a>,
        ty: Option<NonHarmonicType>,
    ) -> Candidates<Pitch> {
        let mut candidates: Option<Candidates<Pitch>> = None;
        for rule in self
            .candidate_rules_before
            .iter()
            .chain(rules.iter())
            .chain(self.candidate_rules_after.iter())
        {
            let c = rule(self, s, current, candidates, ty);
            if c.is_empty() {
                return c;
            }
            candidates = Some(c);
        }
        candidates.expect("at least one candidate rule")
    }

    pub fn fill_non_harmonic_tone<'a>(
        &self,
        types: &[NonHarmonicType],
        s: &'a Score,
        note: NoteCursor<'a>,
        create: &dyn Fn(Note, Pitch) -> Measure,
        cost_offset: f64,
    ) -> Vec<Step> {
        let mut results = Vec::new();
        for ty in types {
            if let Some(rules) = self.non_harmonic_tone_rules.get(ty) {
                results.extend(self.fill_in(rules, s, note, Some(*ty), create, cost_offset));
            }
        }
        results
    }

    pub fn fill_harmonic_tone<'a>(
        &self,
        s: &'a Score,
        note: NoteCursor<'a>,
        create: &dyn Fn(Note, Pitch) -> Measure,
        cost_offset: f64,
    ) -> Vec<Step> {
        self.fill_in(&self.harmonic_tone_rules, s, note, None, create, cost_offset)
    }

    pub fn fill_in<'a>(
        &self,
        rules: &[CandidateRule],
        s: &'a Score,
        note: NoteCursor<'a>,
        ty: Option<NonHarmonicType>,
        create: &dyn Fn(Note, Pitch) -> Measure,
        cost_offset: f64,
    ) -> Vec<Step> {
        let measure = note.parent();
        let voice = measure.container();
        let candidates = self.get_candidates(rules, s, note, ty);

        let mut steps = Vec::new();
        for (p, cost) in candidates.iter() {
            let p = *p;
            let m = create(Note::new(note.span(), Some(p), ty), p);
            let new_voice = voice.replace_measure(measure.index(), m.clone());
            let new_score = s.replace_voice(voice.index(), new_voice);
            let new_cursor = new_score.voices[voice.index()]
                .note_at(note.global_time())
                .expect("new cursor exists");

            let mut cost = *cost;
            let mut debug = Vec::new();
            for rule in &self.local_rules {
                let c = rule(self, &new_score, new_cursor);
                cost += c;
                if c != 0.0 {
                    debug.push(format!("{c}"));
                }
            }
            steps.push(Step {
                measure: m,
                cost: cost + cost_offset,
                advanced: note.span(),
                debug: debug.join("\n"),
                score: Some(new_score),
            });
        }
        steps
    }

    #[must_use]
    pub fn get_chord_candidates<'a>(
        &self,
        s: &'a Score,
        current: ChordCursor<'a>,
    ) -> Candidates<Chord> {
        let mut candidates: Option<Candidates<Chord>> = None;
        for rule in &self.harmony_rules {
            let c = rule(self, s, current, candidates);
            if c.is_empty() {
                return c;
            }
            candidates = Some(c);
        }
        candidates.expect("at least one harmony rule")
    }
}
