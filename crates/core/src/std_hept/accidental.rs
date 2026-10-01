use num_rational::Ratio;

use passacaglia_common::Rational;
use passacaglia_parser::{parse_accidental, ParseError};

/// Parse an accidental expression (`''`, `n`, `s+`, `f+`, `3f`, `3/4s`, …).
pub fn parse(s: &str) -> Result<Rational, ParseError> {
    let (num, den) = parse_accidental(s)?;
    Ok(Ratio::new(num, den))
}

/// Print an accidental (`s` for sharps, `f` for flats).
pub fn print(x: Rational) -> String {
    let num = *x.numer();
    if num == 0 {
        return String::new();
    }
    let abs_num = num.unsigned_abs();
    let letter = if num > 0 { 's' } else { 'f' };
    if *x.denom() == 1 && abs_num <= 2 {
        return letter.to_string().repeat(abs_num as usize);
    }
    if *x.denom() == 1 {
        return format!("{abs_num}{letter}");
    }
    format!("{abs_num}/{}{}", x.denom(), letter)
}
