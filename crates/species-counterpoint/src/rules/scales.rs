use std::collections::HashMap;
use std::rc::Rc;

use passacaglia_core::std_hept::scales;
use passacaglia_core::std_hept::{Interval, Pitch};
use passacaglia_macros::std_hept_interval as interval;

use crate::context::{CandidateRule, Candidates, CounterpointContext};
use crate::rules::utils::{note_pitch, prev_different, sign_of};
use crate::score::Score;
use crate::voice::{NonHarmonicType, NoteCursor};

pub use crate::context::parse_preferred;

#[must_use]
pub fn enforce_scale_tones<'a>(
    _ctx: &CounterpointContext,
    s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let voice = cur.parent().container();
    let Some((lo, hi)) = voice.ranges() else {
        return Candidates::new();
    };
    let scale_tones = Candidates::from_pairs(
        s.harmony
            .scale
            .get_degrees_in_range(&lo, &hi)
            .iter()
            .map(|d| (d.to_pitch(), 0.0)),
    );
    match c {
        None => scale_tones,
        Some(mut c) => {
            c.intersect(&scale_tones);
            c
        }
    }
}

/// A preferred-interval transition for a scale degree.
pub struct DegreeTransition {
    pub next: HashMap<Interval, f64>,
    pub forbid_other: bool,
}

/// Directional degree-transition matrix keyed by scale degree index.
pub struct DegreeMatrix {
    pub upward: HashMap<usize, DegreeTransition>,
    pub downward: HashMap<usize, DegreeTransition>,
}

#[must_use]
pub fn degree_matrix_preset_major() -> DegreeMatrix {
    let upward = HashMap::from([(
        6,
        DegreeTransition {
            next: parse_preferred(&[("m2", -50.0)]),
            forbid_other: false,
        },
    )]);
    let downward = HashMap::from([
        (
            6,
            DegreeTransition {
                next: parse_preferred(&[("m2", -30.0)]),
                forbid_other: false,
            },
        ),
        (
            5,
            DegreeTransition {
                next: parse_preferred(&[("-M2", -20.0)]),
                forbid_other: false,
            },
        ),
        (
            3,
            DegreeTransition {
                next: parse_preferred(&[("-m2", -10.0)]),
                forbid_other: false,
            },
        ),
    ]);
    DegreeMatrix { upward, downward }
}

#[must_use]
pub fn enforce_directional_degree_matrix(m: DegreeMatrix) -> CandidateRule {
    Rc::new(move |_ctx, s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if let Some((prev_cur, prev)) = note_pitch(cur.prev_global())
            && let Some((_, prev2)) = note_pitch(prev_cur.prev_global())
            && let sign = sign_of(prev2.distance_to(&prev))
            && sign != 0
            && let map = if sign > 0 { &m.upward } else { &m.downward }

            && let Some(deg) = s.harmony.scale.get_exact_degree(&prev, false)
            && let Some(pref) = map.get(&deg.index)
        {
            let next_map: HashMap<Pitch, f64> = pref
                .next
                .iter()
                .map(|(x, cost)| (prev.add(x), *cost))
                .collect();

            for (p, cost) in &next_map {
                if let Some(old_cost) = c.get(p) {
                    if *cost == f64::INFINITY {
                        c.remove(p);
                    } else {
                        c.set(*p, old_cost + cost);
                    }
                }
            }
            if pref.forbid_other {
                c.filter(|p, _| next_map.contains_key(p));
            }
        }
        c
    })
}

#[must_use]
pub fn enforce_minor(root: Pitch) -> CandidateRule {
    Rc::new(move |_ctx, _s, cur, c, ty| {
        let voice = cur.parent().container();
        let scale = scales::complete_minor(root);

        let mut c = c.unwrap_or_else(|| {
            let Some((lo, hi)) = voice.ranges() else {
                return Candidates::new();
            };
            Candidates::from_pairs(
                scale
                    .get_degrees_in_range(&lo, &hi)
                    .iter()
                    .map(|d| (d.to_pitch(), 0.0)),
            )
        });

        if ty == Some(NonHarmonicType::Suspension) {
            return c;
        }

        if let Some((n1, p1)) = note_pitch(cur.prev_global())
            && let Some(d1) = scale.get_exact_degree(&p1, false)
            && let Some((_, p0)) = note_pitch(prev_different(n1))
            && scale.get_exact_degree(&p0, false).is_some()
        {
            if d1.index == 6 {
                if p0.interval_to(&p1) != interval!("M2") {
                    return Candidates::new();
                }
                let target = p1.add(&interval!("M2"));
                c.filter(|x, _| *x == target);
                return c;
            }
            if d1.index == 7 {
                if p0.interval_to(&p1) != interval!("-M2") {
                    return Candidates::new();
                }
                let target = p1.add(&interval!("-M2"));
                c.filter(|x, _| *x == target);
                return c;
            }
            if d1.index == 8 {
                let target = p1.add(&interval!("m2"));
                c.filter(|x, _| *x == target);
            }
        }
        c
    })
}
