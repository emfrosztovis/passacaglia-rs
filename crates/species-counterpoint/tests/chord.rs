#![warn(clippy::pedantic)]

use passacaglia_species_counterpoint::chord::Chord;
use passacaglia_macros::std_hept_pitch as pitch;
use passacaglia_macros::std_hept_interval as interval;

#[test]
fn chord_positions() {
    let maj = Chord::from_intervals_stacking(
        &[interval!("M3"), interval!("m3")],
        0,
        pitch!("c"),
    );

    assert_eq!(maj.position, 0);
    assert_eq!(maj.bass, pitch!("c"));

    let first = maj.to_position(1);
    assert_eq!(first.position, 1);
    assert_eq!(first.bass, pitch!("e"));
    assert_eq!(first.root(), pitch!("c"));

    let second = maj.to_position(2);
    assert_eq!(second.position, 2);
    assert_eq!(second.bass, pitch!("g"));
    assert_eq!(second.root(), pitch!("c"));

    assert_eq!(maj.to_position(0), maj);
}
