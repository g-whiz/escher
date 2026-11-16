use core::fmt::{self, Write as _};
use core::num::NonZeroU64;
use core::str::FromStr;

use crate::canonical::canonicalize;
use crate::classify::{
    Derived, Geometry, derive, geometry, has_infinite_order, is_general_non_compact,
};
use crate::error::{Detail, Error, Limit};
use crate::order::Order;
use crate::parse::{Features, scan};
use crate::rational::Rational;

/// A valid orbifold signature in canonical form.
///
/// An orbifold signature, in the notation of John Conway, names a symmetry
/// group of the plane, the sphere, or the hyperbolic plane by its features:
/// handles `o`, crosscaps `x`, mirror boundaries `*`, cone points, and the
/// corner points of each boundary. Each cone and corner point has an
/// [`Order`].
///
/// The only way to get a `Signature` is to parse one with [`str::parse`].
/// Parsing maps every accepted spelling of a group to one canonical form, so
/// two signatures are equal exactly when they name the same group.
/// [`Display`](fmt::Display) writes the canonical string, which parses back
/// to an equal signature.
///
/// # Examples
///
/// ```
/// use escher_signature::Signature;
///
/// let first: Signature = "2*(12)".parse()?;
/// let second: Signature = "(12)*2".parse()?;
/// assert_ne!(first, second);
/// assert_eq!(first.to_string(), "2*(12)");
///
/// let handle_and_crosscap: Signature = "ox".parse()?;
/// assert_eq!(handle_and_crosscap, "XXX".parse()?);
/// assert_eq!(handle_and_crosscap.to_string(), "xxx");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Signature {
    features: Features,
}

/// One mirror boundary of a [`Signature`], with its corner points.
///
/// # Examples
///
/// ```
/// use escher_signature::Signature;
///
/// let signature: Signature = "*2*".parse()?;
/// let corners = signature
///     .boundaries()
///     .iter()
///     .map(|boundary| boundary.corner_orders().len())
///     .collect::<Vec<_>>();
/// assert_eq!(corners, [1, 0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Boundary {
    pub(crate) corners: Vec<Order>,
}

impl Boundary {
    /// Returns the orders of the corner points in their cyclic order along
    /// the boundary.
    ///
    /// A corner point is a point of the boundary where two mirrors meet at
    /// the angle pi divided by its order. The list may be empty.
    #[must_use]
    pub fn corner_orders(&self) -> &[Order] {
        &self.corners
    }
}

impl Signature {
    /// Returns the number of handles, which is 0 when the signature has a
    /// crosscap.
    ///
    /// One handle and one crosscap name the same group as three crosscaps, so
    /// a signature with a crosscap counts each handle as two crosscaps.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "oox".parse()?;
    /// assert_eq!(signature.handles(), 0);
    /// assert_eq!(signature.crosscaps(), 5);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn handles(&self) -> u32 {
        self.features.handles
    }

    /// Returns the orders of the cone points, in descending order.
    ///
    /// A cone point is a point of rotational symmetry whose order is the
    /// number of rotations about it.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "236".parse()?;
    /// let orders = signature
    ///     .cone_orders()
    ///     .iter()
    ///     .filter_map(|order| order.finite())
    ///     .collect::<Vec<_>>();
    /// assert_eq!(orders, [6, 3, 2]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn cone_orders(&self) -> &[Order] {
        &self.features.cone_orders
    }

    /// Returns the mirror boundaries, in descending order of their corner
    /// orders, compared from the first corner.
    ///
    /// The corners of each boundary start at the rotation that compares
    /// greatest.
    #[must_use]
    pub fn boundaries(&self) -> &[Boundary] {
        &self.features.boundaries
    }

    /// Returns the number of crosscaps.
    ///
    /// When the parsed text has a crosscap, each handle of the text counts as
    /// two more crosscaps.
    #[must_use]
    pub fn crosscaps(&self) -> u32 {
        self.features.crosscaps
    }

    /// Returns the exact Euler characteristic of the orbifold.
    ///
    /// The Euler characteristic is 2 minus the sum of the defects of the
    /// features: 2 for a handle, 1 for a crosscap or a mirror boundary,
    /// `1 - 1/n` for a cone point of order `n`, `(1 - 1/n) / 2` for a corner
    /// point of order `n`, 1 for an infinite cone point, and `1/2` for an
    /// infinite corner point.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "*532".parse()?;
    /// assert_eq!(signature.euler_characteristic().to_string(), "1/60");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn euler_characteristic(&self) -> Rational {
        self.derived().chi
    }

    /// Returns the geometry in which the group acts.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::{Geometry, Signature};
    ///
    /// let signature: Signature = "*3333".parse()?;
    /// assert_eq!(signature.geometry(), Geometry::Hyperbolic);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn geometry(&self) -> Geometry {
        geometry(self.derived().chi)
    }

    /// Returns whether the signature has no infinite order, which means that
    /// its orbifold, the quotient of the geometry by the group, is compact.
    #[must_use]
    pub fn is_compact(&self) -> bool {
        !has_infinite_order(&self.features)
    }

    /// Returns whether the signature names a frieze group, the symmetry
    /// group of a pattern that repeats along one direction of the Euclidean
    /// plane.
    ///
    /// This is the case when some order is infinite and the Euler
    /// characteristic is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::Signature;
    ///
    /// let frieze: Signature = "*22(inf)".parse()?;
    /// assert!(frieze.is_frieze());
    /// let wallpaper: Signature = "*2222".parse()?;
    /// assert!(!wallpaper.is_frieze());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_frieze(&self) -> bool {
        !self.is_compact() && self.derived().chi.numerator() == 0
    }

    /// Returns whether the signature has an infinite order, a negative Euler
    /// characteristic, and neither of the forms `*pqr` and `pqr`, the
    /// triangle groups with ideal corners and their rotation subgroups.
    ///
    /// The forms are taken literally: no handle, no crosscap, and either no
    /// cone point and one boundary with exactly three corners, or no boundary
    /// and exactly three cone points.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::Signature;
    ///
    /// let general: Signature = "3*(inf)".parse()?;
    /// assert!(general.is_general_non_compact());
    /// let triangle: Signature = "*23(inf)".parse()?;
    /// assert!(!triangle.is_general_non_compact());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_general_non_compact(&self) -> bool {
        is_general_non_compact(&self.features, self.derived().chi)
    }

    /// Returns the order of the group, which is 2 divided by the Euler
    /// characteristic, for a spherical signature, and `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::Signature;
    ///
    /// let icosahedral: Signature = "*532".parse()?;
    /// assert_eq!(icosahedral.group_order().map(|order| order.get()), Some(120));
    /// let wallpaper: Signature = "*632".parse()?;
    /// assert_eq!(wallpaper.group_order(), None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn group_order(&self) -> Option<NonZeroU64> {
        self.derived().group_order
    }

    /// Returns the area of the orbifold divided by pi, which is twice the
    /// absolute value of the Euler characteristic, for a spherical or a
    /// hyperbolic signature, and `None` for a Euclidean one.
    ///
    /// The area is that of the orbifold on the sphere or the hyperbolic plane
    /// of curvature 1 or -1. A hyperbolic signature with an infinite order
    /// uses the same formula.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "*3333".parse()?;
    /// assert_eq!(signature.area_over_pi().map(|area| area.to_string()).as_deref(), Some("2/3"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn area_over_pi(&self) -> Option<Rational> {
        self.derived().area_over_pi
    }

    /// Returns the dimension of the space of deformations of the orbifold,
    /// its Teichmuller space, for a compact hyperbolic signature, and `None`
    /// otherwise.
    ///
    /// The dimension is `6h + 3c + 3b + 2k + l - 6` for `h` handles, `c`
    /// crosscaps, `b` boundaries, `k` cone points, and `l` corner points. It
    /// counts the independent real parameters that change the hyperbolic
    /// shape of the orbifold.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_signature::Signature;
    ///
    /// let rigid: Signature = "*732".parse()?;
    /// assert_eq!(rigid.deformation_dimension(), Some(0));
    /// let flexible: Signature = "*3333".parse()?;
    /// assert_eq!(flexible.deformation_dimension(), Some(1));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn deformation_dimension(&self) -> Option<u32> {
        self.derived().deformation_dimension
    }

    #[expect(
        clippy::expect_used,
        reason = "the constructor accepted the signature only after the same computation on the same features succeeded"
    )]
    fn derived(&self) -> Derived {
        derive(&self.features).expect(
            "internal error: the derived values of an accepted signature fit, because the constructor computed them",
        )
    }
}

/// Parses a signature, validates it, and stores it in canonical form.
///
/// The text holds `o` or `O` for a handle, `x` or `X` for a crosscap, `*`
/// for a mirror boundary, and the orders. An order is one digit, digits in
/// parentheses such as `(12)`, or `(inf)` in lower case or the infinity sign
/// U+221E for an infinite order. Orders before the first `*` are cone points,
/// and orders after a `*` are corners of that boundary. An order of 1 is
/// dropped, and a text with no other feature, such as `1`, is the trivial
/// signature, which names the trivial group.
///
/// # Examples
///
/// ```
/// use escher_signature::Signature;
///
/// let signature: Signature = "*22(inf)".parse()?;
/// assert_eq!(signature.to_string(), "*(inf)22");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an [`Error`] with its [`ErrorKind`](crate::ErrorKind) and the
/// byte span of the cause when:
///
/// - the text is empty, which is [`ErrorKind::Empty`](crate::ErrorKind::Empty),
/// - a character is not allowed at its place, including whitespace, which is
///   [`ErrorKind::UnexpectedCharacter`](crate::ErrorKind::UnexpectedCharacter),
/// - the text ends inside parentheses, which is
///   [`ErrorKind::UnclosedParenthesis`](crate::ErrorKind::UnclosedParenthesis),
/// - an order is 0, which is [`ErrorKind::ZeroOrder`](crate::ErrorKind::ZeroOrder),
/// - a `1` outside parentheses is followed by a digit, which is
///   [`ErrorKind::MultiDigitOrder`](crate::ErrorKind::MultiDigitOrder),
/// - an order other than 1 follows a `*` with an `o` or an `x` in between,
///   which is [`ErrorKind::AmbiguousOrder`](crate::ErrorKind::AmbiguousOrder),
/// - the text is longer than 1024 bytes, an order is larger than 4294967295,
///   or the signature has more than 64 features, counting each handle twice,
///   which is [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded),
/// - the Euler characteristic or twice it does not fit exact 64-bit
///   arithmetic, which is [`ErrorKind::Overflow`](crate::ErrorKind::Overflow),
/// - the signature is a bad orbifold, which is
///   [`ErrorKind::BadOrbifold`](crate::ErrorKind::BadOrbifold),
/// - the signature has an infinite order and a positive Euler characteristic,
///   which is [`ErrorKind::PositiveNonCompact`](crate::ErrorKind::PositiveNonCompact),
/// - the group order of a spherical signature is not a whole number or does
///   not fit a 64-bit unsigned integer, which is also
///   [`ErrorKind::Overflow`](crate::ErrorKind::Overflow), or
/// - the deformation dimension of a compact hyperbolic signature does not fit
///   a 32-bit unsigned integer, which is also
///   [`ErrorKind::Overflow`](crate::ErrorKind::Overflow).
///
/// A text longer than 1024 bytes is rejected before it is read. Otherwise the
/// first error in the text is returned. The last five errors of the list
/// concern the whole signature, so they arise only when the text has no other
/// error, in the order of this list, and they span the whole text.
impl FromStr for Signature {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let whole = 0..text.len();
        let features = canonicalize(scan(text)?)
            .ok_or_else(|| Error::new(Detail::LimitExceeded(Limit::Size), whole.clone()))?;
        derive(&features).map_err(|detail| Error::new(detail, whole))?;
        Ok(Self { features })
    }
}

/// Writes the canonical string: each handle as `o`, the cone orders, each
/// boundary as `*` followed by its corner orders, each crosscap as `x`, and
/// `1` for the trivial signature.
///
/// Each order is written as [`Order`] writes it, so the string parses back to
/// an equal signature.
impl fmt::Display for Signature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut empty = true;
        for _ in 0..self.features.handles {
            formatter.write_char('o')?;
            empty = false;
        }
        for order in &self.features.cone_orders {
            write!(formatter, "{order}")?;
            empty = false;
        }
        for boundary in &self.features.boundaries {
            formatter.write_char('*')?;
            for order in &boundary.corners {
                write!(formatter, "{order}")?;
            }
            empty = false;
        }
        for _ in 0..self.features.crosscaps {
            formatter.write_char('x')?;
            empty = false;
        }
        if empty {
            formatter.write_char('1')?;
        }
        Ok(())
    }
}
