use passacaglia_common::Rational;
use passacaglia_core::std_hept::Interval;

#[test]
fn parse() {
    let a = Interval::parse("-m3+1/2").unwrap();
    assert_eq!(a.sign, -1);
    assert_eq!(a.steps, 2);
    assert_eq!(a.distance, Rational::new(7, 2));

    let b = Interval::parse("M6").unwrap();
    assert_eq!(b.sign, 1);
    assert_eq!(b.steps, 5);
    assert_eq!(b.distance, Rational::new(9, 1));

    let c = Interval::parse("d4").unwrap();
    assert_eq!(c.sign, 1);
    assert_eq!(c.steps, 3);
    assert_eq!(c.distance, Rational::new(4, 1));

    let d = Interval::parse("d12-2").unwrap();
    assert_eq!(d.sign, 1);
    assert_eq!(d.steps, 11);
    assert_eq!(d.distance, Rational::new(16, 1));
}

#[test]
fn parse_fail() {
    for s in ["", "4", "m3+", "m3+1.5", "m8", "P0"] {
        assert!(Interval::parse(s).is_none(), "expected `{s}` to fail");
    }
}

#[test]
fn abbr() {
    let a = Interval::new(2, Rational::new(7, 2), -1);
    assert_eq!(a.to_abbreviation(false), "-m3+1/2");

    let b = Interval::new(11, Rational::new(16, 1), 1);
    assert_eq!(b.to_abbreviation(false), "d12-2");
    assert_eq!(b.to_abbreviation(true), "+d12-2");

    let c = Interval::new(5, Rational::new(9, 1), 1);
    assert_eq!(c.to_abbreviation(false), "M6");
}

#[test]
fn to_string() {
    let a = Interval::new(3, Rational::new(6, 1), -1);
    assert_eq!(a.to_string(), "augmented fourth downward");

    let b = Interval::new(11, Rational::new(16, 1), 1);
    assert_eq!(b.to_string(), "triply-diminished twelfth");
    assert_eq!(
        b.to_verbose_string(true),
        "triply-diminished twelfth upward"
    );

    assert_eq!(
        Interval::new(5, Rational::new(9, 1), 1).to_string(),
        "major sixth"
    );
    assert_eq!(
        Interval::new(11, Rational::new(21, 1), 1).to_string(),
        "doubly-augmented twelfth"
    );
    assert_eq!(
        Interval::new(11, Rational::new(41, 2), 1).to_string(),
        "3/2×-augmented twelfth"
    );
    assert_eq!(
        Interval::new(13, Rational::new(23, 1), 1).to_string(),
        "major 14th"
    );

    assert_eq!(
        Interval::new(7, Rational::new(12, 1), 1).to_string(),
        "octave"
    );
    assert_eq!(
        Interval::new(14, Rational::new(24, 1), 1).to_string(),
        "double octave"
    );
    assert_eq!(
        Interval::new(35, Rational::new(60, 1), 1).to_string(),
        "5-octave"
    );
    assert_eq!(
        Interval::new(14, Rational::new(25, 1), 1).to_string(),
        "augmented double octave"
    );
}

#[test]
fn equality() {
    assert_eq!(
        Interval::parse("M3+1/4").unwrap(),
        Interval::parse("M3+1/4").unwrap()
    );
    assert_eq!(
        Interval::parse("m3+1/2").unwrap(),
        Interval::parse("M3-1/2").unwrap()
    );
    assert_ne!(
        Interval::parse("M3").unwrap(),
        Interval::parse("d4").unwrap()
    );

    assert!(Interval::parse("M3+1/4")
        .unwrap()
        .equals_enharmonically(&Interval::parse("M3+1/4").unwrap()));
    assert!(Interval::parse("m3+1/2")
        .unwrap()
        .equals_enharmonically(&Interval::parse("M3-1/2").unwrap()));
    assert!(Interval::parse("M3")
        .unwrap()
        .equals_enharmonically(&Interval::parse("d4").unwrap()));
}

#[test]
fn add() {
    assert_eq!(
        Interval::parse("m3")
            .unwrap()
            .add(&Interval::parse("m3").unwrap()),
        Interval::parse("d5").unwrap()
    );
    assert_eq!(
        Interval::parse("M3")
            .unwrap()
            .add(&Interval::parse("-m3").unwrap()),
        Interval::parse("A1").unwrap()
    );
    assert_eq!(
        Interval::parse("-M3")
            .unwrap()
            .add(&Interval::parse("m3").unwrap()),
        Interval::parse("-A1").unwrap()
    );
    assert_eq!(
        Interval::parse("P1")
            .unwrap()
            .add(&Interval::parse("-A1").unwrap()),
        Interval::parse("-A1").unwrap()
    );

    assert_eq!(
        Interval::parse("d2")
            .unwrap()
            .add(&Interval::parse("d2").unwrap()),
        Interval::parse("d3-2").unwrap()
    );
    assert_eq!(
        Interval::parse("-d2")
            .unwrap()
            .add(&Interval::parse("-d2").unwrap()),
        Interval::parse("-d3-2").unwrap()
    );
}

#[test]
fn add_period() {
    assert_eq!(
        Interval::parse("A3").unwrap().add_period(0),
        Interval::parse("A3").unwrap()
    );
    assert_eq!(
        Interval::parse("d3").unwrap().add_period(10),
        Interval::parse("d73").unwrap()
    );
    assert_eq!(
        Interval::parse("d73").unwrap().add_period(-10),
        Interval::parse("d3").unwrap()
    );
}

#[test]
fn negate_abs() {
    let x = Interval::parse("M3+1/4").unwrap();
    assert_eq!(x.negate(), Interval::parse("-M3+1/4").unwrap());
    assert_eq!(x.negate().negate(), x);
    assert_eq!(x.negate().abs(), x);
}

#[test]
fn to_simple() {
    let x = Interval::parse("M17").unwrap();
    assert_eq!(x.to_simple(None), Interval::parse("M3").unwrap());
    assert_eq!(x.to_simple(Some(7)), Interval::parse("M3").unwrap());
    assert_eq!(x.to_simple(Some(12)), Interval::parse("M10").unwrap());
    assert_eq!(x.to_simple(Some(17)), Interval::parse("M17").unwrap());

    let y = Interval::parse("d8").unwrap();
    assert_eq!(y.to_simple(None), Interval::parse("d8").unwrap());
}

#[test]
fn matches() {
    let x = Interval::parse("M10").unwrap();
    assert!(!x.matches(&Interval::parse("M3").unwrap()));
    assert!(!x.matches_enharmonically(&Interval::parse("M3").unwrap()));

    assert!(!x.matches(&Interval::parse("M16").unwrap()));
    assert!(!x.matches_enharmonically(&Interval::parse("M16").unwrap()));

    assert!(x.matches(&Interval::parse("M10").unwrap()));
    assert!(!x.matches(&Interval::parse("d11").unwrap()));
    assert!(x.matches_enharmonically(&Interval::parse("d11").unwrap()));

    assert!(x.matches(&Interval::parse("M17").unwrap()));
    assert!(!x.matches(&Interval::parse("d18").unwrap()));
    assert!(x.matches_enharmonically(&Interval::parse("d18").unwrap()));
}
