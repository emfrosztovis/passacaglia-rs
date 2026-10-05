use std::rc::Rc;

use passacaglia_common::rational_value;
use passacaglia_core::structure::Container;

use crate::{LocalRule, Voice};

/// Avoid consecutive measures with the same rhythm. Only applies to voices selected by a predicate.
#[must_use]
pub fn avoid_consecutive_measures_with_same_rhythm(
    pred: impl Fn(&Voice) -> bool + 'static
) -> LocalRule {
    Rc::new(move |_ctx, _s, cur| {
        let measure = cur.parent();
        // only check at the end of a measure
        // if cur.global_end_time() != measure.global_end_time() { return 0.0; }

        let voice = measure.container();
        if !pred(voice) { return 0.0; }

        let Some(prev_measure) = measure.prev_global() else { return 0.0; };
        let mut cost = 0.0;
        for (a, b) in prev_measure.notes.iter().zip(measure.notes.iter()) {
            if a.pitch.is_none() || b.pitch.is_none() { return cost; }
            if a.duration != b.duration { return cost; }
            cost += 10.0 * rational_value(a.duration);
        }

        if prev_measure.len() == measure.len() {
            f64::INFINITY
        } else {
            cost
        }
    })
}