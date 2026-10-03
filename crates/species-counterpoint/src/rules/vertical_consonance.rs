use std::rc::Rc;

use passacaglia_core::std_hept::Pitch;

use crate::context::LocalRule;
use crate::rules::utils::{is_consonance, note_pitch};
use crate::voice::NonHarmonicType;

/// Enforces that the candidates form consonance with voices that are moving
/// at the same point. Forbids certain intervals if bass is involved.
#[must_use]
pub fn enforce_vertical_consonance_with_moving_local() -> LocalRule {
    Rc::new(move |_ctx, s, cur| {
        let t = cur.global_time();
        let mut pitches: Vec<Pitch> = Vec::new();
        let mut bass_pitch: Option<Pitch> = None;
        let last_index = s.voices.len() - 1;

        for (i, voice) in s.voices.iter().enumerate() {
            let Some((n1, p1)) = note_pitch(voice.note_at(t)) else {
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
    })
}
