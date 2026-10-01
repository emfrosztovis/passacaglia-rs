use crate::ParseError;

/// Interval quality, as used in standard heptatonic interval naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Perfect,
    Major,
    Minor,
    Augmented,
    Diminished,
}

impl Quality {
    /// The single-letter abbreviation (`P`, `M`, `m`, `A`, `d`).
    #[must_use]
    pub fn abbr(self) -> char {
        match self {
            Quality::Perfect => 'P',
            Quality::Major => 'M',
            Quality::Minor => 'm',
            Quality::Augmented => 'A',
            Quality::Diminished => 'd',
        }
    }

    #[must_use]
    pub fn from_abbr(c: u8) -> Option<Quality> {
        Some(match c {
            b'P' => Quality::Perfect,
            b'M' => Quality::Major,
            b'm' => Quality::Minor,
            b'A' => Quality::Augmented,
            b'd' => Quality::Diminished,
            _ => return None,
        })
    }

    /// The full quality name (`perfect`, `major`, `minor`, `augmented`, `diminished`).
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Quality::Perfect => "perfect",
            Quality::Major => "major",
            Quality::Minor => "minor",
            Quality::Augmented => "augmented",
            Quality::Diminished => "diminished",
        }
    }
}

/// The parsed components of a pitch literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitchParts {
    pub index: usize,
    pub acci_num: i64,
    pub acci_den: i64,
    pub period: i32,
}

/// The parsed components of an interval literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntervalParts {
    pub steps: usize,
    pub distance_num: i64,
    pub distance_den: i64,
    pub sign: i8,
}

// ---------------------------------------------------------------------------
// Interval data table (indexed by simple steps 0..=7).
// ---------------------------------------------------------------------------

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

/// The `(semitones, quality)` rows for the given number of simple steps.
#[must_use]
pub fn interval_data(simple_steps: usize) -> &'static [(i64, Quality)] {
    INTERVAL_DATA.get(simple_steps).copied().unwrap_or(&[])
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn reduce(num: i64, den: i64) -> (i64, i64) {
    debug_assert!(den > 0);
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

// ---------------------------------------------------------------------------
// Accidentals
// ---------------------------------------------------------------------------

/// Parse an accidental expression.
///
/// Grammar: `''` / `'n'`, `s+`, `f+`, or `(\d+)(/(\d+))?(s|f)+`.
/// Returns the reduced `(numer, denom)` of the accidental offset.
pub fn parse_accidental(s: &str) -> Result<(i64, i64), ParseError> {
    if s.is_empty() || s == "n" {
        return Ok((0, 1));
    }

    let bytes = s.as_bytes();
    if !bytes.is_empty() && bytes.iter().all(|&b| b == b's') {
        return Ok((bytes.len() as i64, 1));
    }
    if !bytes.is_empty() && bytes.iter().all(|&b| b == b'f') {
        return Ok((-(bytes.len() as i64), 1));
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
    let num = magnitude as i64 * sign;
    let den = den as i64;
    Ok(reduce(num, den))
}

// ---------------------------------------------------------------------------
// Rational
// ---------------------------------------------------------------------------

/// Parse a rational literal (`^([+-]?\d+)(?:/(\d+))?$`), reduced.
pub fn parse_rational(s: &str) -> Result<(i64, i64), ParseError> {
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

// ---------------------------------------------------------------------------
// Pitch
// ---------------------------------------------------------------------------

/// Parse a pitch literal: `^([a-g])([\d\/sf]*?)(\d+)?$` (case-insensitive).
pub fn parse_pitch(s: &str) -> Result<PitchParts, ParseError> {
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
    let (acci_num, acci_den) = parse_accidental(acc)?;

    let period = if oct.is_empty() {
        0
    } else {
        let o = parse_unsigned(oct)?;
        if o > i32::MAX as u64 {
            return Err(ParseError::new(format!("octave out of range in `{s}`")));
        }
        o as i32
    };

    Ok(PitchParts {
        index,
        acci_num,
        acci_den,
        period,
    })
}

// ---------------------------------------------------------------------------
// Interval
// ---------------------------------------------------------------------------

/// Parse an interval abbreviation: `^([+-])?([PMmAd])(\d+)([+-]?\d+(?:\/\d+)?)?$`.
pub fn parse_interval(s: &str) -> Result<IntervalParts, ParseError> {
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

    Ok(IntervalParts {
        steps: steps as usize,
        distance_num: num,
        distance_den: den,
        sign,
    })
}
