use num_traits::Signed;
use passacaglia_common::rational;
use passacaglia_core::std_hept::Pitch;

use crate::context::{Candidates, CounterpointContext};
use crate::rules::utils::note_pitch;
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
    if let Some((p1, _)) = note_pitch(cur.prev_global())
        && p1.non_harmonic == Some(NonHarmonicType::Neighbor)
        && let Some((_, prev2)) = note_pitch(p1.prev_global())
    {
        c.filter(|p, _| *p == prev2);
    }
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
    if let Some((p1, prev)) = note_pitch(cur.prev_global())
        && p1.non_harmonic.is_none()
    {
        c.filter(|p, _| {
            let dist = prev.distance_to(p).abs();
            prev.steps_to(p).unsigned_abs() <= 1 && dist > rational(0) && dist <= rational(2)
        });
    }
    c
}
