//! Integration tests of `escher-render` through its public interface.

#[cfg(test)]
use escher_render::{
    BorderUnit, Colour, CosetFill, Fill, HyperbolicModel, Image, Masks, Paint, RenderOptions,
    RenderOptionsBuilder, Renderer, Shading, encode_png,
};

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
mod image_fills;
#[cfg(test)]
mod images;
#[cfg(test)]
mod masks;
#[cfg(test)]
mod properties;
#[cfg(test)]
mod reference;
