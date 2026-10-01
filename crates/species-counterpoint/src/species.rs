use std::sync::Arc;

use passacaglia_common::{rational, rational_value, Rational};
use passacaglia_core::std_hept::Pitch;

use crate::basic::{
    empty_melodic_context, update_melodic_context, MelodicContext, MelodicSettings, NewMeasure, Step,
    VoiceConstructor,
};
use crate::context::CounterpointContext;
use crate::score::Score;
use crate::voice::{
    Measure, MeasureCursor, MeasureKind, NonHarmonicType, Note, VoiceKind,
};

/// A note schema: either a harmonic/tone slot or a skipped slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NoteSchema {
    Skip { duration: Rational },
    Tone {
        harmonic: bool,
        types: Option<Vec<NonHarmonicType>>,
        duration: Rational,
    },
}

impl NoteSchema {
    #[must_use]
    pub fn duration(&self) -> Rational {
        match self {
            NoteSchema::Skip { duration } | NoteSchema::Tone { duration, .. } => *duration,
        }
    }
}

/// A species measure schema: a name, a note generator, and an optional
/// condition.
pub struct MeasureSchema {
    pub name: String,
    pub notes: Box<dyn Fn(Rational) -> Vec<NoteSchema>>,
    pub condition: Option<Box<dyn for<'a> Fn(MeasureCursor<'a>, &'a Score) -> bool>>,
    pub cost: f64,
}

/// A solver-aware species measure (stored inside a [`Measure`]).
#[derive(Clone)]
pub struct SpeciesMeasure {
    pub ctx: Arc<CounterpointContext>,
    pub melodic_context: MelodicContext,
    pub name: String,
    pub note_schema: Arc<[NoteSchema]>,
}

/// A transient measure that expands into concrete species measures.
#[derive(Clone)]
pub struct FakeMeasure {
    pub ctx: Arc<CounterpointContext>,
    pub melodic_context: MelodicContext,
    pub candidates: Arc<[FakeCandidate]>,
    pub p0: Pitch,
}

#[derive(Debug, Clone)]
pub struct FakeCandidate {
    pub name: String,
    pub note_schema: Arc<[NoteSchema]>,
    pub cost: f64,
}

#[must_use]
pub(crate) fn make_species_measure(
    ctx: Arc<CounterpointContext>,
    mc: MelodicContext,
    name: String,
    note_schema: Arc<[NoteSchema]>,
    mut notes: Vec<Note>,
) -> Measure {
    while notes.len() < note_schema.len() {
        let d = note_schema[notes.len()].duration();
        notes.push(Note::new(d, None, None));
    }
    Measure {
        notes: Arc::from(notes),
        duration: ctx.parameters.measure_length,
        kind: MeasureKind::Species(SpeciesMeasure {
            ctx,
            melodic_context: mc,
            name,
            note_schema,
        }),
    }
}

pub(crate) fn species_get_next_steps<'a>(
    measure: &Measure,
    sm: &SpeciesMeasure,
    s: &'a Score,
    c: MeasureCursor<'a>,
) -> Vec<Step> {
    let idx = (0..measure.notes.len())
        .find(|&i| {
            measure.notes[i].pitch.is_none()
                && matches!(sm.note_schema.get(i), Some(NoteSchema::Tone { .. }))
        })
        .expect("species measure has a writable tone");
    let ci = c.child(idx).expect("child cursor");
    let NoteSchema::Tone { harmonic, types, .. } = &sm.note_schema[idx] else {
        unreachable!("writable position is a tone schema")
    };

    let create = |note: Note, _p: Pitch| -> Measure {
        let pitch = note.pitch;
        let mut notes = measure.notes.to_vec();
        notes[idx] = note;
        Measure {
            notes: Arc::from(notes),
            duration: measure.duration,
            kind: MeasureKind::Species(SpeciesMeasure {
                ctx: sm.ctx.clone(),
                melodic_context: update_melodic_context(sm.melodic_context, pitch),
                name: sm.name.clone(),
                note_schema: sm.note_schema.clone(),
            }),
        }
    };

    let mut next = Vec::new();
    if let Some(types) = types {
        next.extend(sm.ctx.fill_non_harmonic_tone(types, s, ci, &create, 0.0));
    }
    if *harmonic {
        next.extend(sm.ctx.fill_harmonic_tone(s, ci, &create, 0.0));
    }
    next
}

pub(crate) fn fake_get_next_steps(fm: &FakeMeasure) -> Vec<Step> {
    fm.candidates
        .iter()
        .map(|fc| {
            let first = Note::new(fc.note_schema[0].duration(), Some(fm.p0), None);
            let measure = make_species_measure(
                fm.ctx.clone(),
                fm.melodic_context,
                fc.name.clone(),
                fc.note_schema.clone(),
                vec![first],
            );
            Step {
                measure,
                advanced: rational(1),
                cost: fc.cost,
                debug: "from_fake".to_string(),
            }
        })
        .collect()
}

pub(crate) fn species_make_new_measure<'a>(
    cp: &crate::voice::CounterpointVoice,
    schemas: &[MeasureSchema],
    score: &'a Score,
    c: MeasureCursor<'a>,
) -> Vec<NewMeasure> {
    let mc = c
        .prev_global()
        .map_or_else(empty_melodic_context, |p| p.melodic_context());

    let available: Vec<(&MeasureSchema, Vec<NoteSchema>)> = schemas
        .iter()
        .filter(|s| s.condition.as_ref().is_none_or(|cond| cond(c, score)))
        .map(|s| (s, (s.notes)(cp.ctx.parameters.measure_length)))
        .collect();

    let first_is_harmonic: Vec<(&MeasureSchema, &[NoteSchema])> = available
        .iter()
        .filter(|(_, n)| matches!(n.first(), Some(NoteSchema::Tone { harmonic: true, .. })))
        .map(|(s, n)| (*s, n.as_slice()))
        .collect();
    let first_is_not_harmonic: Vec<(&MeasureSchema, &[NoteSchema])> = available
        .iter()
        .filter(|(_, n)| !matches!(n.first(), Some(NoteSchema::Tone { harmonic: true, .. })))
        .map(|(s, n)| (*s, n.as_slice()))
        .collect();

    let mut results = Vec::new();
    for (s, n) in &first_is_not_harmonic {
        results.push(NewMeasure {
            measure: make_species_measure(
                cp.ctx.clone(),
                mc,
                s.name.clone(),
                Arc::from(n.to_vec()),
                vec![],
            ),
            cost: s.cost,
        });
    }

    if !first_is_harmonic.is_empty() {
        let fake_candidates: Arc<[FakeCandidate]> = Arc::from(
            first_is_harmonic
                .iter()
                .map(|(s, n)| FakeCandidate {
                    name: s.name.clone(),
                    note_schema: Arc::from(n.to_vec()),
                    cost: s.cost,
                })
                .collect::<Vec<_>>(),
        );
        let voice = c.container();
        let fake_cursor = voice.note_at(c.global_time()).expect("fake cursor exists");
        let create = move |_note: Note, p: Pitch| -> Measure {
            Measure {
                notes: Arc::from(vec![Note::new(
                    cp.ctx.parameters.measure_length,
                    Some(p),
                    None,
                )]),
                duration: cp.ctx.parameters.measure_length,
                kind: MeasureKind::Fake(FakeMeasure {
                    ctx: cp.ctx.clone(),
                    melodic_context: mc,
                    candidates: fake_candidates.clone(),
                    p0: p,
                }),
            }
        };
        results.extend(
            cp.ctx
                .fill_harmonic_tone(score, fake_cursor, &create, 0.0)
                .into_iter()
                .map(|step| NewMeasure {
                    measure: step.measure,
                    cost: step.cost,
                }),
        );
    }

    results
}

fn first(c: MeasureCursor<'_>, _s: &Score) -> bool {
    c.index() == 0
}

fn later(c: MeasureCursor<'_>, _s: &Score) -> bool {
    c.index() > 0
}

fn hdiff(c: MeasureCursor<'_>, name: &str) -> bool {
    c.prev_global()
        .is_none_or(|p| p.schema_name() != Some(name))
}

fn vdiff(c: MeasureCursor<'_>, score: &Score, name: &str) -> bool {
    if !hdiff(c, name) {
        return false;
    }
    let mut total = 0usize;
    let mut same = 0usize;
    for v in score.voices.iter() {
        if let Some(m) = v.measures().get(c.index()) {
            if let Some(sn) = m.schema_name() {
                if sn == name {
                    same += 1;
                }
                total += 1;
            }
        }
    }
    if total > 0 && same > total - 1 {
        return false;
    }
    true
}

fn repeat_notes(n: f64, f: impl Fn() -> NoteSchema) -> Vec<NoteSchema> {
    let count = n.ceil().max(0.0) as usize;
    (0..count).map(|_| f()).collect()
}

fn define_species(m: MelodicSettings, schema: Vec<MeasureSchema>) -> VoiceConstructor {
    VoiceConstructor {
        melody_settings: m,
        kind: VoiceKind::Species {
            schemas: Arc::from(schema),
        },
    }
}

fn passing_neighbor() -> Vec<NonHarmonicType> {
    vec![NonHarmonicType::PassingTone, NonHarmonicType::Neighbor]
}

#[must_use]
pub fn species1() -> VoiceConstructor {
    let schema = vec![MeasureSchema {
        name: "sp1".to_string(),
        notes: Box::new(|ml| {
            vec![NoteSchema::Tone {
                harmonic: true,
                types: None,
                duration: ml,
            }]
        }),
        condition: None,
        cost: 0.0,
    }];
    define_species(
        MelodicSettings {
            forbid_repeated_notes: false,
            max_consecutive_leaps: 2,
            max_ignorable_3rd_leaps: 2,
            max_unidirectional_consecutive_leaps: 1,
            max_unidirectional_ignorable_3rd_leaps: 1,
        },
        schema,
    )
}

#[must_use]
pub fn species2() -> VoiceConstructor {
    let schema = vec![
        MeasureSchema {
            name: "sp2.0".to_string(),
            condition: Some(Box::new(first)),
            notes: Box::new(|ml| {
                vec![
                    NoteSchema::Skip {
                        duration: ml / rational(2),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: ml / rational(2),
                    },
                ]
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp2.1".to_string(),
            condition: Some(Box::new(later)),
            notes: Box::new(|ml| {
                vec![
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: ml / rational(2),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: Some(passing_neighbor()),
                        duration: ml / rational(2),
                    },
                ]
            }),
            cost: 0.0,
        },
    ];
    define_species(
        MelodicSettings {
            forbid_repeated_notes: true,
            max_consecutive_leaps: 2,
            max_ignorable_3rd_leaps: 1,
            max_unidirectional_consecutive_leaps: 1,
            max_unidirectional_ignorable_3rd_leaps: 0,
        },
        schema,
    )
}

#[must_use]
pub fn species3() -> VoiceConstructor {
    let schema = vec![
        MeasureSchema {
            name: "sp3.0".to_string(),
            condition: Some(Box::new(first)),
            notes: Box::new(|ml| {
                let mut out = vec![
                    NoteSchema::Skip {
                        duration: rational(1),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: rational(1),
                    },
                ];
                out.extend(repeat_notes(rational_value(ml) - 2.0, || NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }));
                out
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp3.1".to_string(),
            condition: Some(Box::new(later)),
            notes: Box::new(|ml| {
                let mut out = vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: rational(1),
                }];
                out.extend(repeat_notes(rational_value(ml) - 1.0, || NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }));
                out
            }),
            cost: 0.0,
        },
    ];
    define_species(
        MelodicSettings {
            forbid_repeated_notes: true,
            max_consecutive_leaps: 2,
            max_ignorable_3rd_leaps: 1,
            max_unidirectional_consecutive_leaps: 1,
            max_unidirectional_ignorable_3rd_leaps: 0,
        },
        schema,
    )
}

#[must_use]
pub fn species4() -> VoiceConstructor {
    let schema = vec![
        MeasureSchema {
            name: "sp4.0".to_string(),
            condition: Some(Box::new(first)),
            notes: Box::new(|ml| {
                vec![
                    NoteSchema::Skip {
                        duration: ml / rational(2),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: ml / rational(2),
                    },
                ]
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp4.1".to_string(),
            condition: Some(Box::new(later)),
            notes: Box::new(|ml| {
                vec![
                    NoteSchema::Tone {
                        harmonic: false,
                        types: Some(vec![NonHarmonicType::Suspension]),
                        duration: ml / rational(2),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: ml / rational(2),
                    },
                ]
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp4.2".to_string(),
            condition: Some(Box::new(later)),
            notes: Box::new(|ml| {
                vec![
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: ml / rational(2),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: Some(passing_neighbor()),
                        duration: ml / rational(2),
                    },
                ]
            }),
            cost: 500.0,
        },
    ];
    define_species(
        MelodicSettings {
            forbid_repeated_notes: true,
            max_consecutive_leaps: 2,
            max_ignorable_3rd_leaps: 1,
            max_unidirectional_consecutive_leaps: 1,
            max_unidirectional_ignorable_3rd_leaps: 0,
        },
        schema,
    )
}

#[must_use]
pub fn species5() -> VoiceConstructor {
    let schema = vec![
        MeasureSchema {
            name: "sp5.1".to_string(),
            condition: Some(Box::new(|c, s| {
                later(c, s) && s.voices.len() > 2 && vdiff(c, s, "sp5.1")
            })),
            notes: Box::new(|ml| {
                vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: ml,
                }]
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.2.0".to_string(),
            condition: Some(Box::new(|c, s| first(c, s) && vdiff(c, s, "sp5.2.0"))),
            notes: Box::new(|ml| {
                vec![
                    NoteSchema::Skip {
                        duration: ml / rational(2),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: ml / rational(2),
                    },
                ]
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.2.1".to_string(),
            condition: Some(Box::new(|c, s| later(c, s) && vdiff(c, s, "sp5.2.1"))),
            notes: Box::new(|ml| {
                vec![
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: ml / rational(2),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: Some(passing_neighbor()),
                        duration: ml / rational(2),
                    },
                ]
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.3.0".to_string(),
            condition: Some(Box::new(|c, s| first(c, s) && vdiff(c, s, "sp5.3.0"))),
            notes: Box::new(|ml| {
                let mut out = vec![
                    NoteSchema::Skip {
                        duration: rational(1),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: rational(1),
                    },
                ];
                out.extend(repeat_notes(rational_value(ml) - 2.0, || NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }));
                out
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.3.1".to_string(),
            condition: Some(Box::new(|c, s| later(c, s) && vdiff(c, s, "sp5.3.1"))),
            notes: Box::new(|ml| {
                let mut out = vec![NoteSchema::Tone {
                    harmonic: true,
                    types: Some(vec![NonHarmonicType::Suspension]),
                    duration: rational(1),
                }];
                out.extend(repeat_notes(rational_value(ml) - 1.0, || NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }));
                out
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.4.1".to_string(),
            condition: Some(Box::new(|c, s| later(c, s) && vdiff(c, s, "sp5.4.1"))),
            notes: Box::new(|ml| {
                vec![
                    NoteSchema::Tone {
                        harmonic: false,
                        types: Some(vec![NonHarmonicType::Suspension]),
                        duration: ml / rational(2),
                    },
                    NoteSchema::Tone {
                        harmonic: true,
                        types: None,
                        duration: ml / rational(2),
                    },
                ]
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.5.1".to_string(),
            condition: Some(Box::new(|c, s| later(c, s) && vdiff(c, s, "sp5.5.1"))),
            notes: Box::new(|ml| {
                let mut out = vec![NoteSchema::Tone {
                    harmonic: true,
                    types: Some(vec![NonHarmonicType::Suspension]),
                    duration: ml / rational(2),
                }];
                out.extend(repeat_notes(rational_value(ml) / 2.0, || NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }));
                out
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.5.2".to_string(),
            condition: Some(Box::new(|c, s| later(c, s) && vdiff(c, s, "sp5.5.2"))),
            notes: Box::new(|ml| {
                let mut out = vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: rational(1),
                }];
                out.extend(repeat_notes(rational_value(ml) / 2.0 - 1.0, || NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }));
                out.push(NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: ml / rational(2),
                });
                out
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.5.4".to_string(),
            condition: Some(Box::new(|c, s| later(c, s) && vdiff(c, s, "sp5.5.4"))),
            notes: Box::new(|ml| {
                let v = rational_value(ml);
                let d1 = (v * 3.0 / 4.0).floor();
                let n = v - d1;
                let mut out = vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: rational(d1 as i64),
                }];
                out.extend(repeat_notes(n, || NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }));
                out
            }),
            cost: 0.0,
        },
    ];
    define_species(
        MelodicSettings {
            forbid_repeated_notes: true,
            max_consecutive_leaps: 3,
            max_ignorable_3rd_leaps: 1,
            max_unidirectional_consecutive_leaps: 1,
            max_unidirectional_ignorable_3rd_leaps: 0,
        },
        schema,
    )
}
