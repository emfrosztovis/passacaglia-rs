use std::rc::Rc;

use passacaglia_common::rational;
use passacaglia_core::std_hept::Scale;
use passacaglia_core::structure::Container;

use crate::chord::{chords, Chord};
use crate::context::{Candidates, HarmonyRule};

fn get_degree_triads(i: usize, scale: &Scale) -> Vec<Chord> {
    let t1 = scale.at(i, rational(0));
    let t2 = t1.next().next();
    let t3 = t2.next().next();
    let chord = Chord::from_pitches(&[t1.to_pitch(), t2.to_pitch(), t3.to_pitch()], 0);
    vec![chord.clone(), chord.to_position(1)]
}

fn triads(array: &[usize], scale: &Scale, c: Option<Candidates<Chord>>) -> Candidates<Chord> {
    let map = Candidates::from_pairs(
        array
            .iter()
            .flat_map(|&x| get_degree_triads(x, scale))
            .map(|ch| (ch, 0.0)),
    );
    match c {
        Some(mut c) => {
            c.intersect(&map);
            c
        }
        None => map,
    }
}

#[must_use]
pub fn enforce_functional_progression_major() -> HarmonyRule {
    Rc::new(move |_ctx, s, cur, c| {
        let scale = &s.harmony.scale;
        let prev = cur.prev().and_then(|p| s.harmony.item(p.index()).chord.as_ref());

        let Some(prev) = prev else {
            let tonic = chords::major().with_root(scale.root());
            return match c {
                Some(mut c) => {
                    c.filter(|x, _| *x == tonic);
                    c
                }
                None => Candidates::from_pairs(vec![(tonic, 0.0)]),
            };
        };

        let Some(deg) = scale.get_exact_degree(&prev.root(), false) else {
            return Candidates::new();
        };

        match deg.index {
            0 => triads(&[0, 1, 2, 3, 4, 5], scale, c),
            1 => triads(&[4, 6], scale, c),
            2 => triads(&[3, 5], scale, c),
            3 => triads(&[0, 1, 4, 6], scale, c),
            4 | 6 => triads(&[0, 5], scale, c),
            5 => triads(&[1, 3, 4], scale, c),
            _ => unreachable!("valid scale degree"),
        }
    })
}
