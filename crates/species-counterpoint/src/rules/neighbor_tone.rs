use passacaglia_common::rational_value;
use passacaglia_core::std_hept::Pitch;

use crate::context::{Candidates, CounterpointContext};
use crate::score::Score;
use crate::voice::{NonHarmonicType, NoteCursor};

#[must_use]
pub fn enforce_neighbor_tones<'a>(
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
    let Some(_prev) = p1.pitch else {
        return c;
    };
    if p1.non_harmonic != Some(NonHarmonicType::Neighbor) {
        return c;
    }
    let Some(p2) = p1.prev_global() else {
        return c;
    };
    let Some(prev2) = p2.pitch else {
        return c;
    };
    c.filter(|p, _| *p == prev2);
    c
}

#[must_use]
pub fn make_neighbor_tone<'a>(
    _ctx: &CounterpointContext,
    _s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let mut c = c.expect("candidates initialized");
    let Some(p1) = cur.prev_global() else {
        return Candidates::new();
    };
    let Some(prev) = p1.pitch else {
        return Candidates::new();
    };
    if p1.non_harmonic.is_some() {
        return Candidates::new();
    }
    c.filter(|p, _| {
        let dist = rational_value(prev.distance_to(p)).abs();
        prev.steps_to(p).unsigned_abs() <= 1 && dist > 0.0 && dist <= 2.0
    });
    c
}
