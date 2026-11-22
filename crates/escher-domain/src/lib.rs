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
//! The crate supports four kinds of signature. A signature with one mirror
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
//! the orders sum to less than 1, and no order may exceed 100. It is
//! Euclidean for `*632`, `*442`, and `*333`, and spherical for `*332`, `*432`,
//! `*532`, and `*n22`, with `n` from 2 to 100. The signature `*2222` has a
//! rectangle, and its shape parameter `aspect`, which
//! [`parameters`](parameters()) lists, sets the ratio of its sides. The
//! signature `*nn`, with `n` from 2 to 100, has a lune, the region of the
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
//! parameter, and the crate returns it only when it passes its checks. The
//! domain of `2222` has a second shape parameter, `skew`. At a `skew` other
//! than 0, the double of the rectangle is sheared along its first axis, which
//! keeps the numbering of its sides, and its pairings are half-turns about
//! the sheared vertices 0 and 3 and translations. The crate returns the
//! domain of `2222` only when it passes its checks, at every `skew`.
//!
//! The third kind is a wallpaper signature: one of the nine Euclidean
//! signatures that are neither a reflection signature nor a rotation
//! signature of the kinds above, which are `o`, `**`, `xx`, `*x`, `22*`,
//! `22x`, `2*22`, `4*2`, and `3*3`. The crate builds all nine. The signature `o` has a parallelogram whose opposite
//! sides are paired by translations, with the shape parameters `aspect` and
//! `skew`. The signatures `**`, `xx`, `*x`, `22x`, `22*`, and `2*22` have the
//! rectangle of `*2222`, with the shape parameter `aspect`. Its bottom and top
//! sides are paired by a translation for `**` and by glide reflections along
//! vertical axes for `xx`, `*x`, and `22x`. Its left and right sides are
//! mirrors for `**` and `*x`, are paired by a translation for `xx`, and are
//! paired by glide reflections along the horizontal axis for `22x`. For `22*`,
//! the bottom and top sides are each split at their midpoint into two sides
//! that the half-turn about that midpoint pairs, and the left and right sides
//! are mirrors, which gives six sides. For `2*22`, only the bottom side is
//! split in this way, and the other three sides are mirrors, which gives five
//! sides. The signatures `4*2` and `3*3` have the kite of `442` or `632`, with
//! no shape parameter. Its sides 0 and 1 are mirrors, and its sides 2 and 3
//! are paired by the rotation about vertex 2, through a quarter turn for
//! `4*2` and a third of a turn for `3*3`. The crate returns such a domain only
//! when it passes its checks.
//!
//! The fourth kind is one of the remaining spherical signatures. A signature
//! with a cone point beside a mirror boundary, `3*2`, `2*n` with `n` from 2 to
//! 50, or `n*` with `n` from 2 to 100, which includes `2*`, has the kite of the double of a spherical
//! triangle with one pair of its sides turned into mirrors, with no shape
//! parameter. The kite of `432` is the domain of `3*2`, and the kite of
//! `(2n)22` is the domain of `2*n`, each with its sides 0 and 1 as mirrors and
//! its sides 2 and 3 paired by the rotation about vertex 2. The kite of `n22`
//! is the domain of `n*`, with its sides 2 and 3 as mirrors and its sides 0
//! and 1 paired by the rotation through `2 pi / n` about vertex 0. The crate
//! returns such a domain only when it passes its checks. The signature `nx`,
//! with `n` from 2 to 100, has the lune of `*nn`, whose two sides are paired
//! by rotary reflections, each the reflection in the wall of its side
//! followed by the half-turn about the axis through the centre of the lune.
//! The signatures `*` and `x` have the lune of order 1, a hemisphere whose two
//! sides lie on one great circle and meet at the angle `pi`. Its sides are
//! mirrors for `*` and are paired by the antipodal map for `x`. The trivial
//! signature `1` has the whole sphere, with no side, no vertex, and the area
//! `4 pi`. The crate returns the domains of `nx`, `x`, and `1` only when they
//! pass their checks. So every spherical signature has a domain within these
//! limits.
//!
//! Every other signature gives an [`Error`]. The causes are an unsupported
//! signature, more than eight corner or cone orders, an order above 100, or
//! above 50 for `2*n`, and a polygon, a double of a polygon, a wallpaper
//! domain, or a domain of `3*2`, `2*n`, `n*`, `nx`, `x`, or `1` that fails its
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
mod kite;
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
