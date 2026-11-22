//! Points, walls, isometries, and the Poincare disk map of the hyperbolic
//! plane in its linear model.
//!
//! A point is a [`DVec3`] `x` on the hyperboloid, where [`form`]`(x, x)` is
//! `-1` and `x.z` is positive. The time coordinate, the one with the negative
//! sign in the form, is always the last coordinate, `.z`. A wall, which is a
//! geodesic line, is represented by a unit normal `u`, a [`DVec3`] with
//! `form(u, u)` equal to `1`. The wall is the set of points `x` with
//! `form(x, u)` equal to `0`. A normal of a wall that bounds a region points
//! out of the region. An isometry, a map that preserves distances, is a
//! [`DMat3`] that maps the hyperboloid to itself. A position in the Poincare
//! disk, which shows the whole plane inside the unit circle, is a [`DVec2`],
//! and so is a position in the upper half-plane, band, and Klein models, which
//! [`disk_from_half_plane`], [`disk_from_band`], and [`disk_from_klein`] map
//! to the disk.
//!
//! `matrix * point` applies an isometry. The product `a * b` of two
//! isometries applies `b` first and is the only way to compose them, so the
//! module has no compose function. Lengths are hyperbolic distances in the
//! metric of curvature `-1`, and angles are in radians.
//!
//! The types carry no proof of what they hold, so each function states its
//! domain. Every function returns without a panic for every input. Outside
//! its domain a function returns an unspecified value, except
//! [`normalize_point`], [`wall_through`], [`from_disk`], and the three
//! functions that map a model to the disk, which check their input. Only
//! [`normalize_point`] and [`wall_through`] scale their result to unit
//! length. Rounding moves a computed point off the hyperboloid, and the
//! caller decides when to put it back with [`normalize_point`].

use core::f64::consts::FRAC_PI_4;

use glam::{DMat2, DMat3, DVec2, DVec3};
use num_complex::Complex64;

use crate::error::{Error, Result};

/// The point at the centre of the Poincare disk, `(0, 0, 1)`.
///
/// It is the usual base point of a construction, such as the centre of a
/// rotation made by [`rotation`].
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, form, to_disk};
/// use glam::DVec2;
///
/// assert_eq!(form(ORIGIN, ORIGIN), -1.0);
/// assert_eq!(to_disk(ORIGIN), DVec2::ZERO);
/// ```
pub const ORIGIN: DVec3 = DVec3::new(0.0, 0.0, 1.0);

/// Returns the Minkowski form of two vectors, `x.x * y.x + x.y * y.y - x.z * y.z`.
///
/// Use it to test the kind of a vector or the relation between two vectors.
/// The form of a point with itself is `-1`, and the form of a unit normal
/// with itself is `1`. A point lies on the wall of a unit normal when their
/// form is `0`. The function takes any vectors and sums the three products
/// in the order written.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, form};
/// use glam::DVec3;
///
/// let normal = DVec3::new(1.0, 0.0, 0.0);
/// assert_eq!(form(normal, normal), 1.0);
/// assert_eq!(form(normal, ORIGIN), 0.0);
/// ```
#[must_use]
pub fn form(x: DVec3, y: DVec3) -> f64 {
    x.x * y.x + x.y * y.y - x.z * y.z
}

/// Returns the covector of a normal, the vector `c` whose dot product with
/// any vector `x` is [`form`]`(x, normal)`.
///
/// Use it to store the side test of a wall once and evaluate it with
/// [`side`] at many points. The covector is the normal with its last
/// coordinate negated. The function takes any vector and is exact.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::covector;
/// use glam::DVec3;
///
/// assert_eq!(covector(DVec3::new(1.0, 2.0, 3.0)), DVec3::new(1.0, 2.0, -3.0));
/// ```
#[must_use]
pub fn covector(normal: DVec3) -> DVec3 {
    DVec3::new(normal.x, normal.y, -normal.z)
}

/// Returns a signed number that tells on which side of a wall a point lies.
///
/// Use it to test points against a wall, such as a wall that bounds a
/// region. The first argument is the [`covector`] of the wall's unit normal.
/// The value is [`form`]`(point, normal)` computed as one dot product. It is
/// positive outside the wall when the normal points out of the region, zero
/// on the wall, and negative inside. The function applies no tolerance and
/// takes any vectors.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, boost, covector, side};
/// use glam::DVec3;
///
/// let wall = covector(DVec3::new(1.0, 0.0, 0.0));
/// assert!(side(wall, boost(1.0) * ORIGIN) > 0.0);
/// assert!(side(wall, boost(-1.0) * ORIGIN) < 0.0);
/// ```
#[must_use]
pub fn side(covector: DVec3, point: DVec3) -> f64 {
    covector.dot(point)
}

/// Returns the point of the hyperboloid on the ray of a vector, the vector
/// divided by the square root of `-form(vector, vector)`.
///
/// Use it to put a computed point back onto the hyperboloid after rounding
/// has moved it, or to turn a sum of points into their midpoint. The vector
/// must be timelike, which means that [`form`]`(vector, vector)` is
/// negative, and its last coordinate must be positive.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, form, normalize_point};
/// use glam::DVec3;
///
/// let point = normalize_point(DVec3::new(0.0, 0.0, 2.0))?;
/// assert_eq!(point, ORIGIN);
///
/// let middle = normalize_point(DVec3::new(0.5, 0.0, 2.0))?;
/// assert!((form(middle, middle) + 1.0).abs() < 1e-15);
/// # Ok::<(), escher_geometry::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error of kind [`ErrorKind::NotAPoint`](crate::ErrorKind::NotAPoint)
/// when the vector is spacelike or null, when its last coordinate is not
/// positive, when a coordinate is NaN or infinite, or when
/// `-form(vector, vector)` underflows to zero or overflows to infinity. A
/// vector with a negative last coordinate is rejected, not flipped onto the
/// hyperboloid.
pub fn normalize_point(vector: DVec3) -> Result<DVec3> {
    let q = -form(vector, vector);
    if q > 0.0 && q.is_finite() && vector.z > 0.0 {
        Ok(vector / q.sqrt())
    } else {
        Err(Error::not_a_point())
    }
}

/// Returns the hyperbolic distance between two points.
///
/// Use it to measure how far apart two points are. The distance is the
/// inverse hyperbolic cosine of `-form(first, second)`, with that argument
/// raised to at least `1`, so the result is never negative and is a number
/// for two points even when rounding has moved them. A NaN in either point
/// gives NaN. For a vector off the hyperboloid, the result is the same
/// clamped formula.
///
/// The result loses accuracy for points that are close together and for
/// points far from [`ORIGIN`]. The rounding error of the form grows with the
/// distance of the points from [`ORIGIN`], and the inverse hyperbolic cosine
/// amplifies that error near `1`. For two equal points other than
/// [`ORIGIN`], the result can be a small positive number instead of `0`.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, boost, distance};
///
/// let point = boost(2.0) * ORIGIN;
/// assert!((distance(ORIGIN, point) - 2.0).abs() < 1e-12);
/// assert_eq!(distance(ORIGIN, ORIGIN), 0.0);
/// ```
#[must_use]
pub fn distance(first: DVec3, second: DVec3) -> f64 {
    let c = -form(first, second);
    let clamped = if c >= 1.0 || c.is_nan() { c } else { 1.0 };
    clamped.acosh()
}

/// Returns a unit normal of the wall that contains two points.
///
/// Use it to build the wall through two vertices, such as a side of a
/// polygon. The points must be two distinct points of the hyperboloid. The
/// result `u` has [`form`]`(u, first)` and `form(u, second)` equal to `0`,
/// and `form(u, u)` equal to `1`, each up to rounding. The function gives no
/// accuracy guarantee for two points that are very close together, and may
/// return the error below for them, because the cross product of nearly
/// parallel vectors loses most of its digits.
///
/// The normal is the covector of the cross product of the points, divided by
/// the square root of the form of that covector with itself. The function
/// states no orientation for the normal, and swapping the points negates it.
/// To make the normal point out of a region, negate it when [`side`] of a
/// point inside the region is positive.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, boost, covector, form, rotation, side, wall_through};
///
/// let first = boost(1.0) * ORIGIN;
/// let second = rotation(1.0) * first;
/// let mut normal = wall_through(first, second)?;
/// if side(covector(normal), ORIGIN) > 0.0 {
///     normal = -normal;
/// }
/// assert!(form(normal, first).abs() < 1e-14);
/// assert!((form(normal, normal) - 1.0).abs() < 1e-14);
/// assert!(side(covector(normal), ORIGIN) < 0.0);
/// # Ok::<(), escher_geometry::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error of kind
/// [`ErrorKind::DegenerateWall`](crate::ErrorKind::DegenerateWall) when the
/// form of the covector of the cross product with itself is not positive and
/// finite. That happens for two equal points, for two points so close that
/// rounding leaves no positive form, and for a NaN or infinite coordinate.
pub fn wall_through(first: DVec3, second: DVec3) -> Result<DVec3> {
    let w = covector(first.cross(second));
    let n = form(w, w);
    if n > 0.0 && n.is_finite() {
        Ok(w / n.sqrt())
    } else {
        Err(Error::degenerate_wall())
    }
}

/// Returns the boost along the first axis by a length, an isometry that
/// moves [`ORIGIN`] that distance towards positive `x`.
///
/// Use it with [`rotation`] to build other orientation-preserving isometries.
/// The length is the full hyperbolic distance. The matrix has the columns
/// `(cosh l, 0, sinh l)`, `(0, 1, 0)`, and `(sinh l, 0, cosh l)`. A length
/// above about 710.5 in absolute value gives infinite entries, and a NaN
/// length gives NaN entries.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, boost, distance};
///
/// let point = boost(3.0) * ORIGIN;
/// assert!(point.x > 0.0);
/// assert!((distance(ORIGIN, point) - 3.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn boost(length: f64) -> DMat3 {
    let cosh = length.cosh();
    let sinh = length.sinh();
    DMat3::from_cols(
        DVec3::new(cosh, 0.0, sinh),
        DVec3::Y,
        DVec3::new(sinh, 0.0, cosh),
    )
}

/// Returns the rotation about [`ORIGIN`] by an angle in radians.
///
/// Use it with [`boost`] to build other orientation-preserving isometries. A
/// positive angle turns the first axis towards the second, which is
/// counterclockwise with the first axis to the right and the second axis up.
/// The function takes any finite angle. A non-finite angle gives NaN
/// entries.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, boost, rotation};
///
/// let point = rotation(core::f64::consts::FRAC_PI_2) * (boost(1.0) * ORIGIN);
/// assert!(point.x.abs() < 1e-15);
/// assert!(point.y > 0.0);
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

/// Returns an isometry that maps [`ORIGIN`] to the point of the hyperboloid
/// whose first two coordinates are those of the target.
///
/// Use it to move a construction made at [`ORIGIN`] to another point. The
/// isometry is `rotation(phi) * boost(rho) * rotation(-phi)`, where `phi` is
/// the direction of the target's first two coordinates and `rho` is the
/// inverse hyperbolic sine of their length. The last coordinate of the
/// target is not read, so for a target that is a point the isometry maps
/// [`ORIGIN`] to that point. The function does not promise that the result is
/// the translation along the geodesic from [`ORIGIN`] to the target. A
/// non-finite target gives infinite or NaN entries.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, translation};
/// use glam::DVec3;
///
/// let target = DVec3::new(0.75, 0.0, 1.25);
/// let image = translation(target) * ORIGIN;
/// assert!((image - target).abs().max_element() < 1e-15);
/// ```
#[must_use]
pub fn translation(target: DVec3) -> DMat3 {
    let rho = target.x.hypot(target.y).asinh();
    let phi = target.y.atan2(target.x);
    rotation(phi) * boost(rho) * rotation(-phi)
}

/// Returns the reflection in the wall of a unit normal.
///
/// Use it to build the mirror symmetry of a wall, such as a side of a
/// reflection triangle. The reflection maps a vector `x` to
/// `x - 2 form(normal, x) normal`. It fixes the points of the wall and maps
/// the normal to its negation. The function does not check that the normal
/// is a unit normal. For a vector that is not one, the result is not a
/// reflection.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, boost, reflection};
/// use glam::DVec3;
///
/// let mirror = reflection(DVec3::new(1.0, 0.0, 0.0));
/// assert_eq!(mirror * ORIGIN, ORIGIN);
/// let image = mirror * (boost(1.0) * ORIGIN);
/// assert!((image - boost(-1.0) * ORIGIN).abs().max_element() < 1e-15);
/// ```
#[must_use]
pub fn reflection(normal: DVec3) -> DMat3 {
    let c = covector(normal);
    DMat3::from_cols(
        DVec3::X - normal * (2.0 * c.x),
        DVec3::Y - normal * (2.0 * c.y),
        DVec3::Z - normal * (2.0 * c.z),
    )
}

/// Returns the inverse of an isometry, its transpose with the four entries
/// that have exactly one index in the last row or column negated.
///
/// Use it instead of a general matrix inverse, because it copies or negates
/// each entry and so adds no rounding error. For a matrix that is not an
/// isometry, the result is the same transpose with sign changes, which is not
/// the inverse of that matrix.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, boost, inverse, rotation};
///
/// let isometry = rotation(0.5) * boost(1.0);
/// let point = isometry * ORIGIN;
/// let back = inverse(isometry) * point;
/// assert!((back - ORIGIN).abs().max_element() < 1e-14);
/// ```
#[must_use]
pub fn inverse(isometry: DMat3) -> DMat3 {
    let m = isometry;
    DMat3::from_cols(
        DVec3::new(m.x_axis.x, m.y_axis.x, -m.z_axis.x),
        DVec3::new(m.x_axis.y, m.y_axis.y, -m.z_axis.y),
        DVec3::new(-m.x_axis.z, -m.y_axis.z, m.z_axis.z),
    )
}

/// Returns how far a matrix is from an isometry of the hyperboloid.
///
/// Use it to check a matrix built by a long computation before relying on
/// it, by comparing the result with a tolerance of the caller's choice. The
/// result is the largest absolute difference between the [`form`] of two
/// columns of the matrix and the form of the matching basis vectors, which
/// is `1` for each of the first two columns with itself, `-1` for the last
/// column with itself, and `0` for two different columns. It is `0` for an
/// exact isometry.
///
/// The result is [`f64::INFINITY`] when the last entry of the last column is
/// not positive, because a matrix that preserves the form and has such an
/// entry maps the hyperboloid onto its other sheet, and when the difference
/// is NaN, so that a check of the form `defect > tolerance` rejects the
/// matrix.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{boost, isometry_defect, rotation};
/// use glam::DMat3;
///
/// assert!(isometry_defect(rotation(0.5) * boost(1.0)) < 1e-14);
/// assert_eq!(isometry_defect(-DMat3::IDENTITY), f64::INFINITY);
/// assert!(isometry_defect(DMat3::from_diagonal([2.0, 1.0, 1.0].into())) > 1.0);
/// ```
#[must_use]
pub fn isometry_defect(matrix: DMat3) -> f64 {
    let (first, second, third) = (matrix.x_axis, matrix.y_axis, matrix.z_axis);
    let preserves_sheets = third.z > 0.0;
    if !preserves_sheets {
        return f64::INFINITY;
    }
    let differences = [
        form(first, first) - 1.0,
        form(second, second) - 1.0,
        form(third, third) + 1.0,
        form(first, second),
        form(first, third),
        form(second, third),
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

/// Returns the position in the Poincare disk of a point of the hyperboloid.
///
/// Use it to draw a point in the Poincare disk model, which shows the whole
/// hyperbolic plane inside the unit circle. The position is
/// `(x.x / (x.z + 1), x.y / (x.z + 1))`. The position of a point far from
/// [`ORIGIN`] is within rounding error of the unit circle, so [`from_disk`]
/// may give [`None`] for it. For a vector off the hyperboloid, the result is
/// the same formula, which can be infinite or NaN.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, boost, to_disk};
/// use glam::DVec2;
///
/// assert_eq!(to_disk(ORIGIN), DVec2::ZERO);
/// let position = to_disk(boost(1.0) * ORIGIN);
/// assert!((position.x - 0.5_f64.tanh()).abs() < 1e-15);
/// ```
#[must_use]
pub fn to_disk(point: DVec3) -> DVec2 {
    let s = point.z + 1.0;
    DVec2::new(point.x / s, point.y / s)
}

/// Returns the point of the hyperboloid at a position of the Poincare disk,
/// or [`None`] for a position on or outside the unit circle.
///
/// Use it to find which point a pixel of a disk image shows, and treat
/// [`None`] as background. The point is
/// `(2 z.x, 2 z.y, 1 + |z|^2) / (1 - |z|^2)`. The result is [`None`] exactly
/// when the computed `1 - |z|^2` is not positive, which includes a NaN or
/// infinite coordinate. Near the unit circle the result loses accuracy,
/// because `1 - |z|^2` cancels, and the relative error of each coordinate
/// grows in proportion to `1 / (1 - |z|^2)`.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::{ORIGIN, from_disk, to_disk};
/// use glam::DVec2;
///
/// assert_eq!(from_disk(DVec2::ZERO), Some(ORIGIN));
/// assert_eq!(from_disk(DVec2::new(0.6, 0.8)), None);
///
/// let position = DVec2::new(0.25, -0.5);
/// let back = from_disk(position).map(to_disk);
/// assert!(back.is_some_and(|back| (back - position).abs().max_element() < 1e-15));
/// ```
#[must_use]
pub fn from_disk(position: DVec2) -> Option<DVec3> {
    let s = position.length_squared();
    let d = 1.0 - s;
    if d > 0.0 {
        Some(DVec3::new(
            2.0 * position.x / d,
            2.0 * position.y / d,
            (1.0 + s) / d,
        ))
    } else {
        None
    }
}

/// Returns the position in the Poincare disk of a position of the upper
/// half-plane model, with the Jacobian of the map, or [`None`] for a position
/// outside the half-plane.
///
/// Use it to draw the hyperbolic plane in the upper half-plane model, in which
/// the plane is the set of complex numbers `w = w.x + i w.y` with `w.y`
/// positive. Pass the result to [`from_disk`] to find the point that the
/// position shows. The map is `(i w + 1) / (w + i)`, the standard
/// transformation of J. G. Ratcliffe, "Foundations of Hyperbolic Manifolds",
/// which sends `i` to the centre of the disk. The picture of the half-plane
/// sits the same way up as the picture of the disk: a position above `i` maps
/// to the positive second axis of the disk, and a position to the right of
/// `i` at the same height maps to a disk position with a positive first
/// coordinate.
///
/// The Jacobian, the matrix of the partial derivatives of the map, has the
/// derivative in `w.x` as its first column and the derivative in `w.y` as its
/// second. The map is holomorphic with the complex derivative
/// `c = -2 / (w + i)^2`, so the columns are `(c.re, c.im)` and
/// `(-c.im, c.re)`. The result is [`None`] exactly when `w.y` is not positive
/// or a coordinate is NaN or infinite. A position close to the real axis or
/// far from `i` can give a disk position on the unit circle or one that is
/// not finite, for which [`from_disk`] gives [`None`].
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::disk_from_half_plane;
/// use glam::{DMat2, DVec2};
///
/// let (position, jacobian) = disk_from_half_plane(DVec2::new(0.0, 1.0)).ok_or("outside")?;
/// assert_eq!(position.length(), 0.0);
/// assert_eq!(jacobian.abs(), DMat2::from_diagonal(DVec2::splat(0.5)));
/// assert_eq!(disk_from_half_plane(DVec2::new(1.0, 0.0)), None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn disk_from_half_plane(position: DVec2) -> Option<(DVec2, DMat2)> {
    (position.y > 0.0 && position.is_finite()).then(|| {
        let w = Complex64::new(position.x, position.y);
        let q = w + Complex64::i();
        let z = (Complex64::i() * w + 1.0) / q;
        let c = -2.0 / (q * q);
        (DVec2::new(z.re, z.im), holomorphic_jacobian(c))
    })
}

/// Returns the position in the Poincare disk of a position of the band model,
/// with the Jacobian of the map, or [`None`] for a position outside the band.
///
/// Use it to draw the hyperbolic plane in the band model, in which the plane
/// is the strip of complex numbers `w = w.x + i w.y` with `|w.y|` below `1`,
/// and one geodesic runs along the first axis. Pass the result to
/// [`from_disk`] to find the point that the position shows. The map is
/// `tanh(pi w / 4)`, which sends `0` to the centre of the disk.
///
/// The Jacobian, the matrix of the partial derivatives of the map, has the
/// derivative in `w.x` as its first column and the derivative in `w.y` as its
/// second. The map is holomorphic with the complex derivative
/// `c = pi / 4 (1 - z^2)`, where `z` is the disk position, so the columns are
/// `(c.re, c.im)` and `(-c.im, c.re)`. The result is [`None`] exactly when
/// `|w.y|` is not below `1` or a coordinate is NaN or infinite. Far along the
/// band, `z` is close to `1` or `-1`, so `1 - z^2` cancels and the Jacobian
/// keeps only an absolute accuracy of a few rounding units. Farther still,
/// the disk position rounds onto the unit circle or is not finite, and
/// [`from_disk`] gives [`None`] for it.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::disk_from_band;
/// use glam::DVec2;
///
/// let (position, jacobian) = disk_from_band(DVec2::ZERO).ok_or("outside")?;
/// assert_eq!(position, DVec2::ZERO);
/// assert_eq!(jacobian.x_axis.x, core::f64::consts::FRAC_PI_4);
/// assert_eq!(disk_from_band(DVec2::new(0.0, 1.0)), None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn disk_from_band(position: DVec2) -> Option<(DVec2, DMat2)> {
    (position.y.abs() < 1.0 && position.x.is_finite()).then(|| {
        let w = Complex64::new(position.x, position.y);
        let z = (w * FRAC_PI_4).tanh();
        let c = (Complex64::new(1.0, 0.0) - z * z) * FRAC_PI_4;
        (DVec2::new(z.re, z.im), holomorphic_jacobian(c))
    })
}

/// Returns the position in the Poincare disk of a position of the Klein disk
/// model, with the Jacobian of the map, or [`None`] for a position on or
/// outside the unit circle.
///
/// Use it to draw the hyperbolic plane in the Klein model, which shows the
/// plane inside the unit circle with every geodesic as a straight chord. Pass
/// the result to [`from_disk`] to find the point that the position shows. The
/// map is `s / (1 + sqrt(1 - |s|^2))`, which sends the centre to the centre
/// and each chord onto the geodesic of the Poincare disk with the same end
/// points.
///
/// The Jacobian, the matrix of the partial derivatives of the map, has the
/// derivative in `s.x` as its first column and the derivative in `s.y` as its
/// second. With `r = sqrt(1 - |s|^2)`, its entry in row `j` and column `k` is
/// `1 / (1 + r)` on the diagonal plus `s_j s_k / (r (1 + r)^2)`. The map
/// stretches by different amounts in different directions, so the Jacobian is
/// not a rotation times a scale. The result is [`None`] exactly when the
/// computed `1 - |s|^2` is not positive, which includes a NaN or infinite
/// coordinate. Near the unit circle `1 - |s|^2` cancels, and the Jacobian
/// grows like `1 / r`.
///
/// # Examples
///
/// ```
/// use escher_geometry::hyperbolic::disk_from_klein;
/// use glam::{DMat2, DVec2};
///
/// let (position, jacobian) = disk_from_klein(DVec2::ZERO).ok_or("outside")?;
/// assert_eq!(position, DVec2::ZERO);
/// assert_eq!(jacobian, DMat2::from_diagonal(DVec2::splat(0.5)));
/// assert_eq!(disk_from_klein(DVec2::new(1.0, 0.0)), None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn disk_from_klein(position: DVec2) -> Option<(DVec2, DMat2)> {
    let d = 1.0 - position.length_squared();
    (d > 0.0).then(|| {
        let root = d.sqrt();
        let denominator = 1.0 + root;
        let diagonal = 1.0 / denominator;
        let outer = root * denominator * denominator;
        let (x, y) = (position.x, position.y);
        let mixed = x * y / outer;
        let jacobian = DMat2::from_cols(
            DVec2::new(diagonal + x * x / outer, mixed),
            DVec2::new(mixed, diagonal + y * y / outer),
        );
        (position / denominator, jacobian)
    })
}

fn holomorphic_jacobian(derivative: Complex64) -> DMat2 {
    DMat2::from_cols(
        DVec2::new(derivative.re, derivative.im),
        DVec2::new(-derivative.im, derivative.re),
    )
}

#[cfg(test)]
mod tests {
    use core::f64::consts::{FRAC_PI_4, SQRT_2};

    use glam::{DMat2, DMat3, DVec2, DVec3};

    use super::{
        ORIGIN, boost, disk_from_band, disk_from_half_plane, disk_from_klein, distance, from_disk,
        isometry_defect, normalize_point, rotation, to_disk, translation, wall_through,
    };
    use crate::ErrorKind;

    fn normalize_kind(vector: DVec3) -> Option<ErrorKind> {
        normalize_point(vector).err().map(|error| error.kind())
    }

    fn assert_from_disk_rejects_the_image(map: fn(DVec2) -> Option<(DVec2, DMat2)>, input: DVec2) {
        let Some((position, _)) = map(input) else {
            panic!("the map rejects {input:?}");
        };
        assert!(
            !position.is_finite() || position.length() >= 1.0 - UNIT_CIRCLE_TOLERANCE,
            "{position:?} from {input:?}"
        );
        assert_eq!(from_disk(position), None, "{position:?} from {input:?}");
    }

    #[test]
    fn normalize_point_rejects_a_spacelike_vector() {
        assert_eq!(
            normalize_kind(DVec3::new(2.0, 0.0, 1.0)),
            Some(ErrorKind::NotAPoint)
        );
    }

    #[test]
    fn normalize_point_rejects_a_null_vector() {
        assert_eq!(
            normalize_kind(DVec3::new(3.0, 4.0, 5.0)),
            Some(ErrorKind::NotAPoint)
        );
    }

    #[test]
    fn normalize_point_rejects_a_vector_with_a_negative_last_coordinate() {
        assert_eq!(
            normalize_kind(DVec3::new(0.0, 0.0, -1.0)),
            Some(ErrorKind::NotAPoint)
        );
    }

    #[test]
    fn normalize_point_rejects_a_vector_whose_form_underflows() {
        assert_eq!(
            normalize_kind(DVec3::new(0.0, 0.0, 1e-200)),
            Some(ErrorKind::NotAPoint)
        );
    }

    #[test]
    fn normalize_point_rejects_a_vector_whose_form_overflows() {
        assert_eq!(
            normalize_kind(DVec3::new(0.0, 0.0, 1e200)),
            Some(ErrorKind::NotAPoint)
        );
    }

    #[test]
    fn normalize_point_rejects_a_nan_vector() {
        assert_eq!(
            normalize_kind(DVec3::new(f64::NAN, 0.0, 1.0)),
            Some(ErrorKind::NotAPoint)
        );
        assert_eq!(
            normalize_kind(DVec3::new(0.0, 0.0, f64::NAN)),
            Some(ErrorKind::NotAPoint)
        );
    }

    #[test]
    fn wall_through_rejects_two_equal_points() {
        let kind = wall_through(ORIGIN, ORIGIN).map_err(|error| error.kind());
        assert_eq!(kind, Err(ErrorKind::DegenerateWall));
    }

    #[test]
    fn from_disk_returns_none_on_the_unit_circle() {
        assert_eq!(from_disk(DVec2::new(1.0, 0.0)), None);
    }

    #[test]
    fn from_disk_returns_none_outside_the_unit_circle() {
        assert_eq!(from_disk(DVec2::new(2.0, 0.0)), None);
    }

    #[test]
    fn from_disk_returns_none_for_nan() {
        assert_eq!(from_disk(DVec2::new(f64::NAN, 0.0)), None);
    }

    #[test]
    fn from_disk_maps_the_centre_to_the_origin() {
        assert_eq!(from_disk(DVec2::ZERO), Some(ORIGIN));
    }

    #[test]
    fn from_disk_is_finite_at_the_largest_radius_below_one() {
        let radius = 1.0 - f64::EPSILON / 2.0;
        let point = from_disk(DVec2::new(radius, 0.0));
        assert!(point.is_some_and(DVec3::is_finite), "{point:?}");
        assert!(point.is_some_and(|point| point.z > 8.9e15 && point.z < 9.1e15));
    }

    #[test]
    fn to_disk_maps_the_origin_to_the_centre() {
        assert_eq!(to_disk(ORIGIN), DVec2::ZERO);
    }

    #[test]
    fn to_disk_of_a_point_at_distance_37_is_within_rounding_of_its_expected_radius() {
        let point = DVec3::new(37.0_f64.sinh(), 0.0, 37.0_f64.cosh());
        let radius = to_disk(point).length();
        assert!(
            (radius - 0.999_999_999_999_999_8).abs() <= TO_DISK_RADIUS_TOLERANCE,
            "{radius}"
        );
    }

    #[test]
    fn to_disk_of_a_point_at_distance_38_point_2_is_within_rounding_of_the_unit_circle() {
        let point = DVec3::new(38.2_f64.sinh(), 0.0, 38.2_f64.cosh());
        let radius = to_disk(point).length();
        assert!((radius - 1.0).abs() <= TO_DISK_RADIUS_TOLERANCE, "{radius}");
    }

    #[test]
    fn to_disk_of_infinite_coordinates_is_nan() {
        let position = to_disk(DVec3::new(f64::INFINITY, 0.0, f64::INFINITY));
        assert!(position.x.is_nan(), "{position:?}");
    }

    #[test]
    fn to_disk_of_a_last_coordinate_of_minus_one_is_not_finite() {
        let position = to_disk(DVec3::new(1.0, 0.0, -1.0));
        assert!(!position.is_finite(), "{position:?}");
    }

    #[test]
    fn from_disk_returns_none_for_an_infinite_coordinate() {
        assert_eq!(from_disk(DVec2::new(f64::INFINITY, 0.0)), None);
        assert_eq!(from_disk(DVec2::new(0.0, f64::NEG_INFINITY)), None);
    }

    #[test]
    fn normalize_point_rejects_an_infinite_vector() {
        assert_eq!(
            normalize_kind(DVec3::new(0.0, 0.0, f64::INFINITY)),
            Some(ErrorKind::NotAPoint)
        );
        assert_eq!(
            normalize_kind(DVec3::new(f64::INFINITY, 0.0, 1.0)),
            Some(ErrorKind::NotAPoint)
        );
    }

    #[test]
    fn wall_through_rejects_a_nan_or_infinite_coordinate() {
        let point = DVec3::new(0.75, 0.0, 1.25);
        for bad in [
            DVec3::new(f64::NAN, 0.0, 1.0),
            DVec3::new(0.0, 0.0, f64::INFINITY),
        ] {
            let forward = wall_through(bad, point).map_err(|error| error.kind());
            assert_eq!(forward, Err(ErrorKind::DegenerateWall), "{bad:?}");
            let backward = wall_through(point, bad).map_err(|error| error.kind());
            assert_eq!(backward, Err(ErrorKind::DegenerateWall), "{bad:?}");
        }
    }

    #[test]
    fn distance_is_nan_for_a_nan_in_either_point() {
        let nan = DVec3::new(f64::NAN, 0.0, 1.0);
        assert!(distance(nan, ORIGIN).is_nan());
        assert!(distance(ORIGIN, nan).is_nan());
    }

    #[test]
    fn boost_of_a_length_beyond_710_point_5_has_infinite_entries() {
        for length in [711.0, -711.0] {
            let matrix = boost(length);
            assert!(matrix.x_axis.x.is_infinite(), "{matrix:?}");
            assert!(matrix.z_axis.z.is_infinite(), "{matrix:?}");
            assert!(matrix.x_axis.z.is_infinite(), "{matrix:?}");
        }
    }

    #[test]
    fn boost_of_a_nan_length_has_nan_entries() {
        let matrix = boost(f64::NAN);
        assert!(
            matrix.x_axis.x.is_nan() && matrix.x_axis.z.is_nan(),
            "{matrix:?}"
        );
    }

    #[test]
    fn rotation_of_a_non_finite_angle_has_nan_entries() {
        for angle in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let matrix = rotation(angle);
            assert!(
                matrix.x_axis.x.is_nan() && matrix.x_axis.y.is_nan(),
                "{matrix:?}"
            );
        }
    }

    #[test]
    fn translation_of_a_non_finite_target_has_a_non_finite_entry() {
        for target in [
            DVec3::new(f64::NAN, 0.0, 1.0),
            DVec3::new(f64::INFINITY, 0.0, 1.0),
            DVec3::new(0.0, f64::NEG_INFINITY, 1.0),
        ] {
            assert!(!translation(target).is_finite(), "{target:?}");
        }
    }

    fn matrix(columns: [[f64; 3]; 3]) -> DMat3 {
        DMat3::from_cols_array_2d(&columns)
    }

    #[test]
    fn isometry_defect_detects_each_condition_violated_alone() {
        let cases = [
            ([[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], 3.0),
            ([[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 1.0]], 3.0),
            ([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 2.0]], 3.0),
            ([[1.0, 0.0, 0.0], [0.6, 0.8, 0.0], [0.0, 0.0, 1.0]], 0.6),
            ([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.75, 0.0, 1.25]], 0.75),
            ([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.75, 1.25]], 0.75),
        ];
        for (columns, expected) in cases {
            let defect = isometry_defect(matrix(columns));
            assert!(
                (defect - expected).abs() <= 1e-12,
                "{defect} for {columns:?}"
            );
        }
        assert_eq!(isometry_defect(DMat3::IDENTITY), 0.0);
    }

    #[test]
    fn isometry_defect_is_infinite_for_a_nan_entry() {
        for index in 0..9 {
            let mut entries = DMat3::IDENTITY.to_cols_array();
            entries[index] = f64::NAN;
            let defect = isometry_defect(DMat3::from_cols_array(&entries));
            assert_eq!(defect, f64::INFINITY, "NaN at entry {index}");
        }
    }

    #[test]
    fn isometry_defect_is_infinite_for_a_last_entry_that_is_not_positive() {
        for last in [0.0, -0.0, -1.0] {
            let defect =
                isometry_defect(matrix([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, last]]));
            assert_eq!(defect, f64::INFINITY, "last entry {last}");
        }
    }

    #[test]
    fn disk_from_half_plane_maps_i_to_the_centre_with_the_jacobian_one_half() {
        let result = disk_from_half_plane(DVec2::new(0.0, 1.0));
        let Some((position, jacobian)) = result else {
            panic!("disk_from_half_plane rejects i");
        };
        assert_eq!(position.to_array().map(f64::abs), [0.0, 0.0]);
        let expected = DMat2::from_diagonal(DVec2::splat(0.5));
        assert_eq!(jacobian.abs(), expected);
    }

    #[test]
    fn disk_from_half_plane_keeps_the_picture_the_same_way_up() {
        let Some((above, _)) = disk_from_half_plane(DVec2::new(0.0, 2.0)) else {
            panic!("disk_from_half_plane rejects 2i");
        };
        assert_eq!(above.x.abs(), 0.0, "{above:?}");
        assert!(above.y > 0.0, "{above:?}");
        let Some((right, _)) = disk_from_half_plane(DVec2::new(1.0, 1.0)) else {
            panic!("disk_from_half_plane rejects 1 + i");
        };
        assert!(right.x > 0.0, "{right:?}");
    }

    #[test]
    fn disk_from_half_plane_near_the_real_axis_or_far_from_i_gives_a_rejected_position() {
        for input in [
            DVec2::new(1e8, 1e-12),
            DVec2::new(0.0, 1e-300),
            DVec2::new(1e300, 1.0),
        ] {
            assert_from_disk_rejects_the_image(disk_from_half_plane, input);
        }
    }

    #[test]
    fn disk_from_half_plane_returns_none_on_and_below_the_real_axis() {
        assert_eq!(disk_from_half_plane(DVec2::new(0.0, 0.0)), None);
        assert_eq!(disk_from_half_plane(DVec2::new(0.0, -0.0)), None);
        assert_eq!(disk_from_half_plane(DVec2::new(0.0, -1.0)), None);
    }

    #[test]
    fn disk_from_half_plane_returns_none_for_nan_or_infinite_coordinates() {
        for position in [
            DVec2::new(f64::NAN, 1.0),
            DVec2::new(f64::INFINITY, 1.0),
            DVec2::new(0.0, f64::NAN),
            DVec2::new(0.0, f64::INFINITY),
        ] {
            assert_eq!(disk_from_half_plane(position), None, "{position:?}");
        }
    }

    #[test]
    fn disk_from_band_maps_zero_to_the_centre_with_the_jacobian_pi_over_4() {
        let expected = DMat2::from_diagonal(DVec2::splat(FRAC_PI_4));
        assert_eq!(disk_from_band(DVec2::ZERO), Some((DVec2::ZERO, expected)));
    }

    #[test]
    fn disk_from_band_maps_1_and_i_over_2_onto_the_positive_first_and_second_axes() {
        let Some((right, _)) = disk_from_band(DVec2::new(1.0, 0.0)) else {
            panic!("disk_from_band rejects 1");
        };
        assert!(right.x > 0.0, "{right:?}");
        assert!(
            (right.x - TANH_OF_PI_OVER_4).abs() <= BAND_VALUE_TOLERANCE,
            "{right:?}"
        );
        assert_eq!(right.y, 0.0, "{right:?}");
        let Some((above, _)) = disk_from_band(DVec2::new(0.0, 0.5)) else {
            panic!("disk_from_band rejects i / 2");
        };
        assert_eq!(above.x, 0.0, "{above:?}");
        assert!(above.y > 0.0, "{above:?}");
        let tan_of_pi_over_8 = SQRT_2 - 1.0;
        assert!(
            (above.y - tan_of_pi_over_8).abs() <= BAND_VALUE_TOLERANCE,
            "{above:?}"
        );
    }

    #[test]
    fn disk_from_band_far_along_the_strip_gives_a_rejected_position() {
        for input in [
            DVec2::new(40.0, 0.0),
            DVec2::new(-40.0, 0.5),
            DVec2::new(400.0, 0.0),
            DVec2::new(1e300, 0.0),
        ] {
            assert_from_disk_rejects_the_image(disk_from_band, input);
        }
    }

    #[test]
    fn disk_from_band_returns_none_on_and_outside_the_edges_of_the_strip() {
        for position in [
            DVec2::new(0.0, 1.0),
            DVec2::new(0.0, -1.0),
            DVec2::new(0.0, 4.0),
        ] {
            assert_eq!(disk_from_band(position), None, "{position:?}");
        }
    }

    #[test]
    fn disk_from_band_returns_none_for_nan_or_infinite_coordinates() {
        for position in [
            DVec2::new(f64::INFINITY, 0.0),
            DVec2::new(f64::NEG_INFINITY, 0.0),
            DVec2::new(f64::NAN, 0.0),
            DVec2::new(0.0, f64::NAN),
            DVec2::new(0.0, f64::INFINITY),
        ] {
            assert_eq!(disk_from_band(position), None, "{position:?}");
        }
    }

    #[test]
    fn disk_from_klein_maps_the_centre_to_the_centre_with_the_jacobian_one_half() {
        let expected = DMat2::from_diagonal(DVec2::splat(0.5));
        assert_eq!(disk_from_klein(DVec2::ZERO), Some((DVec2::ZERO, expected)));
    }

    #[test]
    fn disk_from_klein_maps_the_positive_first_axis_to_itself_closer_to_the_centre() {
        let Some((position, _)) = disk_from_klein(DVec2::new(0.5, 0.0)) else {
            panic!("disk_from_klein rejects (0.5, 0)");
        };
        assert!(position.x > 0.0 && position.x < 0.5, "{position:?}");
        assert_eq!(position.y, 0.0, "{position:?}");
    }

    #[test]
    fn disk_from_klein_of_the_largest_radius_below_one_is_inside_the_disk() {
        let input = DVec2::new(1.0 - f64::EPSILON / 2.0, 0.0);
        let Some((position, jacobian)) = disk_from_klein(input) else {
            panic!("disk_from_klein rejects {input:?}");
        };
        assert!(
            position.is_finite() && jacobian.is_finite(),
            "{position:?} {jacobian:?}"
        );
        assert!(position.length() < 1.0, "{position:?}");
        let point = from_disk(position);
        assert!(point.is_some_and(DVec3::is_finite), "{point:?}");
    }

    #[test]
    fn disk_from_klein_returns_none_on_and_outside_the_unit_circle() {
        assert_eq!(disk_from_klein(DVec2::new(1.0, 0.0)), None);
        assert_eq!(disk_from_klein(DVec2::new(0.6, -0.8)), None);
        assert_eq!(disk_from_klein(DVec2::new(2.0, 0.0)), None);
    }

    #[test]
    fn disk_from_klein_returns_none_for_nan_or_infinite_coordinates() {
        for position in [
            DVec2::new(f64::NAN, 0.0),
            DVec2::new(0.0, f64::NAN),
            DVec2::new(f64::INFINITY, 0.0),
        ] {
            assert_eq!(disk_from_klein(position), None, "{position:?}");
        }
    }

    const TO_DISK_RADIUS_TOLERANCE: f64 = 4.0 * f64::EPSILON;
    const UNIT_CIRCLE_TOLERANCE: f64 = 4.0 * f64::EPSILON;
    const BAND_VALUE_TOLERANCE: f64 = 4.0 * f64::EPSILON;
    const TANH_OF_PI_OVER_4: f64 = 0.655_794_202_632_672_4;
}
