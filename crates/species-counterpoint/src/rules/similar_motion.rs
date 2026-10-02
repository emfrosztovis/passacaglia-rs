use crate::context::CounterpointContext;
use crate::rules::utils::{is_perfect_consonance, note_pitch, sign_of};
use crate::score::Score;
use crate::voice::NoteCursor;

#[must_use]
pub fn forbid_perfects_by_similar_motion<'a>(
    _ctx: &CounterpointContext,
    s: &'a Score,
    x1: NoteCursor<'a>,
) -> f64 {
    let Some((_, px0)) = note_pitch(x1.prev_global()) else {
        return 0.0;
    };
    let Some(px1) = x1.pitch else {
        return 0.0;
    };

    let v = x1.parent().container();
    let sign0 = sign_of(px0.distance_to(&px1));

    for voice in s.voices.iter() {
        if voice.index() == v.index() {
            continue;
        }
        let Some((n1, pn1)) = note_pitch(voice.note_at(x1.global_time())) else {
            continue;
        };
        let d1 = px1.interval_to(&pn1);
        if !is_perfect_consonance(&d1) {
            continue;
        }

        let n0 = if n1.global_time() < x1.global_time() {
            Some(n1)
        } else {
            n1.prev_global()
        };
        let Some((_, pn0)) = note_pitch(n0) else {
            continue;
        };

        let sign1 = sign_of(pn0.distance_to(&pn1));
        if sign0 == sign1 {
            if sign0 == 0 {
                continue;
            }
            return f64::INFINITY;
        }

        let d0 = px0.interval_to(&pn0);
        if is_perfect_consonance(&d0) {
            return f64::INFINITY;
        }
    }
    0.0
}
