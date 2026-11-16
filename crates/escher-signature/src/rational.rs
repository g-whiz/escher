use core::fmt;
use core::num::NonZeroU64;

/// An exact rational number in lowest terms.
///
/// The denominator is at least 1, the numerator and the denominator have no
/// common factor, and zero has the denominator 1.
///
/// # Examples
///
/// ```
/// use escher_signature::Signature;
///
/// let signature: Signature = "4*3".parse()?;
/// let chi = signature.euler_characteristic();
/// assert_eq!(chi.numerator(), -1);
/// assert_eq!(chi.denominator().get(), 12);
/// assert_eq!(chi.to_string(), "-1/12");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rational {
    numerator: i64,
    denominator: NonZeroU64,
}

impl Rational {
    pub(crate) fn new(numerator: i128, denominator: i128) -> Option<Self> {
        let divisor = gcd(numerator.unsigned_abs(), denominator.unsigned_abs());
        let magnitude = numerator.unsigned_abs().checked_div(divisor)?;
        let denominator_magnitude = denominator.unsigned_abs().checked_div(divisor)?;
        let negative = (numerator < 0) != (denominator < 0);
        let signed = i128::try_from(magnitude).ok()?;
        let signed = if negative {
            signed.checked_neg()?
        } else {
            signed
        };
        Some(Self {
            numerator: i64::try_from(signed).ok()?,
            denominator: NonZeroU64::new(u64::try_from(denominator_magnitude).ok()?)?,
        })
    }

    pub(crate) fn abs(self) -> Option<Self> {
        Some(Self {
            numerator: self.numerator.checked_abs()?,
            denominator: self.denominator,
        })
    }

    /// Returns the numerator, which carries the sign.
    #[must_use]
    pub fn numerator(self) -> i64 {
        self.numerator
    }

    /// Returns the denominator.
    #[must_use]
    pub fn denominator(self) -> NonZeroU64 {
        self.denominator
    }
}

/// Writes `n/d`, or `n` when the denominator is 1, as in `-1/84`, `0`, and
/// `2`.
impl fmt::Display for Rational {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator.get() == 1 {
            write!(formatter, "{}", self.numerator)
        } else {
            write!(formatter, "{}/{}", self.numerator, self.denominator)
        }
    }
}

pub(crate) fn gcd(mut first: u128, mut second: u128) -> u128 {
    while let Some(remainder) = first.checked_rem(second) {
        first = second;
        second = remainder;
    }
    first
}

#[cfg(test)]
mod tests {
    use super::{Rational, gcd};

    fn parts(value: Option<Rational>) -> Option<(i64, u64)> {
        value.map(|rational| (rational.numerator(), rational.denominator().get()))
    }

    #[test]
    fn new_reduces_to_lowest_terms() {
        assert_eq!(parts(Rational::new(6, 8)), Some((3, 4)));
        assert_eq!(parts(Rational::new(-84, 7056)), Some((-1, 84)));
    }

    #[test]
    fn new_moves_the_sign_to_the_numerator() {
        assert_eq!(parts(Rational::new(3, -6)), Some((-1, 2)));
        assert_eq!(parts(Rational::new(-3, -6)), Some((1, 2)));
    }

    #[test]
    fn new_writes_zero_as_zero_over_one() {
        assert_eq!(parts(Rational::new(0, -5)), Some((0, 1)));
        assert_eq!(parts(Rational::new(0, 1)), Some((0, 1)));
    }

    #[test]
    fn new_fails_for_a_zero_denominator() {
        assert_eq!(Rational::new(1, 0), None);
        assert_eq!(Rational::new(0, 0), None);
    }

    #[test]
    fn new_fails_when_a_reduced_part_does_not_fit() {
        let too_large = i128::from(i64::MAX) + 1;
        assert_eq!(Rational::new(too_large, 1), None);
        assert_eq!(Rational::new(1, i128::from(u64::MAX) + 1), None);
        assert_eq!(Rational::new(i128::MIN, 3), None);
        assert_eq!(parts(Rational::new(too_large, 2)), Some((1 << 62, 1)));
        assert_eq!(parts(Rational::new(-too_large, 1)), Some((i64::MIN, 1)));
        assert_eq!(parts(Rational::new(i128::MIN, i128::MIN)), Some((1, 1)));
    }

    #[test]
    fn display_omits_a_denominator_of_one() {
        let text = [(-1, 84), (0, 1), (2, 1), (1, 60)]
            .into_iter()
            .filter_map(|(numerator, denominator)| Rational::new(numerator, denominator))
            .map(|rational| rational.to_string())
            .collect::<Vec<_>>();
        assert_eq!(text, ["-1/84", "0", "2", "1/60"]);
    }

    #[test]
    fn abs_fails_only_for_the_smallest_numerator() {
        assert_eq!(
            parts(Rational::new(-3, 4).and_then(Rational::abs)),
            Some((3, 4))
        );
        assert_eq!(
            parts(Rational::new(3, 4).and_then(Rational::abs)),
            Some((3, 4))
        );
        let smallest = Rational::new(i128::from(i64::MIN), 1);
        assert!(smallest.is_some());
        assert_eq!(smallest.and_then(Rational::abs), None);
    }

    #[test]
    fn gcd_follows_euclid() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(gcd(0, 7), 7);
        assert_eq!(gcd(7, 0), 7);
        assert_eq!(gcd(0, 0), 0);
    }
}
