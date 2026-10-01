use passacaglia_core::std_hept::Pitch;

use crate::context::{Candidates, CounterpointContext};
use crate::score::Score;
use crate::voice::{NonHarmonicType, NoteCursor};

#[must_use]
pub fn enforce_leap_preparation<'a>(
    _ctx: &CounterpointContext,
    _s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let mut c = c.expect("candidates initialized");
    let Some(p1) = cur.prev_global() else {
        return c;
    };
    let Some(prev) = p1.pitch else {
        return c;
    };
    let Some(p2) = p1.prev_global() else {
        return c;
    };
    let Some(prev2) = p2.pitch else {
        return c;
    };
    let int0 = prev2.interval_to(&prev);
    c.filter(|x, _| {
        let int = prev.interval_to(x);
        if int.steps < 3 {
            return true;
        }
        int0.steps == 1 && int.sign == -int0.sign
    });
    c
}

#[must_use]
pub fn enforce_leap_resolution<'a>(
    _ctx: &CounterpointContext,
    _s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let mut c = c.expect("candidates initialized");
    let Some(p1) = cur.prev_global() else {
        return c;
    };
    let Some(prev) = p1.pitch else {
        return c;
    };
    let Some(p2) = p1.prev_global() else {
        return c;
    };
    let Some(prev2) = p2.pitch else {
        return c;
    };
    let int0 = prev2.interval_to(&prev);
    if int0.steps < 3 {
        return c;
    }
    c.filter(|x, _| {
        let int = prev.interval_to(x);
        int.steps >= 3 || (int.steps == 1 && int.sign == -int0.sign)
    });
    c
}

#[must_use]
pub fn limit_consecutive_leaps<'a>(
    _ctx: &CounterpointContext,
    _s: &'a Score,
    x1: NoteCursor<'a>,
) -> f64 {
    let m = x1.container().melodic_context();
    let Some(settings) = x1.parent().container().melody_settings() else {
        return 0.0;
    };
    if m.n_consecutive_leaps - m.n3rd_leaps.min(settings.max_ignorable_3rd_leaps)
        > settings.max_consecutive_leaps
        || m.n_unidirectional_consecutive_leaps
            - m.n_unidirectional_3rd_leaps
                .min(settings.max_unidirectional_ignorable_3rd_leaps)
            > settings.max_unidirectional_consecutive_leaps
    {
        return f64::INFINITY;
    }
    0.0
}
