use std::rc::Rc;

use crate::context::LocalRule;
use crate::rules::utils::{is_perfect_consonance, is_stepwise_around, note_pitch};

/// Forbid perfect consonances that are near each other.
///
/// Specifically, if the second consonance is on the first beat of the measure:
/// - all perfect consonances that is less than OR exactly a measure apart from it
///
/// If the second consonance is not so:
/// - only when the first consonance is on the same beat at the second
/// - and NO notes are non-harmonic tones, or surrounded by stepwise motion
/// - (NOT IMPLEMENTED) and IF the two notes of the second consonance don't
///   start simultaneously, only when they are NOT in contrary motion.
#[must_use]
pub fn forbid_nearby_perfects() -> LocalRule {
    Rc::new(move |ctx, s, x1| {
        let Some(px1) = x1.pitch else {
            return 0.0;
        };
        let measure_len = ctx.parameters.measure_length;
        let v = x1.parent().container();

        if x1.index() == 0 {
            let t1 = x1.global_time();
            for voice in s.voices.iter() {
                if voice.index() == v.index() {
                    continue;
                }

                if let Some((y1, py1)) = note_pitch(voice.note_at(t1))
                    && is_perfect_consonance(&px1.interval_to(&py1))
                {
                    let mut y2 = y1.prev_global();
                    while let Some((y2c, py2)) = note_pitch(y2)
                        && t1 - y2c.global_time() <= measure_len
                    {
                        if let Some((_, px2)) = note_pitch(v.note_at(y2c.global_time()))
                            && is_perfect_consonance(&px2.interval_to(&py2))
                        {
                            return f64::INFINITY;
                        }
                        y2 = y2c.prev_global();
                    }

                    let mut x2 = x1.prev_global();
                    while let Some((x2c, px2)) = note_pitch(x2)
                        && t1 - x2c.global_time() <= measure_len
                    {
                        if let Some((_, py2)) = note_pitch(voice.note_at(x2c.global_time()))
                            && is_perfect_consonance(&px2.interval_to(&py2))
                        {
                            return f64::INFINITY;
                        }
                        x2 = x2c.prev_global();
                    }
                }
            }
        } else {
            if x1.non_harmonic.is_some() || is_stepwise_around(x1) == Some(true) {
                return 0.0;
            }
            let t1 = x1.global_time();
            let t2 = t1 - ctx.parameters.measure_length;

            let Some((x2, px2)) = note_pitch(v.note_at(t2)) else {
                return 0.0;
            };
            if x2.global_time() != t2
                || x2.non_harmonic.is_some()
                || is_stepwise_around(x2) == Some(true)
            {
                return 0.0;
            }

            for voice in s.voices.iter() {
                if voice.index() == v.index() {
                    continue;
                }
                if let Some((y1, py1)) = note_pitch(voice.note_at(t1))
                    && y1.non_harmonic.is_none()
                    && is_stepwise_around(y1) != Some(true)

                    && let int1 = px1.interval_to(&py1)
                    && is_perfect_consonance(&int1)

                    && let Some((y2, py2)) = note_pitch(voice.note_at(t2))
                    && y2.non_harmonic.is_none()
                    && is_stepwise_around(y2) != Some(true)

                    && int1 == px2.interval_to(&py2)
                {
                    return f64::INFINITY;
                }
            }
        }
        0.0
    })
}
