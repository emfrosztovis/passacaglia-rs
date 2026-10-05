use std::fmt;

use num_rational::Ratio;

use passacaglia_common::Rational;

use crate::interval::Interval;
use crate::pitch::Pitch;
use crate::std_hept::system::StdHeptatonic;

/// An error produced when a literal string does not conform to the grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        ParseError(message.into())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

/// Interval quality, as used in standard heptatonic interval naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Quality {
    Perfect,
    Major,
    Minor,
    Augmented,
    Diminished,
}

impl Quality {
    pub(crate) fn abbr(self) -> char {
        match self {
            Quality::Perfect => 'P',
            Quality::Major => 'M',
            Quality::Minor => 'm',
            Quality::Augmented => 'A',
            Quality::Diminished => 'd',
        }
    }

    pub(crate) fn from_abbr(c: u8) -> Option<Quality> {
        Some(match c {
            b'P' => Quality::Perfect,
            b'M' => Quality::Major,
            b'm' => Quality::Minor,
            b'A' => Quality::Augmented,
            b'd' => Quality::Diminished,
            _ => return None,
        })
    }

    pub(crate) fn word(self) -> &'static str {
        match self {
            Quality::Perfect => "perfect",
            Quality::Major => "major",
            Quality::Minor => "minor",
            Quality::Augmented => "augmented",
            Quality::Diminished => "diminished",
        }
    }
}

const INTERVAL_DATA: [&[(i64, Quality)]; 8] = [
    &[(0, Quality::Perfect), (1, Quality::Augmented)], // unisons
    &[
        (0, Quality::Diminished),
        (1, Quality::Minor),
        (2, Quality::Major),
        (3, Quality::Augmented),
    ], // seconds
    &[
        (2, Quality::Diminished),
        (3, Quality::Minor),
        (4, Quality::Major),
        (5, Quality::Augmented),
    ], // thirds
    &[
        (4, Quality::Diminished),
        (5, Quality::Perfect),
        (6, Quality::Augmented),
    ], // fourths
    &[
        (6, Quality::Diminished),
        (7, Quality::Perfect),
        (8, Quality::Augmented),
    ], // fifths
    &[
        (7, Quality::Diminished),
        (8, Quality::Minor),
        (9, Quality::Major),
        (10, Quality::Augmented),
    ], // sixths
    &[
        (9, Quality::Diminished),
        (10, Quality::Minor),
        (11, Quality::Major),
        (12, Quality::Augmented),
    ], // sevenths
    &[
        (11, Quality::Diminished),
        (12, Quality::Perfect),
        (13, Quality::Augmented),
    ], // octaves
];

pub(crate) fn interval_data(simple_steps: usize) -> &'static [(i64, Quality)] {
    INTERVAL_DATA.get(simple_steps).copied().unwrap_or(&[])
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn reduce(num: i64, den: i64) -> (i64, i64) {
    assert!(den > 0);
    let g = gcd(num.unsigned_abs(), den as u64) as i64;
    (num / g, den / g)
}

fn parse_unsigned(s: &str) -> Result<u64, ParseError> {
    s.parse::<u64>()
        .map_err(|_| ParseError::new(format!("invalid number `{s}`")))
}

/// Split a string into `(prefix, suffix)` where `suffix` is the longest trailing
/// run of ASCII digits (possibly empty).
fn split_trailing_digits(s: &str) -> (&str, &str) {
    let bytes = s.as_bytes();
    let mut i = s.len();
    while i > 0 && bytes[i - 1].is_ascii_digit() {
        i -= 1;
    }
    (&s[..i], &s[i..])
}

/// Parse an accidental expression.
///
/// Grammar: `''` / `'n'`, `s+`, `f+`, or `(\d+)(/(\d+))?(s|f)+`.
pub(crate) fn parse_accidental(s: &str) -> Result<Rational, ParseError> {
    if s.is_empty() || s == "n" {
        return Ok(Ratio::new(0, 1));
    }

    let bytes = s.as_bytes();
    if !bytes.is_empty() && bytes.iter().all(|&b| b == b's') {
        return Ok(Ratio::new(bytes.len() as i64, 1));
    }
    if !bytes.is_empty() && bytes.iter().all(|&b| b == b'f') {
        return Ok(Ratio::new(-(bytes.len() as i64), 1));
    }

    // ^(\d+)(?:/(\d+))?(s|f)+$
    let n = bytes.len();
    let mut i = 0;

    let num_start = i;
    while i < n && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == num_start {
        return Err(ParseError::new(format!("invalid accidental `{s}`")));
    }
    let magnitude = parse_unsigned(&s[num_start..i])?;

    let mut den: u64 = 1;
    if i < n && bytes[i] == b'/' {
        i += 1;
        let den_start = i;
        while i < n && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == den_start {
            return Err(ParseError::new(format!("invalid accidental `{s}`")));
        }
        den = parse_unsigned(&s[den_start..i])?;
    }

    if i >= n || (bytes[i] != b's' && bytes[i] != b'f') {
        return Err(ParseError::new(format!("invalid accidental `{s}`")));
    }
    let mut last = bytes[i];
    i += 1;
    while i < n && (bytes[i] == b's' || bytes[i] == b'f') {
        last = bytes[i];
        i += 1;
    }
    if i != n {
        return Err(ParseError::new(format!("invalid accidental `{s}`")));
    }

    let sign = if last == b's' { 1 } else { -1 };
    Ok(Ratio::new(magnitude as i64 * sign, den as i64))
}

/// Parse a rational literal (`^([+-]?\d+)(?:/(\d+))?$`), reduced.
pub(crate) fn parse_rational(s: &str) -> Result<(i64, i64), ParseError> {
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0;

    let mut sign = 1i64;
    if i < n && (bytes[i] == b'+' || bytes[i] == b'-') {
        if bytes[i] == b'-' {
            sign = -1;
        }
        i += 1;
    }

    let num_start = i;
    while i < n && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == num_start {
        return Err(ParseError::new(format!("invalid rational `{s}`")));
    }
    let num = parse_unsigned(&s[num_start..i])? as i64 * sign;

    let mut den: i64 = 1;
    if i < n && bytes[i] == b'/' {
        i += 1;
        let den_start = i;
        while i < n && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == den_start {
            return Err(ParseError::new(format!("invalid rational `{s}`")));
        }
        den = parse_unsigned(&s[den_start..i])? as i64;
    }

    if i != n {
        return Err(ParseError::new(format!("invalid rational `{s}`")));
    }
    if den == 0 {
        return Err(ParseError::new(format!("zero denominator in `{s}`")));
    }

    Ok(reduce(num, den))
}

/// Parse a pitch literal: `^([a-g])([\d\/sf]*?)(\d+)?$` (case-insensitive).
pub(crate) fn parse_pitch(s: &str) -> Result<Pitch<StdHeptatonic>, ParseError> {
    let lower = s.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    if bytes.is_empty() {
        return Err(ParseError::new(format!("empty pitch `{s}`")));
    }

    let index = match bytes[0] {
        b'c' => 0,
        b'd' => 1,
        b'e' => 2,
        b'f' => 3,
        b'g' => 4,
        b'a' => 5,
        b'b' => 6,
        _ => return Err(ParseError::new(format!("invalid pitch `{s}`"))),
    };

    let (acc, oct) = split_trailing_digits(&lower[1..]);
    let acci = parse_accidental(acc)?;

    let period = if oct.is_empty() {
        0
    } else {
        let o = parse_unsigned(oct)?;
        if o > i32::MAX as u64 {
            return Err(ParseError::new(format!("octave out of range in `{s}`")));
        }
        o as i32
    };

    Ok(Pitch::new(index, acci, period))
}

/// Parse an interval abbreviation: `^([+-])?([PMmAd])(\d+)([+-]?\d+(?:\/\d+)?)?$`.
pub(crate) fn parse_interval(s: &str) -> Result<Interval<StdHeptatonic>, ParseError> {
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0;

    let mut sign: i8 = 1;
    if i < n && (bytes[i] == b'+' || bytes[i] == b'-') {
        if bytes[i] == b'-' {
            sign = -1;
        }
        i += 1;
    }

    if i >= n {
        return Err(ParseError::new(format!("invalid interval `{s}`")));
    }
    let quality = Quality::from_abbr(bytes[i])
        .ok_or_else(|| ParseError::new(format!("invalid interval `{s}`")))?;
    i += 1;

    let num_start = i;
    while i < n && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == num_start {
        return Err(ParseError::new(format!("invalid interval `{s}`")));
    }
    let number = parse_unsigned(&s[num_start..i])?;
    if number == 0 {
        return Err(ParseError::new(format!("invalid interval `{s}`")));
    }

    let mut remainder: (i64, i64) = (0, 1);
    if i < n {
        remainder = parse_rational(&s[i..])?;
    }

    let steps = number - 1;
    let mut octaves = steps / 7;
    let mut simple_steps = steps % 7;
    if simple_steps == 0 && octaves > 0 {
        simple_steps = 7;
        octaves -= 1;
    }

    let simple_semitones = interval_data(simple_steps as usize)
        .iter()
        .find(|(_, q)| *q == quality)
        .map(|(semi, _)| *semi)
        .ok_or_else(|| ParseError::new(format!("invalid interval `{s}`")))?;

    let whole = simple_semitones + octaves as i64 * 12;
    let (rem_num, rem_den) = remainder;
    let (num, den) = reduce(rem_num + whole * rem_den, rem_den);

    Ok(Interval::new(steps as usize, Ratio::new(num, den), sign))
}
