use std::rc::Rc;

use crate::context::LocalRule;
use crate::rules::utils::{note_pitch, sign_of};

/// Assign heuristic costs according to motion type, based on the settings in
/// [`crate::context::CounterpointContext`].
#[must_use]
pub fn prioritize_voice_motion() -> LocalRule {
    Rc::new(move |ctx, s, x1| {
        if s.voices.len() <= 1 {
            return 0.0;
        }
        let Some((_, px0)) = note_pitch(x1.prev_global()) else {
            return 0.0;
        };
        let Some(px1) = x1.pitch else {
            return 0.0;
        };
        let sign0 = sign_of(px0.distance_to(&px1));
        let v = x1.parent().container();

        let mut cost = 0.0;
        for voice in &s.voices {
            if voice.index() == v.index() {
                continue;
            }
            if let Some((n1, pn1)) = note_pitch(voice.note_at(x1.global_time()))
                && let Some((_, pn0)) = note_pitch(n1.prev_global())
            {
                let sign1 = sign_of(pn0.distance_to(&pn1));
                if sign0 == sign1 {
                    cost += ctx.similar_motion_cost;
                } else if sign0 == 0 || sign1 == 0 {
                    cost += ctx.oblique_motion_cost;
                } else {
                    cost += ctx.contrary_motion_cost;
                }
            }
        }
        cost / (s.voices.len() - 1) as f64
    })
}
