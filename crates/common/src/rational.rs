use num_rational::Ratio;

/// An exact rational number, always reduced on construction and `Copy`.
pub type Rational = Ratio<i64>;

/// Construct an integer [`Rational`] as a constant.
#[must_use]
pub const fn rational(n: i64) -> Rational {
    Ratio::new_raw(n, 1)
}

/// Convert a [`Rational`] to its nearest `f64` value.
#[must_use]
pub fn rational_value(r: Rational) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

/// Format a [`Rational`] as a string, optionally always-signed or as a mixed
/// fraction.
#[must_use]
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
