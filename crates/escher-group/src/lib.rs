//! Builds finitely presented groups and enumerates the cosets of their
//! subgroups with Todd-Coxeter coset enumeration.
//!
//! A [`Presentation`] lists named generators and [`Relator`]s, each a
//! [`Word`] of [`Letter`]s raised to an exponent. [`conway_presentation`]
//! reads the presentation of the symmetry group of an orbifold signature,
//! a [`Signature`](escher_signature::Signature) of `escher-signature`, and
//! [`enumerate`] turns a presentation and the words that generate a subgroup
//! into the [`CosetTable`] of the subgroup, in a standard form in which equal
//! subgroups give equal tables. This crate is the one place in Escher where
//! a presentation becomes a coset table, and it depends only on
//! `escher-signature` within the workspace.
//!
//! The enumeration may define many more cosets than the index, and no
//! computable bound limits how many, so every call takes a [`CosetLimit`]
//! and returns an [`Error`] of kind [`ErrorKind::CosetLimitReached`] when it
//! needs more cosets than the limit allows. That error says nothing about
//! the index. Every input is checked against the limits of the crate before
//! it is used.
//!
//! ```
//! use escher_group::{CosetLimit, Letter, Word, conway_presentation, enumerate};
//! use escher_signature::Signature;
//!
//! let signature: Signature = "532".parse()?;
//! let presentation = conway_presentation(&signature)?;
//! assert_eq!(presentation.generator_names(), ["g1", "g2", "g3"]);
//!
//! let group = enumerate(&presentation, &[], CosetLimit::DEFAULT)?;
//! assert_eq!(group.index(), 60);
//!
//! let rotation: Word = [Letter::generator(0)].into_iter().collect();
//! let cosets = enumerate(&presentation, &[rotation], CosetLimit::DEFAULT)?;
//! assert_eq!(cosets.index(), 12);
//! assert_eq!(cosets.image(0, Letter::generator(0)), Some(0));
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

mod conway;
mod enumerate;
mod error;
mod presentation;
mod table;
mod word;

#[cfg(test)]
#[path = "../tests/group/common.rs"]
mod common;

pub use conway::conway_presentation;
pub use enumerate::enumerate;
pub use error::{Error, ErrorKind, Result};
pub use presentation::{Presentation, Relator, RelatorDisplay, WordDisplay};
pub use table::{CosetLimit, CosetTable};
pub use word::{Letter, Word};

#[cfg(test)]
mod tests {
    use super::{
        CosetLimit, CosetTable, Error, ErrorKind, Letter, Presentation, Relator, RelatorDisplay,
        Word, WordDisplay,
    };

    fn assert_auto_traits<T: Send + Sync + 'static>() {}

    fn assert_borrowing_auto_traits<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_sync_and_static() {
        assert_auto_traits::<Letter>();
        assert_auto_traits::<Word>();
        assert_auto_traits::<Relator>();
        assert_auto_traits::<Presentation>();
        assert_auto_traits::<CosetLimit>();
        assert_auto_traits::<CosetTable>();
        assert_auto_traits::<Error>();
        assert_auto_traits::<ErrorKind>();
        assert_borrowing_auto_traits::<WordDisplay<'_>>();
        assert_borrowing_auto_traits::<RelatorDisplay<'_>>();
    }
}
