/// Standard mathematical modulo (handles negative numbers correctly).
///
/// ```text
/// modulo(-1, 7) -> 6
/// ```
#[must_use]
pub fn modulo(n: i64, m: i64) -> i64 {
    n.rem_euclid(m)
}

/// Rotate a slice by `n` steps. Positive `n` means left-shift, negative right-shift.
///
/// This is the replacement for the original `rotateArray`, using [`slice::rotate_left`].
pub fn rotate_array<T: Clone>(list: &[T], n: i64) -> Vec<T> {
    if list.is_empty() {
        return Vec::new();
    }
    let mut out = list.to_vec();
    out.rotate_left(n.rem_euclid(list.len() as i64) as usize);
    out
}
