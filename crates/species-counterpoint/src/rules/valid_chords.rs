use std::rc::Rc;

use passacaglia_core::Degree;
use passacaglia_core::std_hept::Pitch;
use passacaglia_core::structure::Container;

use crate::chord::{chords, Chord};
use crate::context::{CandidateRule, Candidates, HarmonyRule};

fn permitted_chords() -> Vec<Chord> {
    vec![
        chords::major(),
        chords::major6(),
        chords::minor(),
        chords::minor6(),
        chords::dim6(),
    ]
}

#[must_use]
pub fn enforce_fixed_progression(prog: Vec<Option<Vec<Chord>>>) -> HarmonyRule {
    Rc::new(move |_ctx, _s, cur, c| {
        let c = c.expect("candidates initialized");
        let Some(chords) = prog.get(cur.index()).and_then(|c| c.as_deref()) else {
            return c;
        };
        let candidates = Candidates::from_pairs(chords.iter().map(|x| (x.clone(), 0.0)));
        let mut c = c;
        c.intersect(&candidates);
        c
    })
}

#[must_use]
pub fn enforce_valid_chords() -> HarmonyRule {
    Rc::new(move |_ctx, s, cur, c| {
        let scale = &s.harmony.scale;
        let mut basses: Vec<Pitch> = scale.all_degrees().iter().map(Degree::to_pitch).collect();
        let mut notes: Vec<Pitch> = Vec::new();

        for v in &s.voices {
            let Some(m) = v.measures().get(cur.index()) else {
                continue;
            };
            let mut bass: Option<Pitch> = None;
            for n in m.notes.iter() {
                if !n.is_non_harmonic() && let Some(p) = n.pitch {
                    let p0 = p.with_period(0);
                    if v.index() == s.voices.len() - 1 {
                        if bass.is_none_or(|b| b.ord() > p0.ord()) {
                            bass = Some(p0);
                        }
                    } else {
                        notes.push(p0);
                    }
                }
            }
            if let Some(b) = bass {
                basses = vec![b];
            }
        }

        if let Some(mut c) = c {
            c.filter(|ch, _| !notes.iter().any(|x| !ch.contains(x)));
            c
        } else {
            let mut map = Candidates::new();
            for bass in &basses {
                for ch in permitted_chords() {
                    let chord = ch.with_bass(*bass);
                    let tones_ok = !chord
                        .tones
                        .iter()
                        .any(|x| scale.get_degree(x).is_none());
                    let notes_ok = !notes.iter().any(|x| !chord.contains(x));
                    if tones_ok && notes_ok {
                        map.set(chord, 0.0);
                    }
                }
            }
            map
        }
    })
}

#[must_use]
pub fn enforce_chord_tone() -> CandidateRule {
    Rc::new(move |_ctx, s, cur, c, _ty| {
        let mut c = c.expect("candidates initialized");
        let ch = s
            .harmony
            .item(cur.parent().index())
            .chord
            .as_ref()
            .expect("chord present");

        if cur.parent().container().index() == s.voices.len() - 1 {
            let bass = ch.bass.with_period(0);
            c.filter(|p, _| p.with_period(0) == bass);
            return c;
        }

        c.filter(|p, _| ch.contains(p));
        c
    })
}
