use core::f64::consts::PI;

use escher_geometry::{euclidean, hyperbolic};
use escher_signature::{Geometry, Signature};
use glam::{DMat3, DVec3};

use crate::error::{Error, Result};
use crate::parameters::{Parameter, ShapeParameters, validate, value};
use crate::polygon::{Polygon, rectangle, triangle};
use crate::triangle::{LARGEST_ORDER, Supported, construct, supported};
use crate::verify::Verification;

/// The fundamental domain of a symmetry group of the hyperbolic or the
/// Euclidean plane, a polygon whose copies under the group tile the plane
/// without overlap.
///
/// Use it to fold a point into the domain, or to read the shape of the
/// domain. A domain holds its vertices and its sides. Each side lies on a wall,
/// a line of the plane, and carries the pairing isometry, a map that preserves
/// distances and moves the neighbouring copy of the domain across that side
/// onto the domain. The points and walls are those of the crate
/// [`escher_geometry`]: of its module
/// [`hyperbolic`](escher_geometry::hyperbolic) for a domain whose
/// [geometry](FundamentalDomain::geometry) is hyperbolic, and of its module
/// [`euclidean`](escher_geometry::euclidean) for a Euclidean one. Each module
/// states its conventions.
///
/// The crate builds a reflection triangle for a signature `*pqr`, a triangle
/// whose sides are mirrors and whose corners have the angles `pi / p`,
/// `pi / q`, and `pi / r`, and a reflection rectangle for `*2222`. Vertex `j`
/// is corner `j` of the canonical signature, which lists the largest order
/// first. Side `j` joins vertex `j - 1` and vertex `j`, with indexes modulo
/// the number of sides, so vertex `j` lies between side `j` and side `j + 1`.
/// The vertices are counterclockwise, in the Poincare disk for a hyperbolic
/// domain and in the plane for a Euclidean one, and the domain is placed with
/// its [centre](FundamentalDomain::centre) at the origin.
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
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FundamentalDomain {
    pub(crate) plane: Plane,
    pub(crate) orders: Vec<u32>,
    pub(crate) sides: Vec<Side>,
    pub(crate) vertices: Vec<DVec3>,
    pub(crate) centre: DVec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Plane {
    Hyperbolic { area: f64 },
    Euclidean,
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
    /// The signature must have one mirror boundary with finite corner orders
    /// and no handle, crosscap, or cone point, and be one of these:
    ///
    /// - A hyperbolic triangle `*pqr`, whose corner orders have reciprocals
    ///   that sum to less than 1. Each corner order must be at most 1000.
    /// - A Euclidean triangle, which is `*632`, `*442`, or `*333`.
    /// - The Euclidean rectangle `*2222`, whose shape parameter `aspect` is
    ///   the length of side 0 divided by the length of side 1.
    ///
    /// The triangle of a Euclidean signature has side 0 of length 1, and the
    /// rectangle has area 1. Each name of a setting must be the name of a
    /// parameter that [`parameters`](crate::parameters()) lists for the
    /// signature, and each value must lie in the range of that parameter. A
    /// parameter without a setting takes its default value.
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
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Checks in this order and returns the first error that applies:
    ///
    /// 1. An error of kind
    ///    [`ErrorKind::UnsupportedSignature`](crate::ErrorKind::UnsupportedSignature)
    ///    for a signature that is none of those above.
    /// 2. An error of kind
    ///    [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) for a
    ///    corner order above 1000.
    /// 3. An error of kind
    ///    [`ErrorKind::UnknownParameter`](crate::ErrorKind::UnknownParameter)
    ///    when a setting names no parameter of the signature.
    /// 4. An error of kind
    ///    [`ErrorKind::ParameterOutOfRange`](crate::ErrorKind::ParameterOutOfRange)
    ///    when the value of a setting is NaN, infinite, or outside the range
    ///    of its parameter.
    /// 5. An error of kind
    ///    [`ErrorKind::Numerical`](crate::ErrorKind::Numerical) when rounding
    ///    makes the construction degenerate, which no supported signature is
    ///    known to do.
    pub fn build_with(signature: &Signature, parameters: &ShapeParameters) -> Result<Self> {
        let supported = supported(signature).ok_or_else(|| Error::unsupported(signature))?;
        let orders = supported.orders();
        let largest = orders.iter().copied().fold(0, u32::max);
        if largest > LARGEST_ORDER {
            return Err(Error::limit_exceeded(largest));
        }
        validate(signature, supported, parameters)?;
        match supported {
            Supported::HyperbolicTriangle { orders, area } => {
                let triangle = construct(orders).map_err(Error::numerical)?;
                Ok(Self {
                    plane: Plane::Hyperbolic { area },
                    orders: orders.to_vec(),
                    sides: triangle.sides.to_vec(),
                    vertices: triangle.vertices.to_vec(),
                    centre: triangle.centre,
                })
            }
            Supported::EuclideanTriangle { orders } => {
                Self::euclidean(orders.to_vec(), triangle(orders))
            }
            Supported::Rectangle => {
                Self::euclidean(orders, rectangle(value(parameters, Parameter::Aspect)))
            }
        }
    }

    fn euclidean(
        orders: Vec<u32>,
        polygon: core::result::Result<Polygon, Option<escher_geometry::Error>>,
    ) -> Result<Self> {
        let polygon = polygon.map_err(Error::numerical)?;
        Ok(Self {
            plane: Plane::Euclidean,
            orders,
            sides: polygon.sides,
            vertices: polygon.vertices,
            centre: polygon.centre,
        })
    }

    /// Returns the geometry of the signature, which is
    /// [`Geometry::Hyperbolic`] or [`Geometry::Euclidean`] for every domain
    /// the crate builds.
    #[must_use]
    pub fn geometry(&self) -> Geometry {
        match self.plane {
            Plane::Hyperbolic { .. } => Geometry::Hyperbolic,
            Plane::Euclidean => Geometry::Euclidean,
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
    /// of the plane.
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
    /// [`ORIGIN`](escher_geometry::euclidean::ORIGIN) up to rounding.
    #[must_use]
    pub fn centre(&self) -> DVec3 {
        self.centre
    }

    /// Returns the length of each side, in side order.
    ///
    /// The lengths are distances between the stored vertices, in the metric
    /// of curvature -1 for a hyperbolic domain.
    #[must_use]
    pub fn side_lengths(&self) -> Vec<f64> {
        self.lengths()
    }

    /// Returns the interior angle at each vertex in radians, in vertex
    /// order.
    ///
    /// For a hyperbolic domain each angle is measured from the three side
    /// lengths by the law of cosines of hyperbolic trigonometry, and for a
    /// Euclidean domain from the directions of the two sides at the vertex.
    /// No angle is measured from the walls, and each lies in the interval from
    /// 0 to pi. The law of cosines holds only for a triangle, so a hyperbolic
    /// domain that does not have three vertices gives NaN for every angle.
    #[must_use]
    pub fn corner_angles(&self) -> Vec<f64> {
        self.angles()
    }

    /// Returns the area of the domain.
    ///
    /// For a hyperbolic domain it is pi minus the sum of its corner angles, in
    /// the metric of curvature -1, which is the area of a triangle, and NaN
    /// for a hyperbolic domain that does not have three vertices. For a
    /// Euclidean domain it is the area that the vertices enclose.
    #[must_use]
    pub fn area(&self) -> f64 {
        match self.plane {
            Plane::Hyperbolic { .. } => match self.angles()[..] {
                [first, second, third] => PI - first - second - third,
                _ => f64::NAN,
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

    /// Checks the domain numerically and reports each check with its
    /// residual and its tolerance.
    ///
    /// Use it to confirm that a domain is fit for use before relying on it,
    /// such as before drawing a tiling. The checks test that each pairing is
    /// an isometry, that the sides pair correctly, that each side lies on its
    /// wall, that each corner closes up, that each angle is `pi` divided by
    /// its order, and that the area agrees with the signature. The method
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

    fn lengths(&self) -> Vec<f64> {
        let v = &self.vertices;
        let count = v.len();
        let distance = match self.plane {
            Plane::Hyperbolic { .. } => hyperbolic::distance,
            Plane::Euclidean => euclidean::distance,
        };
        (0..count)
            .map(|index| distance(v[(index + count - 1) % count], v[index]))
            .collect()
    }

    pub(crate) fn angles(&self) -> Vec<f64> {
        let count = self.vertices.len();
        match self.plane {
            Plane::Hyperbolic { .. } => match self.lengths()[..] {
                [first, second, third] => {
                    let lengths = [first, second, third];
                    [0, 1, 2]
                        .map(|vertex| {
                            let a = lengths[vertex];
                            let b = lengths[(vertex + 1) % 3];
                            let c = lengths[(vertex + 2) % 3];
                            let cosine = (a.cosh() * b.cosh() - c.cosh()) / (a.sinh() * b.sinh());
                            cosine.clamp(-1.0, 1.0).acos()
                        })
                        .to_vec()
                }
                _ => vec![f64::NAN; count],
            },
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
}

/// One side of a [`FundamentalDomain`], with its wall and its pairing
/// isometry.
///
/// Use it to test on which side of the wall a point lies and to move a point
/// across the side. In a hyperbolic domain a wall is a geodesic line,
/// represented by a unit normal, a vector whose Minkowski form with itself
/// equals 1. In a Euclidean domain it is a line, represented by a line
/// covector. The pairing isometry is a map that preserves distances and maps
/// the neighbouring copy of the domain across this side onto the domain.
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
    /// vector `(a.x, a.y, 0)` normal to the line.
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
    /// [`escher_geometry::euclidean::side`]. Either test is the dot product of
    /// the covector and the point, and is positive outside the domain, zero on
    /// the wall, and negative inside.
    #[must_use]
    pub fn covector(self) -> DVec3 {
        self.covector
    }

    /// Returns the pairing isometry of the side as a 3 by 3 matrix.
    ///
    /// In a hyperbolic domain it is a matrix of the linear model, and in a
    /// Euclidean domain it is an isometry of the affine plane with last
    /// coordinate 1. The isometry maps the neighbouring copy of the domain across this
    /// side onto the domain. For a side of kind [`PairingKind::Mirror`] it is
    /// the reflection in the wall of the side.
    #[must_use]
    pub fn pairing(self) -> DMat3 {
        self.pairing
    }

    /// Returns the index of the side onto which the pairing isometry maps
    /// this side.
    ///
    /// A mirror side is its own partner.
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
    /// A reflection reverses orientation.
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
    use crate::triangle::LARGEST_ORDER;
    use crate::{ErrorKind, ShapeParameters};

    fn signature(text: &str) -> Signature {
        match text.parse() {
            Ok(signature) => signature,
            Err(error) => panic!("{text:?} is rejected: {error}"),
        }
    }

    #[test]
    fn build_accepts_the_largest_order_and_rejects_the_next() {
        let at_limit = signature(&format!("*({LARGEST_ORDER})32"));
        assert!(FundamentalDomain::build(&at_limit).is_ok());
        let over_limit = signature(&format!("*({})32", LARGEST_ORDER + 1));
        let kind = FundamentalDomain::build(&over_limit).map_err(|error| error.kind());
        assert_eq!(kind.err(), Some(ErrorKind::LimitExceeded));
    }

    #[test]
    fn build_checks_support_before_the_limit() {
        let unsupported = signature(&format!("*({})({})", LARGEST_ORDER + 1, LARGEST_ORDER + 1));
        let kind = FundamentalDomain::build(&unsupported).map_err(|error| error.kind());
        assert_eq!(kind.err(), Some(ErrorKind::UnsupportedSignature));
    }

    #[test]
    fn build_with_settings_checks_the_signature_and_the_limit_first() {
        let mut settings = ShapeParameters::new();
        settings.set("aspect", f64::NAN);
        for (text, expected) in [
            ("*532", ErrorKind::UnsupportedSignature),
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
