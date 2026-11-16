use core::f64::consts::PI;

use escher_geometry::hyperbolic::distance;
use escher_signature::{Geometry, Signature};
use glam::{DMat3, DVec3};

use crate::error::{Error, Result};
use crate::triangle::{LARGEST_ORDER, Supported, construct, supported};
use crate::verify::Verification;

/// The fundamental domain of a symmetry group of the hyperbolic plane, a
/// polygon whose copies under the group tile the plane without overlap.
///
/// Use it to fold a point into the domain, or to read the shape of the
/// domain. A domain holds its vertices and its sides. Each side lies on a wall,
/// a geodesic line of the hyperbolic plane, and carries the pairing isometry,
/// a map that preserves distances and moves the neighbouring copy of the
/// domain across that side onto the domain. The points and walls are in the
/// linear model of the crate [`escher_geometry`], whose module
/// [`hyperbolic`](escher_geometry::hyperbolic) states the conventions.
///
/// The only domain the crate builds is the reflection triangle of a signature
/// `*pqr`, a triangle whose sides are mirrors and whose corners have the
/// angles `pi / p`, `pi / q`, and `pi / r`. Vertex `j` is the corner of order
/// `p`, `q`, or `r` for `j` equal to 0, 1, or 2, in the order of the
/// canonical signature, which lists the largest order first. Side `j` joins
/// vertex `j - 1` and vertex `j`, with indexes modulo 3, so vertex `j` lies
/// between side `j` and side `j + 1`. The vertices are counterclockwise in the
/// Poincare disk, and the domain is placed with its
/// [centre](FundamentalDomain::centre) at the centre of the disk.
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
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FundamentalDomain {
    pub(crate) geometry: Geometry,
    pub(crate) orders: [u32; 3],
    pub(crate) signature_area: f64,
    pub(crate) sides: [Side; 3],
    pub(crate) vertices: [DVec3; 3],
    pub(crate) centre: DVec3,
}

impl FundamentalDomain {
    /// Builds the fundamental domain of a signature.
    ///
    /// Use it once per signature and keep the domain, because the domain is
    /// immutable. The signature must be a compact hyperbolic reflection
    /// triangle `*pqr`: one mirror boundary with three finite corner orders
    /// whose reciprocals sum to less than 1, and no handle, crosscap, or cone
    /// point. Each corner order must be at most 1000.
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
    /// Returns an error of kind
    /// [`ErrorKind::UnsupportedSignature`](crate::ErrorKind::UnsupportedSignature)
    /// for every signature that is not a compact hyperbolic reflection
    /// triangle, of kind
    /// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) for such a
    /// triangle with a corner order above 1000, and of kind
    /// [`ErrorKind::Numerical`](crate::ErrorKind::Numerical) when rounding
    /// makes the construction degenerate, which no supported signature is
    /// known to do.
    pub fn build(signature: &Signature) -> Result<Self> {
        let Supported { orders, area } =
            supported(signature).ok_or_else(|| Error::unsupported(signature))?;
        let largest = orders.into_iter().fold(0, u32::max);
        if largest > LARGEST_ORDER {
            return Err(Error::limit_exceeded(largest));
        }
        let triangle = construct(orders).map_err(Error::numerical)?;
        Ok(Self {
            geometry: signature.geometry(),
            orders,
            signature_area: area,
            sides: triangle.sides,
            vertices: triangle.vertices,
            centre: triangle.centre,
        })
    }

    /// Returns the geometry of the signature, which is
    /// [`Geometry::Hyperbolic`] for every domain the crate builds.
    #[must_use]
    pub fn geometry(&self) -> Geometry {
        self.geometry
    }

    /// Returns the sides of the domain.
    ///
    /// Side `j` joins vertex `j - 1` and vertex `j`, with indexes modulo 3.
    /// Each side has the wall it lies on and the pairing isometry that
    /// replicates the domain across it.
    #[must_use]
    pub fn sides(&self) -> &[Side] {
        &self.sides
    }

    /// Returns the vertices of the domain as points of the hyperboloid, one
    /// for each side, counterclockwise in the Poincare disk.
    ///
    /// The vertices are not scaled back onto the hyperboloid after the
    /// domain is moved into place, because that scaling moves a vertex away
    /// from the true corner. A vertex therefore lies off the hyperboloid by a
    /// rounding error, and a caller should not normalize it.
    #[must_use]
    pub fn vertices(&self) -> &[DVec3] {
        &self.vertices
    }

    /// Returns the centre of the domain, the sum of its vertices scaled onto
    /// the hyperboloid.
    ///
    /// The domain is placed so that its centre is the point
    /// [`ORIGIN`](escher_geometry::hyperbolic::ORIGIN) up to rounding, the
    /// centre of the Poincare disk.
    #[must_use]
    pub fn centre(&self) -> DVec3 {
        self.centre
    }

    /// Returns the hyperbolic length of each side, in side order.
    ///
    /// The lengths are distances between the stored vertices, in the metric
    /// of curvature -1.
    #[must_use]
    pub fn side_lengths(&self) -> Vec<f64> {
        self.lengths().to_vec()
    }

    /// Returns the interior angle at each vertex in radians, in vertex
    /// order.
    ///
    /// Each angle is measured from the three side lengths by the law of
    /// cosines of hyperbolic trigonometry, not from the walls, and lies in
    /// the interval from 0 to pi.
    #[must_use]
    pub fn corner_angles(&self) -> Vec<f64> {
        self.angles().to_vec()
    }

    /// Returns the area of the domain, pi minus the sum of its corner angles.
    ///
    /// The area is in the metric of curvature -1, where a triangle has the
    /// area pi minus its angle sum.
    #[must_use]
    pub fn area(&self) -> f64 {
        let [first, second, third] = self.angles();
        PI - first - second - third
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

    fn lengths(&self) -> [f64; 3] {
        let v = self.vertices;
        [0, 1, 2].map(|index| distance(v[(index + 2) % 3], v[index]))
    }

    pub(crate) fn angles(&self) -> [f64; 3] {
        let lengths = self.lengths();
        [0, 1, 2].map(|vertex| {
            let a = lengths[vertex];
            let b = lengths[(vertex + 1) % 3];
            let c = lengths[(vertex + 2) % 3];
            let cosine = (a.cosh() * b.cosh() - c.cosh()) / (a.sinh() * b.sinh());
            cosine.clamp(-1.0, 1.0).acos()
        })
    }
}

/// One side of a [`FundamentalDomain`], with its wall and its pairing
/// isometry.
///
/// Use it to test on which side of the wall a point lies and to move a point
/// across the side. A wall is a geodesic line of the hyperbolic plane,
/// represented by a unit normal, a vector whose Minkowski form with itself
/// equals 1. The pairing isometry is a map that preserves distances and maps
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
    /// The normal is a vector of the linear model whose Minkowski form with
    /// itself is 1, and the wall is the set of points whose form with it is
    /// 0.
    #[must_use]
    pub fn normal(self) -> DVec3 {
        self.normal
    }

    /// Returns the covector of the wall, the normal with its last coordinate
    /// negated.
    ///
    /// Pass it to [`escher_geometry::hyperbolic::side`] to test a point
    /// against the wall. The test is positive outside the domain, zero on the
    /// wall, and negative inside.
    #[must_use]
    pub fn covector(self) -> DVec3 {
        self.covector
    }

    /// Returns the pairing isometry of the side as a matrix of the linear
    /// model.
    ///
    /// The isometry maps the neighbouring copy of the domain across this
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
    use crate::ErrorKind;
    use crate::triangle::LARGEST_ORDER;

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
