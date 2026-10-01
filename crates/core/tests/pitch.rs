use passacaglia_common::{rational, Rational};
use passacaglia_core::std_hept::{Interval, Pitch};

#[test]
fn parse() {
    assert_eq!(Pitch::parse("c").unwrap().ord(), rational(0));
    assert_eq!(Pitch::parse("C10").unwrap().ord(), rational(120));
    assert_eq!(Pitch::parse("eff4").unwrap().ord(), rational(50));
    assert_eq!(Pitch::parse("esss4").unwrap().ord(), rational(55));
    assert_eq!(Pitch::parse("e5f4").unwrap().ord(), rational(47));
    assert_eq!(Pitch::parse("c1/4s4").unwrap().ord(), Rational::new(193, 4));
}

#[test]
fn to_midi() {
    assert_eq!(Pitch::parse("C4").unwrap().to_midi(), rational(60));
}

#[test]
fn parse_fail() {
    assert!(Pitch::parse("do4").is_none());
    assert!(Pitch::parse("h5").is_none());
    assert!(Pitch::parse("c3+1").is_none());
}

#[test]
fn to_string() {
    assert_eq!(Pitch::parse("c4").unwrap().to_string(), "c4");
    assert_eq!(Pitch::parse("c3f0").unwrap().to_string(), "c3f0");
    assert_eq!(Pitch::parse("gss7").unwrap().to_string(), "gss7");
    assert_eq!(Pitch::parse("g6/17s7").unwrap().to_string(), "g6/17s7");
}

#[test]
fn equality() {
    assert_eq!(
        Pitch::parse("c4").unwrap(),
        Pitch::parse("c4").unwrap()
    );
    assert_eq!(
        Pitch::parse("d12/34s5").unwrap(),
        Pitch::parse("d6/17s5").unwrap()
    );
    assert_ne!(
        Pitch::parse("e3/8s5").unwrap(),
        Pitch::parse("f5/8f5").unwrap()
    );

    assert!(Pitch::parse("d12/34s5")
        .unwrap()
        .enharmonically_equals(&Pitch::parse("d6/17s5").unwrap()));
    assert!(Pitch::parse("e3/8s5")
        .unwrap()
        .enharmonically_equals(&Pitch::parse("f5/8f5").unwrap()));
}

#[test]
fn normalize() {
    assert_eq!(Pitch::parse("cff4").unwrap().normalize().to_string(), "cff4");
    assert_eq!(Pitch::parse("csss4").unwrap().normalize().to_string(), "ds4");
    assert_eq!(Pitch::parse("cfff4").unwrap().normalize().to_string(), "bff3");
    assert_eq!(Pitch::parse("b12s4").unwrap().normalize().to_string(), "ass5");
}

#[test]
fn add_interval() {
    assert_eq!(
        Pitch::parse("c4")
            .unwrap()
            .add(&Interval::parse("m3").unwrap())
            .to_string(),
        "ef4"
    );
    assert_eq!(
        Pitch::parse("cs4")
            .unwrap()
            .add(&Interval::parse("m3").unwrap())
            .to_string(),
        "e4"
    );
    assert_eq!(
        Pitch::parse("c4")
            .unwrap()
            .add(&Interval::parse("M3").unwrap())
            .to_string(),
        "e4"
    );
    assert_eq!(
        Pitch::parse("c4")
            .unwrap()
            .add(&Interval::parse("d6").unwrap())
            .to_string(),
        "aff4"
    );
    assert_eq!(
        Pitch::parse("c4")
            .unwrap()
            .add(&Interval::parse("d6+1/4").unwrap())
            .to_string(),
        "a7/4f4"
    );
}

#[test]
fn interval_to() {
    let a = Pitch::parse("Ef4")
        .unwrap()
        .interval_to(&Pitch::parse("Gss4").unwrap());
    assert_eq!(a.to_abbreviation(false), "A3+1");

    let b = Pitch::parse("F4")
        .unwrap()
        .interval_to(&Pitch::parse("B3").unwrap());
    assert_eq!(b.to_abbreviation(false), "-d5");

    let x = Pitch::parse("G23/45s4").unwrap();
    let y = Pitch::parse("C45/67f3").unwrap();
    assert_eq!(x.add(&x.interval_to(&y)), y);
}
