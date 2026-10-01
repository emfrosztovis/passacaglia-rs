use passacaglia_core::std_hept::Pitch;
use passacaglia_core::{EqualTemperament, Tuning};

fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

#[test]
fn et_tuning() {
    let a440 = EqualTemperament::new(440.0, &Pitch::parse("a4").unwrap());

    assert!(close(
        a440.frequency_of(&Pitch::parse("c4").unwrap()),
        261.63,
        0.01
    ));
    assert!(close(
        a440.frequency_of(&Pitch::parse("bf6").unwrap()),
        1864.66,
        0.01
    ));

    assert!(close(
        a440.ratio_between(&Pitch::parse("g5").unwrap(), &Pitch::parse("g6").unwrap()),
        2.0,
        1e-9
    ));
    assert!(close(
        a440.ratio_between(&Pitch::parse("c5").unwrap(), &Pitch::parse("g5").unwrap()),
        1.498,
        0.001
    ));

    assert!(close(
        a440.cent_between(&Pitch::parse("c5").unwrap(), &Pitch::parse("d5").unwrap()),
        200.0,
        0.0001
    ));
}
