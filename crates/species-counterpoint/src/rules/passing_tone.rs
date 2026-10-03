use std::rc::Rc;

use num_traits::Signed;
use passacaglia_common::rational;

use crate::context::CandidateRule;
use crate::rules::utils::note_pitch;
use crate::voice::NonHarmonicType;

/// Enforce that notes surrounding a passing tone are its neighbors in
/// ascending or descending order.
#[must_use]
pub fn enforce_passing_tones() -> CandidateRule {
    Rc::new(move |_ctx, _s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if let Some((p1, prev)) = note_pitch(cur.prev_global())
            && p1.non_harmonic == Some(NonHarmonicType::PassingTone)
            && let Some((_, prev2)) = note_pitch(p1.prev_global())
        {
            let o2 = prev2.ord();
            let o1 = prev.ord();
            c.filter(|p, _| {
                let op = p.ord();
                (o1 - op).signum() == (o2 - o1).signum()
                    && prev.steps_to(p).unsigned_abs() <= 1
                    && prev.distance_to(p).abs() > rational(0)
            });
        }
        c
    })
}

#[must_use]
pub fn make_passing_tone() -> CandidateRule {
    Rc::new(move |_ctx, _s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        if let Some((_, prev)) = note_pitch(cur.prev_global()) {
            c.filter(|p, _| {
                let dist = prev.distance_to(p).abs();
                prev.steps_to(p).unsigned_abs() <= 1 && dist > rational(0) && dist <= rational(2)
            });
        }
        c
    })
}
