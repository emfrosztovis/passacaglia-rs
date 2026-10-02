use passacaglia_common::rational;
use passacaglia_core::std_hept::Pitch;

use crate::context::{Candidates, CounterpointContext};
use crate::rules::utils::{is_consonance, is_leading_tone, note_pitch};
use crate::score::Score;
use crate::voice::{NonHarmonicType, NoteCursor};

/// Enforce that suspensions are resolved correctly.
#[must_use]
pub fn enforce_suspension<'a>(
    _ctx: &CounterpointContext,
    s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
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
}

#[must_use]
pub fn make_suspension<'a>(
    _ctx: &CounterpointContext,
    _s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let mut c = c.expect("candidates initialized");
    if cur.index() != 0 {
        return Candidates::new();
    }
    if let Some((p1, prev)) = note_pitch(cur.prev_global())
        && p1.non_harmonic.is_none()
        && p1.span() >= cur.span()
    {
        c.filter_map(|p, _| if *p == prev { Some(0.0) } else { None });
    }
    c
}
