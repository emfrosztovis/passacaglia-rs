use std::sync::Arc;

use passacaglia_core::std_hept::Pitch;

use crate::basic::{
    empty_melodic_context, update_melodic_context, MelodicContext, MelodicSettings, NewMeasure, Step,
    VoiceConstructor,
};
use crate::context::{CandidateRule, CounterpointContext};
use crate::score::Score;
use crate::voice::{CounterpointVoice, Measure, MeasureCursor, MeasureKind, NonHarmonicType, Note, VoiceKind};

/// A measure imitating the notes of another voice (or an empty placeholder).
#[derive(Clone)]
pub enum ImitationMeasure {
    Empty {
        ctx: Arc<CounterpointContext>,
        melodic_context: MelodicContext,
        target_voice: usize,
        target_measure: i64,
    },
    Filled {
        ctx: Arc<CounterpointContext>,
        melodic_context: MelodicContext,
        target_voice: usize,
        target_measure: i64,
        target: Arc<[Note]>,
        filled: Arc<[Note]>,
        transform: Arc<dyn Fn(Pitch) -> Vec<Pitch>>,
    },
}

/// A candidate rule constraining candidates to a fixed set of pitches.
fn fixed(pitches: Vec<Pitch>) -> CandidateRule {
    Arc::new(move |_ctx, _s, _cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        c.filter(|p, _| pitches.contains(p));
        c
    })
}

pub(crate) fn imitation_make_new_measure<'a>(
    cp: &CounterpointVoice,
    target_voice: usize,
    delay: i64,
    transform: Arc<dyn Fn(Pitch) -> Vec<Pitch>>,
    score: &'a Score,
    c: MeasureCursor<'a>,
) -> Vec<NewMeasure> {
    let mc = c
        .prev_global()
        .map_or_else(empty_melodic_context, |p| p.melodic_context());

    let target_measure = c.index() as i64 - delay;
    let target: Option<Arc<[Note]>> = if target_measure >= 0 {
        score
            .voices
            .get(target_voice)
            .and_then(|v| v.measures().get(target_measure as usize))
            .map(|m| m.notes.clone())
    } else {
        None
    };

    let new_measure = match target {
        None => Measure {
            notes: Arc::from(vec![Note::new(cp.ctx.parameters.measure_length, None, None)]),
            duration: cp.ctx.parameters.measure_length,
            kind: MeasureKind::Imitation(ImitationMeasure::Empty {
                ctx: cp.ctx.clone(),
                melodic_context: mc,
                target_voice,
                target_measure,
            }),
        },
        Some(target) => {
            let notes: Vec<Note> = target
                .iter()
                .map(|n| Note::new(n.duration, None, None))
                .collect();
            Measure {
                notes: Arc::from(notes),
                duration: cp.ctx.parameters.measure_length,
                kind: MeasureKind::Imitation(ImitationMeasure::Filled {
                    ctx: cp.ctx.clone(),
                    melodic_context: mc,
                    target_voice,
                    target_measure,
                    target,
                    filled: Arc::from(Vec::new()),
                    transform,
                }),
            }
        }
    };

    vec![NewMeasure {
        measure: new_measure,
        cost: 0.0,
    }]
}

pub(crate) fn imitation_get_next_steps<'a>(
    measure: &Measure,
    im: &ImitationMeasure,
    s: &'a Score,
    c: MeasureCursor<'a>,
) -> Vec<Step> {
    let ImitationMeasure::Filled {
        ctx,
        melodic_context,
        target_voice,
        target_measure,
        target,
        filled,
        transform,
    } = im
    else {
        return Vec::new();
    };

    let idx = filled.len();
    let c_this = c.child(idx).expect("note cursor");
    let note = target.get(idx).expect("target note");

    let fill = |n: Note, _p: Pitch| -> Measure {
        let pitch = n.pitch;
        let mut new_filled = filled.to_vec();
        new_filled.push(n.clone());
        let notes: Vec<Note> = target
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i < new_filled.len() {
                    new_filled[i].clone()
                } else {
                    Note::new(t.duration, None, None)
                }
            })
            .collect();
        Measure {
            notes: Arc::from(notes),
            duration: measure.duration,
            kind: MeasureKind::Imitation(ImitationMeasure::Filled {
                ctx: ctx.clone(),
                melodic_context: update_melodic_context(*melodic_context, pitch),
                target_voice: *target_voice,
                target_measure: *target_measure,
                target: target.clone(),
                filled: Arc::from(new_filled),
                transform: transform.clone(),
            }),
        }
    };

    if note.pitch.is_none() {
        return vec![Step {
            measure: fill(note.clone(), Pitch::parse("c0").unwrap()),
            advanced: note.duration,
            cost: 0.0,
            debug: "imitate_blank".to_string(),
        }];
    }

    let pitches = (transform)(note.pitch.expect("non-null pitch"));

    if note.non_harmonic == Some(NonHarmonicType::Suspension) {
        let mut rules = vec![fixed(pitches)];
        if let Some(susp) = ctx.non_harmonic_tone_rules.get(&NonHarmonicType::Suspension) {
            rules.extend(susp.iter().cloned());
        }
        return ctx.fill_in(&rules, s, c_this, Some(NonHarmonicType::Suspension), &fill, 0.0);
    }

    let mut next = Vec::new();
    if !filled.is_empty() {
        for (t, r) in &ctx.non_harmonic_tone_rules {
            let mut rules = vec![fixed(pitches.clone())];
            rules.extend(r.iter().cloned());
            next.extend(ctx.fill_in(&rules, s, c_this, Some(*t), &fill, 0.0));
        }
    }
    let mut rules = vec![fixed(pitches)];
    rules.extend(ctx.harmonic_tone_rules.iter().cloned());
    next.extend(ctx.fill_in(&rules, s, c_this, None, &fill, 0.0));

    next
}

#[must_use]
pub fn define_imitation(
    m: MelodicSettings,
    target_voice: usize,
    delay: i64,
    transform: Arc<dyn Fn(Pitch) -> Vec<Pitch>>,
) -> VoiceConstructor {
    VoiceConstructor {
        melody_settings: m,
        kind: VoiceKind::Imitation {
            target_voice,
            delay,
            transform,
        },
    }
}
