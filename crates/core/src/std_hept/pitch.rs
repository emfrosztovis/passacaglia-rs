use std::fmt;
use std::str::FromStr;

use num_traits::Signed;

use passacaglia_common::rational;

use crate::interval::Interval;
use crate::pitch::Pitch;
use crate::std_hept::accidental;
use crate::std_hept::parse::{parse_pitch, ParseError};
use crate::std_hept::system::StdHeptatonic;
use crate::system::PitchSystem;

impl FromStr for Pitch<StdHeptatonic> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_pitch(s)
    }
}

impl Pitch<StdHeptatonic> {
    /// Parses a string expression of pitch, in the format of note name +
    /// accidental + single-digit octave number.
    ///
    /// For accidentals, use `s` for sharps and `f` for flats. For more than one
    /// sharps or flats, either duplicate the letter or add a number like `3f`.
    /// For microtonal accidentals, write out a fraction like `3/4s`. An empty
    /// accidental or `n` is parsed as `0` (natural).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// parse("c")      // C0 natural
    /// parse("c4")     // C4 natural
    /// parse("gff3")   // G3 double-flat
    /// parse("g3f3")   // G3 triple-flat
    /// parse("e2/3s6") // E6 two-thirds sharp
    /// ```
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::from_str(s).ok()
    }

    /// Returns the positive simple interval between two pitches. For example, it returns a minor 
    /// second for C0 and B0.
    #[must_use]
    pub fn absolute_simple_interval_to(&self, b: &Self) -> Interval<StdHeptatonic> {
        let a = self.with_period(0);
        let b = b.with_period(i32::from(a.index > b.index));
        a.interval_to(&b).abs()
    }

    /// Returns a readable string form, in the format described in the docs for [`Pitch::parse`], 
    /// but omitting the octave.
    #[must_use]
    pub fn to_class_string(&self) -> String {
        const NAMES: [&str; 7] = ["c", "d", "e", "f", "g", "a", "b"];
        format!("{}{}", NAMES[self.index], accidental::print(self.acci))
    }

    /// Normalize the pitch so that it uses at most a single accidental (i.e. `acci.abs()` < 2).
    #[must_use]
    pub fn normalize(&self) -> Self {
        let acci = self.acci;
        if acci.abs() < rational(2) {
            return *self;
        }
        let direction: i64 = if *acci.numer() > 0 { 1 } else { -1 };
        let target = acci + StdHeptatonic::DEGREE_OFFSETS[self.index];

        let mut deg = self.index as i64;
        let mut delta_period = 0i64;
        let mut acci = acci;
        while acci.abs() > rational(2) {
            deg += direction;
            if deg >= StdHeptatonic::N_DEGREES as i64 {
                deg = 0;
                delta_period += 1;
            }
            if deg < 0 {
                deg = StdHeptatonic::N_DEGREES as i64 - 1;
                delta_period -= 1;
            }
            acci = target
                - StdHeptatonic::DEGREE_OFFSETS[deg as usize]
                - rational(delta_period * StdHeptatonic::N_PITCH_CLASSES as i64);
        }
        Pitch::new(deg as usize, acci, self.period + delta_period as i32)
    }
}

/// Returns a readable string form, in the format described in the docs for [`Pitch::parse`].
impl fmt::Display for Pitch<StdHeptatonic> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const NAMES: [&str; 7] = ["c", "d", "e", "f", "g", "a", "b"];
        write!(
            f,
            "{}{}{}",
            NAMES[self.index],
            accidental::print(self.acci),
            self.period
        )
    }
}
