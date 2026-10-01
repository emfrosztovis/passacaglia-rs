use passacaglia_common::{rational, Rational};
use passacaglia_core::std_hept::Pitch;

use crate::context::{Candidates, CounterpointContext};
use crate::score::Score;
use crate::voice::{NonHarmonicType, NoteCursor};

#[must_use]
pub fn forbid_voice_overlapping2<'a>(
    ctx: &CounterpointContext,
    s: &'a Score,
    cur: NoteCursor<'a>,
    c: Option<Candidates<Pitch>>,
    _ty: Option<NonHarmonicType>,
) -> Candidates<Pitch> {
    let mut c = c.expect("candidates initialized");

    let mut upper: Option<Rational> = None;
    let mut lower: Option<Rational> = None;

    let iv = cur.parent().container().index();
    let end = cur.global_end_time();

    if iv > 0 {
        let v = &s.voices[iv - 1];
        let mut cur2 = v.note_at(cur.global_time());
        while let Some(c2) = cur2 {
            if c2.global_time() >= end {
                break;
            }
            if let Some(nord) = c2.pitch.map(|p| p.ord()) {
                if nord != rational(0) && upper.is_none_or(|u| nord < u) {
                    upper = Some(nord);
                }
            }
            cur2 = c2.next_global();
        }
        let before = v.note_at(cur.global_time()).and_then(|n| n.prev_global());
        if let Some(b) = before {
            if let Some(bv) = b.pitch.map(|p| p.ord()) {
                if bv != rational(0) && upper.is_none_or(|u| bv < u) {
                    upper = Some(bv - if ctx.allow_unison { rational(1) } else { rational(0) });
                }
            }
        }
    }

    if iv < s.voices.len() - 1 {
        let v = &s.voices[iv + 1];
        let mut cur2 = v.note_at(cur.global_time());
        while let Some(c2) = cur2 {
            if c2.global_time() >= end {
                break;
            }
            if let Some(nord) = c2.pitch.map(|p| p.ord()) {
                if nord != rational(0) && lower.is_none_or(|l| nord > l) {
                    lower = Some(nord);
                }
            }
            cur2 = c2.next_global();
        }
        let before = v.note_at(cur.global_time()).and_then(|n| n.prev_global());
        if let Some(b) = before {
            if let Some(bv) = b.pitch.map(|p| p.ord()) {
                if bv != rational(0) && lower.is_none_or(|l| bv > l) {
                    lower = Some(bv + if ctx.allow_unison { rational(1) } else { rational(0) });
                }
            }
        }
    }

    c.filter(|p, _| {
        let ord = p.ord();
        if let Some(u) = upper {
            if ord > u || (!ctx.allow_unison && ord == u) {
                return false;
            }
        }
        if let Some(l) = lower {
            if ord < l || (!ctx.allow_unison && ord == l) {
                return false;
            }
        }
        true
    });
    c
}
