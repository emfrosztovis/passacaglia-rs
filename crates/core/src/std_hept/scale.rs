use std::fmt;

use passacaglia_common::rational;

use crate::degree::Degree;
use crate::interval::Interval;
use crate::pitch::Pitch;
use crate::scale::Scale;
use crate::std_hept::accidental;
use crate::std_hept::parse;
use crate::std_hept::system::StandardHeptatonic;
use crate::system::PitchSystem;

const ROMAN_NUMERALS: [&str; 10] = ["i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x"];

impl Scale<StandardHeptatonic> {
    /// Build a scale from a root and its intervals (the last wraps back to the root).
    #[must_use]
    pub fn from_intervals(
        root: Pitch<StandardHeptatonic>,
        intervals: &[Interval<StandardHeptatonic>],
    ) -> Self {
        let mut degrees = vec![root];
        let mut current = root;
        for (i, int) in intervals.iter().enumerate() {
            debug_assert_eq!(int.sign, 1);
            current = current.add(int);
            if i == intervals.len() - 1 {
                debug_assert_eq!(
                    root.distance_to(&current),
                    rational(StandardHeptatonic::N_PITCH_CLASSES as i64)
                );
            } else {
                degrees.push(current);
            }
        }
        Scale::new(degrees, intervals.to_vec())
    }

    /// Build a scale from its degrees (the wrap interval is inferred).
    #[must_use]
    pub fn from_pitches(degrees: &[Pitch<StandardHeptatonic>]) -> Self {
        let mut intervals = Vec::with_capacity(degrees.len());
        for i in 1..degrees.len() {
            let int = degrees[i - 1].interval_to(&degrees[i]);
            debug_assert!(int.sign > 0);
            intervals.push(int);
        }
        let wrap = degrees
            .last()
            .expect("non-empty degrees")
            .interval_to(&degrees[0].add_period(1));
        debug_assert!(wrap.sign > 0);
        debug_assert!(wrap.distance < rational(StandardHeptatonic::N_PITCH_CLASSES as i64));
        intervals.push(wrap);
        Scale::new(degrees.to_vec(), intervals)
    }

    /// Parse a scale-degree expression (roman numeral or `[n]`, plus accidental).
    #[must_use]
    pub fn parse_degree(&self, ex: &str) -> Option<Degree<'_, StandardHeptatonic>> {
        let bytes = ex.as_bytes();
        let (idx, rest) = if bytes.first() == Some(&b'[') {
            let close = ex.find(']')?;
            let num = ex[1..close].parse::<usize>().ok()?;
            if num == 0 {
                return None;
            }
            let idx = num - 1;
            if idx >= self.degrees.len() {
                return None;
            }
            (idx, &ex[close + 1..])
        } else {
            let mut i = 0;
            while i < bytes.len() && matches!(bytes[i], b'i' | b'v' | b'x') {
                i += 1;
            }
            if i == 0 {
                return None;
            }
            let roman = &ex[..i];
            let idx = ROMAN_NUMERALS.iter().position(|r| *r == roman)?;
            (idx, &ex[i..])
        };

        let acci = parse::parse_accidental(rest).ok()?;
        Some(self.at(idx, acci))
    }
}

impl fmt::Display for Degree<'_, StandardHeptatonic> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            ROMAN_NUMERALS[self.index],
            accidental::print(self.acci)
        )
    }
}

impl Degree<'_, StandardHeptatonic> {
    /// The `preferArabic` form (`[n]` instead of roman numerals).
    #[must_use]
    pub fn to_arabic_string(&self) -> String {
        format!("[{}]{}", self.index + 1, accidental::print(self.acci))
    }
}
