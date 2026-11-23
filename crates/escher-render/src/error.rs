use core::fmt;

use crate::options::{
    LARGEST_BORDER_WIDTH, LARGEST_FOLD_CAP, LARGEST_GRID, LARGEST_PIXELS, LARGEST_SIDE,
};

/// A specialized [`Result`](core::result::Result) whose error is [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// The reason why an operation failed. The operations are building options,
/// a coset fill, a renderer, or polygon masks, rendering rows or a mask, and
/// encoding or decoding an image.
///
/// Match on [`Error::kind`] to tell the causes apart. The
/// [`Display`](fmt::Display) text starts with `cannot `, names the operation
/// that failed, states the cause, and is meant for a person, not for
/// matching.
///
/// # Examples
///
/// ```
/// use escher_render::{Colour, ErrorKind, Fill, RenderOptions};
///
/// let error = RenderOptions::builder(Fill::One(Colour::WHITE)).samples(17).build().err();
/// assert_eq!(error.as_ref().map(|error| error.kind()), Some(ErrorKind::LimitExceeded));
/// assert_eq!(
///     error.map(|error| error.to_string()).as_deref(),
///     Some("cannot build render options: the sample grid 17 is larger than 16"),
/// );
/// ```
#[derive(Debug)]
pub struct Error {
    operation: &'static str,
    detail: Detail,
}

const BUILD: &str = "build render options";
const COSET_FILL: &str = "build coset fill";
const RENDERER: &str = "build renderer";
const RENDER: &str = "render rows";
const ENCODE: &str = "encode image";
pub(crate) const MASKS: &str = "build masks";
pub(crate) const RENDER_MASK: &str = "render mask";
pub(crate) const DECODE: &str = "decode image";

impl Error {
    pub(crate) fn empty_size(width: u32, height: u32) -> Self {
        Self::build(Detail::EmptySize { width, height })
    }

    pub(crate) fn empty_grid() -> Self {
        Self::build(Detail::EmptyGrid)
    }

    pub(crate) fn negative_border(width: f64) -> Self {
        Self::build(Detail::NegativeBorder(width))
    }

    pub(crate) fn large_side(width: u32, height: u32) -> Self {
        Self::build(Detail::LargeSide { width, height })
    }

    pub(crate) fn many_pixels(width: u32, height: u32) -> Self {
        Self::build(Detail::ManyPixels { width, height })
    }

    pub(crate) fn large_grid(grid: u32) -> Self {
        Self::build(Detail::LargeGrid(grid))
    }

    pub(crate) fn wide_border(width: f64) -> Self {
        Self::build(Detail::WideBorder(width))
    }

    pub(crate) fn large_cap(cap: u32) -> Self {
        Self::build(Detail::LargeCap(cap))
    }

    pub(crate) fn rotation_angle(angle: f64) -> Self {
        Self::build(Detail::RotationAngle(angle))
    }

    pub(crate) fn light_direction() -> Self {
        Self::build(Detail::LightDirection)
    }

    pub(crate) fn generator_count(generators: usize, sides: usize) -> Self {
        Self {
            operation: COSET_FILL,
            detail: Detail::GeneratorCount { generators, sides },
        }
    }

    pub(crate) fn no_presentation(source: escher_domain::Error) -> Self {
        Self {
            operation: COSET_FILL,
            detail: Detail::NoPresentation(source),
        }
    }

    pub(crate) fn broken_relator(relator: usize) -> Self {
        Self {
            operation: COSET_FILL,
            detail: Detail::BrokenRelator(relator),
        }
    }

    pub(crate) fn other_presentation() -> Self {
        Self {
            operation: RENDERER,
            detail: Detail::OtherPresentation,
        }
    }

    pub(crate) fn no_mask_polygon() -> Self {
        Self {
            operation: RENDERER,
            detail: Detail::NoMaskPolygon,
        }
    }

    pub(crate) fn colour_count(needed: usize, given: usize) -> Self {
        Self {
            operation: COSET_FILL,
            detail: Detail::ColourCount { needed, given },
        }
    }

    pub(crate) fn partial_rows(length: usize, row: usize) -> Self {
        Self {
            operation: RENDER,
            detail: Detail::PartialRows { length, row },
        }
    }

    pub(crate) fn rows_outside(first: u64, last: u64, height: u32) -> Self {
        Self {
            operation: RENDER,
            detail: Detail::RowsOutside {
                first,
                last,
                height,
            },
        }
    }

    pub(crate) fn wrong_image(length: usize, width: u32, height: u32) -> Self {
        Self {
            operation: ENCODE,
            detail: Detail::WrongImage {
                length,
                width,
                height,
            },
        }
    }

    pub(crate) fn encoding(source: png::EncodingError) -> Self {
        Self {
            operation: ENCODE,
            detail: Detail::Encoding(source),
        }
    }

    pub(crate) fn mask_colour(colour: u32, count: u32) -> Self {
        Self {
            operation: RENDER_MASK,
            detail: Detail::MaskColour { colour, count },
        }
    }

    pub(crate) fn sixteen_bits() -> Self {
        Self::decode(Detail::SixteenBits)
    }

    pub(crate) fn decoder_limit(source: png::DecodingError) -> Self {
        Self::decode(Detail::DecoderLimit(source))
    }

    pub(crate) fn unreadable(source: png::DecodingError) -> Self {
        Self::decode(Detail::Unreadable(source))
    }

    pub(crate) fn not_png(source: png::DecodingError) -> Self {
        Self::decode(Detail::NotPng(source))
    }

    pub(crate) fn during(self, operation: &'static str) -> Self {
        Self { operation, ..self }
    }

    fn decode(detail: Detail) -> Self {
        Self {
            operation: DECODE,
            detail,
        }
    }

    fn build(detail: Detail) -> Self {
        Self {
            operation: BUILD,
            detail,
        }
    }

    /// Returns the cause of the error as an [`ErrorKind`].
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self.detail {
            Detail::EmptySize { .. }
            | Detail::EmptyGrid
            | Detail::NegativeBorder(_)
            | Detail::RotationAngle(_)
            | Detail::LightDirection
            | Detail::NoPresentation(_)
            | Detail::GeneratorCount { .. }
            | Detail::BrokenRelator(_)
            | Detail::ColourCount { .. }
            | Detail::OtherPresentation
            | Detail::NoMaskPolygon
            | Detail::MaskColour { .. } => ErrorKind::InvalidOption,
            Detail::LargeSide { .. }
            | Detail::ManyPixels { .. }
            | Detail::LargeGrid(_)
            | Detail::WideBorder(_)
            | Detail::LargeCap(_)
            | Detail::DecoderLimit(_) => ErrorKind::LimitExceeded,
            Detail::PartialRows { .. } | Detail::RowsOutside { .. } | Detail::WrongImage { .. } => {
                ErrorKind::InvalidBuffer
            }
            Detail::Encoding(_) => ErrorKind::Encoding,
            Detail::SixteenBits | Detail::Unreadable(_) | Detail::NotPng(_) => ErrorKind::Decoding,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cannot {}: {}", self.operation, self.detail)
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.detail {
            Detail::Encoding(source) => Some(source),
            Detail::NoPresentation(source) => Some(source),
            Detail::DecoderLimit(source) | Detail::Unreadable(source) | Detail::NotPng(source) => {
                Some(source)
            }
            Detail::EmptySize { .. }
            | Detail::EmptyGrid
            | Detail::NegativeBorder(_)
            | Detail::LargeSide { .. }
            | Detail::ManyPixels { .. }
            | Detail::LargeGrid(_)
            | Detail::WideBorder(_)
            | Detail::LargeCap(_)
            | Detail::RotationAngle(_)
            | Detail::LightDirection
            | Detail::GeneratorCount { .. }
            | Detail::BrokenRelator(_)
            | Detail::ColourCount { .. }
            | Detail::OtherPresentation
            | Detail::NoMaskPolygon
            | Detail::PartialRows { .. }
            | Detail::RowsOutside { .. }
            | Detail::WrongImage { .. }
            | Detail::MaskColour { .. }
            | Detail::SixteenBits => None,
        }
    }
}

/// The cause of an [`Error`], for a caller to match on.
///
/// The enum is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
///
/// # Examples
///
/// ```
/// use escher_render::{ErrorKind, encode_png};
///
/// let kind = encode_png(16, 32, &[0; 1535]).map_err(|error| error.kind());
/// assert_eq!(kind.err(), Some(ErrorKind::InvalidBuffer));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An option has a value that means nothing, such as an empty image, an
    /// empty sample grid, a negative or non-finite border width, a rotation
    /// angle that is not finite, a light direction that is zero or not
    /// finite, a coset fill or polygon masks whose colouring does not fit the
    /// domain, a coset fill whose paints do not fit the colouring, a coset
    /// fill used with a domain of another side presentation, polygon masks or
    /// an image fill for a domain without a mask polygon, or a mask of a
    /// colour that the colouring does not have.
    InvalidOption,
    /// An option or an image is larger than its limit, which bounds the
    /// memory and the time of a render, or the PNG decoder reached its memory
    /// limit.
    LimitExceeded,
    /// A buffer does not fit the image: its length does not hold whole rows
    /// or the whole image, or its rows lie outside the image.
    InvalidBuffer,
    /// The PNG encoder failed, and [`source`](core::error::Error::source)
    /// returns its error.
    Encoding,
    /// An image cannot be decoded: the stream cannot be read, is not a valid
    /// PNG image, or has 16 bits per channel. When the PNG decoder reported
    /// the cause, [`source`](core::error::Error::source) returns its error.
    Decoding,
}

#[derive(Debug, thiserror::Error)]
enum Detail {
    #[error("the image size {width}x{height} is empty")]
    EmptySize { width: u32, height: u32 },
    #[error("the sample grid 0 is empty")]
    EmptyGrid,
    #[error("the border width {0} is negative or not finite")]
    NegativeBorder(f64),
    #[error("the image size {width}x{height} is larger than {LARGEST_SIDE} pixels on a side")]
    LargeSide { width: u32, height: u32 },
    #[error("the image size {width}x{height} has more than {LARGEST_PIXELS} pixels")]
    ManyPixels { width: u32, height: u32 },
    #[error("the sample grid {0} is larger than {LARGEST_GRID}")]
    LargeGrid(u32),
    #[error("the border width {0} is larger than {LARGEST_BORDER_WIDTH}")]
    WideBorder(f64),
    #[error("the fold cap {0} is larger than {LARGEST_FOLD_CAP}")]
    LargeCap(u32),
    #[error("the rotation angle {0} is not finite")]
    RotationAngle(f64),
    #[error("the light direction is zero or not finite")]
    LightDirection,
    #[error("the domain has no side presentation")]
    NoPresentation(escher_domain::Error),
    #[error(
        "the colouring has {}, and the domain has {}",
        counted(*generators, "generator", "generators"),
        counted(*sides, "side", "sides")
    )]
    GeneratorCount { generators: usize, sides: usize },
    #[error(
        "the colouring needs {}, and {given} {} given",
        counted(*needed, "colour", "colours"),
        if *given == 1 { "was" } else { "were" }
    )]
    ColourCount { needed: usize, given: usize },
    #[error("relator {0} of the side presentation of the domain moves a coset of the colouring")]
    BrokenRelator(usize),
    #[error("the coset fill was built for a domain with another side presentation")]
    OtherPresentation,
    #[error("the domain has no mask polygon")]
    NoMaskPolygon,
    #[error("the buffer of {length} bytes does not hold whole rows of {row} bytes")]
    PartialRows { length: usize, row: usize },
    #[error("the rows {first} to {last} are outside the image of {height} rows")]
    RowsOutside { first: u64, last: u64, height: u32 },
    #[error("the buffer of {length} bytes does not hold an image of {width}x{height} pixels")]
    WrongImage {
        length: usize,
        width: u32,
        height: u32,
    },
    #[error("the PNG encoder failed")]
    Encoding(png::EncodingError),
    #[error(
        "the colour {colour} is not below the {} of the colouring",
        counted(usize::try_from(*count).unwrap_or(usize::MAX), "colour", "colours")
    )]
    MaskColour { colour: u32, count: u32 },
    #[error("the image has 16 bits per channel, and at most 8 are supported")]
    SixteenBits,
    #[error("the PNG decoder reached its memory limit")]
    DecoderLimit(png::DecodingError),
    #[error("the image cannot be read")]
    Unreadable(png::DecodingError),
    #[error("the data is not a valid PNG image")]
    NotPng(png::DecodingError),
}

fn counted(count: usize, one: &str, many: &str) -> String {
    let word = if count == 1 { one } else { many };
    format!("{count} {word}")
}

#[cfg(test)]
mod tests {
    use super::{DECODE, Error, ErrorKind, MASKS, RENDER_MASK};

    fn png_error() -> png::EncodingError {
        png::EncodingError::LimitsExceeded
    }

    fn errors() -> Vec<(Error, ErrorKind, &'static str)> {
        let mut errors = option_errors();
        errors.extend(coset_errors());
        errors.extend(buffer_errors());
        errors.extend(mask_errors());
        errors.extend(decode_errors());
        errors
    }

    fn mask_errors() -> Vec<(Error, ErrorKind, &'static str)> {
        vec![
            (
                Error::generator_count(3, 4).during(MASKS),
                ErrorKind::InvalidOption,
                "cannot build masks: the colouring has 3 generators, and the domain has 4 sides",
            ),
            (
                Error::broken_relator(3).during(MASKS),
                ErrorKind::InvalidOption,
                "cannot build masks: relator 3 of the side presentation of the domain moves a coset of the colouring",
            ),
            (
                Error::no_mask_polygon().during(MASKS),
                ErrorKind::InvalidOption,
                "cannot build masks: the domain has no mask polygon",
            ),
            (
                Error::mask_colour(4, 4),
                ErrorKind::InvalidOption,
                "cannot render mask: the colour 4 is not below the 4 colours of the colouring",
            ),
            (
                Error::mask_colour(1, 1),
                ErrorKind::InvalidOption,
                "cannot render mask: the colour 1 is not below the 1 colour of the colouring",
            ),
            (
                Error::empty_size(0, 512).during(RENDER_MASK),
                ErrorKind::InvalidOption,
                "cannot render mask: the image size 0x512 is empty",
            ),
            (
                Error::large_side(16385, 512).during(RENDER_MASK),
                ErrorKind::LimitExceeded,
                "cannot render mask: the image size 16385x512 is larger than 16384 pixels on a side",
            ),
        ]
    }

    fn decoding_error() -> png::DecodingError {
        png::DecodingError::IoError(std::io::ErrorKind::UnexpectedEof.into())
    }

    fn decode_errors() -> Vec<(Error, ErrorKind, &'static str)> {
        vec![
            (
                Error::large_side(16385, 512).during(DECODE),
                ErrorKind::LimitExceeded,
                "cannot decode image: the image size 16385x512 is larger than 16384 pixels on a side",
            ),
            (
                Error::many_pixels(16384, 8192).during(DECODE),
                ErrorKind::LimitExceeded,
                "cannot decode image: the image size 16384x8192 has more than 67108864 pixels",
            ),
            (
                Error::sixteen_bits(),
                ErrorKind::Decoding,
                "cannot decode image: the image has 16 bits per channel, and at most 8 are supported",
            ),
            (
                Error::decoder_limit(decoding_error()),
                ErrorKind::LimitExceeded,
                "cannot decode image: the PNG decoder reached its memory limit",
            ),
            (
                Error::unreadable(decoding_error()),
                ErrorKind::Decoding,
                "cannot decode image: the image cannot be read",
            ),
            (
                Error::not_png(decoding_error()),
                ErrorKind::Decoding,
                "cannot decode image: the data is not a valid PNG image",
            ),
        ]
    }

    fn coset_errors() -> Vec<(Error, ErrorKind, &'static str)> {
        vec![
            (
                Error::generator_count(3, 4),
                ErrorKind::InvalidOption,
                "cannot build coset fill: the colouring has 3 generators, and the domain has 4 sides",
            ),
            (
                Error::generator_count(1, 2),
                ErrorKind::InvalidOption,
                "cannot build coset fill: the colouring has 1 generator, and the domain has 2 sides",
            ),
            (
                Error::colour_count(3, 2),
                ErrorKind::InvalidOption,
                "cannot build coset fill: the colouring needs 3 colours, and 2 were given",
            ),
            (
                Error::colour_count(1, 1),
                ErrorKind::InvalidOption,
                "cannot build coset fill: the colouring needs 1 colour, and 1 was given",
            ),
            (
                Error::broken_relator(3),
                ErrorKind::InvalidOption,
                "cannot build coset fill: relator 3 of the side presentation of the domain moves a coset of the colouring",
            ),
            (
                Error::other_presentation(),
                ErrorKind::InvalidOption,
                "cannot build renderer: the coset fill was built for a domain with another side presentation",
            ),
            (
                Error::no_mask_polygon(),
                ErrorKind::InvalidOption,
                "cannot build renderer: the domain has no mask polygon",
            ),
        ]
    }

    fn option_errors() -> Vec<(Error, ErrorKind, &'static str)> {
        vec![
            (
                Error::empty_size(0, 512),
                ErrorKind::InvalidOption,
                "cannot build render options: the image size 0x512 is empty",
            ),
            (
                Error::empty_grid(),
                ErrorKind::InvalidOption,
                "cannot build render options: the sample grid 0 is empty",
            ),
            (
                Error::negative_border(-1.0),
                ErrorKind::InvalidOption,
                "cannot build render options: the border width -1 is negative or not finite",
            ),
            (
                Error::large_side(16385, 512),
                ErrorKind::LimitExceeded,
                "cannot build render options: the image size 16385x512 is larger than 16384 pixels on a side",
            ),
            (
                Error::many_pixels(16384, 8192),
                ErrorKind::LimitExceeded,
                "cannot build render options: the image size 16384x8192 has more than 67108864 pixels",
            ),
            (
                Error::large_grid(17),
                ErrorKind::LimitExceeded,
                "cannot build render options: the sample grid 17 is larger than 16",
            ),
            (
                Error::wide_border(1025.0),
                ErrorKind::LimitExceeded,
                "cannot build render options: the border width 1025 is larger than 1024",
            ),
            (
                Error::large_cap(100_001),
                ErrorKind::LimitExceeded,
                "cannot build render options: the fold cap 100001 is larger than 100000",
            ),
            (
                Error::rotation_angle(f64::INFINITY),
                ErrorKind::InvalidOption,
                "cannot build render options: the rotation angle inf is not finite",
            ),
            (
                Error::rotation_angle(f64::NEG_INFINITY),
                ErrorKind::InvalidOption,
                "cannot build render options: the rotation angle -inf is not finite",
            ),
            (
                Error::light_direction(),
                ErrorKind::InvalidOption,
                "cannot build render options: the light direction is zero or not finite",
            ),
        ]
    }

    fn buffer_errors() -> Vec<(Error, ErrorKind, &'static str)> {
        vec![
            (
                Error::partial_rows(1535, 1536),
                ErrorKind::InvalidBuffer,
                "cannot render rows: the buffer of 1535 bytes does not hold whole rows of 1536 bytes",
            ),
            (
                Error::rows_outside(510, 513, 512),
                ErrorKind::InvalidBuffer,
                "cannot render rows: the rows 510 to 513 are outside the image of 512 rows",
            ),
            (
                Error::wrong_image(1535, 16, 32),
                ErrorKind::InvalidBuffer,
                "cannot encode image: the buffer of 1535 bytes does not hold an image of 16x32 pixels",
            ),
            (
                Error::encoding(png_error()),
                ErrorKind::Encoding,
                "cannot encode image: the PNG encoder failed",
            ),
        ]
    }

    #[test]
    fn display_of_each_cause_is_the_documented_text() {
        for (error, _, text) in errors() {
            assert_eq!(error.to_string(), text);
        }
    }

    #[test]
    fn display_of_each_cause_is_lower_case_apart_from_png_without_trailing_punctuation() {
        for (error, _, _) in errors() {
            let text = error.to_string();
            let rest = text.replace("PNG", "");
            assert_eq!(rest, rest.to_lowercase(), "{text}");
            assert!(!text.ends_with('.'), "{text}");
        }
    }

    #[test]
    fn kind_matches_each_cause() {
        for (error, kind, text) in errors() {
            assert_eq!(error.kind(), kind, "{text}");
        }
    }

    #[test]
    fn source_is_the_png_error_only_for_an_error_that_png_reported() {
        for (error, kind, text) in errors() {
            let source = core::error::Error::source(&error).map(ToString::to_string);
            if kind == ErrorKind::Encoding {
                assert_eq!(source, Some(png_error().to_string()), "{text}");
            } else if text.contains("PNG") || text.contains("cannot be read") {
                assert_eq!(source, Some(decoding_error().to_string()), "{text}");
            } else {
                assert_eq!(source, None, "{text}");
            }
        }
    }

    #[test]
    fn a_domain_without_a_side_presentation_keeps_the_domain_error_as_its_source() {
        let signature: escher_signature::Signature = match "4*3".parse() {
            Ok(signature) => signature,
            Err(error) => panic!("{error}"),
        };
        let Err(cause) = escher_domain::FundamentalDomain::build(&signature) else {
            panic!("4*3 has a domain");
        };
        let cause_text = cause.to_string();
        let error = Error::no_presentation(cause);
        assert_eq!(error.kind(), ErrorKind::InvalidOption);
        assert_eq!(
            error.to_string(),
            "cannot build coset fill: the domain has no side presentation"
        );
        let source = core::error::Error::source(&error).map(ToString::to_string);
        assert_eq!(source, Some(cause_text));
    }
}
