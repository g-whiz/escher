//! Points, lines, and isometries of the Euclidean plane, as the affine plane
//! whose last coordinate is 1.
//!
//! A Euclidean point is a [`DVec3`] `(x, y, 1)`. A line is represented by a
//! line covector `c = (a.x, a.y, -t)`, where `a` is a unit vector normal to
//! the line and `t` is its offset, so that the line is the set of points `p`
//! with `a . p = t`. For a point `x`, the dot product `c . x` is `a . p - t`,
//! which is zero on the line. A covector of a line that bounds a region is
//! oriented outward when that value is positive outside the region.
//!
//! An isometry, a map that preserves distances, is a [`DMat3`] whose
//! upper-left 2 by 2 block is orthogonal and whose last row is `(0, 0, 1)`.
//! It maps `(p, 1)` to `(A p + b, 1)`. Every isometry that the module returns
//! has the last row `(0, 0, 1)` exactly, so applying it to a point with a
//! finite position keeps the last coordinate exactly 1. `matrix * point`
//! applies an isometry, and the product `a * b` of two isometries applies `b`
//! first. Lengths are in the unit of the coordinates, and angles are in
//! radians.
//!
//! The functions use the names of the module
//! [`hyperbolic`](crate::hyperbolic) for the same operations, so a caller
//! changes only the module path. Every function returns without a panic for
//! every input.

use glam::{DMat3, DVec3};

use crate::error::{Error, Result};

/// The point at the origin of the plane, `(0, 0, 1)`.
///
/// It is the centre of a rotation made by [`rotation`], and [`translation`]
/// moves it to its target.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::{ORIGIN, translation};
/// use glam::DVec3;
///
/// assert_eq!(translation(DVec3::new(2.0, 3.0, 1.0)) * ORIGIN, DVec3::new(2.0, 3.0, 1.0));
/// ```
pub const ORIGIN: DVec3 = DVec3::new(0.0, 0.0, 1.0);

/// Returns a signed number that tells on which side of a line a point lies.
///
/// Use it to test points against a line, such as a line that bounds a
/// region. The first argument is a line covector `(a.x, a.y, -t)`. The value
/// is the dot product of the covector and the point, which is `a . p - t` for
/// the point `(p, 1)`, the signed distance of the point from the line. It
/// is positive outside the line when the covector is oriented outward, zero on
/// the line, and negative inside. The function applies no tolerance and takes
/// any vectors.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::side;
/// use glam::DVec3;
///
/// let line = DVec3::new(1.0, 0.0, -0.5);
/// assert_eq!(side(line, DVec3::new(2.0, 7.0, 1.0)), 1.5);
/// assert_eq!(side(line, DVec3::new(0.0, 7.0, 1.0)), -0.5);
/// ```
#[must_use]
pub fn side(covector: DVec3, point: DVec3) -> f64 {
    covector.dot(point)
}

/// Returns the Euclidean distance from a point to the line of a line
/// covector.
///
/// Use it to measure how far a point lies from a line, such as to draw a
/// border of a given width along a side of a region. The first argument is a
/// line covector `(a.x, a.y, -t)`, with `a` a unit vector, and the second is
/// a point `(p, 1)`. The distance is the absolute value of [`side`], which is
/// `|a . p - t|`, in the unit of the coordinates. The function does not
/// divide by the last coordinate of the point. It is the distance to the
/// whole line, not to a segment of it.
///
/// The result is at least 0, and NaN when a coordinate of either argument is
/// NaN. An infinite coordinate gives an infinite result or NaN.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::wall_distance;
/// use glam::DVec3;
///
/// let line = DVec3::new(1.0, 0.0, -0.5);
/// assert_eq!(wall_distance(line, DVec3::new(2.0, 7.0, 1.0)), 1.5);
/// assert_eq!(wall_distance(line, DVec3::new(0.0, 7.0, 1.0)), 0.5);
/// ```
#[must_use]
pub fn wall_distance(covector: DVec3, point: DVec3) -> f64 {
    side(covector, point).abs()
}

/// Returns a line covector of the line through two points.
///
/// Use it to build the line through two vertices, such as a side of a
/// polygon. The points must be two distinct Euclidean points. The result `c`
/// has [`side`]`(c, first)` and `side(c, second)` equal to `0`, and
/// `c.x * c.x + c.y * c.y` equal to `1`, each up to rounding. It is the cross
/// product of the points divided by the length of the first two coordinates
/// of that product.
///
/// The function states no orientation for the covector, and swapping the
/// points negates it exactly. To orient it outward from a region, negate it
/// when [`side`] of a point inside the region is positive.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::{ORIGIN, side, wall_through};
/// use glam::DVec3;
///
/// let first = DVec3::new(1.0, 0.0, 1.0);
/// let second = DVec3::new(1.0, 2.0, 1.0);
/// let mut line = wall_through(first, second)?;
/// if side(line, ORIGIN) > 0.0 {
///     line = -line;
/// }
/// assert_eq!(line, DVec3::new(1.0, 0.0, -1.0));
/// # Ok::<(), escher_geometry::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error of kind
/// [`ErrorKind::DegenerateWall`](crate::ErrorKind::DegenerateWall) when the
/// sum of the squares of the first two coordinates of the cross product is
/// not positive and finite, or when the result is not finite. That happens
/// for two equal points and for a NaN or infinite coordinate. Because the
/// squares are not scaled, it also happens for two distinct, finite points
/// that are so close that the sum underflows to zero, such as points about
/// `1e-170` apart, or so far apart that it overflows, such as points about
/// `1e200` apart, and for points with coordinates so large that the cross
/// product overflows.
pub fn wall_through(first: DVec3, second: DVec3) -> Result<DVec3> {
    let w = first.cross(second);
    let n = w.x * w.x + w.y * w.y;
    let covector = w / n.sqrt();
    if n > 0.0 && n.is_finite() && covector.is_finite() {
        Ok(covector)
    } else {
        Err(Error::degenerate_wall())
    }
}

/// Returns the reflection in the line of a line covector.
///
/// Use it to build the mirror symmetry of a line, such as a side of a
/// polygon whose sides are mirrors. With `u = (c.x, c.y, 0)`, the reflection
/// maps a vector `x` to `x - 2 (c . x) u`. It fixes the points of the line and
/// maps a point at a signed distance `d` from the line to the point at `-d`.
/// The function does not check that the covector is a line covector. For a
/// vector that is not one, the result is not a reflection, and for a finite
/// vector its last row is still `(0, 0, 1)` exactly.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::reflection;
/// use glam::DVec3;
///
/// let mirror = reflection(DVec3::new(1.0, 0.0, -0.5));
/// assert_eq!(mirror * DVec3::new(0.25, 3.0, 1.0), DVec3::new(0.75, 3.0, 1.0));
/// ```
#[must_use]
pub fn reflection(covector: DVec3) -> DMat3 {
    let column = |basis: DVec3, entry: f64| {
        let twice = 2.0 * entry;
        basis - DVec3::new(covector.x * twice, covector.y * twice, 0.0)
    };
    DMat3::from_cols(
        column(DVec3::X, covector.x),
        column(DVec3::Y, covector.y),
        column(DVec3::Z, covector.z),
    )
}

/// Returns the rotation about [`ORIGIN`] by an angle in radians.
///
/// A positive angle turns the first axis towards the second, which is
/// counterclockwise with the first axis to the right and the second axis up.
/// The matrix is the same as that of
/// [`hyperbolic::rotation`](crate::hyperbolic::rotation). The function takes
/// any finite angle. A non-finite angle gives NaN entries.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::rotation;
/// use glam::DVec3;
///
/// let point = rotation(core::f64::consts::FRAC_PI_2) * DVec3::new(2.0, 0.0, 1.0);
/// assert!(point.x.abs() < 1e-15);
/// assert_eq!(point.y, 2.0);
/// assert_eq!(point.z, 1.0);
/// ```
#[must_use]
pub fn rotation(angle: f64) -> DMat3 {
    let (sin, cos) = angle.sin_cos();
    DMat3::from_cols(
        DVec3::new(cos, sin, 0.0),
        DVec3::new(-sin, cos, 0.0),
        DVec3::Z,
    )
}

/// Returns the translation that maps [`ORIGIN`] to the point whose first two
/// coordinates are those of the target.
///
/// The matrix has the columns `(1, 0, 0)`, `(0, 1, 0)`, and
/// `(target.x, target.y, 1)`, so it copies the target and is exact. The last
/// coordinate of the target is not read.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::translation;
/// use glam::DVec3;
///
/// let shift = translation(DVec3::new(-1.0, 0.5, 1.0));
/// assert_eq!(shift * DVec3::new(3.0, 4.0, 1.0), DVec3::new(2.0, 4.5, 1.0));
/// ```
#[must_use]
pub fn translation(target: DVec3) -> DMat3 {
    DMat3::from_cols(DVec3::X, DVec3::Y, DVec3::new(target.x, target.y, 1.0))
}

/// Returns the Euclidean distance between two points.
///
/// The distance is the length of the difference of their first two
/// coordinates, computed with [`f64::hypot`], which neither overflows nor
/// underflows in the squares. The last coordinates are not read. The result is
/// at least 0. It is infinite when a difference is infinite, and otherwise NaN
/// when a coordinate is NaN.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::{ORIGIN, distance};
/// use glam::DVec3;
///
/// assert_eq!(distance(ORIGIN, DVec3::new(3.0, 4.0, 1.0)), 5.0);
/// ```
#[must_use]
pub fn distance(first: DVec3, second: DVec3) -> f64 {
    (first.x - second.x).hypot(first.y - second.y)
}

/// Returns how far a matrix is from an isometry of the Euclidean plane.
///
/// Use it to check a matrix built by a long computation before relying on
/// it, by comparing the result with a tolerance of the caller's choice. With
/// `A` the upper-left 2 by 2 block of the matrix, the result is the largest
/// of the absolute entries of `A^t A` minus the identity, and of the absolute
/// differences between the last row and `(0, 0, 1)`. It is `0` for an exact
/// isometry. It is [`f64::INFINITY`] when an entry of the matrix is not
/// finite, including an entry of the translation, which the largest value does
/// not read, and when the largest value would be NaN. A check of the form
/// `defect > tolerance` then rejects the matrix.
///
/// # Examples
///
/// ```
/// use escher_geometry::euclidean::{isometry_defect, reflection, rotation, translation};
/// use glam::{DMat3, DVec3};
///
/// assert!(isometry_defect(rotation(0.5) * translation(DVec3::new(2.0, 1.0, 1.0))) < 1e-15);
/// assert!(isometry_defect(reflection(DVec3::new(0.6, 0.8, -3.0))) < 1e-15);
/// assert_eq!(isometry_defect(DMat3::from_diagonal(DVec3::new(2.0, 1.0, 1.0))), 3.0);
/// ```
#[must_use]
pub fn isometry_defect(matrix: DMat3) -> f64 {
    if !matrix.is_finite() {
        return f64::INFINITY;
    }
    let (first, second, third) = (matrix.x_axis, matrix.y_axis, matrix.z_axis);
    let differences = [
        first.x * first.x + first.y * first.y - 1.0,
        second.x * second.x + second.y * second.y - 1.0,
        first.x * second.x + first.y * second.y,
        first.z,
        second.z,
        third.z - 1.0,
    ];
    let mut defect = 0.0_f64;
    for difference in differences {
        let size = difference.abs();
        if size.is_nan() {
            return f64::INFINITY;
        }
        if size > defect {
            defect = size;
        }
    }
    defect
}

#[cfg(test)]
mod tests {
    use glam::{DMat3, DVec3};

    use super::{
        ORIGIN, distance, isometry_defect, reflection, rotation, translation, wall_distance,
        wall_through,
    };
    use crate::ErrorKind;

    #[test]
    fn reflection_in_the_line_x_one_half_maps_dyadic_points_exactly() {
        let mirror = reflection(DVec3::new(1.0, 0.0, -0.5));
        for (x, y) in [
            (0.0, 0.0),
            (0.25, -3.5),
            (1.0, 2.0),
            (-2.75, 0.125),
            (0.5, 7.0),
        ] {
            assert_eq!(
                mirror * DVec3::new(x, y, 1.0),
                DVec3::new(1.0 - x, y, 1.0),
                "({x}, {y})"
            );
        }
    }

    #[test]
    fn translation_maps_the_origin_to_its_target_exactly() {
        for target in [
            DVec3::new(0.1, -0.7, 1.0),
            DVec3::new(-1e300, 3.0, 5.0),
            DVec3::new(5e-324, 0.0, -1.0),
        ] {
            assert_eq!(
                translation(target) * ORIGIN,
                DVec3::new(target.x, target.y, 1.0)
            );
        }
    }

    #[test]
    fn reflection_of_a_finite_covector_has_the_last_row_of_an_affine_map() {
        for covector in [
            DVec3::new(0.6, 0.8, -3.0),
            DVec3::new(-0.1, 1e10, 7.5),
            DVec3::new(-2.0, -0.0, 1e-300),
            DVec3::new(1e154, -1e154, 1e154),
            DVec3::new(1e308, -1e308, f64::MAX),
        ] {
            let matrix = reflection(covector);
            assert_eq!(
                (matrix.x_axis.z, matrix.y_axis.z, matrix.z_axis.z),
                (0.0, 0.0, 1.0),
                "{covector:?}"
            );
        }
    }

    #[test]
    fn rotation_is_the_hyperbolic_rotation() {
        for angle in [0.0, 1.0, -2.5, core::f64::consts::PI, 1000.0, 1e-300] {
            assert_eq!(
                rotation(angle),
                crate::hyperbolic::rotation(angle),
                "{angle}"
            );
        }
    }

    #[test]
    fn wall_through_rejects_two_equal_points() {
        let point = DVec3::new(0.3, -0.2, 1.0);
        assert_eq!(
            wall_through(point, point).map_err(|error| error.kind()),
            Err(ErrorKind::DegenerateWall)
        );
    }

    #[test]
    fn wall_through_rejects_a_nan_or_infinite_coordinate() {
        let point = DVec3::new(0.3, -0.2, 1.0);
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for index in 0..3 {
                let mut broken = DVec3::new(1.0, 2.0, 1.0);
                broken[index] = bad;
                for (first, second) in [(broken, point), (point, broken)] {
                    assert_eq!(
                        wall_through(first, second).map_err(|error| error.kind()),
                        Err(ErrorKind::DegenerateWall),
                        "{first:?} {second:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn wall_through_rejects_an_offset_that_overflows() {
        let first = DVec3::new(1e160, 1e160, 1.0);
        let second = DVec3::new(1e160, 1e160 + 1e145, 1.0);
        assert_eq!(
            wall_through(first, second).map_err(|error| error.kind()),
            Err(ErrorKind::DegenerateWall)
        );
    }

    #[test]
    fn wall_through_rejects_finite_points_whose_squared_length_underflows() {
        let first = DVec3::new(0.0, 0.0, 1.0);
        let second = DVec3::new(1e-170, 0.0, 1.0);
        assert_eq!(
            wall_through(first, second).map_err(|error| error.kind()),
            Err(ErrorKind::DegenerateWall)
        );
    }

    #[test]
    fn wall_through_rejects_finite_points_whose_squared_length_overflows() {
        let first = DVec3::new(1e200, 0.0, 1.0);
        let second = DVec3::new(-1e200, 5.0, 1.0);
        assert_eq!(
            wall_through(first, second).map_err(|error| error.kind()),
            Err(ErrorKind::DegenerateWall)
        );
    }

    #[test]
    fn wall_through_swapped_points_is_the_exact_negation() {
        let pairs = [
            (DVec3::new(0.3, -0.2, 1.0), DVec3::new(-1.7, 4.1, 1.0)),
            (DVec3::new(60.0, 0.1, 1.0), DVec3::new(-3.0, -63.9, 1.0)),
        ];
        for (first, second) in pairs {
            let forward = wall_through(first, second);
            let backward = wall_through(second, first).map(|covector| -covector);
            assert_eq!(forward, backward, "{first:?} {second:?}");
        }
    }

    #[test]
    fn distance_of_a_nan_coordinate_is_nan() {
        assert!(distance(DVec3::new(f64::NAN, 0.0, 1.0), ORIGIN).is_nan());
    }

    #[test]
    fn isometry_defect_is_zero_for_the_identity() {
        assert_eq!(isometry_defect(DMat3::IDENTITY), 0.0);
    }

    #[test]
    fn isometry_defect_is_infinite_for_a_nan_entry() {
        for index in 0..9 {
            let mut entries = DMat3::IDENTITY.to_cols_array();
            entries[index] = f64::NAN;
            assert_eq!(
                isometry_defect(DMat3::from_cols_array(&entries)),
                f64::INFINITY,
                "entry {index}"
            );
        }
    }

    #[test]
    fn isometry_defect_exceeds_1e_minus_9_for_a_scaled_reflection() {
        let mirror = reflection(DVec3::new(0.6, 0.8, -3.0));
        assert!(isometry_defect(mirror * (1.0 + 1e-6)) > 1e-9);
    }

    #[test]
    fn isometry_defect_exceeds_1e_minus_9_for_a_reflection_with_a_wrong_last_row() {
        let mut mirror = reflection(DVec3::new(0.6, 0.8, -3.0));
        mirror.y_axis.z = 1e-6;
        assert!(isometry_defect(mirror) > 1e-9);
    }

    #[test]
    fn wall_distance_of_the_origin_to_the_line_of_the_first_axis_is_zero() {
        assert_eq!(wall_distance(DVec3::X, ORIGIN), 0.0);
    }

    #[test]
    fn wall_distance_is_nan_for_a_nan_in_either_argument() {
        let nan = DVec3::new(f64::NAN, 0.0, 1.0);
        assert!(wall_distance(nan, ORIGIN).is_nan());
        assert!(wall_distance(DVec3::X, nan).is_nan());
    }

    #[test]
    fn wall_distance_of_an_infinite_coordinate_is_infinite_or_nan() {
        let along = wall_distance(DVec3::X, DVec3::new(f64::INFINITY, 0.0, 1.0));
        assert_eq!(along, f64::INFINITY);
        let across = wall_distance(DVec3::X, DVec3::new(0.0, f64::INFINITY, 1.0));
        assert!(across.is_nan(), "{across}");
    }
}
