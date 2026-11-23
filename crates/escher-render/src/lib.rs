//! Renders the tiling of a fundamental domain, in the Poincare disk or another
//! model of the hyperbolic plane for a hyperbolic domain, in the plane for a
//! Euclidean one, and on a lit ball for a spherical one, and encodes the image
//! as a PNG file in memory.
//!
//! The crate draws a [`FundamentalDomain`](escher_domain::FundamentalDomain)
//! that the crate [`escher_domain`] builds. The domain is the reflection
//! triangle of a compact hyperbolic signature `*pqr`, the reflection polygon
//! of a compact hyperbolic signature with four to eight corners, such as
//! `*3333`, the reflection triangle of `*632`, `*442`, or `*333`, the
//! rectangle of `*2222`, the spherical reflection triangle of `*332`, `*432`,
//! `*532`, or `*n22`, or the lune of `*nn`, which is the region between two
//! half great circles of the sphere.
//! It can also be the domain of the signature without the star, such as `732`
//! for `*732`, whose group has no reflections. That domain is two copies of
//! the reflection domain joined across one side, and its side pairings are
//! rotations and translations. It has the place and the view of its reflection
//! domain and draws borders only on its own sides, and [`Fill::Parity`] gives
//! every tile its first colour. The domain of `2222` with a `skew` other than
//! 0 is those two rectangles sheared along the first axis, so its sides no
//! longer lie on the borders of the tiling of `*2222`.
//! It can also be the domain of the Euclidean signature `o`, a parallelogram
//! whose opposite sides are paired by translations, of `**`, a rectangle
//! with two mirror sides and two sides paired by a translation, of `xx`, a
//! rectangle whose sides are paired by glide reflections and by translations,
//! of `*x`, a rectangle with two mirror sides and two sides paired by a glide
//! reflection, of `22x`, a rectangle whose sides are paired by glide
//! reflections, of `22*` and `2*22`, a rectangle with mirror sides whose
//! bottom side, and for `22*` also its top side, is split at its midpoint into
//! two sides paired by the half-turn about that midpoint, or of `4*2` and
//! `3*3`, the kite of `442` or `632` with two mirror sides and two sides
//! paired by the rotation about the vertex between them. Each is drawn in the
//! view of the Euclidean plane, with borders on all of its sides. A glide reflection reverses orientation, so
//! [`Fill::Parity`] gives its two colours across a side paired by one. For
//! `o`, which has no reversing side, [`Fill::Parity`] gives every tile its
//! first colour.
//! It can also be the domain of the spherical signature `3*2`, `2*n`, or `n*`,
//! the kite of `432`, `(2n)22`, or `n22` with two mirror sides and two sides
//! paired by the rotation about the vertex between them. It is drawn on the
//! ball in the view of that kite, with borders on all of its sides, and
//! [`Fill::Parity`] gives its two colours across each mirror side. It can be
//! the domain of `nx`, the lune of `*nn` whose two sides are paired by rotary
//! reflections, which reverse orientation, so its picture with
//! [`Fill::Parity`] is that of `*nn`. It can be the hemisphere of `*`, with
//! two mirror sides, or of `x`, whose two sides are paired by the antipodal
//! map, which draw the same picture. Last, it can be the whole sphere of the
//! trivial signature `1`, which has no side, so every point lies in it, no
//! border is drawn, and [`Fill::Parity`] gives the whole ball its first
//! colour.
//!
//! The view follows the geometry of the domain. For a hyperbolic domain, the
//! Poincare disk is centred in the image, with the centre of the domain at the
//! centre of the disk. The [`HyperbolicModel`] of the options can instead draw
//! it in the upper half-plane or the band, which fill the image, or in the
//! Klein disk, which has the size of the disk, each with the centre of the
//! domain at the centre of the image. For a Euclidean domain, the tiling fills
//! the image, the centre of the domain lies at the centre of the image, and
//! the shorter side of the image spans 8 units of the plane. For a spherical domain, the ball
//! has the size and the place of the disk and shows the front half of the
//! sphere, turned by the viewing rotation and shaded by the [`Shading`] of the
//! options. A [`Renderer`] maps each sample of a regular grid in each pixel to
//! a point of the view, folds the point into the domain with [`fold`], and
//! colours the sample with a [`Fill`] colour, the border colour, or the
//! background colour of its [`RenderOptions`]. The border width is in pixels
//! of the image, or with [`BorderUnit::Intrinsic`] a length in the geometry
//! of the domain. A [`Fill`] gives every tile
//! one colour, two colours by parity, or the paint of its coset in a
//! [`Colouring`](escher_group::Colouring), through a [`CosetFill`]. A
//! [`Paint`] is a solid colour or an [`Image`] that [`Image::decode_png`]
//! reads from a PNG stream, and [`Masks`] draws, for each colour, the flat
//! polygon of the domain over which the image of that colour is painted. On a
//! reflection triangle, a [`WythoffFill`] instead draws the faces of a
//! [`Wythoff`] construction, a tiling by regular polygons, with a colour for
//! each face type and borders on the edges of the faces. The
//! renderer averages the samples of a pixel in linear light and stores the
//! pixel as 8-bit sRGB.
//! [`encode_png`] turns the pixels into the bytes of a PNG file. The crate
//! starts no thread and touches no file, so the caller decides how to split
//! the image into bands of rows and where to write it.
//!
//! ```
//! use escher_domain::FundamentalDomain;
//! use escher_render::{Colour, Fill, RenderOptions, Renderer, encode_png};
//! use escher_signature::Signature;
//!
//! let domain = FundamentalDomain::build(&"*542".parse::<Signature>()?)?;
//! let fill = Fill::Parity([Colour::WHITE, Colour::from_srgb8(220, 20, 60)]);
//! let options = RenderOptions::builder(fill).size(32, 32).build()?;
//! let (pixels, stats) = Renderer::new(&domain, options)?.render();
//! assert_eq!(stats.unconverged(), 0);
//! let file = encode_png(32, 32, &pixels)?;
//! assert!(file.starts_with(b"\x89PNG"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! This crate has no Cargo features.
//!
//! # Platform support
//!
//! The crate is pure Rust with no platform-specific code. The fold, the border
//! test, and the shading use only additions, multiplications, divisions, and
//! square roots, so an image depends on the platform's mathematical library
//! only through the domain, the viewing rotation, the complex hyperbolic
//! tangent of each sample of the band model, and the sRGB transfer function.

mod colour;
mod encode;
mod error;
mod fold;
mod image;
mod mask;
mod options;
mod renderer;
mod view;
mod wythoff;

#[cfg(test)]
#[path = "../tests/render/reference.rs"]
mod reference;

#[cfg(test)]
#[path = "../tests/render/golden_images.rs"]
mod golden_images;

pub use colour::{Colour, CosetFill, Fill, Paint};
pub use encode::encode_png;
pub use error::{Error, ErrorKind, Result};
pub use fold::{Fold, fold};
pub use image::Image;
pub use mask::Masks;
pub use options::{BorderUnit, HyperbolicModel, RenderOptions, RenderOptionsBuilder, Shading};
pub use renderer::{RenderStats, Renderer};
pub use wythoff::{Wythoff, WythoffFill};

#[cfg(test)]
mod tests {
    use super::{
        BorderUnit, Colour, CosetFill, Error, ErrorKind, Fill, Fold, HyperbolicModel, Image, Masks,
        Paint, RenderOptions, RenderOptionsBuilder, RenderStats, Renderer, Shading, Wythoff,
        WythoffFill,
    };

    fn assert_auto_traits<T: Send + Sync + 'static>() {}

    #[test]
    fn public_types_are_send_sync_and_static() {
        assert_auto_traits::<Colour>();
        assert_auto_traits::<Fill>();
        assert_auto_traits::<CosetFill>();
        assert_auto_traits::<Paint>();
        assert_auto_traits::<Image>();
        assert_auto_traits::<Masks>();
        assert_auto_traits::<RenderOptions>();
        assert_auto_traits::<RenderOptionsBuilder>();
        assert_auto_traits::<Shading>();
        assert_auto_traits::<HyperbolicModel>();
        assert_auto_traits::<BorderUnit>();
        assert_auto_traits::<Renderer>();
        assert_auto_traits::<RenderStats>();
        assert_auto_traits::<Fold>();
        assert_auto_traits::<Wythoff>();
        assert_auto_traits::<WythoffFill>();
        assert_auto_traits::<Error>();
        assert_auto_traits::<ErrorKind>();
    }
}
