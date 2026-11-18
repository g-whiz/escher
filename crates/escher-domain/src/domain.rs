use core::f64::consts::PI;

use escher_geometry::{euclidean, hyperbolic, spherical};
use escher_signature::{Geometry, Signature};
use glam::{DMat3, DVec3};

use crate::double::double;
use crate::error::{Error, Result};
use crate::parameters::{ShapeParameters, validate};
use crate::polygon::{Polygon, rectangle, triangle};
use crate::sheets::{Cuts, side_lengths};
use crate::spherical::{lune, triangle as spherical_triangle};
use crate::triangle::{Shape, Supported, admitted, construct};
use crate::verify::Verification;
use crate::walk;

/// The fundamental domain of a symmetry group of the hyperbolic plane, the
/// Euclidean plane, or the sphere, a polygon whose copies under the group
/// tile the plane or the sphere without overlap.
///
/// Use it to fold a point into the domain, or to read the shape of the
/// domain. A domain holds its vertices and its sides. Each side lies on a wall,
/// a line of the plane or a great circle of the sphere, and carries the
/// pairing isometry, a map that preserves distances and moves the
/// neighbouring copy of the domain across that side onto the domain. The
/// points and walls are those of the crate [`escher_geometry`]: of its module
/// [`hyperbolic`](escher_geometry::hyperbolic) for a domain whose
/// [geometry](FundamentalDomain::geometry) is hyperbolic, of its module
/// [`euclidean`](escher_geometry::euclidean) for a Euclidean one, and of its
/// module [`spherical`](escher_geometry::spherical) for a spherical one. Each
/// module states its conventions.
///
/// The crate builds a reflection triangle for a signature `*pqr`, a triangle
/// whose sides are mirrors, the walls in which the group reflects, and whose
/// corners have the angles `pi / p`, `pi / q`, and `pi / r`. It builds a
/// reflection polygon of the hyperbolic plane for a signature with four to
/// eight corners, such as `*3333`, a reflection rectangle for `*2222`, and a
/// lune for `*nn`, the region of the sphere between two half great circles
/// that are mirrors and meet at the angle `pi / n` at two antipodal vertices.
/// In a reflection domain, vertex `j` is
/// corner `j` of the canonical signature, which lists the largest order first.
/// Side `j` joins vertex `j - 1` and vertex `j`, with indexes modulo the number
/// of sides, so vertex `j` lies between side `j` and side `j + 1`. The vertices
/// are counterclockwise, in the Poincare disk for a hyperbolic domain, in the
/// plane for a Euclidean one, and as seen from outside the sphere for a
/// spherical one, and the domain is placed with its
/// [centre](FundamentalDomain::centre) at the origin.
///
/// For a signature without a mirror boundary, whose cone points, the centres of
/// rotation, have the orders of the corners of a signature above, such as `732`
/// for `*732`, the crate builds the double of the reflection domain of that
/// signature. The double is the reflection domain together with its mirror
/// image across the wall of its side 1. It keeps the reflection domain in
/// place, so it has the same centre. Rotations pair its sides with each other
/// in place of mirrors, except that a translation pairs two sides of the double
/// of `2222`.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_signature::Signature;
///
/// let signature: Signature = "*732".parse()?;
/// let domain = FundamentalDomain::build(&signature)?;
/// assert_eq!(domain.sides().len(), 3);
/// assert_eq!(domain.vertices().len(), 3);
/// assert!(domain.verify().passed());
///
/// let rectangle = FundamentalDomain::build(&"*2222".parse::<Signature>()?)?;
/// assert_eq!(rectangle.sides().len(), 4);
/// assert!(rectangle.verify().passed());
///
/// let kite = FundamentalDomain::build(&"732".parse::<Signature>()?)?;
/// assert_eq!(kite.sides().len(), 4);
/// assert_eq!(kite.centre(), domain.centre());
/// assert!(kite.verify().passed());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FundamentalDomain {
    pub(crate) plane: Plane,
    pub(crate) corners: Vec<Corner>,
    pub(crate) sides: Vec<Side>,
    pub(crate) vertices: Vec<DVec3>,
    pub(crate) centre: DVec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Corner {
    pub(crate) order: u32,
    pub(crate) copies: u32,
}

impl Corner {
    pub(crate) const fn single(order: u32) -> Self {
        Self { order, copies: 1 }
    }
}

impl Plane {
    pub(crate) fn side(self, covector: DVec3, point: DVec3) -> f64 {
        match self {
            Self::Hyperbolic { .. } => hyperbolic::side(covector, point),
            Self::Euclidean => euclidean::side(covector, point),
            Self::Spherical { .. } => spherical::side(covector, point),
        }
    }
}

pub(crate) fn triangle_angles(
    plane: Plane,
    [first, second, third]: [DVec3; 3],
) -> Option<[f64; 3]> {
    let distance = match plane {
        Plane::Hyperbolic { .. } => hyperbolic::distance,
        Plane::Spherical { .. } => spherical::distance,
        Plane::Euclidean => return None,
    };
    let lengths = [
        distance(third, first),
        distance(first, second),
        distance(second, third),
    ];
    Some([0, 1, 2].map(|vertex| {
        let a = lengths[vertex];
        let b = lengths[(vertex + 1) % 3];
        let c = lengths[(vertex + 2) % 3];
        let cosine = match plane {
            Plane::Hyperbolic { .. } => (a.cosh() * b.cosh() - c.cosh()) / (a.sinh() * b.sinh()),
            Plane::Euclidean | Plane::Spherical { .. } => {
                (c.cos() - a.cos() * b.cos()) / (a.sin() * b.sin())
            }
        };
        cosine.clamp(-1.0, 1.0).acos()
    }))
}

fn plane(shape: Shape) -> Plane {
    match shape {
        Shape::HyperbolicTriangle { area, .. } | Shape::HyperbolicPolygon { area, .. } => {
            Plane::Hyperbolic { area }
        }
        Shape::EuclideanTriangle { .. } | Shape::Rectangle => Plane::Euclidean,
        Shape::SphericalTriangle { area, .. } | Shape::Lune { area, .. } => {
            Plane::Spherical { area }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Plane {
    Hyperbolic { area: f64 },
    Euclidean,
    Spherical { area: f64 },
}

impl FundamentalDomain {
    /// Builds the fundamental domain of a signature, with each shape
    /// parameter at its default value.
    ///
    /// It is [`build_with`](FundamentalDomain::build_with) with an empty
    /// [`ShapeParameters`]. Use it once per signature and keep the domain,
    /// because the domain is immutable.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_domain::FundamentalDomain;
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "*237".parse()?;
    /// let domain = FundamentalDomain::build(&signature)?;
    /// let angles = domain.corner_angles();
    /// assert!((angles[0] - core::f64::consts::PI / 7.0).abs() < 1e-12);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the errors of
    /// [`build_with`](FundamentalDomain::build_with) other than those of a
    /// setting.
    pub fn build(signature: &Signature) -> Result<Self> {
        Self::build_with(signature, &ShapeParameters::new())
    }

    /// Builds the fundamental domain of a signature, with the shape
    /// parameters that the settings give.
    ///
    /// The signature must have no handle or crosscap, and either one mirror
    /// boundary with finite corner orders and no cone point, or finite cone
    /// orders and no boundary. With a boundary, it must be one of these:
    ///
    /// - A hyperbolic triangle `*pqr`, whose corner orders have reciprocals
    ///   that sum to less than 1.
    /// - A hyperbolic polygon `*p1...pn` with four to eight corners, such as
    ///   `*3333` or `*5432`, whose shape parameters are lengths.
    /// - A Euclidean triangle, which is `*632`, `*442`, or `*333`.
    /// - The Euclidean rectangle `*2222`, whose shape parameter `aspect` is
    ///   the length of side 0 divided by the length of side 1.
    /// - A spherical triangle, which is `*332`, `*432`, `*532`, or `*n22`.
    /// - A lune `*nn`.
    ///
    /// Each corner order must be at most 1000. Without a boundary, its cone
    /// orders must be the corner orders of one of these other than a
    /// hyperbolic polygon, such as `732`, `2222`, or `nn`, and its domain is
    /// the double of the domain of that signature, which is the domain
    /// together with its mirror image across the wall of side 1. The double of
    /// a triangle is a kite with four sides, that of the rectangle has six,
    /// and that of a lune has two. The double of `2222` has the shape
    /// parameter `aspect` of the rectangle that it doubles.
    ///
    /// The reflection triangle of a Euclidean signature has side 0 of length 1,
    /// and the reflection rectangle has area 1. A spherical domain lies on the
    /// unit sphere and has no shape parameter. A hyperbolic polygon has its
    /// corners in the order of the canonical signature, with corner `j` at
    /// vertex `j`, and is placed with its centre at the origin and vertex 0 on
    /// the positive first axis. Each name of a setting must be the name of a
    /// parameter that [`parameters`](crate::parameters()) lists for the
    /// signature, and each value must lie in the range of that parameter. A
    /// parameter without a setting takes its default value.
    ///
    /// A hyperbolic polygon is returned only when it passes every check of
    /// [`verify`](FundamentalDomain::verify), because some settings near the
    /// ends of the ranges give a shape that rounding keeps from meeting the
    /// tolerances of the checks.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_domain::{FundamentalDomain, ShapeParameters};
    /// use escher_signature::Signature;
    ///
    /// let mut settings = ShapeParameters::new();
    /// settings.set("aspect", 0.25);
    /// let domain = FundamentalDomain::build_with(&"*2222".parse::<Signature>()?, &settings)?;
    /// let lengths = domain.side_lengths();
    /// assert!((lengths[0] - 0.5).abs() < 1e-15);
    /// assert!((lengths[1] - 2.0).abs() < 1e-15);
    /// assert!((domain.area() - 1.0).abs() < 1e-15);
    ///
    /// let mut settings = ShapeParameters::new();
    /// settings.set("slice1.length", 2.5);
    /// let polygon = FundamentalDomain::build_with(&"*3333".parse::<Signature>()?, &settings)?;
    /// for angle in polygon.corner_angles() {
    ///     assert!((angle - core::f64::consts::PI / 3.0).abs() < 1e-12);
    /// }
    /// assert!(polygon.verify().passed());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the first of these errors that applies, in this order:
    ///
    /// 1. An error of kind
    ///    [`ErrorKind::UnsupportedSignature`](crate::ErrorKind::UnsupportedSignature)
    ///    for a signature that is none of those above.
    /// 2. An error of kind
    ///    [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) for a
    ///    signature with more than eight corner orders, and then for a corner
    ///    or cone order above 1000.
    /// 3. An error of kind
    ///    [`ErrorKind::UnknownParameter`](crate::ErrorKind::UnknownParameter)
    ///    when a setting names no parameter of the signature.
    /// 4. An error of kind
    ///    [`ErrorKind::ParameterOutOfRange`](crate::ErrorKind::ParameterOutOfRange)
    ///    when the value of a setting is NaN, infinite, or outside the range
    ///    of its parameter.
    /// 5. An error of kind
    ///    [`ErrorKind::Numerical`](crate::ErrorKind::Numerical) when rounding
    ///    makes the construction degenerate, or when a hyperbolic polygon
    ///    fails a check of [`verify`](FundamentalDomain::verify).
    pub fn build_with(signature: &Signature, parameters: &ShapeParameters) -> Result<Self> {
        let supported = admitted(signature)?;
        let cuts = supported.shape().cuts();
        let values = validate(signature, supported, cuts.as_ref(), parameters)?;
        match supported {
            Supported::Reflection(shape) => Self::reflection(shape, cuts.as_ref(), &values),
            Supported::Rotation(shape) => double(
                &Self::reflection(shape.with_half_area(), cuts.as_ref(), &values)?,
                plane(shape),
            ),
        }
    }

    fn reflection(shape: Shape, cuts: Option<&Cuts>, values: &[f64]) -> Result<Self> {
        let orders = shape.orders();
        let polygon = match shape {
            Shape::HyperbolicTriangle { orders, .. } => construct(orders).map(|triangle| Polygon {
                sides: triangle.sides.to_vec(),
                vertices: triangle.vertices.to_vec(),
                centre: triangle.centre,
            }),
            Shape::HyperbolicPolygon { .. } => cuts
                .and_then(|cuts| side_lengths(&orders, cuts, values))
                .ok_or(None)
                .and_then(|lengths| walk::construct(&orders, &lengths)),
            Shape::EuclideanTriangle { orders } => triangle(orders),
            Shape::Rectangle => match values {
                &[aspect] => rectangle(aspect),
                _ => Err(None),
            },
            Shape::SphericalTriangle { orders, .. } => spherical_triangle(orders),
            Shape::Lune { order, .. } => lune(order),
        }
        .map_err(Error::numerical)?;
        let domain = Self {
            plane: plane(shape),
            corners: orders.into_iter().map(Corner::single).collect(),
            sides: polygon.sides,
            vertices: polygon.vertices,
            centre: polygon.centre,
        };
        let guarded = matches!(shape, Shape::HyperbolicPolygon { .. });
        if guarded && !domain.verify().passed() {
            return Err(Error::refused());
        }
        Ok(domain)
    }

    /// Returns the geometry of the signature, which is
    /// [`Geometry::Hyperbolic`], [`Geometry::Euclidean`], or
    /// [`Geometry::Spherical`].
    #[must_use]
    pub fn geometry(&self) -> Geometry {
        match self.plane {
            Plane::Hyperbolic { .. } => Geometry::Hyperbolic,
            Plane::Euclidean => Geometry::Euclidean,
            Plane::Spherical { .. } => Geometry::Spherical,
        }
    }

    /// Returns the sides of the domain.
    ///
    /// Side `j` joins vertex `j - 1` and vertex `j`, with indexes modulo the
    /// number of sides. Each side has the wall it lies on and the pairing
    /// isometry that replicates the domain across it.
    #[must_use]
    pub fn sides(&self) -> &[Side] {
        &self.sides
    }

    /// Returns the vertices of the domain, one for each side, in
    /// counterclockwise order.
    ///
    /// For a hyperbolic domain the vertices are points of the hyperboloid,
    /// counterclockwise in the Poincare disk. They are not scaled back onto
    /// the hyperboloid after the domain is moved into place, because that
    /// scaling moves a vertex away from the true corner. A vertex therefore
    /// lies off the hyperboloid by a rounding error, and a caller should not
    /// normalize it. For a Euclidean domain the vertices are points `(x, y, 1)`
    /// of the plane. For a spherical domain the vertices are points of the
    /// unit sphere. The two vertices of a lune and of its double are the
    /// antipodal points `(0, 1, 0)` and `(0, -1, 0)`.
    ///
    /// The vertices of a double follow those of the domain that it doubles,
    /// which has `n` vertices `V0` to `V(n-1)`. Vertex 0 of the double is
    /// `V0`, vertices 1 to `n - 2` are the mirror images of `V(n-1)` down to
    /// `V2` across the wall of side 1, vertex `n - 1` is `V1`, and the vertices
    /// after it are `V2` to `V(n-1)`.
    #[must_use]
    pub fn vertices(&self) -> &[DVec3] {
        &self.vertices
    }

    /// Returns the centre of the domain.
    ///
    /// For a hyperbolic domain it is the sum of the vertices scaled onto the
    /// hyperboloid, and the domain is placed so that it is the point
    /// [`ORIGIN`](escher_geometry::hyperbolic::ORIGIN) up to rounding, the
    /// centre of the Poincare disk. For a Euclidean domain it is the mean of
    /// the vertices, and the domain is placed so that it is the point
    /// [`ORIGIN`](escher_geometry::euclidean::ORIGIN) up to rounding. For a
    /// spherical triangle it is the sum of the vertices scaled onto the
    /// sphere, and the domain is placed so that it is the point
    /// [`ORIGIN`](escher_geometry::spherical::ORIGIN) up to rounding. The two
    /// vertices of a lune are antipodal and have no midpoint, so its centre is
    /// [`ORIGIN`](escher_geometry::spherical::ORIGIN) exactly, halfway between
    /// its two sides. A double has the centre of the domain that it doubles,
    /// which lies inside that half of the double.
    #[must_use]
    pub fn centre(&self) -> DVec3 {
        self.centre
    }

    /// Returns the length of each side, in side order.
    ///
    /// The lengths are distances between the stored vertices, in the metric
    /// of curvature -1 for a hyperbolic domain and of the unit sphere for a
    /// spherical one. Both sides of a lune and of its double have the length
    /// `pi`.
    #[must_use]
    pub fn side_lengths(&self) -> Vec<f64> {
        self.lengths()
    }

    /// Returns the interior angle at each vertex in radians, in vertex
    /// order.
    ///
    /// An angle measured once lies in the interval from 0 to pi. An angle that
    /// is the sum of two measured angles, as described below, lies in the
    /// interval from 0 to 2 pi. At a vertex of a double where two corners of
    /// the domain that it doubles meet, the angle is twice that of one corner,
    /// so it is `pi` for a corner of order 2.
    ///
    /// For a hyperbolic or spherical triangle, each angle is measured from the
    /// three side lengths by the law of cosines of its geometry. A hyperbolic
    /// or spherical domain with four vertices, such as the double of a
    /// triangle, is split along the diagonal from vertex 0 to vertex 2 into two
    /// triangles, each measured in this way, and the angle at vertex 0 or
    /// vertex 2 is the sum of its angles in the two triangles.
    ///
    /// For a hyperbolic domain with five or more vertices, each angle is
    /// measured in the frame of its vertex, the isometry that moves the vertex
    /// to the origin. It is the two-argument arctangent of the first two
    /// coordinates of the two neighbours of the vertex in that frame. This
    /// holds at every vertex, including a vertex of a double where two corners
    /// of the domain that it doubles meet, whose angle can be pi.
    ///
    /// For a Euclidean domain, each angle is measured from the directions of
    /// the two sides at the vertex.
    ///
    /// The two vertices of a lune or of its double do not fix its sides, so
    /// both of its angles are measured from the unit normals `u0` and `u1` of
    /// its two walls. For a lune it is the inverse cosine of `-u0 . u1`. For
    /// its double it is the two-argument arctangent of the length of `u0 x u1`
    /// and `-u0 . u1`, because its angle can be `pi`, where the inverse cosine
    /// magnifies a rounding error of the normals.
    ///
    /// Every angle is NaN for a hyperbolic domain with fewer than three
    /// vertices and for a spherical domain whose number of vertices is not
    /// two, three, or four. An angle of a Euclidean domain is NaN when a
    /// neighbour of its vertex coincides with the vertex.
    #[must_use]
    pub fn corner_angles(&self) -> Vec<f64> {
        self.angles()
    }

    /// Returns the area of the domain.
    ///
    /// A hyperbolic area is in the metric of curvature -1, and a spherical one
    /// is in the metric of the unit sphere.
    ///
    /// For a hyperbolic triangle it is pi minus the sum of its corner angles.
    /// For a spherical triangle it is the sum of its corner angles minus pi.
    /// For a hyperbolic or spherical domain with four vertices, it is the sum
    /// of the areas of the two triangles that
    /// [`corner_angles`](FundamentalDomain::corner_angles) measures. For a
    /// hyperbolic domain with `m` vertices, five or more, it is `(m - 2) pi`
    /// minus the sum of its corner angles, in which a vertex where two corners
    /// of a doubled domain meet counts with its whole angle. For a lune or its
    /// double it is twice its angle. For a Euclidean domain it is the area that
    /// the vertices enclose.
    ///
    /// The area is NaN for a hyperbolic domain with fewer than three vertices
    /// and for a spherical domain that has neither two, three, nor four
    /// vertices.
    #[must_use]
    pub fn area(&self) -> f64 {
        match self.plane {
            Plane::Hyperbolic { .. } => match self.triangles()[..] {
                [[first, second, third]] => PI - first - second - third,
                [[first, second, third], [fourth, fifth, sixth]] => {
                    (PI - first - second - third) + (PI - fourth - fifth - sixth)
                }
                _ => self.polygon_angles().map_or(f64::NAN, |angles| {
                    polygon_angle_limit(angles.len()) - angles.iter().sum::<f64>()
                }),
            },
            Plane::Spherical { .. } => match self.triangles()[..] {
                [[first, second, third]] => first + second + third - PI,
                [[first, second, third], [fourth, fifth, sixth]] => {
                    (first + second + third - PI) + (fourth + fifth + sixth - PI)
                }
                _ => match self.angles()[..] {
                    [first, _] => 2.0 * first,
                    _ => f64::NAN,
                },
            },
            Plane::Euclidean => {
                let count = self.vertices.len();
                let twice = (0..count).fold(0.0, |sum, index| {
                    let (first, second) =
                        (self.vertices[index], self.vertices[(index + 1) % count]);
                    sum + (first.x * second.y - second.x * first.y)
                });
                (0.5 * twice).abs()
            }
        }
    }

    /// Returns whether the pairing isometry of some side reverses
    /// orientation.
    ///
    /// It is true for a domain whose sides are mirrors, and false for the
    /// domain of a signature without mirrors, such as `732`, whose pairings
    /// are all rotations or translations. Only a domain for which it is true
    /// can be coloured by whether a tile is an even or an odd number of
    /// reflections away from the domain.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_domain::FundamentalDomain;
    /// use escher_signature::Signature;
    ///
    /// assert!(FundamentalDomain::build(&"*732".parse::<Signature>()?)?.reverses_orientation());
    /// assert!(!FundamentalDomain::build(&"732".parse::<Signature>()?)?.reverses_orientation());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn reverses_orientation(&self) -> bool {
        self.sides
            .iter()
            .any(|side| side.orientation == Orientation::Reversing)
    }

    /// Checks the domain numerically and reports each check with its
    /// residual and its tolerance.
    ///
    /// Use it to confirm that a domain is fit for use before relying on it,
    /// such as before drawing a tiling. The checks test that each pairing is
    /// an isometry, that the sides pair correctly, that each side lies on its
    /// wall, that each corner closes up, that each angle agrees with the order
    /// of its corner, and that the area agrees with the signature. The method
    /// computes every check on every call, changes nothing, and never panics.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_domain::FundamentalDomain;
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "*(1000)32".parse()?;
    /// let verification = FundamentalDomain::build(&signature)?.verify();
    /// assert!(verification.relators().residual() <= verification.relators().tolerance());
    /// assert!(verification.passed());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn verify(&self) -> Verification {
        Verification::of(self)
    }

    /// Returns the side presentation of the group of the domain, with one
    /// generator for each side and the relators that the sides give.
    ///
    /// Use it to name the copies of the domain by words, such as to read the
    /// sides that a fold crosses as a word in the generators. The generator of
    /// side `i` is named `s` followed by `i`, such as `s0`, and stands for the
    /// inverse of the pairing of the side, the isometry that maps the domain
    /// onto its neighbouring copy across that side. A word is the product of
    /// its generators from left to right.
    ///
    /// For a domain whose sides are all mirrors, the relators are the square
    /// of each generator in side order, such as `s0^2`, followed by
    /// `(sj sk)^n` for each corner `j` in corner order, where side `k` is the
    /// side that follows side `j`, wrapping around at the last side, and `n`
    /// is the order of the corner. For a domain whose sides are paired with
    /// other sides, the relators are `si sp` for each side `i` whose partner
    /// `p` is larger, followed by one relator for each vertex cycle, a set of
    /// vertices that the side pairings join. The base of a cycle relator is
    /// the generators of the sides that a walk around the cycle crosses, in
    /// the order in which it crosses them, and its exponent is the order of
    /// the cycle. The method changes nothing and never panics.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_domain::FundamentalDomain;
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "22".parse()?;
    /// let presentation = FundamentalDomain::build(&signature)?.side_presentation()?;
    /// assert_eq!(presentation.generator_names(), ["s0", "s1"]);
    /// let relators: Vec<String> = presentation
    ///     .relators()
    ///     .iter()
    ///     .filter_map(|relator| presentation.display_relator(relator))
    ///     .map(|text| text.to_string())
    ///     .collect();
    /// assert_eq!(relators, ["s0 s1", "s1^2", "s0^2"]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) of kind
    /// [`ErrorKind::InvalidPairing`](crate::ErrorKind::InvalidPairing) when the
    /// domain has both mirror sides and paired sides, when it has a different
    /// number of corners than of sides, when it has more than 256 sides, when
    /// the side pairings do not lead around a vertex back to it, when the
    /// corners of a vertex cycle differ in order or do not join two corners of
    /// the reflection domain in total, or when the relators do not form a
    /// valid presentation. The causes are checked in this order.
    /// No domain that [`FundamentalDomain::build`] or
    /// [`FundamentalDomain::build_with`] returns gives this error.
    pub fn side_presentation(&self) -> Result<escher_group::Presentation> {
        crate::presentation::side_presentation(self)
    }

    pub(crate) fn triangles(&self) -> Vec<[f64; 3]> {
        let angles = |corners| triangle_angles(self.plane, corners);
        match self.vertices[..] {
            [first, second, third] => angles([first, second, third]).into_iter().collect(),
            [first, second, third, fourth] => angles([first, second, third])
                .into_iter()
                .chain(angles([first, third, fourth]))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn lengths(&self) -> Vec<f64> {
        let v = &self.vertices;
        let count = v.len();
        let distance = match self.plane {
            Plane::Hyperbolic { .. } => hyperbolic::distance,
            Plane::Euclidean => euclidean::distance,
            Plane::Spherical { .. } => spherical::distance,
        };
        (0..count)
            .map(|index| distance(v[(index + count - 1) % count], v[index]))
            .collect()
    }

    pub(crate) fn angles(&self) -> Vec<f64> {
        if let Some(angles) = self.polygon_angles() {
            return angles;
        }
        let count = self.vertices.len();
        match self.plane {
            Plane::Hyperbolic { .. } | Plane::Spherical { .. } => {
                match (self.triangles().as_slice(), self.sides.as_slice()) {
                    ([triangle], _) => triangle.to_vec(),
                    ([first, second], _) => {
                        vec![
                            first[0] + second[0],
                            first[1],
                            first[2] + second[1],
                            second[2],
                        ]
                    }
                    ([], [first, second])
                        if count == 2 && matches!(self.plane, Plane::Spherical { .. }) =>
                    {
                        let (normal, other) = (first.normal, second.normal);
                        let angle = match first.kind {
                            PairingKind::Mirror => (-normal.dot(other)).clamp(-1.0, 1.0).acos(),
                            PairingKind::Rotation | PairingKind::Translation => {
                                normal.cross(other).length().atan2(-normal.dot(other))
                            }
                        };
                        vec![angle; 2]
                    }
                    _ => vec![f64::NAN; count],
                }
            }
            Plane::Euclidean => (0..count)
                .map(|vertex| {
                    let corner = self.vertices[vertex].truncate();
                    let next = self.vertices[(vertex + 1) % count].truncate() - corner;
                    let previous = self.vertices[(vertex + count - 1) % count].truncate() - corner;
                    let cosine = next.dot(previous) / (next.length() * previous.length());
                    cosine.clamp(-1.0, 1.0).acos()
                })
                .collect(),
        }
    }

    pub(crate) fn polygon_angles(&self) -> Option<Vec<f64>> {
        let v = &self.vertices;
        let count = v.len();
        if count < 5 || !matches!(self.plane, Plane::Hyperbolic { .. }) {
            return None;
        }
        Some(
            (0..count)
                .map(|vertex| {
                    frame_angle(
                        v[vertex],
                        v[(vertex + 1) % count],
                        v[(vertex + count - 1) % count],
                    )
                })
                .collect(),
        )
    }
}

pub(crate) fn polygon_angle_limit(count: usize) -> f64 {
    u32::try_from(count.saturating_sub(2)).map_or(f64::NAN, f64::from) * PI
}

fn frame_angle(vertex: DVec3, next: DVec3, previous: DVec3) -> f64 {
    let frame = hyperbolic::inverse(hyperbolic::translation(vertex));
    let (first, second) = ((frame * next).truncate(), (frame * previous).truncate());
    first.perp_dot(second).abs().atan2(first.dot(second))
}

/// One side of a [`FundamentalDomain`], with its wall and its pairing
/// isometry.
///
/// Use it to test on which side of the wall a point lies and to move a point
/// across the side. In a hyperbolic domain a wall is a geodesic line,
/// represented by a unit normal, a vector whose Minkowski form with itself
/// equals 1. In a Euclidean domain it is a line, represented by a line
/// covector. In a spherical domain it is a great circle, represented by a
/// unit normal of its plane. The pairing isometry is a map that preserves
/// distances and maps the neighbouring copy of the domain across this side onto
/// the domain.
///
/// # Examples
///
/// ```
/// use escher_domain::{FundamentalDomain, Orientation, PairingKind};
/// use escher_geometry::hyperbolic::side;
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*444".parse::<Signature>()?)?;
/// for (index, wall) in domain.sides().iter().enumerate() {
///     assert_eq!(wall.kind(), PairingKind::Mirror);
///     assert_eq!(wall.partner(), index);
///     assert_eq!(wall.orientation(), Orientation::Reversing);
///     assert!(side(wall.covector(), domain.centre()) < 0.0);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Side {
    pub(crate) normal: DVec3,
    pub(crate) covector: DVec3,
    pub(crate) pairing: DMat3,
    pub(crate) partner: usize,
    pub(crate) kind: PairingKind,
    pub(crate) orientation: Orientation,
}

impl Side {
    /// Returns the unit normal of the wall of the side, which points out of
    /// the domain.
    ///
    /// In a hyperbolic domain the normal is a vector of the linear model
    /// whose Minkowski form with itself is 1, and the wall is the set of
    /// points whose form with it is 0. In a Euclidean domain it is the unit
    /// vector `(a.x, a.y, 0)` normal to the line. In a spherical domain it is
    /// the unit normal of the plane of the great circle.
    #[must_use]
    pub fn normal(self) -> DVec3 {
        self.normal
    }

    /// Returns the covector of the wall.
    ///
    /// In a hyperbolic domain it is the normal with its last coordinate
    /// negated, and a caller passes it to
    /// [`escher_geometry::hyperbolic::side`] to test a point against the wall.
    /// In a Euclidean domain it is the line covector `(a.x, a.y, -t)` of the
    /// line of points `p` with `a . p = t`, and a caller passes it to
    /// [`escher_geometry::euclidean::side`]. In a spherical domain it is the
    /// normal itself, and a caller passes it to
    /// [`escher_geometry::spherical::side`]. Each test is the dot product of
    /// the covector and the point, and is positive outside the domain, zero on
    /// the wall, and negative inside.
    #[must_use]
    pub fn covector(self) -> DVec3 {
        self.covector
    }

    /// Returns the pairing isometry of the side as a 3 by 3 matrix.
    ///
    /// In a hyperbolic domain it is a matrix of the linear model, in a
    /// Euclidean domain it is an isometry of the affine plane with last
    /// coordinate 1, and in a spherical domain it is an orthogonal matrix. The
    /// isometry maps the neighbouring copy of the domain across this side onto
    /// the domain. For a side of kind [`PairingKind::Mirror`] it is
    /// the reflection in the wall of the side. For a side of a double it is
    /// the product of two reflections of the domain that the double doubles,
    /// in the wall of its side 1 and in the wall of the side that this side is,
    /// or is the mirror image of. Each entry of the product is computed with
    /// the rounding errors of its products and sums carried along and added
    /// back at the end, so it is close to the exact product of the two stored
    /// matrices rounded once.
    #[must_use]
    pub fn pairing(self) -> DMat3 {
        self.pairing
    }

    /// Returns the index of the side onto which the pairing isometry maps
    /// this side.
    ///
    /// A mirror side is its own partner. A side of a double has another side
    /// as its partner, and its pairing isometry maps it onto that side with
    /// the endpoints exchanged.
    #[must_use]
    pub fn partner(self) -> usize {
        self.partner
    }

    /// Returns the kind of the pairing isometry of the side.
    #[must_use]
    pub fn kind(self) -> PairingKind {
        self.kind
    }

    /// Returns whether the pairing isometry preserves or reverses
    /// orientation.
    ///
    /// A reflection reverses orientation, and a rotation or a translation
    /// preserves it.
    #[must_use]
    pub fn orientation(self) -> Orientation {
        self.orientation
    }
}

/// The kind of the pairing isometry of a [`Side`].
///
/// The enum is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
///
/// # Examples
///
/// ```
/// use escher_domain::{FundamentalDomain, PairingKind};
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*832".parse::<Signature>()?)?;
/// assert!(domain.sides().iter().all(|side| side.kind() == PairingKind::Mirror));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PairingKind {
    /// The side is a mirror, its own partner, and its pairing isometry is the
    /// reflection in its wall.
    Mirror,
    /// The pairing isometry is a rotation, which maps the side onto its
    /// partner, another side of the domain.
    Rotation,
    /// The pairing isometry is a translation of the Euclidean plane, which
    /// maps the side onto its partner, another side of the domain, and fixes
    /// no point.
    Translation,
}

/// Whether an isometry preserves or reverses orientation, the sense in which
/// the corners of a polygon go around it.
///
/// # Examples
///
/// ```
/// use escher_domain::{FundamentalDomain, Orientation};
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*732".parse::<Signature>()?)?;
/// let reversing = domain
///     .sides()
///     .iter()
///     .filter(|side| side.orientation() == Orientation::Reversing)
///     .count();
/// assert_eq!(reversing, 3);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[expect(
    clippy::exhaustive_enums,
    reason = "an isometry either preserves or reverses orientation"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Orientation {
    /// The isometry keeps the sense of rotation, as a rotation does.
    Preserving,
    /// The isometry reverses the sense of rotation, as a reflection does.
    Reversing,
}

#[cfg(test)]
mod tests {
    use escher_signature::Signature;

    use super::FundamentalDomain;
    use crate::triangle::{LARGEST_ORDER, MOST_ORDERS, admitted};
    use crate::{ErrorKind, ShapeParameters};

    fn signature(text: &str) -> Signature {
        match text.parse() {
            Ok(signature) => signature,
            Err(error) => panic!("{text:?} is rejected: {error}"),
        }
    }

    #[test]
    fn build_accepts_the_largest_order_and_rejects_the_next() {
        for star in ["*", ""] {
            let at_limit = signature(&format!("{star}({LARGEST_ORDER})32"));
            assert!(FundamentalDomain::build(&at_limit).is_ok(), "{star}");
            let over_limit = signature(&format!("{star}({})32", LARGEST_ORDER + 1));
            let kind = FundamentalDomain::build(&over_limit).map_err(|error| error.kind());
            assert_eq!(kind.err(), Some(ErrorKind::LimitExceeded), "{star}");
        }
    }

    #[test]
    fn reflection_refuses_settings_of_another_shape() {
        let shape = |text| match admitted(&signature(text)) {
            Ok(supported) => supported.shape(),
            Err(error) => panic!("{text}: {error}"),
        };
        let polygon = shape("*3333");
        let cuts = polygon.cuts();
        assert!(FundamentalDomain::reflection(polygon, cuts.as_ref(), &[1.317]).is_ok());
        let numerical =
            |built: crate::Result<FundamentalDomain>| built.map_err(|error| error.kind()).err();
        assert_eq!(
            numerical(FundamentalDomain::reflection(polygon, None, &[1.317])),
            Some(ErrorKind::Numerical)
        );
        assert_eq!(
            numerical(FundamentalDomain::reflection(shape("*2222"), None, &[])),
            Some(ErrorKind::Numerical)
        );
    }

    #[test]
    fn build_checks_support_before_the_limit() {
        let unsupported = signature(&format!("({})*32", LARGEST_ORDER + 1));
        let kind = FundamentalDomain::build(&unsupported).map_err(|error| error.kind());
        assert_eq!(kind.err(), Some(ErrorKind::UnsupportedSignature));
    }

    #[test]
    fn build_checks_the_number_of_orders_before_the_largest_order() {
        let most = "2".repeat(MOST_ORDERS);
        assert!(FundamentalDomain::build(&signature(&format!("*{most}"))).is_ok());
        for text in [
            format!("*2{most}"),
            format!("*({}){most}", LARGEST_ORDER + 1),
        ] {
            let error = FundamentalDomain::build(&signature(&text)).err();
            assert_eq!(
                error.as_ref().map(crate::Error::kind),
                Some(ErrorKind::LimitExceeded),
                "{text}"
            );
            assert_eq!(
                error.map(|error| error.to_string()),
                Some(format!(
                    "cannot build fundamental domain: the signature has {} orders, more than {MOST_ORDERS}",
                    MOST_ORDERS + 1
                )),
                "{text}"
            );
        }
    }

    #[test]
    fn build_with_settings_checks_the_signature_and_the_limit_first() {
        let mut settings = ShapeParameters::new();
        settings.set("aspect", f64::NAN);
        for (text, expected) in [
            ("3*2", ErrorKind::UnsupportedSignature),
            (
                &*format!("*({})32", LARGEST_ORDER + 1),
                ErrorKind::LimitExceeded,
            ),
            ("*632", ErrorKind::UnknownParameter),
            ("*2222", ErrorKind::ParameterOutOfRange),
        ] {
            let kind = FundamentalDomain::build_with(&signature(text), &settings)
                .map_err(|error| error.kind());
            assert_eq!(kind.err(), Some(expected), "{text}");
        }
    }

    #[test]
    fn sides_join_consecutive_vertices() {
        let domain = FundamentalDomain::build(&signature("*732"));
        let Ok(domain) = domain else {
            panic!("*732 has no domain");
        };
        for (index, side) in domain.sides().iter().enumerate() {
            assert_eq!(side.partner(), index);
            for vertex in [(index + 2) % 3, index] {
                let value =
                    escher_geometry::hyperbolic::side(side.covector(), domain.vertices[vertex]);
                assert!(value.abs() < 1e-12, "side {index} vertex {vertex}: {value}");
            }
        }
    }
}
