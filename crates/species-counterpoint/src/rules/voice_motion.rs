use crate::context::CounterpointContext;
use crate::rules::utils::sign_of;
use crate::score::Score;
use crate::voice::NoteCursor;

#[must_use]
pub fn prioritize_voice_motion<'a>(
    ctx: &CounterpointContext,
    s: &'a Score,
    x1: NoteCursor<'a>,
) -> f64 {
    if s.voices.len() <= 1 {
        return 0.0;
    }
    let Some(x0) = x1.prev_global() else {
        return 0.0;
    };
    let (Some(px0), Some(px1)) = (x0.pitch, x1.pitch) else {
        return 0.0;
    };
    let sign0 = sign_of(px0.distance_to(&px1));
    let v = x1.parent().container();

    let mut cost = 0.0;
    for voice in s.voices.iter() {
        if voice.index() == v.index() {
            continue;
        }
        let Some(n1) = voice.note_at(x1.global_time()) else {
            continue;
        };
        let Some(n0) = n1.prev_global() else {
            continue;
        };
        let (Some(pn0), Some(pn1)) = (n0.pitch, n1.pitch) else {
            continue;
        };
        let sign1 = sign_of(pn0.distance_to(&pn1));
        if sign0 == sign1 {
            cost += ctx.similar_motion_cost;
        } else if sign0 == 0 || sign1 == 0 {
            cost += ctx.oblique_motion_cost;
        } else {
            cost += ctx.contrary_motion_cost;
        }
    }
    cost / (s.voices.len() - 1) as f64
}
