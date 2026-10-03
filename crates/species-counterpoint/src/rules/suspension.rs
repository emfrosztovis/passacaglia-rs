use std::rc::Rc;

use passacaglia_common::rational;

use crate::context::{CandidateRule, Candidates};
use crate::rules::utils::{is_consonance, is_leading_tone, note_pitch};
use crate::voice::NonHarmonicType;

/// Enforce that suspensions are resolved correctly.
#[must_use]
pub fn enforce_suspension() -> CandidateRule {
    Rc::new(move |_ctx, s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if let Some((p1, prev)) = note_pitch(cur.prev_global())
            && p1.non_harmonic == Some(NonHarmonicType::Suspension)
        {
            let v = cur.parent().container();
            let mut is_chord_tone = true;
            for voice in s.voices.iter() {
                if voice.index() == v.index() {
                    continue;
                }
                let Some((_, pn)) = note_pitch(voice.note_at(p1.global_time())) else {
                    continue;
                };
                if !is_consonance(&pn.absolute_simple_interval_to(&prev), false) {
                    is_chord_tone = false;
                    break;
                }
            }

            c.filter_map(|p, val| {
                if prev.steps_to(p) == -1
                    || (is_leading_tone(&prev, &s.harmony.scale)
                        && prev.distance_to(p) == rational(1))
                {
                    Some(val)
                } else if is_chord_tone && prev.steps_to(p) == 1 {
                    Some(val + 100.0)
                } else {
                    None
                }
            });
        }
        c
    })
}

#[must_use]
pub fn make_suspension() -> CandidateRule {
    Rc::new(move |_ctx, _s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if cur.index() != 0 {
            return Candidates::new();
        }
        let Some((p1, prev)) = note_pitch(cur.prev_global()) else {
            return Candidates::new();
        };
        if p1.non_harmonic.is_some() || p1.span() < cur.span() {
            return Candidates::new();
        }
        c.filter_map(|p, _| if *p == prev { Some(0.0) } else { None });
        c
    })
}
