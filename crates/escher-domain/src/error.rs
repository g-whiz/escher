use core::fmt;

use escher_signature::Signature;

use crate::parameters::Parameter;
use crate::presentation::MAX_SIDES;
use crate::triangle::{LARGEST_ORDER, MOST_ORDERS};

/// A specialized [`Result`](core::result::Result) whose error is [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// The reason why no fundamental domain was built for a signature, or why a
/// domain gave no side presentation.
///
/// Match on [`Error::kind`] to tell the causes apart. The
/// [`Display`](fmt::Display) text states the cause and is meant for a person,
/// not for matching. It starts with `cannot derive side presentation: ` for an
/// error of
/// [`FundamentalDomain::side_presentation`](crate::FundamentalDomain::side_presentation)
/// and with `cannot build fundamental domain: ` for every other error.
///
/// # Examples
///
/// ```
/// use escher_domain::{ErrorKind, FundamentalDomain};
/// use escher_signature::Signature;
///
/// let signature: Signature = "3*2".parse()?;
/// let error = FundamentalDomain::build(&signature).err();
/// assert_eq!(error.as_ref().map(|error| error.kind()), Some(ErrorKind::UnsupportedSignature));
/// assert_eq!(
///     error.map(|error| error.to_string()).as_deref(),
///     Some("cannot build fundamental domain: the signature 3*2 is not supported"),
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    operation: &'static str,
    detail: Detail,
}

const OPERATION: &str = "build fundamental domain";
const PRESENTATION: &str = "derive side presentation";

impl Error {
    pub(crate) fn unsupported(signature: &Signature) -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::UnsupportedSignature(signature.to_string()),
        }
    }

    pub(crate) fn limit_exceeded(order: u32) -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::LimitExceeded(order),
        }
    }

    pub(crate) fn too_many_orders(count: usize) -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::TooManyOrders(count),
        }
    }

    pub(crate) fn refused() -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::Refused,
        }
    }

    pub(crate) fn refused_double() -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::RefusedDouble,
        }
    }

    pub(crate) fn refused_wallpaper() -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::RefusedWallpaper,
        }
    }

    pub(crate) fn numerical(source: Option<escher_geometry::Error>) -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::Numerical(source),
        }
    }

    pub(crate) fn unknown_parameter(signature: &Signature) -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::UnknownParameter(signature.to_string()),
        }
    }

    pub(crate) fn parameter_out_of_range(parameter: Parameter) -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::ParameterOutOfRange(parameter),
        }
    }

    pub(crate) fn invalid_pairing(defect: PairingDefect) -> Self {
        Self {
            operation: PRESENTATION,
            detail: Detail::InvalidPairing(defect),
        }
    }

    pub(crate) fn rejected_presentation(source: escher_group::Error) -> Self {
        Self {
            operation: PRESENTATION,
            detail: Detail::RejectedPresentation(source),
        }
    }

    /// Returns the cause of the error as an [`ErrorKind`].
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self.detail {
            Detail::UnsupportedSignature(_) => ErrorKind::UnsupportedSignature,
            Detail::LimitExceeded(_) | Detail::TooManyOrders(_) => ErrorKind::LimitExceeded,
            Detail::Numerical(_)
            | Detail::Refused
            | Detail::RefusedDouble
            | Detail::RefusedWallpaper => ErrorKind::Numerical,
            Detail::UnknownParameter(_) => ErrorKind::UnknownParameter,
            Detail::ParameterOutOfRange(_) => ErrorKind::ParameterOutOfRange,
            Detail::InvalidPairing(_) | Detail::RejectedPresentation(_) => {
                ErrorKind::InvalidPairing
            }
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cannot {}: {}", self.operation, self.detail)
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.detail {
            Detail::Numerical(Some(source)) => Some(source),
            Detail::RejectedPresentation(source) => Some(source),
            Detail::UnsupportedSignature(_)
            | Detail::LimitExceeded(_)
            | Detail::TooManyOrders(_)
            | Detail::Numerical(None)
            | Detail::Refused
            | Detail::RefusedDouble
            | Detail::RefusedWallpaper
            | Detail::UnknownParameter(_)
            | Detail::ParameterOutOfRange(_)
            | Detail::InvalidPairing(_) => None,
        }
    }
}

/// The cause of an [`Error`], for a caller to match on.
///
/// The enum is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
///
/// # Examples
///
/// ```
/// use escher_domain::{ErrorKind, FundamentalDomain};
/// use escher_signature::Signature;
///
/// let signature: Signature = "*(1001)32".parse()?;
/// let kind = FundamentalDomain::build(&signature).map_err(|error| error.kind());
/// assert_eq!(kind.err(), Some(ErrorKind::LimitExceeded));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The crate does not build a fundamental domain for the signature.
    ///
    /// Only a reflection domain, its double, and the domains of `o`, `**`,
    /// `xx`, `*x`, `22x`, `22*`, `2*22`, `4*2`, and `3*3` have one. A reflection domain
    /// is a triangle, a hyperbolic polygon, the rectangle, or the lune, and
    /// its sides are mirrors. The signature of a triangle has the form `*pqr`
    /// with three finite corner orders. It is hyperbolic when their
    /// reciprocals sum to less than 1, or it is one of the Euclidean
    /// signatures `*632`, `*442`, and `*333`, or one of the spherical
    /// signatures `*332`, `*432`, `*532`, and `*n22`. The signature of a
    /// hyperbolic polygon has four or more finite corner orders and is
    /// hyperbolic, such as `*3333`. The signature of the rectangle is `*2222`,
    /// and that of the lune is `*nn`. A corner of order `n` is a corner where
    /// two mirrors meet at the angle `pi / n`. The double of a reflection
    /// domain is the domain together with its mirror image across one of its
    /// walls. Its signature has the same orders as cone points, the centres of
    /// rotation, and no mirror boundary, such as `732`, `2222`, `nn`, or
    /// `3333`. The signature `o` has a parallelogram whose sides are paired by
    /// translations, `**` has a rectangle with two mirror sides and two sides
    /// paired by a translation, `xx` has a rectangle whose sides are paired by
    /// glide reflections and translations, `*x` has a rectangle with two
    /// mirror sides and two sides paired by glide reflections, and `22x` has a
    /// rectangle whose sides are paired by glide reflections. `22*` and `2*22`
    /// have a rectangle with mirror sides whose bottom side, and for `22*` also
    /// its top side, is split at its midpoint into two sides paired by a
    /// half-turn. `4*2` and `3*3` have the kite of `442` or `632` with two
    /// mirror sides and two sides paired by a rotation. Every Euclidean
    /// signature without an infinite order has a domain.
    UnsupportedSignature,
    /// The signature has more than eight corner or cone orders, or a corner or
    /// cone order above 1000, beyond which the accuracy of the domain is not
    /// established.
    LimitExceeded,
    /// Rounding made the construction degenerate, such as a sum of vertices
    /// that is not a point of the hyperbolic plane or of the sphere, a wall
    /// that cannot be found or is not finite, or a hyperbolic polygon, its
    /// double, or a domain of `o`, `**`, `xx`, `*x`, `22x`, `22*`, `2*22`,
    /// `4*2`, `3*3`, or `2222` that failed a check of
    /// [`FundamentalDomain::verify`](crate::FundamentalDomain::verify).
    ///
    /// A hyperbolic polygon gives it when its shape parameters are too
    /// extreme for the construction to meet the tolerances of the checks. The
    /// rotation signature of a polygon gives it when the double of the polygon
    /// at its default shape fails a check, which happens for some signatures
    /// with eight cone orders of which several are large, such as
    /// `(967)(967)(967)(967)(967)(967)(967)(967)`, because the side pairings
    /// of the double, stored as `f64` matrices, are too inexact for the check
    /// of its vertex cycles. The text of the error then names the side
    /// pairings. No other rotation signature is known to give it. The domain
    /// of `o`, `**`, `xx`, `*x`, `22x`, `22*`, `2*22`, `4*2`, `3*3`, or `2222`
    /// gives it when it fails a check, which no setting in the ranges of its parameters is
    /// known to cause, and its text then names the verification.
    Numerical,
    /// A setting names no shape parameter of the signature.
    ///
    /// The names of the parameters are those that
    /// [`parameters`](crate::parameters()) lists, and a name matches only when
    /// it is equal byte for byte.
    UnknownParameter,
    /// The value of a setting is NaN, infinite, or outside the range of its
    /// shape parameter.
    ParameterOutOfRange,
    /// The sides of the domain give no side presentation.
    ///
    /// The domain has a different number of corners than of sides, it has
    /// more than 256 sides, the side pairings do not lead around a vertex back
    /// to it, the corners of a vertex cycle differ in order, the corners of a
    /// vertex cycle of a double do not join two corners of the reflection
    /// domain in total, or the relators that the sides give are not a valid
    /// presentation. No domain that
    /// [`FundamentalDomain::build`](crate::FundamentalDomain::build) or
    /// [`FundamentalDomain::build_with`](crate::FundamentalDomain::build_with)
    /// returns gives this error.
    InvalidPairing,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
enum Detail {
    #[error("the signature {0} is not supported")]
    UnsupportedSignature(String),
    #[error("order {0} is larger than {LARGEST_ORDER}")]
    LimitExceeded(u32),
    #[error("the signature has {0} orders, more than {MOST_ORDERS}")]
    TooManyOrders(usize),
    #[error("the construction is numerically degenerate")]
    Numerical(Option<escher_geometry::Error>),
    #[error("the shape is too extreme to build accurately")]
    Refused,
    #[error("the side pairings of the domain cannot be stored accurately enough")]
    RefusedDouble,
    #[error("the wallpaper domain fails its verification")]
    RefusedWallpaper,
    #[error("the signature {0} has no shape parameter with one of the given names")]
    UnknownParameter(String),
    #[error(
        "the parameter {} is outside its range of {} to {}",
        .0.name(),
        .0.minimum(),
        .0.maximum()
    )]
    ParameterOutOfRange(Parameter),
    #[error("{0}")]
    InvalidPairing(PairingDefect),
    #[error("the side relators do not form a presentation")]
    RejectedPresentation(escher_group::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum PairingDefect {
    #[error("the domain has a different number of corners than of sides")]
    CornerCount,
    #[error("the domain has more than {MAX_SIDES} sides")]
    TooManySides,
    #[error("the side pairings do not close a vertex cycle")]
    OpenCycle,
    #[error("a vertex cycle joins corners of different orders")]
    CycleOrders,
}

#[cfg(test)]
mod tests {
    use escher_signature::Signature;

    use super::{Error, ErrorKind, PairingDefect};
    use crate::parameters::Parameter;

    fn signature(text: &str) -> Signature {
        match text.parse() {
            Ok(signature) => signature,
            Err(error) => panic!("{text:?} is rejected: {error}"),
        }
    }

    fn geometry_error() -> escher_geometry::Error {
        match escher_geometry::hyperbolic::normalize_point(glam::DVec3::X) {
            Ok(point) => panic!("a spacelike vector normalizes to {point:?}"),
            Err(error) => error,
        }
    }

    fn group_error() -> escher_group::Error {
        match escher_group::Relator::new(escher_group::Word::new(), 2) {
            Ok(relator) => panic!("an empty base gives the relator {relator:?}"),
            Err(error) => error,
        }
    }

    fn errors() -> [Error; 13] {
        [
            Error::unsupported(&signature("3*2")),
            Error::limit_exceeded(1001),
            Error::numerical(Some(geometry_error())),
            Error::numerical(None),
            Error::unknown_parameter(&signature("*236")),
            Error::parameter_out_of_range(Parameter::Aspect),
            Error::too_many_orders(9),
            Error::refused(),
            Error::parameter_out_of_range(Parameter::Slice(1)),
            Error::parameter_out_of_range(Parameter::CornerPair(2)),
            Error::refused_double(),
            Error::refused_wallpaper(),
            Error::parameter_out_of_range(Parameter::Skew),
        ]
    }

    fn pairing_errors() -> [Error; 5] {
        [
            Error::invalid_pairing(PairingDefect::CornerCount),
            Error::invalid_pairing(PairingDefect::TooManySides),
            Error::invalid_pairing(PairingDefect::OpenCycle),
            Error::invalid_pairing(PairingDefect::CycleOrders),
            Error::rejected_presentation(group_error()),
        ]
    }

    #[test]
    fn display_of_each_kind_starts_with_the_operation_and_is_lower_case() {
        let cases = errors()
            .map(|error| (error, "cannot build fundamental domain: "))
            .into_iter()
            .chain(pairing_errors().map(|error| (error, "cannot derive side presentation: ")));
        for (error, prefix) in cases {
            let text = error.to_string();
            assert!(text.starts_with(prefix), "{text}");
            let rest = text.replace("*632", "");
            assert_eq!(rest, rest.to_lowercase(), "{text}");
            assert!(!text.ends_with('.'), "{text}");
        }
    }

    #[test]
    fn display_of_each_kind_is_the_documented_text() {
        let texts = errors().map(|error| error.to_string());
        assert_eq!(
            texts,
            [
                "cannot build fundamental domain: the signature 3*2 is not supported",
                "cannot build fundamental domain: order 1001 is larger than 1000",
                "cannot build fundamental domain: the construction is numerically degenerate",
                "cannot build fundamental domain: the construction is numerically degenerate",
                "cannot build fundamental domain: the signature *632 has no shape parameter with one of the given names",
                "cannot build fundamental domain: the parameter aspect is outside its range of 0.125 to 8",
                "cannot build fundamental domain: the signature has 9 orders, more than 8",
                "cannot build fundamental domain: the shape is too extreme to build accurately",
                "cannot build fundamental domain: the parameter slice1.length is outside its range of 0.5 to 3.5",
                "cannot build fundamental domain: the parameter cornerpair2.length is outside its range of 0.5 to 3.5",
                "cannot build fundamental domain: the side pairings of the domain cannot be stored accurately enough",
                "cannot build fundamental domain: the wallpaper domain fails its verification",
                "cannot build fundamental domain: the parameter skew is outside its range of -0.5 to 0.5",
            ],
        );
        let texts = pairing_errors().map(|error| error.to_string());
        assert_eq!(
            texts,
            [
                "cannot derive side presentation: the domain has a different number of corners than of sides",
                "cannot derive side presentation: the domain has more than 256 sides",
                "cannot derive side presentation: the side pairings do not close a vertex cycle",
                "cannot derive side presentation: a vertex cycle joins corners of different orders",
                "cannot derive side presentation: the side relators do not form a presentation",
            ],
        );
    }

    #[test]
    fn kind_matches_each_constructor() {
        assert_eq!(
            errors().map(|error| error.kind()),
            [
                ErrorKind::UnsupportedSignature,
                ErrorKind::LimitExceeded,
                ErrorKind::Numerical,
                ErrorKind::Numerical,
                ErrorKind::UnknownParameter,
                ErrorKind::ParameterOutOfRange,
                ErrorKind::LimitExceeded,
                ErrorKind::Numerical,
                ErrorKind::ParameterOutOfRange,
                ErrorKind::ParameterOutOfRange,
                ErrorKind::Numerical,
                ErrorKind::Numerical,
                ErrorKind::ParameterOutOfRange,
            ],
        );
        assert_eq!(
            pairing_errors().map(|error| error.kind()),
            [ErrorKind::InvalidPairing; 5]
        );
    }

    #[test]
    fn source_is_the_geometry_error_only_for_a_numerical_error_that_one_caused() {
        let caused = Error::numerical(Some(geometry_error()));
        for error in errors() {
            let source = core::error::Error::source(&error).map(ToString::to_string);
            let expected = (error == caused).then(|| geometry_error().to_string());
            assert_eq!(source, expected, "{error}");
        }
    }

    #[test]
    fn source_is_the_group_error_only_for_a_rejected_presentation() {
        let [corners, sides, open, orders, rejected] = pairing_errors();
        let source = core::error::Error::source(&rejected).map(ToString::to_string);
        assert_eq!(source, Some(group_error().to_string()));
        for error in [corners, sides, open, orders] {
            assert!(core::error::Error::source(&error).is_none(), "{error}");
        }
    }
}
