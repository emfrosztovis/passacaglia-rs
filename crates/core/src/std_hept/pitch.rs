use std::fmt;
use std::str::FromStr;

use num_traits::Signed;

use passacaglia_common::{rational, Rational};
use passacaglia_parser::{parse_pitch, ParseError};

use crate::interval::Interval;
use crate::pitch::Pitch;
use crate::std_hept::accidental;
use crate::std_hept::system::StandardHeptatonic;
use crate::system::PitchSystem;

impl FromStr for Pitch<StandardHeptatonic> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = parse_pitch(s)?;
        Ok(Pitch::new(
            p.index,
            Rational::new(p.acci_num, p.acci_den),
            p.period,
        ))
    }
}

impl Pitch<StandardHeptatonic> {
    /// Parse a pitch literal, or `None` on failure.
    pub fn parse(s: &str) -> Option<Self> {
        Self::from_str(s).ok()
    }

    /// The positive simple interval between two pitches (period ignored).
    pub fn absolute_simple_interval_to(&self, b: &Self) -> Interval<StandardHeptatonic> {
        let a = self.with_period(0);
        let b = b.with_period(0);
        a.interval_to(&b).abs()
    }

    /// Rewrite using at most double accidentals (`abs(acci) <= 2`).
    pub fn normalize(&self) -> Self {
        let acci = self.acci;
        if acci.abs() < rational(2) {
            return *self;
        }
        let direction: i64 = if *acci.numer() > 0 { 1 } else { -1 };
        let target = acci + StandardHeptatonic::DEGREE_OFFSETS[self.index];

        let mut deg = self.index as i64;
        let mut delta_period = 0i64;
        let mut acci = acci;
        while acci.abs() > rational(2) {
            deg += direction;
            if deg >= StandardHeptatonic::N_DEGREES as i64 {
                deg = 0;
                delta_period += 1;
            }
            if deg < 0 {
                deg = StandardHeptatonic::N_DEGREES as i64 - 1;
                delta_period -= 1;
            }
            acci = target
                - StandardHeptatonic::DEGREE_OFFSETS[deg as usize]
                - rational(delta_period * StandardHeptatonic::N_PITCH_CLASSES as i64);
        }
        Pitch::new(deg as usize, acci, self.period + delta_period as i32)
    }
}

impl fmt::Display for Pitch<StandardHeptatonic> {
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
