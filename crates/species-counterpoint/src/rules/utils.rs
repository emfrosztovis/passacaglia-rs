use passacaglia_common::{rational, Rational};
use passacaglia_core::std_hept::{Interval, Pitch, Scale};

use crate::voice::NoteCursor;

/// Sign of a rational (`-1`, `0`, or `1`).
#[must_use]
pub(crate) fn sign_of(r: Rational) -> i64 {
    let n = *r.numer();
    i64::from(n > 0) - i64::from(n < 0)
}

#[must_use]
pub fn is_stepwise_before(c: NoteCursor<'_>) -> Option<bool> {
    let pc = c.pitch?;
    let b = c.prev_global()?;
    let pb = b.pitch?;
    Some(pb.steps_to(&pc).unsigned_abs() == 1)
}

#[must_use]
pub fn is_stepwise_after(c: NoteCursor<'_>) -> Option<bool> {
    let pc = c.pitch?;
    let b = c.next_global()?;
    let pb = b.pitch?;
    Some(pb.steps_to(&pc).unsigned_abs() == 1)
}

#[must_use]
pub fn is_stepwise_around(c: NoteCursor<'_>) -> Option<bool> {
    let a = is_stepwise_before(c)?;
    let b = is_stepwise_after(c)?;
    Some(a && b)
}

#[must_use]
pub fn is_perfect_consonance(i: &Interval) -> bool {
    let d = i.to_simple(None).distance;
    d == rational(0) || d == rational(7) || d == rational(12)
}

/// Returns `true` if `i` is a perfect unison, major/minor third, perfect fourth
/// (unless with the bass), perfect fifth, major/minor sixth, perfect octave, or
/// any compound version of the above.
#[must_use]
pub fn is_consonance(i: &Interval, with_bass: bool) -> bool {
    let d = i.to_simple(None).distance;
    d == rational(0)
        || d == rational(3)
        || d == rational(4)
        || (!with_bass && d == rational(5))
        || d == rational(7)
        || d == rational(8)
        || d == rational(9)
        || d == rational(12)
}

#[must_use]
pub fn is_leading_tone(p: &Pitch, s: &Scale) -> bool {
    s.get_exact_degree(p, false)
        .is_some_and(|deg| deg.index == s.degrees.len() - 1)
}

#[must_use]
pub fn prev_different(mut c: NoteCursor<'_>) -> Option<NoteCursor<'_>> {
    let target = c.pitch.expect("pitch present");
    loop {
        let n = c.prev_global()?;
        if n.pitch.is_none() || n.pitch != Some(target) {
            return Some(n);
        }
        c = n;
    }
}

#[must_use]
pub fn next_different(mut c: NoteCursor<'_>) -> Option<NoteCursor<'_>> {
    let target = c.pitch.expect("pitch present");
    loop {
        let n = c.next_global()?;
        if n.pitch.is_none() || n.pitch != Some(target) {
            return Some(n);
        }
        c = n;
    }
}
