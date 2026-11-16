use core::fmt;
use core::ops::Range;

use crate::order::Order;

/// A specialized [`Result`](core::result::Result) whose error is [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// The reason why a text is not a valid orbifold signature.
///
/// Match on [`Error::kind`] to tell the causes apart, and use [`Error::span`]
/// to point at the part of the text that caused the error. The
/// [`Display`](core::fmt::Display) text starts with
/// `cannot parse orbifold signature: ` and is meant for a person, not for
/// matching.
///
/// # Examples
///
/// ```
/// use escher_signature::{Error, ErrorKind, Signature};
///
/// let error = "*7q2".parse::<Signature>().err();
/// assert_eq!(error.as_ref().map(Error::kind), Some(ErrorKind::UnexpectedCharacter));
/// assert_eq!(error.as_ref().map(Error::span), Some(2..3));
/// assert_eq!(
///     error.map(|error| error.to_string()).as_deref(),
///     Some("cannot parse orbifold signature: unexpected character 'q'"),
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("cannot {operation}: {detail}")]
pub struct Error {
    operation: &'static str,
    detail: Detail,
    span: Range<usize>,
}

impl Error {
    pub(crate) fn new(detail: Detail, span: Range<usize>) -> Self {
        Self {
            operation: "parse orbifold signature",
            detail,
            span,
        }
    }

    /// Returns the cause of the error as an [`ErrorKind`].
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self.detail {
            Detail::Empty => ErrorKind::Empty,
            Detail::UnexpectedCharacter(_) => ErrorKind::UnexpectedCharacter,
            Detail::UnclosedParenthesis => ErrorKind::UnclosedParenthesis,
            Detail::ZeroOrder => ErrorKind::ZeroOrder,
            Detail::MultiDigitOrder => ErrorKind::MultiDigitOrder,
            Detail::AmbiguousOrder(_) => ErrorKind::AmbiguousOrder,
            Detail::LimitExceeded(_) => ErrorKind::LimitExceeded,
            Detail::BadOrbifold => ErrorKind::BadOrbifold,
            Detail::PositiveNonCompact => ErrorKind::PositiveNonCompact,
            Detail::Overflow(_) => ErrorKind::Overflow,
        }
    }

    /// Returns the byte range of the text that caused the error.
    ///
    /// The range lies within the text and starts and ends on character
    /// boundaries. An error that concerns the signature as a whole spans the
    /// whole text, so the error for an empty text spans `0..0`.
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

/// The cause of an [`Error`], for a caller to match on.
///
/// The enum is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
///
/// # Examples
///
/// ```
/// use escher_signature::{ErrorKind, Signature};
///
/// let kind = "23".parse::<Signature>().map_err(|error| error.kind());
/// assert_eq!(kind, Err(ErrorKind::BadOrbifold));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The text is empty.
    Empty,
    /// The text holds a character that the signature grammar does not allow
    /// at that place, including whitespace.
    UnexpectedCharacter,
    /// The text ends inside a parenthesized order.
    UnclosedParenthesis,
    /// An order is 0.
    ZeroOrder,
    /// A digit `1` outside parentheses is directly followed by another digit,
    /// which could be one order of 10 or more, or several orders.
    MultiDigitOrder,
    /// An order other than 1 follows a mirror boundary with a handle or a
    /// crosscap in between, so it could be a corner of that boundary or a cone
    /// point.
    AmbiguousOrder,
    /// The text is longer than 1024 bytes, an order is larger than
    /// 4294967295, or the signature has more than 64 features, counting each
    /// handle twice.
    LimitExceeded,
    /// The signature is a bad orbifold, which no symmetry group has.
    ///
    /// A bad orbifold has no handle, no crosscap, and no infinite order. It
    /// is either one cone point, two cone points of different orders, or one
    /// mirror boundary with one corner or with two corners of different
    /// orders, and it has no other feature.
    BadOrbifold,
    /// The signature has an infinite order and a positive Euler
    /// characteristic.
    PositiveNonCompact,
    /// A value that parsing computes does not fit, for one of three causes:
    ///
    /// - The Euler characteristic or twice it does not fit exact 64-bit
    ///   arithmetic. This error arises before [`ErrorKind::BadOrbifold`] and
    ///   [`ErrorKind::PositiveNonCompact`].
    /// - The group order of a spherical signature is not a whole number or
    ///   does not fit a 64-bit unsigned integer.
    /// - The deformation dimension of a compact hyperbolic signature does not
    ///   fit a 32-bit unsigned integer, which includes a negative value.
    ///
    /// The last two errors arise after the other checks, in this order.
    Overflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum Detail {
    #[error("the signature is empty")]
    Empty,
    #[error("unexpected character {0:?}")]
    UnexpectedCharacter(char),
    #[error("unclosed parenthesis")]
    UnclosedParenthesis,
    #[error("order 0 is not allowed")]
    ZeroOrder,
    #[error(
        "a digit 1 followed by another digit is ambiguous, write an order of 10 or more in parentheses, such as (12)"
    )]
    MultiDigitOrder,
    #[error("order {} after a boundary and a handle or crosscap is ambiguous", .0.word())]
    AmbiguousOrder(Order),
    #[error("{0}")]
    LimitExceeded(Limit),
    #[error("bad orbifold, no symmetry group has this signature")]
    BadOrbifold,
    #[error("an infinite order needs an Euler characteristic of 0 or less")]
    PositiveNonCompact,
    #[error("{0}")]
    Overflow(Value),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Limit {
    Length,
    Order,
    Size,
}

impl fmt::Display for Limit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Length => "the signature is longer than 1024 bytes",
            Self::Order => "order is larger than 4294967295",
            Self::Size => "the signature has more than 64 features",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Value {
    EulerCharacteristic,
    GroupOrder,
    DeformationDimension,
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EulerCharacteristic => {
                "the Euler characteristic does not fit exact 64-bit arithmetic"
            }
            Self::GroupOrder => {
                "the group order is not a whole number or does not fit a 64-bit unsigned integer"
            }
            Self::DeformationDimension => {
                "the deformation dimension does not fit a 32-bit unsigned integer"
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Detail, Error, ErrorKind, Limit, Value};
    use crate::order::Order;

    const PREFIX: &str = "cannot parse orbifold signature: ";

    fn every_detail() -> Vec<(Detail, ErrorKind, &'static str)> {
        let three = Order::from_value(3).expect("3 is a valid order");
        vec![
            (Detail::Empty, ErrorKind::Empty, "the signature is empty"),
            (
                Detail::UnexpectedCharacter('Q'),
                ErrorKind::UnexpectedCharacter,
                "unexpected character 'Q'",
            ),
            (
                Detail::UnexpectedCharacter('\n'),
                ErrorKind::UnexpectedCharacter,
                "unexpected character '\\n'",
            ),
            (
                Detail::UnclosedParenthesis,
                ErrorKind::UnclosedParenthesis,
                "unclosed parenthesis",
            ),
            (
                Detail::ZeroOrder,
                ErrorKind::ZeroOrder,
                "order 0 is not allowed",
            ),
            (
                Detail::MultiDigitOrder,
                ErrorKind::MultiDigitOrder,
                "a digit 1 followed by another digit is ambiguous, write an order of 10 or more in parentheses, such as (12)",
            ),
            (
                Detail::AmbiguousOrder(three),
                ErrorKind::AmbiguousOrder,
                "order 3 after a boundary and a handle or crosscap is ambiguous",
            ),
            (
                Detail::AmbiguousOrder(Order::INFINITE),
                ErrorKind::AmbiguousOrder,
                "order inf after a boundary and a handle or crosscap is ambiguous",
            ),
            (
                Detail::LimitExceeded(Limit::Length),
                ErrorKind::LimitExceeded,
                "the signature is longer than 1024 bytes",
            ),
            (
                Detail::LimitExceeded(Limit::Order),
                ErrorKind::LimitExceeded,
                "order is larger than 4294967295",
            ),
            (
                Detail::LimitExceeded(Limit::Size),
                ErrorKind::LimitExceeded,
                "the signature has more than 64 features",
            ),
            (
                Detail::BadOrbifold,
                ErrorKind::BadOrbifold,
                "bad orbifold, no symmetry group has this signature",
            ),
            (
                Detail::PositiveNonCompact,
                ErrorKind::PositiveNonCompact,
                "an infinite order needs an Euler characteristic of 0 or less",
            ),
            (
                Detail::Overflow(Value::EulerCharacteristic),
                ErrorKind::Overflow,
                "the Euler characteristic does not fit exact 64-bit arithmetic",
            ),
            (
                Detail::Overflow(Value::GroupOrder),
                ErrorKind::Overflow,
                "the group order is not a whole number or does not fit a 64-bit unsigned integer",
            ),
            (
                Detail::Overflow(Value::DeformationDimension),
                ErrorKind::Overflow,
                "the deformation dimension does not fit a 32-bit unsigned integer",
            ),
        ]
    }

    #[test]
    fn display_writes_the_prefix_and_the_text_of_each_kind() {
        for (detail, _, text) in every_detail() {
            let error = Error::new(detail, 0..0);
            assert_eq!(error.to_string(), format!("{PREFIX}{text}"));
        }
    }

    #[test]
    fn display_is_lower_case_apart_from_the_unexpected_character() {
        for (detail, _, _) in every_detail() {
            let shown = Error::new(detail, 0..0).to_string();
            let checked = shown.replace("'Q'", "").replace("Euler", "euler");
            assert!(checked.starts_with(PREFIX), "{checked}");
            assert_eq!(checked, checked.to_lowercase(), "{checked}");
            assert!(!checked.ends_with('.'), "{checked}");
        }
    }

    #[test]
    fn kind_matches_each_detail() {
        for (detail, kind, _) in every_detail() {
            assert_eq!(Error::new(detail, 0..0).kind(), kind);
        }
    }

    #[test]
    fn span_returns_the_stored_range() {
        assert_eq!(Error::new(Detail::Empty, 2..5).span(), 2..5);
    }

    #[test]
    fn source_is_none() {
        let error = Error::new(Detail::Empty, 0..0);
        assert!(core::error::Error::source(&error).is_none());
    }
}
