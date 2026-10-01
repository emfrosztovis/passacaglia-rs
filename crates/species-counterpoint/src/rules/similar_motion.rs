use crate::context::CounterpointContext;
use crate::rules::utils::{is_perfect_consonance, sign_of};
use crate::score::Score;
use crate::voice::NoteCursor;

#[must_use]
pub fn forbid_perfects_by_similar_motion<'a>(
    _ctx: &CounterpointContext,
    s: &'a Score,
    x1: NoteCursor<'a>,
) -> f64 {
    let Some(x0) = x1.prev_global() else {
        return 0.0;
    };
    let (Some(px0), Some(px1)) = (x0.pitch, x1.pitch) else {
        return 0.0;
    };

    let v = x1.parent().container();
    let sign0 = sign_of(px0.distance_to(&px1));

    for voice in s.voices.iter() {
        if voice.index() == v.index() {
            continue;
        }
        let Some(n1) = voice.note_at(x1.global_time()) else {
            continue;
        };
        let Some(pn1) = n1.pitch else {
            continue;
        };
        let d1 = px1.interval_to(&pn1);
        if !is_perfect_consonance(&d1) {
            continue;
        }

        let n0 = if n1.global_time() < x1.global_time() {
            n1
        } else {
            let Some(n0) = n1.prev_global() else {
                continue;
            };
            n0
        };
        let Some(pn0) = n0.pitch else {
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
