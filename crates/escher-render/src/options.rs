use glam::DVec3;

use crate::colour::{Colour, Fill};
use crate::error::{Error, Result};

pub(crate) const LARGEST_SIDE: u32 = 16384;
pub(crate) const LARGEST_PIXELS: u64 = 1 << 26;
pub(crate) const LARGEST_GRID: u32 = 16;
pub(crate) const LARGEST_BORDER_WIDTH: f64 = 1024.0;
pub(crate) const LARGEST_FOLD_CAP: u32 = 100_000;

const DEFAULT_SIDE: u32 = 512;
const DEFAULT_GRID: u32 = 4;
const DEFAULT_BORDER_WIDTH: f64 = 1.0;
const DEFAULT_FOLD_CAP: u32 = 1500;
const DEFAULT_LIGHT: DVec3 = DVec3::new(-1.0, 1.0, 2.0);

/// How the renderer shades the ball of a spherical domain.
///
/// Pass it to [`RenderOptionsBuilder::shading`]. The renderer reads it only
/// for a spherical domain.
///
/// # Examples
///
/// ```
/// use escher_render::{Colour, Fill, RenderOptions, Shading};
///
/// let options = RenderOptions::builder(Fill::One(Colour::WHITE)).shading(Shading::Flat).build()?;
/// assert_eq!(options.width(), 512);
/// # Ok::<(), escher_render::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Shading {
    /// Shades the colour of each sample of the ball with the Phong reflection
    /// model, which has one white light that stays fixed to the viewer, an
    /// ambient term and a diffuse term that scale the colour, and a specular
    /// highlight, the bright spot on a glossy surface.
    Phong,
    /// Leaves the colour of each sample unchanged, so the ball looks like a
    /// flat disk.
    Flat,
}

/// The model of the hyperbolic plane in which the renderer draws a hyperbolic
/// domain.
///
/// Pass it to [`RenderOptionsBuilder::model`]. The renderer reads it only for
/// a hyperbolic domain. Every model shows the same tiling, with the centre of
/// the domain at the centre of the image. The Klein disk has the size of the
/// Poincare disk. The half-plane and the band fill the image, and the height
/// of the image spans 2 units of the model.
///
/// # Examples
///
/// ```
/// use escher_render::{Colour, Fill, HyperbolicModel, RenderOptions};
///
/// let options = RenderOptions::builder(Fill::One(Colour::WHITE)).model(HyperbolicModel::Band).build()?;
/// assert_eq!(options.width(), 512);
/// # Ok::<(), escher_render::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HyperbolicModel {
    /// The Poincare disk, which shows the whole plane inside a circle, with
    /// the walls as arcs that meet the circle at right angles.
    Disk,
    /// The upper half-plane, which shows the whole plane above a line, the
    /// real axis, with the walls as half circles centred on that line and as
    /// vertical half lines. The image of the centre of the disk, the point
    /// `i`, lies at the centre of the image, and the real axis lies along the
    /// bottom edge of the image. A wide image shows more of the real axis.
    HalfPlane,
    /// The band, which shows the whole plane in a horizontal strip whose two
    /// edges lie along the top and bottom edges of the image, with the image
    /// of the centre of the disk at the centre of the image. A wide image
    /// shows more of the strip.
    Band,
    /// The Klein disk, which shows the whole plane inside a circle of the size
    /// of the Poincare disk, with every wall as a straight chord.
    Klein,
}

/// The validated options of a render: the image size, the sample grid, the
/// border, the colours, the fold cap, the model of a hyperbolic domain, and
/// the viewing rotation, the light, and the shading of the ball of a spherical
/// domain.
///
/// Build it with [`RenderOptions::builder`], which starts from the defaults
/// and checks every limit, so a value of this type always holds options that
/// the renderer accepts.
///
/// # Examples
///
/// ```
/// use escher_render::{Colour, Fill, RenderOptions};
///
/// let options = RenderOptions::builder(Fill::One(Colour::WHITE)).size(640, 480).build()?;
/// assert_eq!((options.width(), options.height()), (640, 480));
/// # Ok::<(), escher_render::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct RenderOptions {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) samples: u32,
    pub(crate) border_width: f64,
    pub(crate) border_colour: Colour,
    pub(crate) background: Colour,
    pub(crate) max_folds: u32,
    pub(crate) rotation: [f64; 3],
    pub(crate) light: DVec3,
    pub(crate) shading: Shading,
    pub(crate) model: HyperbolicModel,
    pub(crate) fill: Fill,
}

impl RenderOptions {
    /// Returns a builder that starts from the default options with the
    /// given fill.
    ///
    /// The defaults are an image of 512 by 512 pixels, a grid of 4 by 4
    /// samples in each pixel, a border 1 pixel wide in
    /// [`Colour::BLACK`], a [`Colour::WHITE`] background, a fold cap of 1500
    /// steps, no viewing rotation, a light in the direction `(-1, 1, 2)`,
    /// [`Shading::Phong`], and [`HyperbolicModel::Disk`].
    pub fn builder(fill: Fill) -> RenderOptionsBuilder {
        RenderOptionsBuilder {
            options: Self {
                width: DEFAULT_SIDE,
                height: DEFAULT_SIDE,
                samples: DEFAULT_GRID,
                border_width: DEFAULT_BORDER_WIDTH,
                border_colour: Colour::BLACK,
                background: Colour::WHITE,
                max_folds: DEFAULT_FOLD_CAP,
                rotation: [0.0; 3],
                light: DEFAULT_LIGHT,
                shading: Shading::Phong,
                model: HyperbolicModel::Disk,
                fill,
            },
        }
    }

    /// Returns the width of the image in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// Sets the options of a render one at a time and checks them together.
///
/// Get one from [`RenderOptions::builder`]. Each setter takes any value, and
/// [`build`](RenderOptionsBuilder::build) reports the first value outside its
/// accepted range.
///
/// # Examples
///
/// ```
/// use escher_render::{Colour, ErrorKind, Fill, RenderOptions};
///
/// let builder = RenderOptions::builder(Fill::One(Colour::WHITE)).border_width(2.5).samples(2);
/// assert!(builder.clone().build().is_ok());
/// let kind = builder.size(0, 512).build().map_err(|error| error.kind());
/// assert_eq!(kind.err(), Some(ErrorKind::InvalidOption));
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct RenderOptionsBuilder {
    options: RenderOptions,
}

impl RenderOptionsBuilder {
    /// Sets the width and the height of the image in pixels.
    ///
    /// The width and the height must each be from 1 to 16384, and the image
    /// must have at most 67108864 pixels, which is 2 to the power 26.
    ///
    /// For a hyperbolic domain, the disk is centred in the image and spans the
    /// shorter side of the image apart from a margin of 1/32 of that side at
    /// each end, and the Klein disk has the same size. The half-plane and the
    /// band fill the image, and the height of the image spans 2 units of the
    /// model. For a Euclidean domain, the tiling fills the image with no
    /// margin, and the shorter side spans 8 units of the plane. For a
    /// spherical domain, the ball has the size and the place of the disk.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.options.width = width;
        self.options.height = height;
        self
    }

    /// Sets the number of samples along each axis of a pixel.
    ///
    /// Each pixel takes the mean colour, in linear light, of a regular grid
    /// of `grid` by `grid` samples. The grid must be from 1 to 16.
    pub fn samples(mut self, grid: u32) -> Self {
        self.options.samples = grid;
        self
    }

    /// Sets the width of the border of each tile, in pixels of the image.
    ///
    /// A sample takes the border colour when its estimated distance to a side
    /// of its tile is less than half this width. A sample whose side value is
    /// NaN, or whose rate of change across the image is not finite, also takes
    /// the border colour. The width must be a finite number from 0 to 1024. A
    /// width of 0 draws no border.
    pub fn border_width(mut self, pixels: f64) -> Self {
        self.options.border_width = pixels;
        self
    }

    /// Sets the colour of the borders.
    ///
    /// The samples whose fold did not converge take this colour too.
    pub fn border_colour(mut self, colour: Colour) -> Self {
        self.options.border_colour = colour;
        self
    }

    /// Sets the colour of the samples outside the view.
    ///
    /// For a hyperbolic domain these are the samples outside the region of its
    /// model, such as those outside the disk, and for
    /// a spherical domain those on or outside the rim of the ball. The
    /// background is never shaded. A Euclidean domain has no such sample.
    pub fn background(mut self, colour: Colour) -> Self {
        self.options.background = colour;
        self
    }

    /// Sets the fold cap, the largest number of steps that the fold of one
    /// sample may take.
    ///
    /// A sample that still lies outside the fundamental domain after that many
    /// steps counts as unconverged. The cap must be at most 100000. A cap of 0
    /// leaves every sample outside the fundamental domain unconverged.
    pub fn max_folds(mut self, cap: u32) -> Self {
        self.options.max_folds = cap;
        self
    }

    /// Sets the viewing rotation of the ball of a spherical domain, as three
    /// angles in radians.
    ///
    /// The rotation turns the ball, while the viewer and the light stay fixed.
    /// The yaw turns the front of the ball to the right, then the pitch turns
    /// it up, and then the roll turns the image counterclockwise, each about a
    /// fixed axis of the screen. Each angle must be finite. The rotation has no
    /// effect on a hyperbolic or Euclidean domain.
    pub fn rotation(mut self, yaw: f64, pitch: f64, roll: f64) -> Self {
        self.options.rotation = [yaw, pitch, roll];
        self
    }

    /// Sets the direction from the ball of a spherical domain to the light.
    ///
    /// The direction is in the coordinates of the screen, with the first axis
    /// to the right, the second axis up, and the third axis toward the viewer.
    /// Its length has no effect. Each component must be finite, and at least
    /// one must be other than zero. The light has no effect on a hyperbolic or
    /// Euclidean domain.
    pub fn light(mut self, direction: DVec3) -> Self {
        self.options.light = direction;
        self
    }

    /// Sets the shading of the ball of a spherical domain.
    ///
    /// The shading applies to every sample of the ball, including the border
    /// samples, and never to the background. It has no effect on a hyperbolic
    /// or Euclidean domain.
    pub fn shading(mut self, shading: Shading) -> Self {
        self.options.shading = shading;
        self
    }

    /// Sets the model of the hyperbolic plane in which a hyperbolic domain is
    /// drawn.
    ///
    /// Every model is accepted. Samples outside the region of the model, such
    /// as those outside the Klein disk, take the background colour, and so do
    /// samples far along a wide band or next to the real axis far from `i`,
    /// whose position in the Poincare disk rounds onto its edge. The model has
    /// no effect on a Euclidean or spherical domain.
    pub fn model(mut self, model: HyperbolicModel) -> Self {
        self.options.model = model;
        self
    }

    /// Checks the options and returns them.
    ///
    /// The options are checked in the order size, sample grid, border width,
    /// fold cap, rotation, and light, and the first value outside its accepted
    /// range is reported. The rotation and the light are checked for every
    /// domain, although only a spherical domain reads them.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidOption`](crate::ErrorKind::InvalidOption) for a
    /// width or height of 0, a sample grid of 0, a negative or non-finite
    /// border width, a rotation angle that is not finite, or a light direction
    /// that is the zero vector or has a component that is not finite, and of
    /// kind
    /// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) for a
    /// value above its limit.
    pub fn build(self) -> Result<RenderOptions> {
        let options = self.options;
        let (width, height) = (options.width, options.height);
        if width == 0 || height == 0 {
            return Err(Error::empty_size(width, height));
        }
        if width > LARGEST_SIDE || height > LARGEST_SIDE {
            return Err(Error::large_side(width, height));
        }
        if u64::from(width) * u64::from(height) > LARGEST_PIXELS {
            return Err(Error::many_pixels(width, height));
        }
        if options.samples == 0 {
            return Err(Error::empty_grid());
        }
        if options.samples > LARGEST_GRID {
            return Err(Error::large_grid(options.samples));
        }
        let border = options.border_width;
        if !border.is_finite() || border < 0.0 {
            return Err(Error::negative_border(border));
        }
        if border > LARGEST_BORDER_WIDTH {
            return Err(Error::wide_border(border));
        }
        if options.max_folds > LARGEST_FOLD_CAP {
            return Err(Error::large_cap(options.max_folds));
        }
        if let Some(&angle) = options.rotation.iter().find(|angle| !angle.is_finite()) {
            return Err(Error::rotation_angle(angle));
        }
        let light = options.light;
        let largest = light.abs().max_element();
        if !light.is_finite() || largest == 0.0 {
            return Err(Error::light_direction());
        }
        let scaled = light / largest;
        Ok(RenderOptions {
            light: scaled / scaled.length(),
            ..options
        })
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use super::{HyperbolicModel, RenderOptions, RenderOptionsBuilder, Shading};
    use crate::colour::{Colour, Fill};
    use crate::error::ErrorKind;

    fn builder() -> RenderOptionsBuilder {
        RenderOptions::builder(Fill::One(Colour::WHITE))
    }

    fn accepted(builder: RenderOptionsBuilder) -> RenderOptions {
        match builder.build() {
            Ok(options) => options,
            Err(error) => panic!("rejected: {error}"),
        }
    }

    fn rejected(builder: RenderOptionsBuilder, kind: ErrorKind, text: &str) {
        match builder.build() {
            Ok(options) => panic!("accepted {options:?}, expected {text}"),
            Err(error) => {
                assert_eq!(error.kind(), kind, "{error}");
                assert_eq!(error.to_string(), text);
            }
        }
    }

    #[test]
    fn builder_starts_from_the_defaults() {
        let options = accepted(builder());
        assert_eq!((options.width(), options.height()), (512, 512));
        assert_eq!(options.samples, 4);
        assert_eq!(options.border_width.to_bits(), 1.0_f64.to_bits());
        assert_eq!(options.border_colour, Colour::BLACK);
        assert_eq!(options.background, Colour::WHITE);
        assert_eq!(options.max_folds, 1500);
        assert_eq!(options.rotation, [0.0; 3]);
        let length = 6.0_f64.sqrt() / 2.0;
        assert_eq!(options.light, DVec3::new(-0.5, 0.5, 1.0) / length);
        assert_eq!(options.shading, Shading::Phong);
        assert_eq!(options.model, HyperbolicModel::Disk);
        assert_eq!(options.fill, Fill::One(Colour::WHITE));
    }

    #[test]
    fn rotation_accepts_finite_angles_and_rejects_the_first_angle_that_is_not_finite() {
        let options = accepted(builder().rotation(-1.0e300, 0.5, 7.0));
        assert_eq!(options.rotation, [-1.0e300, 0.5, 7.0]);
        for (value, shown) in [
            (f64::NAN, "NaN"),
            (f64::INFINITY, "inf"),
            (f64::NEG_INFINITY, "-inf"),
        ] {
            for slot in 0..3 {
                let mut angles = [0.25; 3];
                angles[slot] = value;
                let [yaw, pitch, roll] = angles;
                rejected(
                    builder().rotation(yaw, pitch, roll),
                    ErrorKind::InvalidOption,
                    &format!(
                        "cannot build render options: the rotation angle {shown} is not finite"
                    ),
                );
            }
        }
        rejected(
            builder().rotation(1.0, f64::INFINITY, f64::NAN),
            ErrorKind::InvalidOption,
            "cannot build render options: the rotation angle inf is not finite",
        );
    }

    #[test]
    fn light_rejects_a_component_that_is_not_finite_and_the_zero_direction() {
        let text = "cannot build render options: the light direction is zero or not finite";
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for slot in 0..3 {
                let mut components = [1.0; 3];
                components[slot] = value;
                rejected(
                    builder().light(DVec3::from_array(components)),
                    ErrorKind::InvalidOption,
                    text,
                );
            }
        }
        rejected(builder().light(DVec3::ZERO), ErrorKind::InvalidOption, text);
        rejected(
            builder().light(-DVec3::ZERO),
            ErrorKind::InvalidOption,
            text,
        );
    }

    #[test]
    fn light_is_normalized_without_overflow_or_underflow() {
        const LENGTH_BOUND: f64 = 9e-16;
        let pairs = [
            (DVec3::new(1.0e300, 1.0e300, 0.0), DVec3::new(1.0, 1.0, 0.0)),
            (DVec3::new(1.0e-310, 0.0, 0.0), DVec3::X),
            (DVec3::new(-4.0, 4.0, 8.0), DVec3::new(-1.0, 1.0, 2.0)),
        ];
        let mut largest = (0.0_f64, DVec3::ZERO);
        for (direction, plain) in pairs {
            let found = accepted(builder().light(direction)).light;
            let expected = accepted(builder().light(plain)).light;
            assert_eq!(
                found.to_array().map(f64::to_bits),
                expected.to_array().map(f64::to_bits)
            );
            let error = (found.length() - 1.0).abs();
            if error >= largest.0 {
                largest = (error, direction);
            }
        }
        println!(
            "normalized light: largest distance of the length from 1 {:e} for {:?}, bound {LENGTH_BOUND:e}, margin {:.1}",
            largest.0,
            largest.1,
            LENGTH_BOUND / largest.0
        );
        assert!(largest.0 <= LENGTH_BOUND, "{largest:?}");
        assert_eq!(
            accepted(builder().light(DVec3::new(1.0e-310, 0.0, 0.0))).light,
            DVec3::X
        );
    }

    #[test]
    fn shading_is_kept() {
        assert_eq!(
            accepted(builder().shading(Shading::Flat)).shading,
            Shading::Flat
        );
        assert_eq!(
            accepted(builder().shading(Shading::Phong)).shading,
            Shading::Phong
        );
    }

    #[test]
    fn model_is_kept() {
        for model in [
            HyperbolicModel::Disk,
            HyperbolicModel::HalfPlane,
            HyperbolicModel::Band,
            HyperbolicModel::Klein,
        ] {
            assert_eq!(accepted(builder().model(model)).model, model);
        }
    }

    #[test]
    fn size_accepts_its_limits_and_rejects_one_past_them() {
        for (width, height) in [(1, 1), (16384, 4096), (4096, 16384), (8192, 8192)] {
            let options = accepted(builder().size(width, height));
            assert_eq!((options.width(), options.height()), (width, height));
        }
        rejected(
            builder().size(0, 512),
            ErrorKind::InvalidOption,
            "cannot build render options: the image size 0x512 is empty",
        );
        rejected(
            builder().size(512, 0),
            ErrorKind::InvalidOption,
            "cannot build render options: the image size 512x0 is empty",
        );
        rejected(
            builder().size(16385, 512),
            ErrorKind::LimitExceeded,
            "cannot build render options: the image size 16385x512 is larger than 16384 pixels on a side",
        );
        rejected(
            builder().size(512, 16385),
            ErrorKind::LimitExceeded,
            "cannot build render options: the image size 512x16385 is larger than 16384 pixels on a side",
        );
        rejected(
            builder().size(16384, 4097),
            ErrorKind::LimitExceeded,
            "cannot build render options: the image size 16384x4097 has more than 67108864 pixels",
        );
        rejected(
            builder().size(16384, 8192),
            ErrorKind::LimitExceeded,
            "cannot build render options: the image size 16384x8192 has more than 67108864 pixels",
        );
    }

    #[test]
    fn samples_accept_their_limits_and_reject_one_past_them() {
        assert_eq!(accepted(builder().samples(1)).samples, 1);
        assert_eq!(accepted(builder().samples(16)).samples, 16);
        rejected(
            builder().samples(0),
            ErrorKind::InvalidOption,
            "cannot build render options: the sample grid 0 is empty",
        );
        rejected(
            builder().samples(17),
            ErrorKind::LimitExceeded,
            "cannot build render options: the sample grid 17 is larger than 16",
        );
    }

    #[test]
    fn border_width_accepts_its_limits_and_rejects_values_past_them() {
        assert_eq!(accepted(builder().border_width(0.0)).border_width, 0.0);
        assert_eq!(
            accepted(builder().border_width(1024.0)).border_width,
            1024.0
        );
        rejected(
            builder().border_width(-1.0),
            ErrorKind::InvalidOption,
            "cannot build render options: the border width -1 is negative or not finite",
        );
        rejected(
            builder().border_width(f64::NAN),
            ErrorKind::InvalidOption,
            "cannot build render options: the border width NaN is negative or not finite",
        );
        rejected(
            builder().border_width(f64::INFINITY),
            ErrorKind::InvalidOption,
            "cannot build render options: the border width inf is negative or not finite",
        );
        rejected(
            builder().border_width(1025.0),
            ErrorKind::LimitExceeded,
            "cannot build render options: the border width 1025 is larger than 1024",
        );
        rejected(
            builder().border_width(1024.5),
            ErrorKind::LimitExceeded,
            "cannot build render options: the border width 1024.5 is larger than 1024",
        );
    }

    #[test]
    fn max_folds_accepts_its_limits_and_rejects_one_past_them() {
        assert_eq!(accepted(builder().max_folds(0)).max_folds, 0);
        assert_eq!(accepted(builder().max_folds(100_000)).max_folds, 100_000);
        rejected(
            builder().max_folds(100_001),
            ErrorKind::LimitExceeded,
            "cannot build render options: the fold cap 100001 is larger than 100000",
        );
    }

    #[test]
    fn colours_and_fill_are_kept() {
        let red = Colour::from_srgb8(255, 0, 0);
        let fill = Fill::Parity([red, Colour::BLACK]);
        let options = accepted(
            RenderOptions::builder(fill.clone())
                .border_colour(red)
                .background(Colour::BLACK),
        );
        assert_eq!(options.fill, fill);
        assert_eq!(options.border_colour, red);
        assert_eq!(options.background, Colour::BLACK);
    }

    #[test]
    fn build_reports_the_first_failing_option_in_table_order() {
        rejected(
            builder()
                .size(0, 0)
                .samples(0)
                .border_width(f64::NAN)
                .max_folds(u32::MAX),
            ErrorKind::InvalidOption,
            "cannot build render options: the image size 0x0 is empty",
        );
        rejected(
            builder().samples(17).border_width(-1.0).max_folds(u32::MAX),
            ErrorKind::LimitExceeded,
            "cannot build render options: the sample grid 17 is larger than 16",
        );
        rejected(
            builder().border_width(2000.0).max_folds(u32::MAX),
            ErrorKind::LimitExceeded,
            "cannot build render options: the border width 2000 is larger than 1024",
        );
        rejected(
            builder()
                .max_folds(u32::MAX)
                .rotation(f64::NAN, 0.0, 0.0)
                .light(DVec3::ZERO),
            ErrorKind::LimitExceeded,
            "cannot build render options: the fold cap 4294967295 is larger than 100000",
        );
        rejected(
            builder().rotation(0.0, 0.0, f64::NAN).light(DVec3::ZERO),
            ErrorKind::InvalidOption,
            "cannot build render options: the rotation angle NaN is not finite",
        );
    }
}
