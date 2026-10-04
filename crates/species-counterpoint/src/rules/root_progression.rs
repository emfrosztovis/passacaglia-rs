use std::rc::Rc;

use passacaglia_core::std_hept::Interval;
use passacaglia_core::structure::Container;

use crate::chord::{chords, Chord};
use crate::context::{Candidates, HarmonyRule};

#[must_use]
pub fn enforce_root_progression(root_intervals: Vec<Interval>, chords: Vec<Chord>) -> HarmonyRule {
    Rc::new(move |_ctx, s, cur, c| {
        let scale = &s.harmony.scale;
        let prev = cur.prev().and_then(|p| s.harmony.item(p.index()).chord.as_ref());

        let Some(prev) = prev else {
            let tonic_major = chords::major().with_root(scale.root().pitch);
            let tonic_minor = chords::minor().with_root(scale.root().pitch);
            return match c {
                Some(mut c) => {
                    c.filter(|x, _| *x == tonic_major || *x == tonic_minor);
                    c
                }
                None => Candidates::from_pairs(vec![(tonic_major, 0.0), (tonic_minor, 0.0)]),
            };
        };

        let mut new_chords = Vec::new();
        for int in &root_intervals {
            for ch in &chords {
                let chord = ch.with_root(prev.root().add(int));
                let tones_ok = !chord
                    .tones
                    .iter()
                    .any(|x| scale.get_degree(x).is_none());
                if tones_ok {
                    new_chords.push((chord, 0.0));
                }
            }
        }
        let new_chords = Candidates::from_pairs(new_chords);

        match c {
            Some(mut c) => {
                c.intersect(&new_chords);
                c
            }
            None => new_chords,
        }
    })
}
