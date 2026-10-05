use std::rc::Rc;

use crate::context::LocalRule;
use crate::rules::utils::{is_perfect_consonance, note_pitch, prev_non_tied, sign_of, start_of_tie};

/// Forbid arriving at perfect consonances 1) by similar motion or
/// 2) immediately from perfect consonances.
#[must_use]
pub fn forbid_perfects_by_similar_motion() -> LocalRule {
    Rc::new(move |_ctx, s, x1| {
        if x1.is_tied() { return 0.0; }

        let Some((x0, px0)) = note_pitch(prev_non_tied(x1)) else {
            return 0.0;
        };
        let Some(px1) = x1.pitch else {
            return 0.0;
        };

        let v = x1.parent().container();
        let sign0 = sign_of(px0.distance_to(&px1));

        for voice in &s.voices {
            if voice.index() == v.index() { continue; }

            // get the note sounding at the beginning of our note
            let Some((n1, pn1)) = note_pitch(voice.note_at(x1.global_time())) else {
                continue;
            };
            if !is_perfect_consonance(&px1.interval_to(&pn1)) {
                continue;
            }

            let Some((n0, pn0)) = note_pitch(start_of_tie(n1).prev_global()) else {
                continue;
            };

            if n0.global_end_time() <= x0.global_time() {
                //          [--x0--][--x1--]
                // [--n0--][----n1----]
                continue;
            }

            if sign0 == sign_of(pn0.distance_to(&pn1)) {
                if sign0 == 0 {
                    // just repeated
                    continue;
                }
                return f64::INFINITY;
            }

            if is_perfect_consonance(&px0.interval_to(&pn0)) {
                return f64::INFINITY;
            }
        }
        0.0
    })
}
