use passacaglia_common::{rational, Rational};
use passacaglia_core::std_hept::{scales, Interval, Pitch, Scale, PITCH_CLASSES};

#[test]
fn simple_well_known_scales() {
    let s = scales::major(PITCH_CLASSES.b);
    assert_eq!(s.at(2, rational(0)).to_pitch().to_string(), "ds1");

    let s = scales::harmonic_minor(PITCH_CLASSES.c);
    assert_eq!(s.at(2, rational(0)).to_pitch().to_string(), "ef0");

    let s = scales::major(PITCH_CLASSES.e);
    assert_eq!(
        s.at(2, rational(0)).with_period(4).to_pitch().to_string(),
        "gs4"
    );

    let s = scales::harmonic_minor(PITCH_CLASSES.e);
    assert_eq!(
        s.at(2, rational(0)).with_period(4).to_pitch().to_string(),
        "g4"
    );
}

#[test]
fn parse_degree() {
    let d = scales::c::MAJOR.parse_degree("iii").unwrap();
    assert_eq!(d.index, 2);
    assert_eq!(d.acci, rational(0));

    let d = scales::c::MAJOR.parse_degree("[4]f").unwrap();
    assert_eq!(d.index, 3);
    assert_eq!(d.acci, rational(-1));

    let d = scales::c::MAJOR.parse_degree("iv5/4s").unwrap();
    assert_eq!(d.index, 3);
    assert_eq!(d.acci, Rational::new(5, 4));
}

#[test]
fn parse_degree_fail() {
    for s in ["?", "iiii", "[0]", "[2]+1"] {
        assert!(
            scales::c::MAJOR.parse_degree(s).is_none(),
            "expected `{s}` to fail"
        );
    }
}

#[test]
fn degree_to_string() {
    assert_eq!(scales::c::MAJOR.at(5, rational(0)).to_string(), "vi");
    assert_eq!(
        scales::c::MAJOR.at(5, rational(0)).to_arabic_string(),
        "[6]"
    );

    assert_eq!(
        scales::c::MAJOR.at(4, Rational::new(-3, 2)).to_string(),
        "v3/2f"
    );
    assert_eq!(
        scales::c::MAJOR.at(5, Rational::new(-3, 2)).to_arabic_string(),
        "[6]3/2f"
    );
}

#[test]
fn equality() {
    let built = Scale::from_intervals(
        PITCH_CLASSES.c,
        &["M2", "M2", "m2", "M2", "M2", "M2", "m2"]
            .iter()
            .map(|s| Interval::parse(s).unwrap())
            .collect::<Vec<_>>(),
    );
    assert_eq!(*scales::c::MAJOR, built);

    assert!(scales::major(PITCH_CLASSES.d).interval_equals(&scales::major(PITCH_CLASSES.b)));
}

#[test]
fn get_degrees_in_range() {
    let s = scales::major(Pitch::parse("fs").unwrap());
    let got: Vec<String> = s
        .get_degrees_in_range(&Pitch::parse("c4").unwrap(), &Pitch::parse("g4").unwrap())
        .iter()
        .map(|d| d.to_pitch().to_string())
        .collect();
    assert_eq!(got, vec!["cs4", "ds4", "es4", "fs4"]);

    let s2 = scales::major(Pitch::parse("b").unwrap());
    let got: Vec<String> = s2
        .get_degrees_in_range(&Pitch::parse("c4").unwrap(), &Pitch::parse("g4").unwrap())
        .iter()
        .map(|d| d.to_pitch().to_string())
        .collect();
    assert_eq!(got, vec!["cs4", "ds4", "e4", "fs4"]);
}

#[test]
fn get_exact_degree() {
    assert!(scales::major(PITCH_CLASSES.c)
        .get_exact_degree(&Pitch::parse("es").unwrap(), false)
        .is_none());
    assert_eq!(
        scales::major(PITCH_CLASSES.c)
            .get_exact_degree(&Pitch::parse("es").unwrap(), true)
            .unwrap()
            .to_string(),
        "iv"
    );
    assert_eq!(
        scales::harmonic_minor(PITCH_CLASSES.c)
            .get_exact_degree(&Pitch::parse("ef").unwrap(), false)
            .unwrap()
            .to_string(),
        "iii"
    );
}

#[test]
fn rotate() {
    assert_eq!(
        scales::c::MAJOR.rotate(2, false),
        scales::phrygian(PITCH_CLASSES.c)
    );
    assert_eq!(
        scales::c::MAJOR.rotate(2, true),
        scales::phrygian(PITCH_CLASSES.e)
    );
}

#[test]
fn transpose() {
    assert_eq!(
        scales::c::MAJOR.transpose(&Interval::parse("m3").unwrap()),
        scales::major(Pitch::parse("ef").unwrap())
    );
    assert_eq!(
        scales::c::MAJOR.transpose(&Interval::parse("-m2").unwrap()),
        scales::major(PITCH_CLASSES.b)
    );
}

#[test]
fn degree_next_previous() {
    let s = scales::major(Pitch::parse("b").unwrap());
    assert_eq!(s.at(0, rational(0)).next().to_pitch().to_string(), "cs1");
    assert_eq!(s.at(6, rational(0)).next().to_pitch().to_string(), "b1");
    assert_eq!(s.at(0, rational(0)).previous().to_pitch().to_string(), "as0");
    assert_eq!(s.at(6, rational(0)).previous().to_pitch().to_string(), "gs1");
}
