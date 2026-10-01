use passacaglia_parser::{parse_accidental, parse_interval, parse_pitch, parse_rational};

#[test]
fn pitch_parse() {
    let p = parse_pitch("c4").unwrap();
    assert_eq!((p.index, p.acci_num, p.acci_den, p.period), (0, 0, 1, 4));

    let p = parse_pitch("g6/17s7").unwrap();
    assert_eq!((p.index, p.acci_num, p.acci_den, p.period), (4, 6, 17, 7));

    assert!(parse_pitch("do4").is_err());
    assert!(parse_pitch("h5").is_err());
    assert!(parse_pitch("c3+1").is_err());
}

#[test]
fn accidental_parse() {
    assert_eq!(parse_accidental("").unwrap(), (0, 1));
    assert_eq!(parse_accidental("n").unwrap(), (0, 1));
    assert_eq!(parse_accidental("s").unwrap(), (1, 1));
    assert_eq!(parse_accidental("sss").unwrap(), (3, 1));
    assert_eq!(parse_accidental("ff").unwrap(), (-2, 1));
    assert_eq!(parse_accidental("5f").unwrap(), (-5, 1));
    assert_eq!(parse_accidental("3/4s").unwrap(), (3, 4));
    assert_eq!(parse_accidental("6/8s").unwrap(), (3, 4));
    assert!(parse_accidental("x").is_err());
}

#[test]
fn rational_parse() {
    assert_eq!(parse_rational("5").unwrap(), (5, 1));
    assert_eq!(parse_rational("-3/4").unwrap(), (-3, 4));
    assert_eq!(parse_rational("+7/2").unwrap(), (7, 2));
    assert_eq!(parse_rational("6/8").unwrap(), (3, 4));
    assert!(parse_rational("1/0").is_err());
    assert!(parse_rational("12/").is_err());
    assert!(parse_rational("").is_err());
}

#[test]
fn interval_parse() {
    let i = parse_interval("-m3+1/2").unwrap();
    assert_eq!(
        (i.steps, i.distance_num, i.distance_den, i.sign),
        (2, 7, 2, -1)
    );

    let d = parse_interval("d12-2").unwrap();
    assert_eq!(
        (d.steps, d.distance_num, d.distance_den, d.sign),
        (11, 16, 1, 1)
    );

    for s in ["", "4", "m3+", "m3+1.5", "m8", "P0"] {
        assert!(parse_interval(s).is_err(), "expected `{s}` to fail");
    }
}
