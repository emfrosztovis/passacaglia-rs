use passacaglia_common::rational_value;
use passacaglia_core::std_hept::{Interval, Pitch};

use crate::context::{Candidates, CounterpointContext};
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
    let Some(p1) = cur.prev_global() else {
        return c;
    };
    let Some(prev) = p1.pitch else {
        return c;
    };
    let p2 = p1.prev_global();
    let prev2 = p2.and_then(|p| p.pitch);

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
    let Some(p1) = cur.prev_global() else {
        return c;
    };
    let Some(prev) = p1.pitch else {
        return c;
    };
    if rational_value(cur.span()) >= 1.0 && rational_value(p1.span()) >= 1.0 {
        return c;
    }
    c.filter(|p, _| prev.steps_to(p).unsigned_abs() == 1);
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
    let Some(p1) = cur.prev_global() else {
        return c;
    };
    let Some(prev) = p1.pitch else {
        return c;
    };
    let Some(p2) = p1.prev_global() else {
        return c;
    };
    let Some(prev2) = p2.pitch else {
        return c;
    };
    let Some(p3) = p2.prev_global() else {
        return c;
    };
    let Some(prev3) = p3.pitch else {
        return c;
    };
    if prev3 == prev {
        c.filter(|x, _| *x != prev2);
    }
    c
}
