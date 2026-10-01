use passacaglia_common::rational_value;

use crate::context::CounterpointContext;
use crate::rules::utils::{is_perfect_consonance, is_stepwise_around};
use crate::score::Score;
use crate::voice::NoteCursor;

#[must_use]
pub fn forbid_nearby_perfects<'a>(
    ctx: &CounterpointContext,
    s: &'a Score,
    x1: NoteCursor<'a>,
) -> f64 {
    let Some(px1) = x1.pitch else {
        return 0.0;
    };
    let measure_len = rational_value(ctx.parameters.measure_length);
    let v = x1.parent().container();

    if x1.index() == 0 {
        let t1 = x1.global_time();
        for voice in s.voices.iter() {
            if voice.index() == v.index() {
                continue;
            }
            let Some(y1) = voice.note_at(t1) else {
                continue;
            };
            let Some(py1) = y1.pitch else {
                continue;
            };
            let int1 = px1.interval_to(&py1);
            if !is_perfect_consonance(&int1) {
                continue;
            }

            let mut y2 = y1.prev_global();
            while let Some(y2c) = y2 {
                let Some(py2) = y2c.pitch else {
                    break;
                };
                if rational_value(t1 - y2c.global_time()) > measure_len {
                    break;
                }
                if let Some(x2) = v.note_at(y2c.global_time()) {
                    if let Some(px2) = x2.pitch {
                        let int2 = px2.interval_to(&py2);
                        if is_perfect_consonance(&int2) {
                            return f64::INFINITY;
                        }
                    }
                }
                y2 = y2c.prev_global();
            }

            let mut x2 = x1.prev_global();
            while let Some(x2c) = x2 {
                let Some(px2) = x2c.pitch else {
                    break;
                };
                if rational_value(t1 - x2c.global_time()) > measure_len {
                    break;
                }
                if let Some(y2) = voice.note_at(x2c.global_time()) {
                    if let Some(py2) = y2.pitch {
                        let int2 = px2.interval_to(&py2);
                        if is_perfect_consonance(&int2) {
                            return f64::INFINITY;
                        }
                    }
                }
                x2 = x2c.prev_global();
            }
        }
    } else {
        if x1.non_harmonic.is_some() {
            return 0.0;
        }
        if is_stepwise_around(x1) == Some(true) {
            return 0.0;
        }
        let t1 = x1.global_time();
        let t2 = t1 - ctx.parameters.measure_length;

        let Some(x2) = v.note_at(t2) else {
            return 0.0;
        };
        let Some(px2) = x2.pitch else {
            return 0.0;
        };
        if x2.global_time() != t2 {
            return 0.0;
        }
        if x2.non_harmonic.is_some() {
            return 0.0;
        }
        if is_stepwise_around(x2) == Some(true) {
            return 0.0;
        }

        for voice in s.voices.iter() {
            if voice.index() == v.index() {
                continue;
            }
            let Some(y1) = voice.note_at(t1) else {
                continue;
            };
            let Some(py1) = y1.pitch else {
                continue;
            };
            if y1.non_harmonic.is_some() {
                continue;
            }
            if is_stepwise_around(y1) == Some(true) {
                continue;
            }
            let int1 = px1.interval_to(&py1);
            if !is_perfect_consonance(&int1) {
                continue;
            }
            let Some(y2) = voice.note_at(t2) else {
                continue;
            };
            let Some(py2) = y2.pitch else {
                continue;
            };
            if y2.non_harmonic.is_some() {
                continue;
            }
            if is_stepwise_around(y2) == Some(true) {
                continue;
            }
            let int2 = px2.interval_to(&py2);
            if int1 == int2 {
                return f64::INFINITY;
            }
        }
    }
    0.0
}
