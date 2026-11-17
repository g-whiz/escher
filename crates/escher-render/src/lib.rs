//! Renders the tiling of a fundamental domain, in the Poincare disk for a
//! hyperbolic domain, in the plane for a Euclidean one, and on a lit ball for
//! a spherical one, and encodes the image as a PNG file in memory.
//!
//! The crate draws a [`FundamentalDomain`](escher_domain::FundamentalDomain)
//! that the crate [`escher_domain`] builds. The domain is the reflection
//! triangle of a compact hyperbolic signature `*pqr`, the reflection triangle
//! of `*632`, `*442`, or `*333`, the rectangle of `*2222`, the spherical
//! reflection triangle of `*332`, `*432`, `*532`, or `*n22`, or the lune of
//! `*nn`, which is the region between two half great circles of the sphere.
//! The view follows the geometry of the domain. For a hyperbolic domain, the
//! Poincare disk is centred in the image, with the centre of the domain at the
//! centre of the disk. For a Euclidean domain, the tiling fills the image, the
//! centre of the domain lies at the centre of the image, and the shorter side
//! of the image spans 8 units of the plane. For a spherical domain, the ball
//! has the size and the place of the disk and shows the front half of the
//! sphere, turned by the viewing rotation and shaded by the [`Shading`] of the
//! options. A [`Renderer`] maps each sample of a regular grid in each pixel to
//! a point of the view, folds the point into the domain with [`fold`], and
//! colours the sample with a [`Fill`] colour, the border colour, or the
//! background colour of its [`RenderOptions`]. It averages the samples of a
//! pixel in linear light and stores the pixel as 8-bit sRGB.
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
//! let (pixels, stats) = Renderer::new(&domain, options).render();
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
//! only through the domain, the viewing rotation, and the sRGB transfer
//! function.

mod colour;
mod encode;
mod error;
mod fold;
mod options;
mod renderer;
mod view;

#[cfg(test)]
#[path = "../tests/render/reference.rs"]
mod reference;

#[cfg(test)]
#[path = "../tests/render/golden_images.rs"]
mod golden_images;

pub use colour::{Colour, Fill};
pub use encode::encode_png;
pub use error::{Error, ErrorKind, Result};
pub use fold::{Fold, fold};
pub use options::{RenderOptions, RenderOptionsBuilder, Shading};
pub use renderer::{RenderStats, Renderer};

#[cfg(test)]
mod tests {
    use super::{
        Colour, Error, ErrorKind, Fill, Fold, RenderOptions, RenderOptionsBuilder, RenderStats,
        Renderer, Shading,
    };

    fn assert_auto_traits<T: Send + Sync + 'static>() {}

    #[test]
    fn public_types_are_send_sync_and_static() {
        assert_auto_traits::<Colour>();
        assert_auto_traits::<Fill>();
        assert_auto_traits::<RenderOptions>();
        assert_auto_traits::<RenderOptionsBuilder>();
        assert_auto_traits::<Shading>();
        assert_auto_traits::<Renderer>();
        assert_auto_traits::<RenderStats>();
        assert_auto_traits::<Fold>();
        assert_auto_traits::<Error>();
        assert_auto_traits::<ErrorKind>();
    }
}
