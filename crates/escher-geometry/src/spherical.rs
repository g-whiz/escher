//! Points, great circles, and isometries of the unit sphere.
//!
//! A sphere point is a unit [`DVec3`], and any of its coordinates may have
//! either sign, so a point of the lower hemisphere is as valid as one of the
//! upper. A wall is a great circle, the intersection of the sphere with a
//! plane through the centre of the sphere, represented by a unit normal `u` of
//! that plane. The side value of a point `x` is the dot product `u . x`, so a
//! normal is its own covector. A normal of a great circle that bounds a region
//! is oriented outward when that value is positive outside the region.
//!
//! An isometry, a map that preserves distances on the sphere, is an
//! orthogonal [`DMat3`], of determinant `1` or `-1`. `matrix * point` applies
//! an isometry, and the product `a * b` of two isometries applies `b` first.
//! A length is the angle between two points as seen from the centre of the
//! sphere, and angles are in radians.
//!
//! The functions use the names of the module
//! [`hyperbolic`](crate::hyperbolic) for the same operations, so a caller
//! changes only the module path. Every function returns without a panic for
//! every input.

use glam::{DMat3, DVec3};

use crate::error::{Error, Result};

/// The point `(0, 0, 1)` of the sphere.
///
/// It is the centre of a rotation made by [`rotation`]. A [`boost`] moves it
/// along the great circle through it and the point `(1, 0, 0)`, and
/// [`translation`] moves it to the point on the ray of its target.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, translation};
/// use glam::DVec3;
///
/// let image = translation(DVec3::new(0.0, 2.0, 0.0)) * ORIGIN;
/// assert!((image - DVec3::Y).abs().max_element() < 1e-15);
/// ```
pub const ORIGIN: DVec3 = DVec3::new(0.0, 0.0, 1.0);

/// Returns a signed number that tells on which side of a great circle a point
/// lies.
///
/// Use it to test points against a great circle, such as a side of a
/// spherical polygon. The first argument is the unit normal of the great
/// circle, which is its own covector. The value is the dot product of the
/// normal and the point. It is positive outside the great circle when the
/// normal points out of the region, zero on the great circle, and negative
/// inside. The function applies no tolerance and takes any vectors.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, side};
/// use glam::DVec3;
///
/// let equator = DVec3::new(0.0, 0.0, -1.0);
/// assert_eq!(side(equator, ORIGIN), -1.0);
/// assert_eq!(side(equator, DVec3::X), 0.0);
/// ```
#[must_use]
pub fn side(covector: DVec3, point: DVec3) -> f64 {
    covector.dot(point)
}

/// Returns the sphere point on the ray of a vector, the vector divided by its
/// length.
///
/// Use it to put a computed point back onto the sphere after rounding has
/// moved it, or to turn a sum of points into their midpoint. The function
/// puts no condition on the sign of any coordinate.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, normalize_point};
/// use glam::DVec3;
///
/// assert_eq!(normalize_point(DVec3::new(0.0, 0.0, 2.0))?, ORIGIN);
/// assert_eq!(normalize_point(DVec3::new(0.0, -3.0, 0.0))?, DVec3::new(0.0, -1.0, 0.0));
/// # Ok::<(), escher_geometry::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error of kind [`ErrorKind::NotAPoint`](crate::ErrorKind::NotAPoint)
/// when the dot product of the vector with itself is not positive and
/// finite. That happens for the zero vector, for a NaN or infinite
/// coordinate, and for a vector so short or so long that the dot product
/// underflows to zero or overflows to infinity.
pub fn normalize_point(vector: DVec3) -> Result<DVec3> {
    let q = vector.dot(vector);
    if q > 0.0 && q.is_finite() {
        Ok(vector / q.sqrt())
    } else {
        Err(Error::no_length())
    }
}

/// Returns the spherical distance between two points, the angle between them
/// in radians.
///
/// Use it to measure how far apart two points are. The distance is the
/// inverse cosine of their dot product, with the dot product clamped to
/// `[-1, 1]`, so the result is in `[0, pi]` even when rounding has moved the
/// points. A NaN dot product gives NaN.
///
/// The result loses accuracy for points that are close together and for
/// points that are nearly antipodal, because the inverse cosine amplifies the
/// rounding error of the dot product near `1` and `-1`. For two equal points,
/// the result can be a small positive number instead of `0`.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, boost, distance};
///
/// let point = boost(2.0) * ORIGIN;
/// assert!((distance(ORIGIN, point) - 2.0).abs() < 1e-12);
/// assert_eq!(distance(ORIGIN, -ORIGIN), core::f64::consts::PI);
/// ```
#[must_use]
pub fn distance(first: DVec3, second: DVec3) -> f64 {
    first.dot(second).clamp(-1.0, 1.0).acos()
}

/// Returns the spherical distance from the sphere point on the ray of a
/// vector to a great circle, an angle in radians.
///
/// Use it to measure how far a point lies from a great circle, such as to
/// draw a border of a given width along a side of a spherical polygon. The
/// first argument is the unit normal of the great circle, which is its own
/// covector. The second is any vector other than the zero vector, which need
/// not have length 1. The distance is the inverse sine of the absolute value
/// of [`side`]`(normal, point)` divided by the length of the vector, with
/// that ratio lowered to at most `1`, so a vector and each positive multiple
/// of it give the same distance up to rounding. It is the distance to the
/// whole great circle, not to an arc of it.
///
/// The result is in `[0, pi/2]`, and NaN when a coordinate of either
/// argument is NaN or the vector is the zero vector. It loses accuracy for a
/// point near a pole of the great circle, where the inverse sine amplifies
/// the rounding error of its argument.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, boost, wall_distance};
/// use glam::DVec3;
///
/// let point = boost(0.5) * ORIGIN;
/// assert!((wall_distance(DVec3::Z, point) - (core::f64::consts::FRAC_PI_2 - 0.5)).abs() < 1e-14);
/// assert!((wall_distance(DVec3::X, 2.0 * point) - 0.5).abs() < 1e-14);
/// assert_eq!(wall_distance(DVec3::X, ORIGIN), 0.0);
/// ```
#[must_use]
pub fn wall_distance(covector: DVec3, point: DVec3) -> f64 {
    let s = side(covector, point).abs() / point.dot(point).sqrt();
    let clamped = if s > 1.0 { 1.0 } else { s };
    clamped.asin()
}

/// Returns a unit normal of the great circle that contains two points.
///
/// Use it to build the great circle through two vertices, such as a side of
/// a spherical polygon. The points must be two sphere points that are neither
/// equal nor antipodal. The result `u` has [`side`]`(u, first)` and
/// `side(u, second)` equal to `0`, and `u . u` equal to `1`, each up to
/// rounding. It is the cross product of the points divided by its length.
/// The function gives no accuracy guarantee for two points that are very
/// close together or very close to antipodal, because the cross product then
/// loses most of its digits.
///
/// The function states no orientation for the normal, and swapping the
/// points negates it exactly. To make the normal point out of a region,
/// negate it when [`side`] of a point inside the region is positive.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, side, wall_through};
/// use glam::DVec3;
///
/// let mut normal = wall_through(DVec3::X, DVec3::Y)?;
/// if side(normal, ORIGIN) > 0.0 {
///     normal = -normal;
/// }
/// assert_eq!(normal, DVec3::new(0.0, 0.0, -1.0));
/// # Ok::<(), escher_geometry::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error of kind
/// [`ErrorKind::DegenerateWall`](crate::ErrorKind::DegenerateWall) when the
/// squared length of the cross product is not positive and finite, or when
/// the result is not finite. That happens for two equal points, for two
/// exactly antipodal points, and for a NaN or infinite coordinate. It also
/// happens for two distinct points whose squared cross product underflows to
/// zero or overflows to infinity.
pub fn wall_through(first: DVec3, second: DVec3) -> Result<DVec3> {
    let w = first.cross(second);
    let n = w.dot(w);
    let normal = w / n.sqrt();
    if n > 0.0 && n.is_finite() && normal.is_finite() {
        Ok(normal)
    } else {
        Err(Error::degenerate_wall())
    }
}

/// Returns the rotation of the sphere that moves [`ORIGIN`] by an angle along
/// the great circle through it and the point `(1, 0, 0)`, towards positive
/// `x`.
///
/// Use it with [`rotation`] to build other rotations of the sphere. It has the
/// name of [`hyperbolic::boost`](crate::hyperbolic::boost), the map that does
/// the same in the hyperbolic plane. The matrix has the columns
/// `(cos a, 0, -sin a)`, `(0, 1, 0)`, and `(sin a, 0, cos a)`, so it maps
/// [`ORIGIN`] to `(sin a, 0, cos a)`. The function takes any finite angle. A
/// non-finite angle gives NaN entries.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, boost};
///
/// let point = boost(core::f64::consts::FRAC_PI_2) * ORIGIN;
/// assert_eq!(point.x, 1.0);
/// assert!(point.z.abs() < 1e-15);
/// ```
#[must_use]
pub fn boost(angle: f64) -> DMat3 {
    let (sin, cos) = angle.sin_cos();
    DMat3::from_cols(
        DVec3::new(cos, 0.0, -sin),
        DVec3::Y,
        DVec3::new(sin, 0.0, cos),
    )
}

/// Returns the rotation about [`ORIGIN`] by an angle in radians.
///
/// Use it with [`boost`] to build other rotations of the sphere. A positive
/// angle turns the first axis towards the second, which is counterclockwise
/// as seen from outside the sphere above [`ORIGIN`]. The matrix is the same
/// as that of [`hyperbolic::rotation`](crate::hyperbolic::rotation). The
/// function takes any finite angle. A non-finite angle gives NaN entries.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::rotation;
/// use glam::DVec3;
///
/// let point = rotation(core::f64::consts::FRAC_PI_2) * DVec3::X;
/// assert!(point.x.abs() < 1e-15);
/// assert_eq!(point.y, 1.0);
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

/// Returns a rotation of the sphere that maps [`ORIGIN`] to the sphere point
/// on the ray of the target.
///
/// Use it to move a construction made at [`ORIGIN`] to another point. The
/// rotation is `rotation(phi) * boost(rho) * rotation(-phi)`, where `phi` is
/// the direction of the target's first two coordinates and `rho` is the angle
/// of the target from [`ORIGIN`], computed as the two-argument arctangent of
/// the length of the first two coordinates and the last coordinate. All
/// three coordinates are read, and the length of the target does not matter.
/// For [`ORIGIN`] and for the zero vector, the result is the identity. A NaN
/// coordinate in the target gives NaN entries.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, translation};
/// use glam::DVec3;
///
/// let target = DVec3::new(0.6, 0.0, -0.8);
/// let image = translation(target) * ORIGIN;
/// assert!((image - target).abs().max_element() < 1e-15);
/// ```
#[must_use]
pub fn translation(target: DVec3) -> DMat3 {
    let planar = target.x.hypot(target.y);
    let rho = if planar == 0.0 && target.z == 0.0 {
        0.0
    } else {
        planar.atan2(target.z)
    };
    let phi = if planar == 0.0 {
        0.0
    } else {
        target.y.atan2(target.x)
    };
    rotation(phi) * boost(rho) * rotation(-phi)
}

/// Returns the reflection in the great circle of a unit normal.
///
/// Use it to build the mirror symmetry of a great circle, such as a side of a
/// spherical reflection triangle. The reflection maps a vector `x` to
/// `x - 2 (normal . x) normal`. It fixes the points of the great circle and
/// maps the normal to its negation. The function does not check that the
/// normal is a unit vector. For a vector that is not one, the result is not a
/// reflection.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::reflection;
/// use glam::DVec3;
///
/// let mirror = reflection(DVec3::X);
/// assert_eq!(mirror * DVec3::new(0.5, 0.25, -0.75), DVec3::new(-0.5, 0.25, -0.75));
/// ```
#[must_use]
pub fn reflection(normal: DVec3) -> DMat3 {
    DMat3::from_cols(
        DVec3::X - normal * (2.0 * normal.x),
        DVec3::Y - normal * (2.0 * normal.y),
        DVec3::Z - normal * (2.0 * normal.z),
    )
}

/// Returns the inverse of an isometry of the sphere, its transpose.
///
/// Use it instead of a general matrix inverse, because it copies each entry
/// and so adds no rounding error. For a matrix that is not orthogonal, the
/// result is its transpose, which is not its inverse.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{ORIGIN, boost, inverse, rotation};
///
/// let isometry = rotation(0.5) * boost(1.0);
/// let back = inverse(isometry) * (isometry * ORIGIN);
/// assert!((back - ORIGIN).abs().max_element() < 1e-15);
/// ```
#[must_use]
pub fn inverse(isometry: DMat3) -> DMat3 {
    isometry.transpose()
}

/// Returns how far a matrix is from an isometry of the sphere.
///
/// Use it to check a matrix built by a long computation before relying on
/// it, by comparing the result with a tolerance of the caller's choice. The
/// result is the largest absolute difference between the dot product of two
/// columns of the matrix and that of the matching basis vectors, which is `1`
/// for a column with itself and `0` for two different columns. It is `0` for
/// an exact isometry, of either determinant. It is [`f64::INFINITY`] when an
/// entry of the matrix is not finite, and when the largest value would be
/// NaN, so that a check of the form `defect > tolerance` rejects the matrix.
///
/// # Examples
///
/// ```
/// use escher_geometry::spherical::{boost, isometry_defect, reflection, rotation};
/// use glam::{DMat3, DVec3};
///
/// assert!(isometry_defect(rotation(0.5) * boost(1.0)) < 1e-15);
/// assert!(isometry_defect(reflection(DVec3::new(0.6, 0.0, 0.8))) < 1e-15);
/// assert_eq!(isometry_defect(DMat3::from_diagonal(DVec3::new(2.0, 1.0, 1.0))), 3.0);
/// ```
#[must_use]
pub fn isometry_defect(matrix: DMat3) -> f64 {
    if !matrix.is_finite() {
        return f64::INFINITY;
    }
    let (first, second, third) = (matrix.x_axis, matrix.y_axis, matrix.z_axis);
    let differences = [
        first.dot(first) - 1.0,
        second.dot(second) - 1.0,
        third.dot(third) - 1.0,
        first.dot(second),
        first.dot(third),
        second.dot(third),
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
        ORIGIN, boost, inverse, isometry_defect, normalize_point, reflection, rotation,
        translation, wall_distance, wall_through,
    };
    use crate::ErrorKind;

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
    fn boost_of_zero_is_the_identity() {
        assert_eq!(boost(0.0), DMat3::IDENTITY);
    }

    #[test]
    fn translation_of_the_origin_and_of_the_zero_vector_is_the_identity() {
        assert_eq!(translation(ORIGIN), DMat3::IDENTITY);
        assert_eq!(translation(DVec3::ZERO), DMat3::IDENTITY);
    }

    #[test]
    fn translation_of_a_zero_vector_with_negative_zeros_is_the_identity() {
        for target in [
            -DVec3::ZERO,
            DVec3::new(-0.0, 0.0, -0.0),
            DVec3::new(0.0, -0.0, -0.0),
            DVec3::new(-0.0, -0.0, 0.0),
        ] {
            assert_eq!(translation(target), DMat3::IDENTITY, "{target:?}");
        }
    }

    #[test]
    fn inverse_is_the_transpose() {
        let matrix = DMat3::from_cols_array(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        assert_eq!(
            inverse(matrix),
            DMat3::from_cols_array(&[1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0])
        );
    }

    #[test]
    fn reflection_in_the_first_axis_negates_the_first_coordinate_of_dyadic_points() {
        let mirror = reflection(DVec3::X);
        for (x, y, z) in [
            (0.0, 0.0, 1.0),
            (0.25, -3.5, 0.5),
            (1.0, 2.0, -0.125),
            (-2.75, 0.125, 8.0),
        ] {
            assert_eq!(
                mirror * DVec3::new(x, y, z),
                DVec3::new(-x, y, z),
                "({x}, {y}, {z})"
            );
        }
    }

    #[test]
    fn normalize_point_rejects_a_vector_without_a_finite_positive_length() {
        let mut cases = vec![DVec3::ZERO, DVec3::splat(1e-170), DVec3::splat(1e160)];
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for index in 0..3 {
                let mut broken = DVec3::new(0.5, -0.5, 0.25);
                broken[index] = bad;
                cases.push(broken);
            }
        }
        for vector in cases {
            assert_eq!(
                normalize_point(vector).map_err(|error| error.kind()),
                Err(ErrorKind::NotAPoint),
                "{vector:?}"
            );
        }
    }

    #[test]
    fn normalize_point_accepts_the_negated_origin() {
        assert_eq!(normalize_point(-ORIGIN), Ok(-ORIGIN));
    }

    #[test]
    fn wall_through_rejects_equal_and_antipodal_points() {
        let point = DVec3::new(0.48, -0.6, 0.64);
        for (first, second) in [(point, point), (point, -point)] {
            assert_eq!(
                wall_through(first, second).map_err(|error| error.kind()),
                Err(ErrorKind::DegenerateWall),
                "{first:?} {second:?}"
            );
        }
    }

    #[test]
    fn wall_through_rejects_a_nan_or_infinite_coordinate() {
        let point = DVec3::new(0.48, -0.6, 0.64);
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for index in 0..3 {
                let mut broken = DVec3::new(0.6, 0.0, 0.8);
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
    fn wall_through_swapped_points_is_the_exact_negation() {
        let pairs = [
            (DVec3::new(0.48, -0.6, 0.64), DVec3::new(0.0, 0.6, -0.8)),
            (DVec3::new(0.1, 0.2, 0.97), DVec3::new(-0.9, 0.3, -0.31)),
        ];
        for (first, second) in pairs {
            let forward = wall_through(first, second);
            let backward = wall_through(second, first).map(|normal| -normal);
            assert_eq!(forward, backward, "{first:?} {second:?}");
        }
    }

    #[test]
    fn isometry_defect_is_zero_for_the_identity() {
        assert_eq!(isometry_defect(DMat3::IDENTITY), 0.0);
    }

    #[test]
    fn isometry_defect_is_infinite_for_a_nan_or_infinite_entry() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for index in 0..9 {
                let mut entries = DMat3::IDENTITY.to_cols_array();
                entries[index] = bad;
                assert_eq!(
                    isometry_defect(DMat3::from_cols_array(&entries)),
                    f64::INFINITY,
                    "{bad} at entry {index}"
                );
            }
        }
    }

    #[test]
    fn wall_distance_of_the_origin_to_the_great_circle_of_the_first_axis_is_zero() {
        assert_eq!(wall_distance(DVec3::X, ORIGIN), 0.0);
    }

    #[test]
    fn wall_distance_is_nan_for_a_nan_in_either_argument() {
        let nan = DVec3::new(f64::NAN, 0.0, 1.0);
        assert!(wall_distance(nan, ORIGIN).is_nan());
        assert!(wall_distance(DVec3::X, nan).is_nan());
    }

    #[test]
    fn wall_distance_of_the_zero_vector_is_nan() {
        assert!(wall_distance(DVec3::X, DVec3::ZERO).is_nan());
    }

    #[test]
    fn wall_distance_of_a_ratio_above_one_is_the_arcsine_of_one() {
        let result = wall_distance(DVec3::new(2.0, 0.0, 0.0), DVec3::X);
        assert_eq!(result, 1.0_f64.asin());
    }

    #[test]
    fn wall_distance_of_an_infinite_point_coordinate_is_nan() {
        for point in [
            DVec3::new(f64::INFINITY, 0.0, 0.0),
            DVec3::new(0.0, 0.0, f64::NEG_INFINITY),
        ] {
            let result = wall_distance(DVec3::X, point);
            assert!(result.is_nan(), "{result} for {point:?}");
        }
    }

    #[test]
    fn wall_distance_of_an_infinite_covector_coordinate_is_the_arcsine_of_one() {
        let result = wall_distance(DVec3::new(f64::INFINITY, 0.0, 0.0), DVec3::X);
        assert_eq!(result, 1.0_f64.asin());
    }
}
