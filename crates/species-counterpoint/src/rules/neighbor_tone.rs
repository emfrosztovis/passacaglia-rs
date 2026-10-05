use std::rc::Rc;

use num_traits::{Signed, Zero};
use passacaglia_common::rational;

use crate::Candidates;
use crate::context::CandidateRule;
use crate::rules::utils::note_pitch;
use crate::voice::NonHarmonicType;

/// Enforce that neighbor tones resolve to the pitch before it.
#[must_use]
pub fn enforce_neighbor_tones() -> CandidateRule {
    Rc::new(move |_ctx, _s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if let Some((p1, _)) = note_pitch(cur.prev_global())
            && p1.non_harmonic == Some(NonHarmonicType::Neighbor)
            && let Some((_, prev2)) = note_pitch(p1.prev_global())
        {
            c.filter(|p, _| *p == prev2);
        }
        c
    })
}

#[must_use]
pub fn make_neighbor_tone() -> CandidateRule {
    Rc::new(move |ctx, _s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if cur.time().is_zero() || cur.time() * rational(2) == ctx.parameters.measure_length {
            // can't occur on downbeats
            return Candidates::new();
        }
        if let Some((p1, prev)) = note_pitch(cur.prev_global())
            && p1.non_harmonic.is_none()
        {
            c.filter(|p, _| {
                let dist = prev.distance_to(p).abs();
                prev.steps_to(p).unsigned_abs() <= 1 && dist > rational(0) && dist <= rational(2)
            });
        }
        c
    })
}
