use core::f64::consts::{LN_2 as LN_2_HIGH, PI as PI_HIGH};
use core::ops::{Add, Div, Mul, Neg, Sub};

use escher_geometry::hyperbolic::{covector, normalize_point, side, wall_through};
use glam::{DMat3, DVec3};

use crate::domain::Side;
use crate::double::{two_product, two_sum};
use crate::polygon::Polygon;

const PI: Dd = Dd {
    high: PI_HIGH,
    low: 1.224_646_799_147_353_2e-16,
};

const LN_2: Dd = Dd {
    high: LN_2_HIGH,
    low: 2.319_046_813_846_299_6e-17,
};

const EXP_REDUCTION_SQUARINGS: u32 = 10;

const EXP_REDUCTION: f64 = 1.0 / 1024.0;

const EXP_TERMS: u32 = 12;

const TRIG_TERMS: u32 = 22;

const LARGEST_EXPONENT: f64 = 700.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Dd {
    high: f64,
    low: f64,
}

pub(crate) type Matrix = [[Dd; 3]; 3];

const IDENTITY: Matrix = [
    [Dd::ONE, Dd::ZERO, Dd::ZERO],
    [Dd::ZERO, Dd::ONE, Dd::ZERO],
    [Dd::ZERO, Dd::ZERO, Dd::ONE],
];

type Point = [Dd; 3];

impl Dd {
    pub(crate) const ZERO: Self = Self::from_f64(0.0);

    pub(crate) const ONE: Self = Self::from_f64(1.0);

    pub(crate) const fn from_f64(value: f64) -> Self {
        Self {
            high: value,
            low: 0.0,
        }
    }

    fn normalized(high: f64, low: f64) -> Self {
        let sum = high + low;
        Self {
            high: sum,
            low: low - (sum - high),
        }
    }

    pub(crate) fn to_f64(self) -> f64 {
        self.high + self.low
    }

    fn scaled(self, power_of_two: f64) -> Self {
        Self {
            high: self.high * power_of_two,
            low: self.low * power_of_two,
        }
    }

    fn times(self, factor: f64) -> Self {
        let (high, error) = two_product(self.high, factor);
        Self::normalized(high, self.low.mul_add(factor, error))
    }

    fn over(self, divisor: f64) -> Self {
        let first = self.high / divisor;
        let (product, error) = two_product(first, divisor);
        let (difference, carried) = two_sum(self.high, -product);
        let remainder = difference + (carried - error + self.low);
        Self::normalized(first, remainder / divisor)
    }

    pub(crate) fn sqrt(self) -> Self {
        if self.high <= 0.0 {
            return Self::from_f64(self.high.sqrt());
        }
        let root = self.high.sqrt();
        let (square, error) = two_product(root, root);
        let remainder = (self - Self::normalized(square, error)).high;
        Self::normalized(root, remainder / (2.0 * root))
    }
}

impl Add for Dd {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let (high, high_error) = two_sum(self.high, other.high);
        let (low, low_error) = two_sum(self.low, other.low);
        let first = Self::normalized(high, high_error + low);
        Self::normalized(first.high, first.low + low_error)
    }
}

impl Neg for Dd {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            high: -self.high,
            low: -self.low,
        }
    }
}

impl Sub for Dd {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + -other
    }
}

impl Mul for Dd {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let (high, error) = two_product(self.high, other.high);
        let cross = self.high.mul_add(other.low, self.low * other.high);
        Self::normalized(high, error + cross)
    }
}

impl Div for Dd {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let first = self.high / other.high;
        let remainder = self - other.times(first);
        let second = remainder.high / other.high;
        let remainder = remainder - other.times(second);
        let third = remainder.high / other.high;
        Self::normalized(first, second) + Self::from_f64(third)
    }
}

pub(crate) fn double_double_angle(order: u32) -> Dd {
    PI.over(f64::from(order))
}

fn exp_parts(value: f64) -> Option<(Dd, Dd)> {
    if !value.is_finite() || value.abs() > LARGEST_EXPONENT {
        return None;
    }
    let multiple = (value / LN_2.high).round();
    let reduced = (Dd::from_f64(value) - LN_2.times(multiple)).scaled(EXP_REDUCTION);
    let mut term = reduced;
    let mut minus_one = reduced;
    for index in 2..=EXP_TERMS {
        term = (term * reduced).over(f64::from(index));
        minus_one = minus_one + term;
    }
    for _ in 0..EXP_REDUCTION_SQUARINGS {
        minus_one = minus_one.scaled(2.0) + minus_one * minus_one;
    }
    let mut power = 1.0;
    let mut remaining = multiple;
    while remaining > 0.0 {
        power *= 2.0;
        remaining -= 1.0;
    }
    while remaining < 0.0 {
        power /= 2.0;
        remaining += 1.0;
    }
    if multiple == 0.0 {
        Some((minus_one + Dd::ONE, minus_one))
    } else {
        let exponential = (minus_one + Dd::ONE).scaled(power);
        Some((exponential, exponential - Dd::ONE))
    }
}

pub(crate) fn cosh_sinh(value: f64) -> (Dd, Dd) {
    let Some((exponential, minus_one)) = exp_parts(value) else {
        let nan = Dd::from_f64(f64::NAN);
        return (nan, nan);
    };
    let reciprocal = Dd::ONE / exponential;
    let cosh = (exponential + reciprocal).scaled(0.5);
    let sinh = (minus_one * (Dd::ONE + reciprocal)).scaled(0.5);
    (cosh, sinh)
}

pub(crate) fn cos_sin(angle: Dd) -> (Dd, Dd) {
    let square = angle * angle;
    let (mut cos, mut sin) = (Dd::ONE, angle);
    let (mut cos_term, mut sin_term) = (Dd::ONE, angle);
    for index in 1..=TRIG_TERMS {
        let even = f64::from(2 * index);
        cos_term = -(cos_term * square).over(even * (even - 1.0));
        sin_term = -(sin_term * square).over(even * (even + 1.0));
        cos = cos + cos_term;
        sin = sin + sin_term;
    }
    (cos, sin)
}

fn product(left: &Matrix, right: &Matrix) -> Matrix {
    let entry = |row: usize, column: usize| {
        left[row][0] * right[0][column]
            + left[row][1] * right[1][column]
            + left[row][2] * right[2][column]
    };
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| entry(row, column)))
}

fn apply(matrix: &Matrix, point: &Point) -> Point {
    matrix.map(|row| row[0] * point[0] + row[1] * point[1] + row[2] * point[2])
}

fn boost(length: f64) -> Matrix {
    let (cosh, sinh) = cosh_sinh(length);
    [
        [cosh, Dd::ZERO, sinh],
        [Dd::ZERO, Dd::ONE, Dd::ZERO],
        [sinh, Dd::ZERO, cosh],
    ]
}

fn turn(order: u32) -> Matrix {
    let (cos, sin) = cos_sin(double_double_angle(order));
    [
        [-cos, -sin, Dd::ZERO],
        [sin, -cos, Dd::ZERO],
        [Dd::ZERO, Dd::ZERO, Dd::ONE],
    ]
}

pub(crate) fn walk(orders: &[u32], lengths: &[f64]) -> Vec<Point> {
    let mut frame = IDENTITY;
    let mut points = vec![[Dd::ZERO, Dd::ZERO, Dd::ONE]];
    for (&order, &length) in orders.iter().zip(lengths).skip(1) {
        frame = product(&frame, &boost(length));
        points.push([frame[0][2], frame[1][2], frame[2][2]]);
        frame = product(&frame, &turn(order));
    }
    points
}

pub(crate) fn place(points: &[Point]) -> Option<Vec<DVec3>> {
    let sum = points.iter().fold([Dd::ZERO; 3], |sum, point| {
        [sum[0] + point[0], sum[1] + point[1], sum[2] + point[2]]
    });
    let norm = (sum[2] * sum[2] - sum[0] * sum[0] - sum[1] * sum[1]).sqrt();
    let [x, y, t] = sum.map(|coordinate| coordinate / norm);
    let factor = Dd::ONE / (Dd::ONE + t);
    let mixed = x * y * factor;
    let to_origin = [
        [Dd::ONE + x * x * factor, mixed, -x],
        [mixed, Dd::ONE + y * y * factor, -y],
        [-x, -y, t],
    ];
    let centred: Vec<Point> = points
        .iter()
        .map(|point| apply(&to_origin, point))
        .collect();
    let [first_x, first_y, _] = centred.first().copied().unwrap_or([Dd::ZERO; 3]);
    let radius = (first_x * first_x + first_y * first_y).sqrt();
    let (cos, sin) = (first_x / radius, -(first_y / radius));
    let vertices: Vec<DVec3> = centred
        .iter()
        .map(|&[x, y, t]| {
            DVec3::new(
                (cos * x - sin * y).to_f64(),
                (sin * x + cos * y).to_f64(),
                t.to_f64(),
            )
        })
        .collect();
    let placed = norm.high > 0.0
        && t.high > 0.0
        && radius.high > 0.0
        && vertices.iter().all(|vertex| vertex.is_finite());
    placed.then_some(vertices)
}

pub(crate) fn construct(
    orders: &[u32],
    lengths: &[f64],
) -> Result<Polygon, Option<escher_geometry::Error>> {
    let vertices = place(&walk(orders, lengths)).ok_or(None)?;
    if vertices.len() != orders.len() {
        return Err(None);
    }
    mirrors(vertices)
}

fn mirrors(vertices: Vec<DVec3>) -> Result<Polygon, Option<escher_geometry::Error>> {
    let count = vertices.len();
    let sum = vertices
        .iter()
        .fold(DVec3::ZERO, |sum, &vertex| sum + vertex);
    let centre = normalize_point(sum).map_err(Some)?;
    let mut sides = Vec::with_capacity(count);
    for index in 0..count {
        let previous = vertices[(index + count - 1) % count];
        let mut normal = wall_through(previous, vertices[index]).map_err(Some)?;
        if side(covector(normal), centre) > 0.0 {
            normal = -normal;
        }
        let value = side(covector(normal), centre);
        if value.is_nan() || value >= 0.0 {
            return Err(None);
        }
        sides.push(Side::mirror(normal, index));
    }
    Ok(Polygon {
        sides,
        vertices,
        centre,
    })
}

fn matrix_of(matrix: DMat3) -> Matrix {
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| Dd::from_f64(matrix.col(column)[row])))
}

fn point_of(point: DVec3) -> Point {
    point.to_array().map(Dd::from_f64)
}

pub(crate) fn chain(factors: &[DMat3]) -> Matrix {
    factors.iter().fold(IDENTITY, |chained, factor| {
        product(&chained, &matrix_of(*factor))
    })
}

pub(crate) fn displacement(transformation: &Matrix, point: DVec3) -> [f64; 3] {
    let start = point_of(point);
    let image = apply(transformation, &start);
    [0, 1, 2].map(|index| (image[index] - start[index]).to_f64())
}

pub(crate) fn framed_power_residual(vertex: DVec3, transformation: &Matrix, order: u32) -> f64 {
    let [x, y, t] = point_of(vertex);
    let norm = (t * t - x * x - y * y).sqrt();
    let [x, y, t] = [x, y, t].map(|coordinate| coordinate / norm);
    let factor = Dd::ONE / (Dd::ONE + t);
    let mixed = x * y * factor;
    let frame = [
        [Dd::ONE + x * x * factor, mixed, x],
        [mixed, Dd::ONE + y * y * factor, y],
        [x, y, t],
    ];
    let inverse = [
        [Dd::ONE + x * x * factor, mixed, -x],
        [mixed, Dd::ONE + y * y * factor, -y],
        [-x, -y, t],
    ];
    let rotation = product(&product(&inverse, transformation), &frame);
    let mut power = rotation;
    for _ in 1..order {
        power = product(&power, &rotation);
    }
    let mut largest: f64 = 0.0;
    for (row, entries) in power.iter().enumerate() {
        for (column, &entry) in entries.iter().enumerate() {
            let difference = if row == column {
                entry - Dd::ONE
            } else {
                entry
            };
            let value = difference.to_f64().abs();
            if value.is_nan() || largest.is_nan() {
                largest = f64::NAN;
            } else {
                largest = largest.max(value);
            }
        }
    }
    largest / f64::from(order)
}

#[cfg(test)]
mod tests {
    use astro_float_num::BigFloat;

    use escher_geometry::hyperbolic::{ORIGIN, boost, rotation};
    use glam::DVec3;

    use super::{Dd, PI, construct, cos_sin, cosh_sinh, double_double_angle, mirrors, place, walk};
    use crate::reference::{Reference, add, div, int, num, sub};

    const TWO_TO_THE_MINUS_30: f64 = 1.0 / 1_073_741_824.0;

    const TWO_TO_THE_MINUS_60: f64 = TWO_TO_THE_MINUS_30 * TWO_TO_THE_MINUS_30;

    const UNIT: f64 = TWO_TO_THE_MINUS_60 * TWO_TO_THE_MINUS_30 / 65_536.0;

    const HYPERBOLIC_BOUND: f64 = 2.0e-30;

    const CIRCULAR_BOUND: f64 = 2.0e-31;

    fn dd(high: f64, low: f64) -> Dd {
        Dd { high, low }
    }

    #[test]
    fn arithmetic_is_exact_on_representable_results() {
        let near_one = dd(1.0 + TWO_TO_THE_MINUS_30, 0.0);
        let square = dd(1.0 + 2.0 * TWO_TO_THE_MINUS_30, TWO_TO_THE_MINUS_60);
        assert_eq!(near_one * near_one, square);
        assert_eq!(square / near_one, near_one);
        assert_eq!(square.sqrt(), near_one);
        assert_eq!(square - near_one * near_one, Dd::ZERO);
        assert_eq!(
            dd(1.0, TWO_TO_THE_MINUS_60) + dd(-1.0, 0.0),
            dd(TWO_TO_THE_MINUS_60, 0.0)
        );
        assert_eq!(dd(3.0, 0.0) / dd(4.0, 0.0), dd(0.75, 0.0));
        assert_eq!(dd(9.0, 0.0).sqrt(), dd(3.0, 0.0));
        assert_eq!(dd(6.0, 0.0).over(4.0), dd(1.5, 0.0));
        assert_eq!(square.to_f64(), 1.0 + 2.0 * TWO_TO_THE_MINUS_30);
    }

    #[test]
    fn walk_starts_at_the_origin_and_turns_counterclockwise() {
        let points = walk(&[3, 3, 3, 3], &[1.0, 1.0, 1.0, 1.0]);
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], [Dd::ZERO, Dd::ZERO, Dd::ONE]);
        assert_eq!(points[1][1], Dd::ZERO);
        assert!(points[1][0].high > 0.0);
        assert!(points[2][1].high > 0.0);
    }

    #[test]
    fn cosh_and_sinh_of_a_length_that_is_not_finite_or_above_700_are_nan() {
        for length in [701.0, -701.0, f64::NAN, f64::INFINITY] {
            let (cosh, sinh) = cosh_sinh(length);
            assert!(cosh.high.is_nan() && sinh.high.is_nan(), "{length}");
        }
        let (cosh, sinh) = cosh_sinh(-1.0);
        assert!((cosh.to_f64() - 1.0_f64.cosh()).abs() <= 1e-15);
        assert!((sinh.to_f64() + 1.0_f64.sinh()).abs() <= 1e-15);
    }

    #[test]
    fn place_refuses_points_that_give_no_centre_or_no_direction() {
        let origin = [Dd::ZERO, Dd::ZERO, Dd::ONE];
        let below = [Dd::ZERO, Dd::ZERO, -Dd::ONE];
        let nan = [Dd::from_f64(f64::NAN), Dd::ZERO, Dd::ONE];
        for points in [vec![], vec![origin], vec![below], vec![origin, nan]] {
            assert_eq!(place(&points), None, "{points:?}");
        }
        assert!(place(&walk(&[3, 3, 3, 3], &[1.0; 4])).is_some());
    }

    #[test]
    fn construct_refuses_a_walk_that_does_not_place_every_corner() {
        assert!(construct(&[3, 3, 3, 3], &[1.0; 4]).is_ok());
        assert!(matches!(construct(&[3, 3, 3, 3], &[1.0; 2]), Err(None)));
        assert!(matches!(construct(&[3, 3, 3, 3], &[0.0; 4]), Err(None)));
    }

    #[test]
    fn mirrors_refuse_a_centre_on_a_wall_and_vertices_without_walls() {
        let (sinh, cosh) = (1.0_f64.sinh(), 1.0_f64.cosh());
        let on_a_wall = vec![
            DVec3::new(sinh, 0.0, cosh),
            DVec3::new(-sinh, 0.0, cosh),
            DVec3::new(0.0, sinh, cosh),
            DVec3::new(0.0, -sinh, cosh),
        ];
        assert!(matches!(mirrors(on_a_wall), Err(None)));
        let point = boost(1.0) * ORIGIN;
        let repeated = vec![point, point, rotation(2.0) * point, rotation(4.0) * point];
        assert!(matches!(mirrors(repeated), Err(Some(_))));
        assert!(matches!(
            mirrors(vec![DVec3::X, DVec3::Y, -DVec3::X]),
            Err(Some(_))
        ));
    }

    fn big(value: Dd) -> BigFloat {
        add(&num(value.high), &num(value.low))
    }

    fn print_largest(name: &str, largest: &(f64, String), bound: f64) {
        println!(
            "{name}: largest error {:.3e}, {:.2} units of 2^-106, at {}, bound {bound:e}, margin {:.1}",
            largest.0,
            largest.0 / UNIT,
            largest.1,
            bound / largest.0
        );
    }

    #[test]
    fn cosh_and_sinh_match_the_reference_relatively() {
        let mut reference = Reference::new();
        let mut largest = [(0.0, String::new()), (0.0, String::new())];
        for step in 0..=30 * 64 {
            let length = f64::from(step) / 64.0;
            let (cosh, sinh) = cosh_sinh(length);
            let exact = [reference.cosh(&num(length)), reference.sinh(&num(length))];
            for ((value, exact), slot) in [cosh, sinh].into_iter().zip(&exact).zip(&mut largest) {
                if step == 0 && exact.is_zero() {
                    assert_eq!(value, Dd::ZERO);
                    continue;
                }
                let error = div(&sub(&big(value), exact), exact).abs();
                let error = reference.approximate(&error);
                if error > slot.0 {
                    *slot = (error, format!("length {length}"));
                }
            }
        }
        for (name, largest) in ["cosh", "sinh"].into_iter().zip(largest) {
            assert!(largest.0 <= HYPERBOLIC_BOUND, "{name}: {largest:?}");
            print_largest(name, &largest, HYPERBOLIC_BOUND);
        }
    }

    #[test]
    fn cos_and_sin_of_the_corner_angles_match_the_reference() {
        let mut reference = Reference::new();
        let pi = reference.pi();
        assert!(reference.approximate(&sub(&big(PI), &pi).abs()) < 2.0 * UNIT);
        let mut largest = [(0.0, String::new()), (0.0, String::new())];
        for order in 2..=100 {
            let exact_angle = div(&pi, &int(order));
            let (cos, sin) = cos_sin(double_double_angle(order));
            let exact = [reference.cos(&exact_angle), reference.sin(&exact_angle)];
            for ((value, exact), slot) in [cos, sin].into_iter().zip(&exact).zip(&mut largest) {
                let error = reference.approximate(&sub(&big(value), exact).abs());
                if error > slot.0 {
                    *slot = (error, format!("pi / {order}"));
                }
            }
        }
        for (name, largest) in ["cos", "sin"].into_iter().zip(largest) {
            assert!(largest.0 <= CIRCULAR_BOUND, "{name}: {largest:?}");
            print_largest(name, &largest, CIRCULAR_BOUND);
        }
    }
}
