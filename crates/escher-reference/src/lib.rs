//! High-precision reference arithmetic for the accuracy tests of the Escher
//! crates.
//!
//! This is a test support crate. The crates of the workspace use it only as a
//! development dependency, to compute the exact value that a test compares a
//! floating-point result with, and to record and assert the largest error over
//! the inputs of a test. It depends on no Escher crate, so a reference value
//! shares no code with the code that it checks. Each operation works on
//! [`BigFloat`] values with 256 bits of mantissa and rounds to the nearest
//! value, with ties to even.
//!
//! This example checks the double nearest to the square root of 2 against the
//! reference value:
//!
//! ```
//! use escher_reference::{Reference, Worst, error, int, sqrt};
//!
//! let mut reference = Reference::new();
//! let mut worst = Worst::new("square root of 2", 1);
//! let exact = sqrt(&int(2));
//! worst.record(&error(2.0_f64.sqrt(), &exact), &int(1), || "2".to_owned());
//! worst.assert_at_most(&mut reference, 1e-15);
//! ```
//!
//! # Features
//!
//! The crate has no Cargo features.
//!
//! # Platform support
//!
//! The crate runs on every platform that `astro-float-num` supports.

use astro_float_num::{BigFloat, Consts, Radix, RoundingMode};

const PRECISION: usize = 256;
const ROUNDING: RoundingMode = RoundingMode::ToEven;

/// A vector of three reference values.
pub type Vector = [BigFloat; 3];

/// A 3 by 3 matrix of reference values, indexed by row and then by column.
pub type Matrix = [[BigFloat; 3]; 3];

/// Returns the reference value of a double, which holds it exactly.
#[must_use]
pub fn num(value: f64) -> BigFloat {
    BigFloat::from_f64(value, PRECISION)
}

/// Returns the reference value of a whole number.
#[must_use]
pub fn int(value: u32) -> BigFloat {
    num(f64::from(value))
}

/// Returns the sum of two values, rounded to the reference precision.
#[must_use]
pub fn add(left: &BigFloat, right: &BigFloat) -> BigFloat {
    left.add(right, PRECISION, ROUNDING)
}

/// Returns the difference of two values, rounded to the reference precision.
#[must_use]
pub fn sub(left: &BigFloat, right: &BigFloat) -> BigFloat {
    left.sub(right, PRECISION, ROUNDING)
}

/// Returns the product of two values, rounded to the reference precision.
#[must_use]
pub fn mul(left: &BigFloat, right: &BigFloat) -> BigFloat {
    left.mul(right, PRECISION, ROUNDING)
}

/// Returns the quotient of two values, rounded to the reference precision.
#[must_use]
pub fn div(left: &BigFloat, right: &BigFloat) -> BigFloat {
    left.div(right, PRECISION, ROUNDING)
}

/// Returns the square root of a value, rounded to the reference precision.
#[must_use]
pub fn sqrt(value: &BigFloat) -> BigFloat {
    value.sqrt(PRECISION, ROUNDING)
}

/// Returns the larger of two values, or the left one when they are equal.
#[must_use]
pub fn max(left: &BigFloat, right: &BigFloat) -> BigFloat {
    if left >= right {
        left.clone()
    } else {
        right.clone()
    }
}

/// Returns the absolute difference between a double and a reference value.
#[must_use]
pub fn error(value: f64, reference: &BigFloat) -> BigFloat {
    sub(&num(value), reference).abs()
}

/// Returns the reference vector of three doubles.
#[must_use]
pub fn vector(values: [f64; 3]) -> Vector {
    values.map(num)
}

/// Returns the reference matrix of nine doubles given column by column.
#[must_use]
pub fn matrix(columns: [f64; 9]) -> Matrix {
    let entry = |row: usize, column: usize| num(columns[3 * column + row]);
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| entry(row, column)))
}

/// Returns the dot product of two vectors, `x0 y0 + x1 y1 + x2 y2`.
#[must_use]
pub fn dot(x: &Vector, y: &Vector) -> BigFloat {
    add(
        &add(&mul(&x[0], &y[0]), &mul(&x[1], &y[1])),
        &mul(&x[2], &y[2]),
    )
}

/// Returns the Lorentzian form of two vectors, `x0 y0 + x1 y1 - x2 y2`.
///
/// The hyperboloid model of the hyperbolic plane measures lengths and angles
/// with this form instead of the dot product.
#[must_use]
pub fn form(x: &Vector, y: &Vector) -> BigFloat {
    sub(
        &add(&mul(&x[0], &y[0]), &mul(&x[1], &y[1])),
        &mul(&x[2], &y[2]),
    )
}

/// Returns the cross product of two vectors, which is perpendicular to both
/// under the dot product.
///
/// Its coordinates are `x1 y2 - x2 y1`, `x2 y0 - x0 y2`, and `x0 y1 - x1 y0`.
#[must_use]
pub fn cross(x: &Vector, y: &Vector) -> Vector {
    [
        sub(&mul(&x[1], &y[2]), &mul(&x[2], &y[1])),
        sub(&mul(&x[2], &y[0]), &mul(&x[0], &y[2])),
        sub(&mul(&x[0], &y[1]), &mul(&x[1], &y[0])),
    ]
}

/// Returns a vector multiplied by a value.
#[must_use]
pub fn scale(factor: &BigFloat, x: &Vector) -> Vector {
    [mul(factor, &x[0]), mul(factor, &x[1]), mul(factor, &x[2])]
}

/// Returns the difference of two vectors.
#[must_use]
pub fn difference(x: &Vector, y: &Vector) -> Vector {
    [sub(&x[0], &y[0]), sub(&x[1], &y[1]), sub(&x[2], &y[2])]
}

/// Returns the largest absolute value of the coordinates of a vector.
#[must_use]
pub fn largest_magnitude(x: &Vector) -> BigFloat {
    max(&max(&x[0].abs(), &x[1].abs()), &x[2].abs())
}

/// Returns the point of the hyperboloid on the ray of a vector, the vector
/// divided by the square root of `-form(x, x)`.
///
/// The hyperboloid is the set of vectors with `form(x, x) = -1` and a positive
/// last coordinate, which models the hyperbolic plane. The vector must have a
/// negative [`form`] with itself and a positive last coordinate.
#[must_use]
pub fn normalize(x: &Vector) -> Vector {
    scale(&div(&int(1), &sqrt(&form(x, x).neg())), x)
}

/// Returns the product of two matrices.
#[must_use]
pub fn product(left: &Matrix, right: &Matrix) -> Matrix {
    [0, 1, 2].map(|row| {
        [0, 1, 2].map(|column| {
            let first = mul(&left[row][0], &right[0][column]);
            let second = mul(&left[row][1], &right[1][column]);
            let third = mul(&left[row][2], &right[2][column]);
            add(&add(&first, &second), &third)
        })
    })
}

/// Returns a matrix applied to a vector.
#[must_use]
pub fn apply(matrix: &Matrix, x: &Vector) -> Vector {
    [0, 1, 2].map(|row| {
        let first = mul(&matrix[row][0], &x[0]);
        let second = mul(&matrix[row][1], &x[1]);
        let third = mul(&matrix[row][2], &x[2]);
        add(&add(&first, &second), &third)
    })
}

/// Computes the transcendental functions and constants at the reference
/// precision.
///
/// Create one for each test, because it caches the constants that the
/// functions need, such as pi.
#[derive(Debug)]
pub struct Reference {
    consts: Consts,
}

impl Reference {
    /// Creates a reference with an empty cache of constants.
    ///
    /// # Panics
    ///
    /// Panics when the cache cannot be allocated.
    #[expect(
        clippy::new_without_default,
        reason = "creating the cache can panic, which a default value should not do"
    )]
    #[expect(
        clippy::expect_used,
        reason = "a test cannot go on without the cache of constants"
    )]
    #[must_use]
    pub fn new() -> Self {
        Self {
            consts: Consts::new().expect("the constant cache of the reference allocates"),
        }
    }

    /// Returns pi.
    pub fn pi(&mut self) -> BigFloat {
        self.consts.pi(PRECISION, ROUNDING)
    }

    /// Returns the cosine of an angle in radians.
    pub fn cos(&mut self, value: &BigFloat) -> BigFloat {
        value.cos(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the sine of an angle in radians.
    pub fn sin(&mut self, value: &BigFloat) -> BigFloat {
        value.sin(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the hyperbolic cosine of a value.
    pub fn cosh(&mut self, value: &BigFloat) -> BigFloat {
        value.cosh(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the hyperbolic sine of a value.
    pub fn sinh(&mut self, value: &BigFloat) -> BigFloat {
        value.sinh(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the inverse hyperbolic sine of a value.
    pub fn asinh(&mut self, value: &BigFloat) -> BigFloat {
        value.asinh(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the inverse hyperbolic cosine of a value of at least 1.
    pub fn acosh(&mut self, value: &BigFloat) -> BigFloat {
        value.acosh(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the arc cosine, in radians, of a value from -1 to 1.
    pub fn acos(&mut self, value: &BigFloat) -> BigFloat {
        value.acos(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the arc sine, in radians, of a value from -1 to 1.
    pub fn asin(&mut self, value: &BigFloat) -> BigFloat {
        value.asin(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the arc tangent of a value, in radians.
    pub fn atan(&mut self, value: &BigFloat) -> BigFloat {
        value.atan(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the natural logarithm of a positive value.
    pub fn ln(&mut self, value: &BigFloat) -> BigFloat {
        value.ln(PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns a base raised to a power.
    pub fn pow(&mut self, base: &BigFloat, exponent: &BigFloat) -> BigFloat {
        base.pow(exponent, PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the hyperbolic distance between two points of the hyperboloid,
    /// the inverse hyperbolic cosine of `-form(x, y)`.
    ///
    /// Use it for points that lie on the hyperboloid, such as the results of
    /// [`normalize`]. A value of `-form(x, y)` below 1, which rounding can
    /// produce for two equal points, counts as 1, so the distance is then 0.
    pub fn distance(&mut self, x: &Vector, y: &Vector) -> BigFloat {
        let c = form(x, y).neg();
        self.acosh(&max(&int(1), &c))
    }

    /// Returns the value of a decimal number written as text, such as `"0.25"`.
    ///
    /// Use it for a constant that a double cannot hold exactly.
    pub fn parse(&mut self, text: &str) -> BigFloat {
        BigFloat::parse(text, Radix::Dec, PRECISION, ROUNDING, &mut self.consts)
    }

    /// Returns the double nearest to a value, or NaN when the value has no
    /// decimal form.
    ///
    /// # Panics
    ///
    /// Panics when the value cannot be formatted as decimal text.
    #[expect(
        clippy::expect_used,
        reason = "a value that cannot be formatted leaves the test nothing to report"
    )]
    pub fn approximate(&mut self, value: &BigFloat) -> f64 {
        let text = value
            .format(Radix::Dec, ROUNDING, &mut self.consts)
            .expect("the reference formats a number");
        text.parse().unwrap_or(f64::NAN)
    }
}

/// Records the largest error of a bound over the inputs of a test, with its
/// input, and asserts the bound.
///
/// Use it in an accuracy test that measures the error of a result against a
/// reference value for each input. The test records each error with
/// [`Worst::record`], then calls [`Worst::assert_at_most`] once, which prints
/// the largest error with its input and the margin of the bound.
#[derive(Debug)]
pub struct Worst {
    name: String,
    unit: &'static str,
    minimum: usize,
    count: usize,
    ratio: BigFloat,
    input: String,
}

impl Worst {
    /// Creates a record named for the bound, which asserts that the test
    /// measures at least `minimum` inputs.
    #[must_use]
    pub fn new(name: impl Into<String>, minimum: usize) -> Self {
        Self::with_unit(name.into(), "", minimum)
    }

    /// Creates a record like [`Worst::new`] whose printed errors say that
    /// they are fractions of the scale passed to [`Worst::record`].
    #[must_use]
    pub fn of_the_scale(name: impl Into<String>, minimum: usize) -> Self {
        Self::with_unit(name.into(), " of the scale", minimum)
    }

    fn with_unit(name: String, unit: &'static str, minimum: usize) -> Self {
        Self {
            name,
            unit,
            minimum,
            count: 0,
            ratio: int(0),
            input: String::new(),
        }
    }

    /// Records the error of one input divided by its scale, and keeps it with
    /// the description of the input when it is the largest so far.
    ///
    /// The description is built only when it is needed.
    ///
    /// # Panics
    ///
    /// Panics when the error divided by the scale is NaN.
    pub fn record(&mut self, error: &BigFloat, scale: &BigFloat, input: impl FnOnce() -> String) {
        self.count += 1;
        let ratio = if error.is_zero() {
            int(0)
        } else {
            div(error, scale)
        };
        assert!(
            !ratio.is_nan(),
            "{}: the error is NaN for {}",
            self.name,
            input()
        );
        if ratio > self.ratio {
            self.ratio = ratio;
            self.input = input();
        }
    }

    /// Returns the number of recorded inputs.
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Prints the largest error with its bound, margin, number of inputs, and
    /// input, then asserts the number of inputs and the bound.
    ///
    /// The margin is the bound divided by the largest error.
    ///
    /// # Panics
    ///
    /// Panics when fewer inputs than the minimum were recorded, or when the
    /// largest error exceeds the bound.
    #[expect(
        clippy::print_stdout,
        reason = "the test prints its largest error so that a reviewer can check each bound on every run"
    )]
    pub fn assert_at_most(&self, reference: &mut Reference, bound: f64) {
        let ratio = reference.approximate(&self.ratio);
        println!(
            "{}: largest error {ratio:.3e}{}, bound {bound:e}, margin {:.1}, {} samples, at {}",
            self.name,
            self.unit,
            bound / ratio,
            self.count,
            self.input
        );
        assert!(
            self.count >= self.minimum,
            "{}: measured {} samples, fewer than {}",
            self.name,
            self.count,
            self.minimum
        );
        assert!(
            self.ratio <= num(bound),
            "{}: largest error {ratio}{} exceeds {bound} at {}",
            self.name,
            self.unit,
            self.input
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Reference, Vector, Worst, apply, cross, difference, div, dot, error, form, int,
        largest_magnitude, matrix, mul, normalize, num, product, sqrt, sub, vector,
    };

    fn vector_error(x: &Vector, y: &Vector) -> astro_float_num::BigFloat {
        largest_magnitude(&difference(x, y))
    }

    #[test]
    fn reference_arithmetic_reproduces_known_values() {
        let mut reference = Reference::new();
        let tolerance = reference.parse("1e-70");
        let pi = reference.pi();
        let literal = reference.parse(
            "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899",
        );
        let third = div(&pi, &int(3));
        let sixth = div(&pi, &int(6));
        let root = sqrt(&int(2));
        let one = int(1);
        let half = div(&one, &int(2));
        let quarter = div(&one, &int(4));
        let inner = reference.acosh(&int(2));
        let asinh_one = reference.parse(
            "0.88137358701954302523260932497979230902816032826163541075329560865337718422202608",
        );
        let power = reference.pow(&int(2), &half);
        let e = reference.parse(
            "2.71828182845904523536028747135266249775724709369995957496696762772407663035354759",
        );
        let cosh_one = div(&add_inverse(&e), &int(2));
        let boosted = [reference.sinh(&one), int(0), reference.cosh(&one)];
        let origin = vector([0.0, 0.0, 1.0]);
        let shear = matrix([1.0, 0.0, 0.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
        let scaling = matrix([1.0, 3.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0]);
        let composed = product(&shear, &scaling);
        let expected = matrix([7.0, 3.0, 0.0, 2.0, 1.0, 0.0, 0.0, 0.0, 2.0]);
        let composed_error = (0..3)
            .map(|row| vector_error(&composed[row], &expected[row]))
            .fold(int(0), |largest, row| super::max(&largest, &row));
        let checks = [
            ("pi", sub(&pi, &literal)),
            ("cos(pi / 3)", sub(&reference.cos(&third), &half)),
            ("sin(pi / 6)", sub(&reference.sin(&sixth), &half)),
            ("cosh(acosh(2))", sub(&reference.cosh(&inner), &int(2))),
            ("sqrt(2)^2", sub(&mul(&root, &root), &int(2))),
            ("asinh(1)", sub(&reference.asinh(&one), &asinh_one)),
            ("sinh(asinh(1))", sub(&reference.sinh(&asinh_one), &one)),
            ("acosh(cosh(1))", sub(&reference.acosh(&cosh_one), &one)),
            ("2^(1/2)", sub(&power, &root)),
            ("ln(e)", sub(&reference.ln(&e), &one)),
            ("acos(1 / 2)", sub(&reference.acos(&half), &third)),
            ("asin(1 / 2)", sub(&reference.asin(&half), &sixth)),
            ("atan(1)", sub(&reference.atan(&one), &div(&pi, &int(4)))),
            (
                "distance of a boost by 1",
                sub(&reference.distance(&origin, &boosted), &one),
            ),
            ("error(0.75, 1 / 2)", sub(&error(0.75, &half), &quarter)),
            ("error(0.25, 1 / 2)", sub(&error(0.25, &half), &quarter)),
            (
                "approximate(1 / 4)",
                sub(&num(reference.approximate(&quarter)), &quarter),
            ),
            (
                "cross",
                vector_error(
                    &cross(&vector([1.0, 2.0, 3.0]), &vector([4.0, 5.0, 6.0])),
                    &vector([-3.0, 6.0, -3.0]),
                ),
            ),
            (
                "dot",
                sub(
                    &dot(&vector([1.0, 2.0, 3.0]), &vector([4.0, 5.0, 6.0])),
                    &int(32),
                ),
            ),
            (
                "form",
                sub(
                    &form(&vector([1.0, 2.0, 3.0]), &vector([4.0, 5.0, 6.0])),
                    &int(4).neg(),
                ),
            ),
            (
                "normalize",
                vector_error(&normalize(&vector([0.0, 0.0, 2.0])), &origin),
            ),
            ("matrix product", composed_error),
            (
                "apply",
                vector_error(
                    &apply(&shear, &vector([1.0, 1.0, 1.0])),
                    &vector([3.0, 1.0, 1.0]),
                ),
            ),
            (
                "largest magnitude",
                sub(&largest_magnitude(&vector([-5.0, 3.0, 4.0])), &int(5)),
            ),
        ];
        for (name, difference) in checks {
            let size = difference.abs();
            let shown = reference.approximate(&size);
            println!("{name}: error {shown:e}");
            assert!(size <= tolerance, "{name}: error {shown:e}");
        }
    }

    #[test]
    fn worst_passes_a_largest_error_within_the_bound() {
        let mut reference = Reference::new();
        let mut worst = Worst::new("within", 2);
        worst.record(&num(1.0), &int(4), || "first".to_owned());
        worst.record(&num(3.0), &int(4), || "second".to_owned());
        worst.assert_at_most(&mut reference, 0.75);
        assert_eq!(worst.count(), 2);
    }

    #[test]
    #[should_panic(expected = "above: largest error 0.75 exceeds 0.5 at second")]
    fn worst_rejects_a_largest_error_above_the_bound() {
        let mut reference = Reference::new();
        let mut worst = Worst::new("above", 1);
        worst.record(&num(1.0), &int(4), || "first".to_owned());
        worst.record(&num(3.0), &int(4), || "second".to_owned());
        worst.assert_at_most(&mut reference, 0.5);
    }

    #[test]
    #[should_panic(expected = "few: measured 1 samples, fewer than 2")]
    fn worst_rejects_fewer_samples_than_the_minimum() {
        let mut reference = Reference::new();
        let mut worst = Worst::new("few", 2);
        worst.record(&num(1.0), &int(4), || "first".to_owned());
        worst.assert_at_most(&mut reference, 1.0);
    }

    fn add_inverse(value: &astro_float_num::BigFloat) -> astro_float_num::BigFloat {
        super::add(value, &div(&int(1), value))
    }
}
