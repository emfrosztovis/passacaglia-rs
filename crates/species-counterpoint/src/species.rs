use std::rc::Rc;

use num_traits::Zero;
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

/// Returns the [`NoteSchema`]s that are possible for the note at the position
/// that follows the already-chosen schemas, in a measure of the given length.
///
/// The returned list is a set of alternatives: each one leads to a distinct
/// candidate measure. Returning an empty list means the measure is complete.
pub type NoteSchemaNext = dyn Fn(Rational, Rational, &[NoteSchema]) -> Vec<NoteSchema>;

/// A species measure schema: a name, a positional note generator, and an
/// optional condition.
pub struct MeasureSchema {
    pub name: String,
    pub next: Rc<NoteSchemaNext>,
    pub condition: Option<Box<dyn for<'a> Fn(MeasureCursor<'a>, &'a Score) -> bool>>,
    pub cost: f64,
}

/// A solver-aware species measure (stored inside a [`Measure`]).
///
/// `note_schema` holds the schemas chosen so far (the prefix). The note at the
/// next position is obtained by calling `next` with this prefix.
#[derive(Clone)]
pub struct SpeciesMeasure {
    pub ctx: Rc<CounterpointContext>,
    pub melodic_context: MelodicContext,
    pub name: String,
    pub note_schema: Rc<[NoteSchema]>,
    pub next: Rc<NoteSchemaNext>,
}

#[derive(Clone)]
pub struct FakeCandidate {
    pub name: String,
    pub note_schema: Rc<[NoteSchema]>,
    pub next: Rc<NoteSchemaNext>,
    pub cost: f64,
}

#[must_use]
pub(crate) fn make_species_measure(
    ctx: Rc<CounterpointContext>,
    mc: MelodicContext,
    name: String,
    next: Rc<NoteSchemaNext>,
    note_schema: Rc<[NoteSchema]>,
    mut notes: Vec<Note>,
) -> Measure {
    while notes.len() < note_schema.len() {
        let d = note_schema[notes.len()].duration();
        notes.push(Note::new(d, None, None));
    }
    let duration: Rational = note_schema.iter().map(NoteSchema::duration).sum();
    if duration < ctx.parameters.measure_length {
        notes.push(Note::new(
            ctx.parameters.measure_length - duration, None, None));
    }
    Measure {
        notes: Rc::from(notes),
        duration: ctx.parameters.measure_length,
        kind: MeasureKind::Species(SpeciesMeasure {
            ctx,
            melodic_context: mc,
            name,
            note_schema,
            next,
        }),
    }
}

pub(crate) fn species_get_next_steps<'a>(
    measure: &Measure,
    sm: &SpeciesMeasure,
    s: &'a Score,
    c: MeasureCursor<'a>,
) -> Vec<Step> {
    if let Some(note_idx) = sm.note_schema.iter()
        .zip(measure.notes.iter())
        .position(|(a, b)| matches!(a, NoteSchema::Tone { .. }) && b.pitch.is_none())
    {
        return fill_species_tone(measure, sm, note_idx, s, c);
    }

    // Every chosen position is complete. If the measure is not full yet, ask
    // the schema for the schemas that are possible at the next position.
    let ml = sm.ctx.parameters.measure_length;
    let total = given_total(&sm.note_schema);
    if total >= ml {
        return Vec::new();
    }

    (sm.next)(ml, total, &sm.note_schema)
        .into_iter()
        .filter(|schema| schema.duration() + total <= ml)
        .map(|option| {
            let mut schema = sm.note_schema.to_vec();
            let mut notes = measure.notes.to_vec();
            notes.pop().unwrap(); // remove the placeholder rest
            notes.push(Note::new(option.duration(), None, None));
            schema.push(option);
            
            Step {
                measure: make_species_measure(
                    sm.ctx.clone(), 
                    sm.melodic_context, 
                    sm.name.clone(), 
                    sm.next.clone(), 
                    Rc::from(schema), 
                    notes
                ),
                advanced: rational(0),
                cost: 0.0,
                debug: "extend_schema".to_string(),
                score: None,
            }
        })
        .collect()
}

fn fill_species_tone<'a>(
    measure: &Measure,
    sm: &SpeciesMeasure,
    note_idx: usize,
    s: &'a Score,
    c: MeasureCursor<'a>,
) -> Vec<Step> {
    let c_note = c.child(note_idx).expect("child cursor");
    let NoteSchema::Tone { harmonic, types, .. } = &sm.note_schema[note_idx] else {
        unreachable!("writable position is a tone schema")
    };

    let create = |note: Note, _p: Pitch| -> Measure {
        let pitch = note.pitch;
        let mut notes = measure.notes.to_vec();
        notes[note_idx] = note;
        Measure {
            notes: Rc::from(notes),
            duration: measure.duration,
            kind: MeasureKind::Species(SpeciesMeasure {
                ctx: sm.ctx.clone(),
                melodic_context: update_melodic_context(sm.melodic_context, pitch),
                name: sm.name.clone(),
                note_schema: sm.note_schema.clone(),
                next: sm.next.clone(),
            }),
        }
    };

    let mut next = Vec::new();
    let prev_note_non_harmonic = c_note.prev().is_none_or(|c| c.is_non_harmonic());
    if let Some(types) = types
        && (types.contains(&NonHarmonicType::Suspension) || !prev_note_non_harmonic)
    {
        next.extend(sm.ctx.fill_non_harmonic_tone(types, s, c_note, &create, 0.0));
    }
    if *harmonic {
        next.extend(sm.ctx.fill_harmonic_tone(s, c_note, &create, 0.0));
    }
    next
}

#[must_use]
pub(crate) fn given_total(notes: &[NoteSchema]) -> Rational {
    notes.iter().map(NoteSchema::duration).sum()
}

pub(crate) fn species_make_new_measure<'a>(
    cp: &crate::voice::CounterpointVoice,
    schemas: &[MeasureSchema],
    score: &'a Score,
    c: MeasureCursor<'a>,
) -> Vec<NewMeasure> {
    let ml = cp.ctx.parameters.measure_length;
    let mc = c
        .prev_global()
        .map_or_else(empty_melodic_context, |p| p.melodic_context());

    let mut results = Vec::new();
    let applicable = schemas.iter()
        .filter(|s| s.condition.as_ref().is_some_and(|cond| cond(c, score)));
    for s in applicable {
        for option in (s.next)(ml, Rational::ZERO, &[]) {
            results.push(NewMeasure {
                measure: make_species_measure(
                    cp.ctx.clone(),
                    mc,
                    s.name.clone(),
                    s.next.clone(),
                    Rc::from(vec![option]),
                    vec![],
                ),
                cost: s.cost,
            });
        }
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
    for v in &score.voices {
        if let Some(m) = v.measures().get(c.index())
            && let Some(sn) = m.schema_name()
        {
            if sn == name {
                same += 1;
            }
            total += 1;
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

#[must_use]
fn total_duration(chosen: &[NoteSchema]) -> Rational {
    chosen.iter().fold(rational(0), |acc, n| acc + n.duration())
}

/// Adapts a `measure length -> full note schema` function into a positional
/// [`NoteSchemaNext`] by looking up the note that follows the chosen prefix.
fn fixed_schema(notes: impl Fn(Rational) -> Vec<NoteSchema> + 'static) -> Rc<NoteSchemaNext> {
    Rc::new(move |ml, _t, chosen| {
        notes(ml)
            .get(chosen.len())
            .cloned()
            .into_iter()
            .collect()
    })
}

fn define_species(name: &str, m: MelodicSettings, schema: Vec<MeasureSchema>) -> VoiceConstructor {
    VoiceConstructor {
        melody_settings: m,
        kind: VoiceKind::Species {
            name: name.to_string(),
            schemas: Rc::from(schema),
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
        next: Rc::new(|ml, _t, chosen| {
            if chosen.is_empty() {
                vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: ml,
                }]
            } else {
                Vec::new()
            }
        }),
        condition: None,
        cost: 0.0,
    }];
    define_species(
        "sp1",
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
            next: Rc::new(|ml, _t, chosen| match chosen {
                [] => vec![NoteSchema::Skip {
                    duration: ml / rational(2),
                }],
                [_] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: ml / rational(2),
                }],
                _ => Vec::new(),
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp2.1".to_string(),
            condition: Some(Box::new(later)),
            next: Rc::new(|ml, _t, chosen| match chosen {
                [] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: ml / rational(2),
                }],
                [_] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: ml / rational(2),
                }],
                _ => Vec::new(),
            }),
            cost: 0.0,
        },
    ];
    define_species(
        "sp2",
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
            next: Rc::new(|ml, _t, chosen| match chosen {
                [] => vec![NoteSchema::Skip {
                    duration: rational(1),
                }],
                [_] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: rational(1),
                }],
                _ if total_duration(chosen) < ml => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }],
                _ => Vec::new(),
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp3.1".to_string(),
            condition: Some(Box::new(later)),
            next: Rc::new(|ml, _t, chosen| match chosen {
                [] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: rational(1),
                }],
                _ if total_duration(chosen) < ml => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                }],
                _ => Vec::new(),
            }),
            cost: 0.0,
        },
    ];
    define_species(
        "sp3",
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
            next: Rc::new(|ml, _t, chosen| match chosen {
                [] => vec![NoteSchema::Skip {
                    duration: ml / rational(2),
                }],
                [_] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: ml / rational(2),
                }],
                _ => Vec::new(),
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp4.1".to_string(),
            condition: Some(Box::new(later)),
            next: Rc::new(|ml, _t, chosen| match chosen {
                [] => vec![NoteSchema::Tone {
                    harmonic: false,
                    types: Some(vec![NonHarmonicType::Suspension]),
                    duration: ml / rational(2),
                }],
                [_] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: ml / rational(2),
                }],
                _ => Vec::new(),
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp4.2".to_string(),
            condition: Some(Box::new(later)),
            next: Rc::new(|ml, _t, chosen| match chosen {
                [] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: None,
                    duration: ml / rational(2),
                }],
                [_] => vec![NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: ml / rational(2),
                }],
                _ => Vec::new(),
            }),
            cost: 500.0,
        },
    ];
    define_species(
        "sp4",
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
            name: "sp5".to_string(),
            condition: Some(Box::new(later)),
            next: Rc::new(|ml, t, _chosen| {
                let mut result: Vec<NoteSchema> = vec![];
                if t.is_zero() {
                    return vec![
                        NoteSchema::Tone {
                            harmonic: true,
                            types: None,
                            duration: rational(1),
                        },
                        NoteSchema::Tone {
                            harmonic: true,
                            types: None,
                            duration: ml / rational(2),
                        },
                        NoteSchema::Tone {
                            harmonic: false,
                            types: Some(vec![NonHarmonicType::Suspension]),
                            duration: rational(1),
                        },
                        NoteSchema::Tone {
                            harmonic: false,
                            types: Some(vec![NonHarmonicType::Suspension]),
                            duration: ml / rational(2),
                        },
                        NoteSchema::Tone {
                            harmonic: true,
                            types: None,
                            duration: ml,
                        },
                    ];
                }
                result.push(NoteSchema::Tone {
                    harmonic: true,
                    types: Some(passing_neighbor()),
                    duration: rational(1),
                });
                if t * 2 >= ml {
                    result.push(NoteSchema::Tone {
                        harmonic: true,
                        types: Some(passing_neighbor()),
                        duration: ml / rational(2),
                    });
                }
                result
            }),
            cost: 0.0,
        },
        MeasureSchema {
            name: "sp5.2.0".to_string(),
            condition: Some(Box::new(|c, s| first(c, s) && vdiff(c, s, "sp5.2.0"))),
            next: fixed_schema(|ml| {
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
            name: "sp5.3.0".to_string(),
            condition: Some(Box::new(|c, s| first(c, s) && vdiff(c, s, "sp5.3.0"))),
            next: fixed_schema(|ml| {
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
    ];

    define_species(
        "sp5",
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
