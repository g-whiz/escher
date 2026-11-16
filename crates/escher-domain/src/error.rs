use core::fmt;

use escher_signature::Signature;

use crate::triangle::LARGEST_ORDER;

/// A specialized [`Result`](core::result::Result) whose error is [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// The reason why no fundamental domain was built for a signature.
///
/// Match on [`Error::kind`] to tell the causes apart. The
/// [`Display`](fmt::Display) text starts with
/// `cannot build fundamental domain: `, states the cause, and is meant for a
/// person, not for matching.
///
/// # Examples
///
/// ```
/// use escher_domain::{ErrorKind, FundamentalDomain};
/// use escher_signature::Signature;
///
/// let signature: Signature = "*632".parse()?;
/// let error = FundamentalDomain::build(&signature).err();
/// assert_eq!(error.as_ref().map(|error| error.kind()), Some(ErrorKind::UnsupportedSignature));
/// assert_eq!(
///     error.map(|error| error.to_string()).as_deref(),
///     Some("cannot build fundamental domain: the signature *632 is not supported"),
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    operation: &'static str,
    detail: Detail,
}

const OPERATION: &str = "build fundamental domain";

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

    pub(crate) fn numerical(source: Option<escher_geometry::Error>) -> Self {
        Self {
            operation: OPERATION,
            detail: Detail::Numerical(source),
        }
    }

    /// Returns the cause of the error as an [`ErrorKind`].
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self.detail {
            Detail::UnsupportedSignature(_) => ErrorKind::UnsupportedSignature,
            Detail::LimitExceeded(_) => ErrorKind::LimitExceeded,
            Detail::Numerical(_) => ErrorKind::Numerical,
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
            Detail::UnsupportedSignature(_)
            | Detail::LimitExceeded(_)
            | Detail::Numerical(None) => None,
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
    /// Only a compact hyperbolic reflection triangle has one. Its signature
    /// has the form `*pqr` with three finite corner orders whose reciprocals
    /// sum to less than 1. A corner of order `n` is a corner where two mirrors
    /// meet at the angle `pi / n`.
    UnsupportedSignature,
    /// A corner order of a supported signature is larger than 1000, the
    /// largest order for which the accuracy of the domain is established.
    LimitExceeded,
    /// Rounding made the construction degenerate, such as a sum of vertices
    /// that is not a point of the hyperbolic plane or a wall that cannot be
    /// found or is not finite.
    ///
    /// No supported signature is known to produce it.
    Numerical,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
enum Detail {
    #[error("the signature {0} is not supported")]
    UnsupportedSignature(String),
    #[error("order {0} is larger than {LARGEST_ORDER}")]
    LimitExceeded(u32),
    #[error("the construction is numerically degenerate")]
    Numerical(Option<escher_geometry::Error>),
}

#[cfg(test)]
mod tests {
    use escher_signature::Signature;

    use super::{Error, ErrorKind};

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

    fn errors() -> [Error; 4] {
        [
            Error::unsupported(&signature("*236")),
            Error::limit_exceeded(1001),
            Error::numerical(Some(geometry_error())),
            Error::numerical(None),
        ]
    }

    #[test]
    fn display_of_each_kind_starts_with_the_operation_and_is_lower_case() {
        let prefix = "cannot build fundamental domain: ";
        for error in errors() {
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
                "cannot build fundamental domain: the signature *632 is not supported",
                "cannot build fundamental domain: order 1001 is larger than 1000",
                "cannot build fundamental domain: the construction is numerically degenerate",
                "cannot build fundamental domain: the construction is numerically degenerate",
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
            ],
        );
    }

    #[test]
    fn source_is_the_geometry_error_only_for_a_numerical_error_that_one_caused() {
        let [unsupported, limit, caused, uncaused] = errors();
        let source = core::error::Error::source(&caused).map(ToString::to_string);
        assert_eq!(source, Some(geometry_error().to_string()));
        for error in [unsupported, limit, uncaused] {
            assert!(core::error::Error::source(&error).is_none(), "{error}");
        }
    }
}
