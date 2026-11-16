//! Parses orbifold signatures into a canonical form, computes their exact
//! Euler characteristic, and classifies them.
//!
//! An orbifold signature, in the notation of John Conway, names a symmetry
//! group of the plane, the sphere, or the hyperbolic plane, such as `*632`
//! for the symmetries of a hexagonal tiling. This crate is the one place in
//! Escher where the text of a signature becomes a [`Signature`], and it
//! depends on no other crate of the workspace.
//!
//! A [`Signature`] is valid by construction. Parsing rejects malformed text,
//! text that could name two different groups, bad orbifolds, which no
//! symmetry group has, signatures with an infinite order and a positive Euler
//! characteristic, signatures whose Euler characteristic, group order, or
//! deformation dimension overflows, and input over the size limits. The
//! [`Error`] carries an [`ErrorKind`] and the byte span of the cause.
//!
//! A signature reports its [`Geometry`], whether it is compact, a frieze, or
//! general non-compact, and, where they are defined, its group order, its area
//! divided by pi, and its deformation dimension.
//!
//! ```
//! use escher_signature::{ErrorKind, Signature};
//!
//! let signature: Signature = "*236".parse()?;
//! assert_eq!(signature.to_string(), "*632");
//! assert_eq!(signature.euler_characteristic().to_string(), "0");
//!
//! let error = "*7 32".parse::<Signature>().err();
//! assert_eq!(error.map(|error| error.kind()), Some(ErrorKind::UnexpectedCharacter));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! This crate has no Cargo features.
//!
//! # Platform support
//!
//! The crate is pure Rust with no platform-specific code.

mod canonical;
mod classify;
mod error;
mod order;
mod parse;
mod rational;
mod signature;

pub use classify::Geometry;
pub use error::{Error, ErrorKind, Result};
pub use order::Order;
pub use rational::Rational;
pub use signature::{Boundary, Signature};

#[cfg(test)]
mod tests {
    use super::{Boundary, Error, ErrorKind, Geometry, Order, Rational, Signature};

    fn assert_auto_traits<T: Send + Sync + 'static>() {}

    #[test]
    fn public_types_are_send_sync_and_static() {
        assert_auto_traits::<Signature>();
        assert_auto_traits::<Boundary>();
        assert_auto_traits::<Order>();
        assert_auto_traits::<Rational>();
        assert_auto_traits::<Error>();
        assert_auto_traits::<ErrorKind>();
        assert_auto_traits::<Geometry>();
    }
}
