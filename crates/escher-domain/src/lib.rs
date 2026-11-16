//! Builds the fundamental domain of an orbifold signature as a polygon of the
//! hyperbolic plane, and checks it numerically.
//!
//! A fundamental domain of a symmetry group is a polygon whose copies under
//! the group tile the plane without overlap. This crate turns a
//! [`Signature`](escher_signature::Signature) of the crate
//! [`escher_signature`] into a [`FundamentalDomain`]. It computes with the
//! linear model of the hyperbolic plane of the crate [`escher_geometry`]. A
//! renderer can use the domain to fold each pixel into one tile.
//!
//! The crate supports compact hyperbolic signatures of the form `*pqr`, which
//! have one mirror boundary and three finite corner orders, with no corner
//! order above 1000. The domain of such a signature is a reflection triangle,
//! a triangle whose sides are mirrors and whose corners have the angles
//! `pi / p`, `pi / q`, and `pi / r`. Every other signature gives an [`Error`].
//! A signature of another form is unsupported, and a corner order above 1000
//! is over the limit. [`FundamentalDomain::verify`] checks a domain
//! numerically.
//!
//! ```
//! use escher_domain::FundamentalDomain;
//! use escher_signature::Signature;
//!
//! let signature: Signature = "*237".parse()?;
//! let domain = FundamentalDomain::build(&signature)?;
//! let lengths = domain.side_lengths();
//! assert_eq!(lengths.len(), 3);
//! assert!((domain.area() - core::f64::consts::PI / 42.0).abs() < 1e-12);
//! assert!(domain.verify().passed());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! This crate has no Cargo features.
//!
//! # Platform support
//!
//! The crate is pure Rust with no platform-specific code. The last bits of
//! the results of the transcendental functions, such as [`f64::acosh`], come
//! from the platform's mathematical library and can differ between platforms.

mod domain;
mod error;
mod triangle;
mod verify;

pub use domain::{FundamentalDomain, Orientation, PairingKind, Side};
pub use error::{Error, ErrorKind, Result};
pub use verify::{Check, Verification};

#[cfg(test)]
mod tests {
    use super::{
        Check, Error, ErrorKind, FundamentalDomain, Orientation, PairingKind, Side, Verification,
    };

    fn assert_auto_traits<T: Send + Sync + 'static>() {}

    #[test]
    fn public_types_are_send_sync_and_static() {
        assert_auto_traits::<FundamentalDomain>();
        assert_auto_traits::<Side>();
        assert_auto_traits::<PairingKind>();
        assert_auto_traits::<Orientation>();
        assert_auto_traits::<Verification>();
        assert_auto_traits::<Check>();
        assert_auto_traits::<Error>();
        assert_auto_traits::<ErrorKind>();
    }
}
