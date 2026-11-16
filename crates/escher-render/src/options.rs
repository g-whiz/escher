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

/// The validated options of a render: the image size, the sample grid, the
/// border, the colours, and the fold cap.
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
    pub(crate) fill: Fill,
}

impl RenderOptions {
    /// Returns a builder that starts from the default options with the
    /// given fill.
    ///
    /// The defaults are an image of 512 by 512 pixels, a grid of 4 by 4
    /// samples in each pixel, a border 1 pixel wide in
    /// [`Colour::BLACK`], a [`Colour::WHITE`] background outside the disk,
    /// and a fold cap of 1500 steps.
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
    /// must have at most 67108864 pixels, which is 2 to the power 26. The disk
    /// is centred in the image and spans the shorter side of the image apart
    /// from a margin of 1/32 of that side at each end.
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

    /// Sets the colour of the image outside the disk.
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

    /// Checks the options and returns them.
    ///
    /// The options are checked in the order size, sample grid, border width,
    /// and fold cap, and the first value outside its accepted range is
    /// reported.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidOption`](crate::ErrorKind::InvalidOption) for a
    /// width or height of 0, a sample grid of 0, or a negative or non-finite
    /// border width, and of kind
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
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderOptions, RenderOptionsBuilder};
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
        assert_eq!(options.fill, Fill::One(Colour::WHITE));
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
            RenderOptions::builder(fill)
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
    }
}
