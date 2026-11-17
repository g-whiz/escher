//! Computes with points, walls, and isometries of the hyperbolic plane in its
//! linear model, of the Euclidean plane, and of the sphere, and maps points of
//! the hyperbolic plane to and from the Poincare disk.
//!
//! The crate is the one place in Escher where the conventions of the linear
//! model become code, and it depends on no other crate of the workspace. In
//! the linear model, also called the hyperboloid model, a point of the
//! hyperbolic plane is a vector on a sheet of a hyperboloid, a surface in
//! three dimensions. The functions of the hyperbolic plane are in the module
//! [`hyperbolic`], those of the Euclidean plane, whose points have the last
//! coordinate 1, in the module [`euclidean`], and those of the unit sphere,
//! whose points are unit vectors, in the module [`spherical`]. Each module
//! states its conventions. The functions take and return the types [`glam::DVec3`],
//! [`glam::DMat3`], and [`glam::DVec2`], and the crate defines no vector or
//! matrix type of its own.
//!
//! ```
//! use escher_geometry::hyperbolic::{ORIGIN, boost, distance, from_disk, to_disk};
//! use glam::DVec2;
//!
//! let point = boost(1.5) * ORIGIN;
//! assert!((distance(ORIGIN, point) - 1.5).abs() < 1e-12);
//!
//! let position = to_disk(point);
//! assert!(position.length() < 1.0);
//! assert_eq!(from_disk(DVec2::new(1.0, 0.0)), None);
//! ```
//!
//! # Features
//!
//! This crate has no Cargo features.
//!
//! # Platform support
//!
//! The crate is pure Rust with no platform-specific code. The last bits of
//! the results of the transcendental functions, such as [`f64::cosh`], come
//! from the platform's mathematical library and can differ between platforms.

mod error;
pub mod euclidean;
pub mod hyperbolic;
pub mod spherical;

pub use error::{Error, ErrorKind, Result};

#[cfg(test)]
mod tests {
    use super::{Error, ErrorKind};

    fn assert_auto_traits<T: Send + Sync + 'static>() {}

    #[test]
    fn error_and_error_kind_are_send_sync_and_static() {
        assert_auto_traits::<Error>();
        assert_auto_traits::<ErrorKind>();
    }
}
