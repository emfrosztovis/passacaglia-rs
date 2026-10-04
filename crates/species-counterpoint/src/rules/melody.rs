use std::rc::Rc;

use passacaglia_common::{rational, rational_value};
use passacaglia_core::std_hept::Interval;

use crate::context::{CandidateRule, Candidates};
use crate::rules::utils::{note_pitch, nth_prev_pitch, prev_non_tied};
use crate::voice::NonHarmonicType;

/// Only allow melodic intervals specified in [`crate::context::CounterpointContext`] in the
/// melody.
#[must_use]
pub fn enforce_melody_intervals() -> CandidateRule {
    Rc::new(move |ctx, _s, cur, c, ty| {
        let mut c = c.expect("candidates initialized");
        if ty == Some(NonHarmonicType::Suspension) {
            return c;
        }
        if let Some((pc, prev)) = note_pitch(cur.prev_global()) {
            let prev2 = nth_prev_pitch(cur, 2);
            let v = cur.parent().container();
            let mut ints: Vec<(Interval, f64)> = ctx.melodic_intervals.iter().map(|(i, c)| (*i, *c)).collect();
            if v.melody_settings().is_some_and(|ms| ms.forbid_repeated_notes) {
                ints.retain(|(x, _)| *x.distance.numer() > 0);
            }

            let sign: i8 = if prev2.is_some_and(|p| p.ord() > prev.ord()) {
                -1
            } else {
                1
            };
            let nexts = Candidates::from_pairs(
                ints.iter()
                    .map(|(x, cost)| (
                        prev.add(&x.with_sign(x.sign * sign)), 
                        *cost / rational_value(pc.duration)
                    )),
            );
            c.intersect_with(&nexts, |a, b| a + b);
        }
        c
    })
}

/// Only allow stepwise motion around notes shorter than a quarter note.
#[must_use]
pub fn enforce_stepwise_around_short_notes() -> CandidateRule {
    Rc::new(move |_ctx, _s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if let Some((p1, prev)) = note_pitch(cur.prev_global())
            && (cur.span() < rational(1) || p1.span() < rational(1))
        {
            c.filter(|p, _| prev.steps_to(p).unsigned_abs() == 1);
        }
        c
    })
}

#[must_use]
pub fn avoid_repeat2() -> CandidateRule {
    Rc::new(move |_ctx, _s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if let Some(prev) = nth_prev_pitch(cur, 1)
            && let Some(prev2) = nth_prev_pitch(cur, 2)
            && let Some(prev3) = nth_prev_pitch(cur, 3)
            && prev3 == prev
        {
            c.filter(|x, _| *x != prev2);
        }
        c
    })
}
#[must_use]
pub fn avoid_stagnation() -> CandidateRule {
    Rc::new(move |_ctx, _s, cur, c, _ty| {
        let c = c.expect("candidates initialized");
        let Some((mut cur, p)) = note_pitch(cur.prev_global()) else {
            return c;
        };
        let mut lo = p;
        let mut hi = p;
        for i in 2..=14 {
            let Some((cur1, p1)) = note_pitch(prev_non_tied(cur)) else {
                return c;
            };
            if p1.ord() < lo.ord() { lo = p1; }
            if p1.ord() > hi.ord() { hi = p1; }
            cur = cur1;
        
            let int = lo.interval_to(&hi);
            if i == 5 && int.steps <= 1
                || i == 9 && int.steps <= 2
                || i == 14 && int.steps <= 3
            {
                return Candidates::new();
            }
        }
        c
    })
}
