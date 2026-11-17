//! Integration tests of `escher-render` through its public interface.

#[cfg(test)]
use escher_render::{Colour, Fill, RenderOptions, Shading};

#[cfg(test)]
mod borders;
#[cfg(test)]
mod common;
#[cfg(test)]
mod encoding;
#[cfg(test)]
mod folds;
#[cfg(test)]
mod golden_images;
#[cfg(test)]
mod images;
#[cfg(test)]
mod properties;
#[cfg(test)]
mod reference;
