use std::rc::Rc;

use im::Vector;

use passacaglia_common::Rational;
use passacaglia_core::std_hept::{Pitch, Scale};
use passacaglia_macros::std_hept_pitch as pitch;

use crate::chord::{Chord, ChordElement, Harmony};
use crate::clef::Clef;
use crate::context::CounterpointContext;
use crate::score::Score;
use crate::voice::{CounterpointVoice, FixedVoice, Measure, MeasureKind, Note, Voice, VoiceKind};

/// Leap-tracking state threaded through a voice's measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MelodicContext {
    pub last_pitch: Option<Pitch>,
    pub leap_direction: i64,
    pub n_consecutive_leaps: i64,
    pub n3rd_leaps: i64,
    pub n_unidirectional_consecutive_leaps: i64,
    pub n_unidirectional_3rd_leaps: i64,
}

#[must_use]
pub fn empty_melodic_context() -> MelodicContext {
    MelodicContext {
        last_pitch: None,
        leap_direction: 0,
        n_consecutive_leaps: 0,
        n3rd_leaps: 0,
        n_unidirectional_consecutive_leaps: 0,
        n_unidirectional_3rd_leaps: 0,
    }
}

/// Update the melodic context after writing a note with pitch `p`.
#[must_use]
pub fn update_melodic_context(old: MelodicContext, p: Option<Pitch>) -> MelodicContext {
    let cleared = |last_pitch| MelodicContext {
        last_pitch,
        leap_direction: 0,
        n_consecutive_leaps: 0,
        n3rd_leaps: 0,
        n_unidirectional_consecutive_leaps: 0,
        n_unidirectional_3rd_leaps: 0,
    };

    let Some(last) = old.last_pitch else {
        return MelodicContext { last_pitch: p, ..old };
    };

    let Some(pp) = p else {
        return cleared(None);
    };
    let int = last.interval_to(&pp);

    if int.steps <= 1 {
        return cleared(Some(pp));
    }

    let is_third = int.steps == 2;
    let is_unidirectional = i64::from(int.sign) == old.leap_direction;
    MelodicContext {
        last_pitch: Some(pp),
        leap_direction: i64::from(int.sign),
        n_consecutive_leaps: old.n_consecutive_leaps + 1,
        n3rd_leaps: old.n3rd_leaps + i64::from(is_third),
        n_unidirectional_consecutive_leaps: if is_unidirectional {
            old.n_unidirectional_consecutive_leaps + 1
        } else {
            0
        },
        n_unidirectional_3rd_leaps: if is_unidirectional {
            old.n_unidirectional_3rd_leaps + i64::from(is_third)
        } else {
            0
        },
    }
}

/// A candidate successor measure, as produced by the solver.
#[derive(Clone)]
pub struct Step {
    pub measure: Measure,
    pub advanced: Rational,
    pub cost: f64,
    pub debug: String,
    /// The score that results from writing `measure`, when it was already
    /// constructed (e.g. to evaluate local rules). Reused by the solver instead
    /// of rebuilding it with `replace_measure` + `replace_voice`.
    pub score: Option<Score>,
}

/// A new measure (and cost) produced by a voice's `make_new_measure`.
#[derive(Clone)]
pub struct NewMeasure {
    pub measure: Measure,
    pub cost: f64,
}

/// Melodic constraints for a counterpoint voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MelodicSettings {
    pub forbid_repeated_notes: bool,
    pub max_consecutive_leaps: i64,
    pub max_ignorable_3rd_leaps: i64,
    pub max_unidirectional_consecutive_leaps: i64,
    pub max_unidirectional_ignorable_3rd_leaps: i64,
}

/// A factory for a counterpoint voice (produced by `define_species` /
/// `define_imitation`).
#[derive(Clone)]
pub struct VoiceConstructor {
    pub melody_settings: MelodicSettings,
    pub kind: VoiceKind,
}

impl VoiceConstructor {
    #[must_use]
    pub fn make_voice(
        &self,
        index: usize,
        ctx: &Rc<CounterpointContext>,
        measures: Vector<Measure>,
        lower_range: Pitch,
        higher_range: Pitch,
        name: String,
        clef: Clef,
    ) -> Voice {
        Voice::Counterpoint(CounterpointVoice {
            index,
            ctx: ctx.clone(),
            measures,
            lower_range,
            higher_range,
            name,
            clef,
            melody_settings: self.melody_settings,
            kind: self.kind.clone(),
        })
    }
}

/// Builds a [`Score`] from fixed (cantus) and counterpoint voices.
pub struct CounterpointScoreBuilder {
    ctx: Rc<CounterpointContext>,
    voices: Vec<Voice>,
}

impl CounterpointScoreBuilder {
    #[must_use]
    pub fn new(ctx: Rc<CounterpointContext>) -> Self {
        CounterpointScoreBuilder {
            ctx,
            voices: Vec::new(),
        }
    }

    #[must_use]
    pub fn build(&self, scale: &Scale, chords: Option<&[Chord]>) -> Score {
        let ms: Vec<ChordElement> = (0..self.ctx.target_measures)
            .map(|i| ChordElement {
                duration: self.ctx.parameters.measure_length,
                chord: chords.and_then(|c| c.get(i).cloned()),
            })
            .collect();
        let h = Harmony::new(scale.clone(), ms);
        Score::new(self.ctx.parameters, self.voices.clone(), h)
    }

    pub fn voice(
        &mut self,
        v: &VoiceConstructor,
        clef: Clef,
        name: &str,
        lower: Pitch,
        higher: Pitch,
    ) -> &mut Self {
        let ms: Vec<Measure> = (0..self.ctx.target_measures)
            .map(|_| Measure::blank(&self.ctx))
            .collect();
        let voice = v.make_voice(
            self.voices.len(),
            &self.ctx,
            ms.into_iter().collect(),
            lower,
            higher,
            name.to_string(),
            clef,
        );
        self.voices.push(voice);
        self
    }

    pub fn cantus(&mut self, clef: Clef, measures: &[Vec<Note>]) -> &mut Self {
        let ms: Vec<Measure> = measures
            .iter()
            .map(|x| Measure {
                notes: Rc::from(x.clone()),
                duration: self.ctx.parameters.measure_length,
                kind: MeasureKind::Fixed,
            })
            .collect();
        let voice = Voice::Fixed(FixedVoice {
            index: self.voices.len(),
            measures: ms.into_iter().collect(),
            clef,
            name: "Cantus".to_string(),
        });
        self.voices.push(voice);
        self
    }

    pub fn soprano(&mut self, v: &VoiceConstructor) -> &mut Self {
        self.voice(
            v,
            Clef::TREBLE,
            "Soprano",
            pitch!("c4"),
            pitch!("a5"),
        )
    }

    pub fn alto(&mut self, v: &VoiceConstructor) -> &mut Self {
        self.voice(
            v,
            Clef::TREBLE,
            "Alto",
            pitch!("f3"),
            pitch!("d5"),
        )
    }

    pub fn tenor(&mut self, v: &VoiceConstructor) -> &mut Self {
        self.voice(
            v,
            Clef::ALTO,
            "Tenor",
            pitch!("c3"),
            pitch!("a4"),
        )
    }

    pub fn bass(&mut self, v: &VoiceConstructor) -> &mut Self {
        self.voice(
            v,
            Clef::BASS,
            "Bass",
            pitch!("f2"),
            pitch!("d4"),
        )
    }
}
