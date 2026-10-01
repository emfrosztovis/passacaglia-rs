use passacaglia_core::std_hept::{scales, Interval, Pitch, Scale};
use passacaglia_macros::std_hept_interval as interval;
use passacaglia_macros::std_hept_pitch as pitch;
use passacaglia_macros::std_hept_scale as scale;

#[test]
fn pitch_macro() {
    let p: Pitch = pitch!("c4");
    assert_eq!(p.to_string(), "c4");
    assert_eq!(p, Pitch::parse("c4").unwrap());

    let g: Pitch = pitch!("g6/17s7");
    assert_eq!(g.to_string(), "g6/17s7");
}

#[test]
fn interval_macro() {
    let i: Interval = interval!("M3");
    assert_eq!(i.to_abbreviation(false), "M3");
    assert_eq!(i, Interval::parse("M3").unwrap());

    let d: Interval = interval!("d12-2");
    assert_eq!(d.to_abbreviation(false), "d12-2");
}

#[test]
fn scale_macro() {
    let s: Scale = scale!["M2", "M2", "m2", "M2", "M2", "M2", "m2"];
    assert_eq!(*scales::c::MAJOR, s);
}
