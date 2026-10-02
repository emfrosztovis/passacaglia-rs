use passacaglia_common::rational;
use passacaglia_core::std_hept::{Interval, Pitch};

use crate::context::{Candidates, CounterpointContext};
use crate::rules::utils::{note_pitch, nth_prev_pitch};
use crate::score::Score;
use crate::voice::{NonHarmonicType, NoteCursor};

#[must_use]
pub fn enforce_melody_intervals<'a>(
    ctx: &CounterpointContext,
    _s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let mut c = c.expect("candidates initialized");
    if ty == Some(NonHarmonicType::Suspension) {
        return c;
    }
    if let Some(prev) = nth_prev_pitch(cur, 1) {
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
                .map(|(x, cost)| (prev.add(&x.with_sign(x.sign * sign)), *cost)),
        );
        c.intersect_with(&nexts, |a, b| a + b);
    }
    c
}

#[must_use]
pub fn enforce_stepwise_around_short_notes<'a>(
    _ctx: &CounterpointContext,
    _s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let mut c = c.expect("candidates initialized");
    if let Some((p1, prev)) = note_pitch(cur.prev_global())
        && (cur.span() < rational(1) || p1.span() < rational(1))
    {
        c.filter(|p, _| prev.steps_to(p).unsigned_abs() == 1);
    }
    c
}

#[must_use]
pub fn avoid_repeat2<'a>(
    _ctx: &CounterpointContext,
    _s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let mut c = c.expect("candidates initialized");
    if let Some(prev) = nth_prev_pitch(cur, 1)
        && let Some(prev2) = nth_prev_pitch(cur, 2)
        && let Some(prev3) = nth_prev_pitch(cur, 3)
        && prev3 == prev
    {
        c.filter(|x, _| *x != prev2);
    }
    c
}
