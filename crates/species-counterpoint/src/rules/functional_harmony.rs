use std::rc::Rc;

use passacaglia_core::std_hept::Scale;
use passacaglia_core::structure::Container;

use crate::chord::{Chord, chords};
use crate::context::{Candidates, HarmonyRule};

fn triads(
    array: &[(usize, Chord, f64)], scale: &Scale, 
    c: Option<Candidates<Chord>>
) -> Candidates<Chord> {
    let map = Candidates::from_pairs(
        array
            .iter()
            .flat_map(|&(x, ref ch, cost)| {
                let chord = ch.with_root(scale.at(x).to_pitch());
                vec![(chord.clone(), cost), (chord.to_position(1), cost)]
            })
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
            let tonic = chords::major().with_root(scale.root().pitch);
            return match c {
                Some(mut c) => {
                    c.filter(|x, _| *x == tonic);
                    c
                }
                None => Candidates::from_pairs(vec![(tonic, 0.0)]),
            };
        };

        let Some(deg) = scale.get_degree(&prev.root()) else {
            return Candidates::new();
        };

        // 0  1   2  3  4  5   6
        // I II III IV  V VI VII
        // M  m   m  M  M  m dim
        match deg.index {
            0 => triads(&[
                (3, chords::major(), 0.0), 
                (4, chords::major(), 0.0), 
                (5, chords::minor(), 10.0), 
                (1, chords::minor(), 40.0), 
                (2, chords::minor(), 40.0), 
                (0, chords::major(), 40.0)
            ], scale, c),
            1 => triads(&[
                (4, chords::major(), 0.0), 
                (6, chords::dim(), 30.0)
            ], scale, c),
            2 => triads(&[
                (5, chords::minor(), 0.0), 
                (3, chords::major(), 10.0)
            ], scale, c),
            3 => triads(&[
                (4, chords::major(), 0.0), 
                (0, chords::major(), 10.0), 
                (1, chords::minor(), 10.0), 
                (6, chords::dim(), 30.0)
            ], scale, c),
            4 => triads(&[
                (0, chords::major(), 0.0), 
                (5, chords::minor(), 10.0)
            ], scale, c),
            5 => triads(&[
                (1, chords::minor(), 0.0), 
                (4, chords::major(), 0.0),
                (3, chords::major(), 10.0)
            ], scale, c),
            6 => triads(&[
                (2, chords::minor(), 0.0), 
                (5, chords::minor(), 10.0)
            ], scale, c),
            _ => unreachable!("valid scale degree"),
        }
    })
}

#[must_use]
pub fn enforce_functional_progression_minor() -> HarmonyRule {
    Rc::new(move |_ctx, s, cur, c| {
        let scale = &s.harmony.scale;
        let prev = cur.prev().and_then(|p| s.harmony.item(p.index()).chord.as_ref());

        let Some(prev) = prev else {
            let tonic = chords::minor().with_root(scale.root().pitch);
            return match c {
                Some(mut c) => {
                    c.filter(|x, _| *x == tonic);
                    c
                }
                None => Candidates::from_pairs(vec![(tonic, 0.0)]),
            };
        };

        let Some(deg) = scale.get_degree(&prev.root()) else {
            return Candidates::new();
        };

        // 0   1   2  3  4    5   6
        // i  ii iii iv  v   vi vii
        // m dim   M  m  m    M   M
        // -   m aug  M  M [dim dim]
        match deg.index {
            0 => triads(&[
                (3, chords::minor(), 0.0), 
                (3, chords::major(), 0.0), 
                (4, chords::minor(), 0.0), 
                (4, chords::major(), 0.0), 
                (5, chords::major(), 10.0), 
                (1, chords::minor(), 40.0), 
                (1, chords::dim(), 40.0), 
                (2, chords::major(), 40.0), 
                // (2, chords::aug(), 40.0), 
                (0, chords::minor(), 40.0)
            ], scale, c),
            1 => triads(&[
                (4, chords::minor(), 0.0), 
                (4, chords::major(), 0.0), 
                (6, chords::major(), 30.0),
            ], scale, c),
            2 => triads(&[
                (5, chords::major(), 0.0), 
                (3, chords::minor(), 10.0),
                (3, chords::major(), 10.0),
            ], scale, c),
            3 => triads(&[
                (4, chords::minor(), 0.0), 
                (4, chords::major(), 0.0), 
                (0, chords::major(), 10.0), 
                (1, chords::minor(), 10.0), 
                (6, chords::major(), 30.0),
            ], scale, c),
            4 => triads(&[
                (0, chords::minor(), 0.0), 
                (5, chords::major(), 10.0)
            ], scale, c),
            5 => triads(&[
                (1, chords::minor(), 0.0), 
                (4, chords::minor(), 0.0), 
                (4, chords::major(), 0.0), 
                (3, chords::minor(), 10.0),
                (3, chords::major(), 10.0),
            ], scale, c),
            6 => triads(&[
                (2, chords::major(), 0.0), 
                // (2, chords::aug(), 0.0), 
                (5, chords::major(), 10.0)
            ], scale, c),
            _ => unreachable!("valid scale degree"),
        }
    })
}