use passacaglia_common::{rational, Rational};
use passacaglia_core::std_hept::{Interval, Pitch, Scale};
use passacaglia_macros::std_hept_interval as interval;

use crate::voice::NoteCursor;

#[must_use]
pub(crate) fn note_pitch(c: Option<NoteCursor<'_>>) -> Option<(NoteCursor<'_>, Pitch)> {
    if let Some(x) = c
        && let Some(p) = x.pitch
    {
        Some((x, p))
    } else {
        None
    }
}

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

/// The pitch of the note `n` steps before `c` in global timeline order.
#[must_use]
pub fn nth_prev_pitch(mut c: NoteCursor<'_>, n: usize) -> Option<Pitch> {
    for _ in 0..n {
        c = c.prev_global()?;
    }
    c.pitch
}

#[must_use]
pub fn is_perfect_consonance(i: &Interval) -> bool {
    // let d = i.to_simple(None).distance;
    // d == rational(0) || d == rational(7) || d == rational(12)
    let d = i.to_simple(None).abs();
    d == interval!("P1") || d == interval!("P5") || d == interval!("P8")
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
    s.get_degree(p)
        .is_some_and(|deg| deg.index == s.degrees.len() - 1
            && deg.to_pitch().absolute_simple_interval_to(&s.root().pitch) == interval!("m2"))
}

/// If this is tied, first go to the start of the tie. Then go to the previous note. If this note 
/// is tied, go to the start of that tie. Return a cursor at this position.
/// 
/// Equivalent to `start_of_tie(start_of_tie(c).prev_global()?)`
#[must_use]
pub fn prev_non_tied(c: NoteCursor<'_>) -> Option<NoteCursor<'_>> {
    let mut c = start_of_tie(c);
    loop {
        let n = c.prev_global()?;
        if !n.is_tied() {
            return Some(n);
        }
        c = n;
    }
}

/// Returns the start of the tie of `c` is tied, or returns `c` itself.
#[must_use]
pub fn start_of_tie(c: NoteCursor<'_>) -> NoteCursor<'_> {
    let mut c = c;
    while c.is_tied() && let Some(prev) = c.prev_global() {
        c = prev;
    }
    c
}

/// Returns the end of the tie of `c` is tied, or returns `c` itself.
#[must_use]
pub fn end_of_tie(c: NoteCursor<'_>) -> NoteCursor<'_> {
    let mut c = c;
    while let Some(next) = c.next_global() && next.is_tied() {
        c = next;
    }
    c
}


#[must_use]
pub fn prev_different(mut c: NoteCursor<'_>) -> Option<NoteCursor<'_>> {
    let target = c.pitch.expect("pitch present");
    loop {
        let n = c.prev_global()?;
        if n.pitch != Some(target) {
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
        if n.pitch != Some(target) {
            return Some(n);
        }
        c = n;
    }
}
