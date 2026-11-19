//! Builds the fundamental domain of an orbifold signature as a polygon of the
//! hyperbolic plane, the Euclidean plane, or the sphere, and checks it
//! numerically.
//!
//! A fundamental domain of a symmetry group is a polygon whose copies under
//! the group tile the plane or the sphere without overlap. This crate turns a
//! [`Signature`](escher_signature::Signature) of the crate
//! [`escher_signature`] into a [`FundamentalDomain`]. It computes with the
//! linear model of the hyperbolic plane, with the Euclidean plane, and with
//! the unit sphere of the crate [`escher_geometry`]. A renderer can use the
//! domain to fold each pixel into one tile.
//!
//! The crate supports three kinds of signature. A signature with one mirror
//! boundary, such as `*632`, has a reflection domain, a polygon whose sides
//! are mirrors, the walls in which the group reflects. A signature with cone
//! points, the centres of rotation, and no mirror boundary, such as `632`, has
//! the double of a reflection domain, which is the union of the reflection
//! domain and its mirror image across the wall of its side 1, the side that
//! joins its vertices 0 and 1. The sides of a double are paired by rotations
//! and translations. A translation joins one pair of sides of the double of
//! `2222`, and each pair of sides of the double of a hyperbolic polygon whose
//! walls do not meet the wall of side 1.
//!
//! The reflection domains are these. A signature `*pqr` with three finite
//! corner orders has a triangle whose corners have the angles `pi / p`,
//! `pi / q`, and `pi / r`. The signature is hyperbolic when the reciprocals of
//! the orders sum to less than 1, and no order may exceed 1000. It is
//! Euclidean for `*632`, `*442`, and `*333`, and spherical for `*332`, `*432`,
//! `*532`, and `*n22`, with `n` from 2 to 1000. The signature `*2222` has a
//! rectangle, and its shape parameter `aspect`, which
//! [`parameters`](parameters()) lists, sets the ratio of its sides. The
//! signature `*nn`, with `n` from 2 to 1000, has a lune, the region of the
//! sphere between two half great circles that meet at two antipodal vertices
//! at the angle `pi / n`. A hyperbolic signature with four to eight corners,
//! such as `*3333` or `*5432`, has a polygon of the hyperbolic plane with
//! those corners in the order of the signature. A polygon with `n` corners
//! has `n - 3` shape parameters, which are lengths from 0.5 to 3.5. The crate
//! returns a polygon only when it passes its checks. A signature without a
//! mirror boundary has a domain when its cone orders are the corner orders of
//! a triangle, the rectangle, a lune, or a hyperbolic polygon, such as `732`,
//! `632`, `2222`, `532`, `n22`, `nn`, and `3333`. The double of a polygon
//! doubles the polygon with every length at its default, so it has no shape
//! parameter, and the crate returns it only when it passes its checks.
//!
//! The third kind is a wallpaper signature: one of the nine Euclidean
//! signatures that are neither a reflection signature nor a rotation
//! signature of the kinds above, which are `o`, `**`, `xx`, `*x`, `22*`,
//! `22x`, `2*22`, `4*2`, and `3*3`. The crate builds five of them. The
//! signature `o` has a parallelogram whose opposite sides are paired by
//! translations, with the shape parameters `aspect` and `skew`. The
//! signatures `**`, `xx`, `*x`, and `22x` have the rectangle of `*2222`, with
//! the shape parameter `aspect`. Its bottom and top sides are paired by a
//! translation for `**` and by glide reflections along vertical axes for the
//! other three. Its left and right sides are mirrors for `**` and `*x`, are
//! paired by a translation for `xx`, and are paired by glide reflections along
//! the horizontal axis for `22x`. The crate returns such a domain only when it
//! passes its checks.
//!
//! Every other signature gives an [`Error`]. The causes are an unsupported
//! signature, more than eight corner or cone orders, an order above 1000, and
//! a polygon, a double of a polygon, or a wallpaper domain that fails its
//! checks.
//! [`FundamentalDomain::verify`] checks a domain numerically, and
//! [`FundamentalDomain::side_presentation`] reads a
//! [`Presentation`](escher_group::Presentation) of its group from its sides.
//! The crate [`escher_group`] defines that type.
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
mod double;
mod error;
mod parameters;
mod polygon;
mod presentation;
mod sheets;
mod spherical;
mod triangle;
mod verify;
mod walk;
mod wallpaper;

#[cfg(test)]
extern crate self as escher_domain;

#[cfg(test)]
mod fixtures;

#[cfg(test)]
#[path = "../tests/domain/shared.rs"]
mod shared;

#[cfg(test)]
#[expect(
    dead_code,
    reason = "the unit tests of the crate use only a part of the reference arithmetic"
)]
#[path = "../tests/domain/reference.rs"]
mod reference;

pub use domain::{FundamentalDomain, Orientation, PairingKind, Side};
pub use error::{Error, ErrorKind, Result};
pub use parameters::{ParameterSpec, ShapeParameters, parameters};
pub use verify::{Check, Verification};

#[cfg(test)]
mod tests {
    use super::{
        Check, Error, ErrorKind, FundamentalDomain, Orientation, PairingKind, ParameterSpec,
        ShapeParameters, Side, Verification,
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
        assert_auto_traits::<ParameterSpec>();
        assert_auto_traits::<ShapeParameters>();
    }
}
