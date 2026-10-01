use passacaglia_common::Rational;

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
