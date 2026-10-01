use passacaglia_core::std_hept::Pitch;

use crate::context::CounterpointContext;
use crate::rules::utils::is_consonance;
use crate::score::Score;
use crate::voice::{NonHarmonicType, NoteCursor};

#[must_use]
pub fn enforce_vertical_consonance_with_moving_local<'a>(
    _ctx: &CounterpointContext,
    s: &'a Score,
    cur: NoteCursor<'a>,
) -> f64 {
    let t = cur.global_time();
    let mut pitches: Vec<Pitch> = Vec::new();
    let mut bass_pitch: Option<Pitch> = None;
    let last_index = s.voices.len() - 1;

    for (i, voice) in s.voices.iter().enumerate() {
        let Some(n1) = voice.note_at(t) else {
            continue;
        };
        let Some(p1) = n1.pitch else {
            continue;
        };
        if n1.global_time() != t || n1.non_harmonic == Some(NonHarmonicType::Suspension) {
            continue;
        }
        let moving = match n1.prev_global() {
            None => true,
            Some(n2) => n2.pitch != Some(p1),
        };
        if moving {
            pitches.push(p1);
            if i == last_index {
                bass_pitch = Some(p1);
            }
        }
    }

    for i in 0..pitches.len() {
        for j in (i + 1)..pitches.len() {
            let a = pitches[i];
            let b = pitches[j];
            let int = a.interval_to(&b);
            if !is_consonance(&int, bass_pitch == Some(b)) {
                return f64::INFINITY;
            }
        }
    }
    0.0
}
