#![warn(clippy::pedantic)]

use passacaglia_core::std_hept::{Interval, Pitch, PITCH_CLASSES};
use passacaglia_species_counterpoint::chord::Chord;

#[test]
fn chord_positions() {
    let maj = Chord::from_intervals_stacking(
        &[
            Interval::parse("M3").unwrap(),
            Interval::parse("m3").unwrap(),
        ],
        0,
        PITCH_CLASSES.c,
    );

    assert_eq!(maj.position, 0);
    assert_eq!(maj.bass, PITCH_CLASSES.c);

    let first = maj.to_position(1);
    assert_eq!(first.position, 1);
    assert_eq!(first.bass, Pitch::parse("e").unwrap());

    let second = maj.to_position(2);
    assert_eq!(second.position, 2);
    assert_eq!(second.bass, Pitch::parse("g").unwrap());

    assert_eq!(maj.to_position(0), maj);
}
