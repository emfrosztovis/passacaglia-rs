use num_rational::Ratio;

/// An exact rational number.
///
/// The original TypeScript `Rational` stored `num`/`den` as IEEE-754 `f64`s and
/// performed lossy float-to-rational conversion via `Rational::from`. We replace
/// it with [`num_rational::Ratio<i64>`], which is always reduced on construction,
/// `Copy`, and fully ordered/hashable. All musical values in this domain are tiny
/// integers, so `i64` is more than sufficient.
pub type Rational = Ratio<i64>;

/// Construct an integer [`Rational`] as a constant.
///
/// This is the replacement for the removed `Rational::from` float path; every
/// literal must be written exactly.
pub const fn rational(n: i64) -> Rational {
    Ratio::new_raw(n, 1)
}

/// Convert a [`Rational`] to its nearest `f64` value.
///
/// Only for display or external (floating-point) APIs — never for control flow.
pub fn rational_value(r: Rational) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

/// Format a [`Rational`] following the original `Rational::toString` options.
pub fn rational_to_string(r: Rational, always_signed: bool, mixed_fraction: bool) -> String {
    let sign = if *r.numer() < 0 {
        "-"
    } else if always_signed {
        "+"
    } else {
        ""
    };
    let abs_num = r.numer().unsigned_abs();
    let den = *r.denom() as u64;
    if den == 1 {
        return format!("{sign}{abs_num}");
    }
    if mixed_fraction {
        let whole = abs_num / den;
        if whole == 0 {
            return format!("{sign}{abs_num}/{den}");
        }
        let rem = abs_num - whole * den;
        return format!("{sign}{whole} {rem}/{den}");
    }
    format!("{sign}{abs_num}/{den}")
}
