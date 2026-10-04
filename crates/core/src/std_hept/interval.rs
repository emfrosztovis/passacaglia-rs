use std::fmt;
use std::str::FromStr;

use num_traits::Signed;

use passacaglia_common::{rational, rational_to_string, Rational};

use crate::interval::Interval;
use crate::std_hept::parse::{interval_data, parse_interval, ParseError, Quality};
use crate::std_hept::system::StdHeptatonic;

const MULTIPLIER_ADVERBS: [&str; 4] = ["", "", "doubly", "triply"];
const MULTIPLIERS: [&str; 4] = ["", "single", "double", "triple"];
const ORDINALS: [&str; 13] = [
    "unison",
    "second",
    "third",
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "octave",
    "ninth",
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
];

fn get_multiplier_adverb(n: Rational, word: &str) -> String {
    debug_assert!(*n.numer() != 0);
    if *n.numer() == 1 {
        return word.to_string();
    }
    if *n.denom() == 1 && *n.numer() < MULTIPLIER_ADVERBS.len() as i64 {
        return format!("{}-{word}", MULTIPLIER_ADVERBS[*n.numer() as usize]);
    }
    format!("{n}×-{word}")
}

fn get_multiplier(n: Rational, word: &str) -> String {
    debug_assert!(*n.numer() != 0);
    if *n.numer() == 1 {
        return word.to_string();
    }
    if *n.denom() == 1 && *n.numer() < MULTIPLIERS.len() as i64 {
        return format!("{} {word}", MULTIPLIERS[*n.numer() as usize]);
    }
    format!("{n}-{word}")
}

fn steps_to_ordinal(n: usize) -> String {
    if n < ORDINALS.len() {
        return ORDINALS[n].to_string();
    }
    if n.is_multiple_of(7) {
        return get_multiplier(rational((n / 7) as i64), "octave");
    }
    let ord = n + 1;
    if ord % 10 == 1 {
        return format!("{ord}st");
    }
    if ord % 10 == 2 {
        return format!("{ord}nd");
    }
    if ord % 10 == 3 {
        return format!("{ord}rd");
    }
    format!("{ord}th")
}

/// Find the closest well-known interval (a `(difference, quality)` pair) by
/// comparing against the interval data for the simple interval.
fn get_closest_well_known(int: &Interval<StdHeptatonic>) -> (Rational, Quality) {
    let simple = int.to_simple(Some(7));

    let mut diff: Option<Rational> = None;
    let mut q: Option<Quality> = None;
    for &(semitones, quality) in interval_data(simple.steps) {
        let d = simple.distance - rational(semitones);
        let better = match diff {
            None => true,
            Some(cur) => {
                let d_abs = d.abs();
                let cur_abs = cur.abs();
                d_abs < cur_abs
                    || (d_abs == cur_abs
                        && ((*d.numer() < 0 && quality == Quality::Diminished)
                            || (*d.numer() > 0 && quality == Quality::Augmented)))
            }
        };
        if better {
            diff = Some(d);
            q = Some(quality);
        }
    }

    let diff = diff.expect("well-known interval data is non-empty");
    let q = q.expect("well-known interval data is non-empty");
    (diff, q)
}

impl FromStr for Interval<StdHeptatonic> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_interval(s)
    }
}

impl Interval<StdHeptatonic> {
    /// Parses an interval abbreviation in the format of sign (optional) + quality +
    /// number + further semitone differences (optional). Available qualities are `P`
    /// (perfect), `M` (major), `m` (minor), `A` (augmented) and `d` (diminished).
    ///
    /// "Further semitone differences" consists of a sign (`+` or `-`) and an
    /// integer or a fraction. For example, a doubly augmented third is `A3+1`. In
    /// this way you can also express complex intervals that have no official
    /// names, such as `d12+1/4`.
    ///
    /// The algorithm does *not* distinguish between intervals with the same steps
    /// and same semitones, such as `m3+1/2` and `M3-1/2`. They parse to the same
    /// interval object.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::from_str(s).ok()
    }

    /// Returns a readable abbreviation of the interval, in the format described in the docs for 
    /// [`Interval::parse`].
    #[must_use]
    pub fn to_abbreviation(&self, always_signed: bool) -> String {
        let (diff, q) = get_closest_well_known(self);
        let quality = q.abbr();
        let remainder = if *diff.numer() == 0 {
            String::new()
        } else {
            rational_to_string(diff, true, false)
        };
        let sign = if self.sign > 0 && !always_signed {
            ""
        } else if self.sign < 0 {
            "-"
        } else {
            "+"
        };
        format!("{sign}{quality}{}{remainder}", self.steps + 1)
    }

    /// Returns a verbose readable form of the interval in English, such as "major sixth" or 
    /// "doubly-diminished fifth downward". The qualifier "upward" is omitted unless `always_signed`
    /// is set to true.
    #[must_use]
    pub fn to_verbose_string(&self, always_signed: bool) -> String {
        let (diff, q) = get_closest_well_known(self);
        let name = steps_to_ordinal(self.steps);

        let quality = if self.steps.is_multiple_of(7) && q == Quality::Perfect {
            String::new()
        } else {
            format!("{} ", q.word())
        };

        let main = if *diff.numer() == 0 {
            format!("{quality}{name}")
        } else if (*diff.numer() <= 0 && q == Quality::Diminished)
            || (*diff.numer() >= 0 && q == Quality::Augmented)
        {
            format!(
                "{}{name}",
                get_multiplier_adverb(diff.abs() + rational(1), &quality)
            )
        } else {
            format!("{quality}{name} {}", rational_to_string(diff, true, true))
        };

        let sign = if self.sign > 0 && !always_signed {
            ""
        } else if self.sign < 0 {
            " downward"
        } else {
            " upward"
        };

        format!("{main}{sign}")
    }
}

impl fmt::Display for Interval<StdHeptatonic> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_verbose_string(false))
    }
}
