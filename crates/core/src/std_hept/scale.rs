use std::fmt;

use crate::degree::Degree;
use crate::scale::Scale;
use crate::std_hept::accidental;
use crate::std_hept::parse;
use crate::std_hept::system::StdHeptatonic;

// FIXME: we assume indices don't go up over 10
const ROMAN_NUMERALS: [&str; 10] = ["i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x"];

impl Scale<StdHeptatonic> {
    /// Parse a scale-degree expression. The degree index can be written as a Roman numeral (1-10 
    /// are supported), or as a positive integer in square brackets. The accidental (alteration) is
    /// written like in a pitch expression.
    /// 
    /// To be compatible with music notation conventions, indices are 1-based despite the fact that 
    /// [`Degree`] actually uses 0-based indices.
    /// 
    /// Examples: `i` (=`[1]`), `viis` (=`[7]s`), `ix2/3f` (=`[9]2/3f`)
    #[must_use]
    pub fn parse_degree(&self, ex: &str) -> Option<Degree<'_, StdHeptatonic>> {
        let ex = ex.to_ascii_lowercase();
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
        Some(self.at(idx).with_acci(acci))
    }
}

impl fmt::Display for Degree<'_, StdHeptatonic> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            ROMAN_NUMERALS[self.index],
            accidental::print(self.acci)
        )
    }
}

impl Degree<'_, StdHeptatonic> {
    #[must_use]
    pub fn to_arabic_string(&self) -> String {
        format!("[{}]{}", self.index + 1, accidental::print(self.acci))
    }
}
