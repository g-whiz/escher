use core::fmt;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

use glam::DVec2;

use crate::error::{DECODE, Error, Result};
use crate::options::check_size;
use crate::view::MARGIN;

const TEXEL_BYTES: usize = 4;

/// The most bytes that the PNG decoder may allocate besides the texels, 16
/// MiB.
///
/// The decoder allocates no texels, which the crate allocates after its own
/// size check. It allocates the buffer of each chunk that it reads, at most
/// 1024 bytes for a palette and its transparency table, and one output row,
/// at most 4 bytes for each of the 16384 pixels of the widest accepted image.
/// The bound leaves room far beyond both for the one metadata chunk that the
/// decoder keeps, the Exif chunk. The decoder doubles the buffer of a chunk
/// as it reads it, so every image within the size limits fits with an Exif
/// chunk of at most 8388608 bytes, and no image fits with an Exif chunk of
/// more than 16777216 bytes. It skips text and colour profile chunks without
/// storing them, so they count against no bound.
const DECODER_BYTES: usize = 1 << 24;

/// A decoded image of 8-bit sRGB texels with an 8-bit alpha, which a
/// [`Paint::Image`](crate::Paint::Image) shows.
///
/// Build it with [`Image::decode_png`]. It has at least 1 by 1 and at most
/// 16384 by 16384 texels. Its [`Debug`](fmt::Debug) output shows only its
/// size.
///
/// # Examples
///
/// ```
/// use escher_render::{Image, encode_png};
///
/// let file = encode_png(2, 1, &[255, 0, 0, 0, 0, 255])?;
/// let image = Image::decode_png(file.as_slice())?;
/// assert_eq!((image.width(), image.height()), (2, 1));
/// # Ok::<(), escher_render::Error>(())
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Image {
    width: u32,
    height: u32,
    texels: Vec<u8>,
}

impl fmt::Debug for Image {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

impl Image {
    /// Decodes one PNG image from a reader.
    ///
    /// It accepts every colour type of PNG at a bit depth of 8 or less,
    /// interlaced or not: greyscale, truecolour, and indexed colour, each with
    /// or without a transparency chunk, and greyscale and truecolour with
    /// alpha. It reads the samples as sRGB whatever colour chunk the file
    /// holds, and reads only the default image of an animated PNG. Indexed
    /// colour and bit depths below 8 are expanded to 8-bit channels,
    /// greyscale fills the three channels, the transparency chunk becomes the
    /// alpha, and an image without alpha is opaque.
    ///
    /// The image has the size limits of a rendered image: at most 16384 pixels
    /// on a side and at most 67108864 pixels, which is 2 to the power 26. The
    /// decoder reads the width, the height, and the bit depth from the header
    /// and checks them before it allocates the texels, so a header that names
    /// a large image costs no memory. It also limits the memory that the PNG
    /// decoder allocates besides the texels to 16777216 bytes, which every
    /// image within the size limits fits with an Exif chunk of at most
    /// 8388608 bytes. It skips text and colour profile chunks of any length.
    /// It reads the stream to its end, and the time it takes grows with the
    /// length of the stream, which it does not limit.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when the
    /// width or the height is above 16384, when the image has more than 2^26
    /// pixels, and when the PNG decoder reaches its memory limit, which an
    /// Exif chunk of more than 16777216 bytes always does and one of more
    /// than 8388608 bytes can do. Returns an error of kind
    /// [`ErrorKind::Decoding`](crate::ErrorKind::Decoding) for an image with 16
    /// bits per channel, a reader that returns an error, and a stream that
    /// is not a valid PNG image, such as one that ends early.
    pub fn decode_png(mut reader: impl Read) -> Result<Self> {
        decode(&mut reader)
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

    #[expect(
        clippy::expect_used,
        reason = "the size check bounds the width by 16384, and the sampling clamps each index to the texels of the image, which holds four bytes for each of them"
    )]
    fn texel(&self, column: usize, row: usize) -> [u8; 4] {
        let width = usize::try_from(self.width)
            .expect("internal error: the width of a checked size fits a usize");
        let start = (row * width + column) * TEXEL_BYTES;
        self.texels
            .get(start..start + TEXEL_BYTES)
            .and_then(|texel| texel.try_into().ok())
            .expect("internal error: a clamped texel index lies inside the image")
    }
}

fn decode(reader: &mut dyn Read) -> Result<Image> {
    let mut stream = Stream {
        reader: BufReader::new(reader),
        failed: false,
    };
    let result = decode_stream(&mut stream);
    let failed = stream.failed;
    result.map_err(|failure| match failure {
        Failure::Png(source @ png::DecodingError::LimitsExceeded) => Error::decoder_limit(source),
        Failure::Png(source @ png::DecodingError::IoError(_)) if failed => {
            Error::unreadable(source)
        }
        Failure::Png(source) => Error::not_png(source),
        Failure::Crate(error) => error,
    })
}

enum Failure {
    Png(png::DecodingError),
    Crate(Error),
}

impl From<png::DecodingError> for Failure {
    fn from(source: png::DecodingError) -> Self {
        Self::Png(source)
    }
}

#[expect(
    clippy::expect_used,
    reason = "the size check allows at most 2 to the power 26 pixels, which a usize of 32 bits or more holds"
)]
fn decode_stream(stream: &mut Stream<'_>) -> core::result::Result<Image, Failure> {
    let mut decoder = png::Decoder::new(stream);
    decoder.set_limits(png::Limits {
        bytes: DECODER_BYTES,
    });
    decoder.set_ignore_text_chunk(true);
    decoder.set_ignore_iccp_chunk(true);
    decoder.set_transformations(png::Transformations::EXPAND);
    let header = decoder.read_header_info()?;
    let (width, height) = (header.width, header.height);
    check_size(width, height).map_err(|error| Failure::Crate(error.during(DECODE)))?;
    if header.bit_depth == png::BitDepth::Sixteen {
        return Err(Failure::Crate(Error::sixteen_bits()));
    }
    let mut reader = decoder.read_info()?;
    let pixels = usize::try_from(u64::from(width) * u64::from(height))
        .expect("internal error: the pixel count of a checked size fits a usize");
    let mut texels = vec![0; pixels * TEXEL_BYTES];
    let output = reader.next_frame(&mut texels)?;
    reader.finish()?;
    expand(&mut texels, pixels, output.color_type.samples());
    Ok(Image {
        width,
        height,
        texels,
    })
}

fn expand(texels: &mut [u8], pixels: usize, channels: usize) {
    for index in (0..pixels).rev() {
        let source = index * channels;
        let mut texel = [0, 0, 0, u8::MAX];
        if let Some(read) = texels.get(source..source + channels) {
            match *read {
                [grey] => texel = [grey, grey, grey, u8::MAX],
                [grey, alpha] => texel = [grey, grey, grey, alpha],
                [red, green, blue] => texel = [red, green, blue, u8::MAX],
                [red, green, blue, alpha] => texel = [red, green, blue, alpha],
                _ => {}
            }
        }
        let target = index * TEXEL_BYTES;
        if let Some(write) = texels.get_mut(target..target + TEXEL_BYTES) {
            write.copy_from_slice(&texel);
        }
    }
}

struct Stream<'reader> {
    reader: BufReader<&'reader mut dyn Read>,
    failed: bool,
}

impl Read for Stream<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let result = self.reader.read(buffer);
        self.failed |= result.is_err();
        result
    }
}

impl BufRead for Stream<'_> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self.reader.fill_buf() {
            Ok(buffer) => Ok(buffer),
            Err(error) => {
                self.failed = true;
                Err(error)
            }
        }
    }

    fn consume(&mut self, amount: usize) {
        self.reader.consume(amount);
    }
}

impl Seek for Stream<'_> {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        self.failed = true;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "the stream of an image cannot seek",
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placement {
    scale: f64,
    centre: DVec2,
    size: DVec2,
}

impl Placement {
    pub(crate) fn new(polygon: &[DVec2], width: u32, height: u32) -> Self {
        let (low, high) = polygon.iter().fold(
            (DVec2::INFINITY, DVec2::NEG_INFINITY),
            |(low, high), &vertex| (low.min(vertex), high.max(vertex)),
        );
        let size = DVec2::new(f64::from(width), f64::from(height));
        let margin = 2.0 * MARGIN * size.min_element();
        let extent = high - low;
        Self {
            scale: ((size.x - margin) / extent.x).min((size.y - margin) / extent.y),
            centre: (low + high) / 2.0,
            size,
        }
    }

    pub(crate) fn arrangement(polygons: &[Vec<DVec2>], width: u32, height: u32) -> Vec<Self> {
        let placements: Vec<Self> = polygons
            .iter()
            .map(|polygon| Self::new(polygon, width, height))
            .collect();
        let scale = placements
            .iter()
            .map(|placement| placement.scale)
            .fold(f64::INFINITY, f64::min);
        placements
            .into_iter()
            .map(|placement| Self { scale, ..placement })
            .collect()
    }

    pub(crate) fn scale(&self) -> f64 {
        self.scale
    }

    pub(crate) fn position(&self, point: DVec2) -> DVec2 {
        DVec2::new(
            self.size.x / 2.0 + self.scale * (point.x - self.centre.x),
            self.size.y / 2.0 - self.scale * (point.y - self.centre.y),
        )
    }

    pub(crate) fn point(&self, position: DVec2) -> DVec2 {
        DVec2::new(
            self.centre.x + (position.x - self.size.x / 2.0) / self.scale,
            self.centre.y - (position.y - self.size.y / 2.0) / self.scale,
        )
    }

    pub(crate) fn inside(&self, position: DVec2) -> Option<DVec2> {
        let inside = position.is_finite()
            && (0.0..=self.size.x).contains(&position.x)
            && (0.0..=self.size.y).contains(&position.y);
        inside.then_some(position)
    }
}

#[expect(
    clippy::expect_used,
    reason = "each index is clamped to the range from 0 to the last texel of a side of at most 16384 texels"
)]
pub(crate) fn sample(
    image: &Image,
    position: DVec2,
    background: [f64; 3],
    linear: &[f64; 256],
) -> [f64; 3] {
    let corner = |coordinate: f64, size: u32| {
        let shifted = coordinate - 0.5;
        let floor = shifted.floor();
        let last = i64::from(size) - 1;
        let first = to_index(floor).clamp(0, last);
        let second = (to_index(floor) + 1).clamp(0, last);
        let index = |value: i64| {
            usize::try_from(value).expect("internal error: a clamped texel index fits a usize")
        };
        (index(first), index(second), shifted - floor)
    };
    let (left, right, across) = corner(position.x, image.width);
    let (top, bottom, down) = corner(position.y, image.height);
    let texel = |column: usize, row: usize| {
        let [red, green, blue, alpha] = image.texel(column, row);
        let opacity = f64::from(alpha) / 255.0;
        let mut value = [0.0; 3];
        for ((channel, byte), behind) in value.iter_mut().zip([red, green, blue]).zip(background) {
            *channel = opacity * linear[usize::from(byte)] + (1.0 - opacity) * behind;
        }
        value
    };
    let weights = [
        ((1.0 - across) * (1.0 - down), texel(left, top)),
        (across * (1.0 - down), texel(right, top)),
        ((1.0 - across) * down, texel(left, bottom)),
        (across * down, texel(right, bottom)),
    ];
    let mut value = [0.0; 3];
    for (weight, texel) in weights {
        for (channel, part) in value.iter_mut().zip(texel) {
            *channel += weight * part;
        }
    }
    value
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "the caller passes the floor of a position inside an image of at most 16384 pixels on a side, which an i64 holds exactly"
)]
fn to_index(floor: f64) -> i64 {
    floor as i64
}

#[cfg(test)]
mod tests {
    use glam::DVec2;

    use super::{Image, Placement, sample};
    use crate::colour::{Colour, decode_table};
    use crate::encode::encode_png;

    const WIDTH: u32 = 5;
    const HEIGHT: u32 = 3;
    const PIXELS: usize = 15;

    struct Encoded {
        colour: png::ColorType,
        depth: png::BitDepth,
        samples: Vec<Vec<u8>>,
        palette: Option<Vec<u8>>,
        transparency: Option<Vec<u8>>,
    }

    fn bits(depth: png::BitDepth) -> u8 {
        match depth {
            png::BitDepth::One => 1,
            png::BitDepth::Two => 2,
            png::BitDepth::Four => 4,
            png::BitDepth::Eight => 8,
            png::BitDepth::Sixteen => 16,
        }
    }

    fn packed_row(pixels: &[Vec<u8>], depth: u8) -> Vec<u8> {
        let mut row = Vec::new();
        let mut filled = 0_u8;
        for sample in pixels.iter().flatten() {
            if filled == 0 {
                row.push(0);
            }
            if let Some(last) = row.last_mut() {
                *last |= sample << (8 - depth - filled);
            }
            filled = (filled + depth) % 8;
        }
        row
    }

    fn packed(samples: &[Vec<u8>], width: usize, depth: u8) -> Vec<u8> {
        samples
            .chunks(width)
            .flat_map(|row| packed_row(row, depth))
            .collect()
    }

    const ADAM7: [[usize; 4]; 7] = [
        [0, 0, 8, 8],
        [4, 0, 8, 8],
        [0, 4, 4, 8],
        [2, 0, 4, 4],
        [0, 2, 2, 4],
        [1, 0, 2, 2],
        [0, 1, 1, 2],
    ];

    fn interlaced(samples: &[Vec<u8>], width: usize, depth: u8) -> Vec<u8> {
        let height = samples.len() / width;
        let mut data = Vec::new();
        for [left, top, across, down] in ADAM7 {
            let columns: Vec<usize> = (left..width).step_by(across).collect();
            if columns.is_empty() {
                continue;
            }
            for row in (top..height).step_by(down) {
                let pixels: Vec<Vec<u8>> = columns
                    .iter()
                    .map(|&column| samples[row * width + column].clone())
                    .collect();
                data.push(0);
                data.extend(packed_row(&pixels, depth));
            }
        }
        data
    }

    fn stored_zlib(data: &[u8]) -> Vec<u8> {
        let mut stream = vec![0x78, 0x01];
        let blocks: Vec<&[u8]> = data.chunks(0xffff).collect();
        for (index, block) in blocks.iter().enumerate() {
            stream.push(u8::from(index + 1 == blocks.len()));
            let length = u16::try_from(block.len()).unwrap_or(u16::MAX);
            stream.extend(length.to_le_bytes());
            stream.extend((!length).to_le_bytes());
            stream.extend_from_slice(block);
        }
        let (mut first, mut second) = (1_u32, 0_u32);
        for &byte in data {
            first = (first + u32::from(byte)) % 65521;
            second = (second + first) % 65521;
        }
        stream.extend(((second << 16) | first).to_be_bytes());
        stream
    }

    fn encode(image: &Encoded, interlace: bool) -> Vec<u8> {
        let mut file = Vec::new();
        let mut info = png::Info::with_size(WIDTH, HEIGHT);
        info.color_type = image.colour;
        info.bit_depth = image.depth;
        info.interlaced = interlace;
        let mut encoder = match png::Encoder::with_info(&mut file, info) {
            Ok(encoder) => encoder,
            Err(error) => panic!("{error}"),
        };
        if let Some(palette) = &image.palette {
            encoder.set_palette(palette.clone());
        }
        if let Some(transparency) = &image.transparency {
            encoder.set_trns(transparency.clone());
        }
        let mut writer = match encoder.write_header() {
            Ok(writer) => writer,
            Err(error) => panic!("{error}"),
        };
        let width = usize::try_from(WIDTH).unwrap_or(usize::MAX);
        let depth = bits(image.depth);
        let result = if interlace {
            let data = stored_zlib(&interlaced(&image.samples, width, depth));
            writer.write_chunk(png::chunk::IDAT, &data)
        } else {
            writer.write_image_data(&packed(&image.samples, width, depth))
        };
        if let Err(error) = result.and_then(|()| writer.finish()) {
            panic!("{error}");
        }
        file
    }

    fn decoded(file: &[u8]) -> Image {
        match Image::decode_png(file) {
            Ok(image) => image,
            Err(error) => panic!("{error}"),
        }
    }

    fn grey(depth: png::BitDepth) -> Vec<Vec<u8>> {
        let levels = 1_usize << bits(depth);
        (0..PIXELS)
            .map(|index| vec![u8::try_from((index * 7) % levels).unwrap_or(0)])
            .collect()
    }

    fn scaled(value: u8, depth: png::BitDepth) -> u8 {
        let top = (1_u32 << bits(depth)) - 1;
        u8::try_from(u32::from(value) * 255 / top).unwrap_or(u8::MAX)
    }

    fn assert_texels(name: &str, file: &[u8], expected: &[[u8; 4]]) {
        let image = decoded(file);
        assert_eq!((image.width(), image.height()), (WIDTH, HEIGHT), "{name}");
        let texels: Vec<[u8; 4]> = image.texels.as_chunks::<4>().0.to_vec();
        assert_eq!(texels.len(), PIXELS, "{name}");
        assert_eq!(texels, expected, "{name}");
    }

    fn assert_both_layouts(name: &str, image: &Encoded, expected: &[[u8; 4]]) {
        assert_texels(name, &encode(image, false), expected);
        assert_texels(
            &format!("{name}, interlaced"),
            &encode(image, true),
            expected,
        );
    }

    #[test]
    fn decoding_gives_the_texels_of_greyscale_at_every_bit_depth_with_and_without_transparency() {
        for depth in [
            png::BitDepth::One,
            png::BitDepth::Two,
            png::BitDepth::Four,
            png::BitDepth::Eight,
        ] {
            let samples = grey(depth);
            let key = samples[1][0];
            let texel = |sample: u8, alpha: u8| {
                let value = scaled(sample, depth);
                [value, value, value, alpha]
            };
            let opaque: Vec<[u8; 4]> = samples.iter().map(|s| texel(s[0], 255)).collect();
            let keyed: Vec<[u8; 4]> = samples
                .iter()
                .map(|s| texel(s[0], if s[0] == key { 0 } else { 255 }))
                .collect();
            assert!(keyed.iter().any(|texel| texel[3] == 0));
            assert!(keyed.iter().any(|texel| texel[3] == 255));
            let image = |transparency| Encoded {
                colour: png::ColorType::Grayscale,
                depth,
                samples: samples.clone(),
                palette: None,
                transparency,
            };
            assert_both_layouts(&format!("greyscale {depth:?}"), &image(None), &opaque);
            assert_both_layouts(
                &format!("greyscale {depth:?} with transparency"),
                &image(Some(vec![0, key])),
                &keyed,
            );
        }
    }

    #[test]
    fn decoding_gives_the_texels_of_indexed_colour_at_every_bit_depth_with_and_without_transparency()
     {
        for depth in [
            png::BitDepth::One,
            png::BitDepth::Two,
            png::BitDepth::Four,
            png::BitDepth::Eight,
        ] {
            let samples = grey(depth);
            let entries = 1_usize << bits(depth).min(4);
            let palette: Vec<u8> = (0..entries)
                .flat_map(|entry| {
                    let entry = u8::try_from(entry).unwrap_or(0);
                    [entry * 13, 255 - entry * 11, entry * 5 + 40]
                })
                .collect();
            let alphas = [10_u8, 200];
            let texel = |sample: u8, transparent: bool| {
                let index = usize::from(sample);
                let alpha = if transparent {
                    alphas.get(index).copied().unwrap_or(255)
                } else {
                    255
                };
                [
                    palette[3 * index],
                    palette[3 * index + 1],
                    palette[3 * index + 2],
                    alpha,
                ]
            };
            let samples: Vec<Vec<u8>> = samples
                .into_iter()
                .map(|sample| vec![sample[0] % u8::try_from(entries).unwrap_or(u8::MAX)])
                .collect();
            let opaque: Vec<[u8; 4]> = samples.iter().map(|s| texel(s[0], false)).collect();
            let translucent: Vec<[u8; 4]> = samples.iter().map(|s| texel(s[0], true)).collect();
            let image = |transparency| Encoded {
                colour: png::ColorType::Indexed,
                depth,
                samples: samples.clone(),
                palette: Some(palette.clone()),
                transparency,
            };
            assert_both_layouts(&format!("indexed {depth:?}"), &image(None), &opaque);
            assert_both_layouts(
                &format!("indexed {depth:?} with transparency"),
                &image(Some(alphas.to_vec())),
                &translucent,
            );
        }
    }

    #[test]
    fn decoding_gives_the_texels_of_truecolour_and_of_the_colour_types_with_alpha() {
        let channel = |index: usize, step: usize| u8::try_from((index * step) % 256).unwrap_or(0);
        let rgb: Vec<Vec<u8>> = (0..PIXELS)
            .map(|index| vec![channel(index, 17), channel(index, 29), channel(index, 53)])
            .collect();
        let key = rgb[2].clone();
        let opaque: Vec<[u8; 4]> = rgb.iter().map(|s| [s[0], s[1], s[2], 255]).collect();
        let keyed: Vec<[u8; 4]> = rgb
            .iter()
            .map(|s| [s[0], s[1], s[2], if *s == key { 0 } else { 255 }])
            .collect();
        let truecolour = |transparency| Encoded {
            colour: png::ColorType::Rgb,
            depth: png::BitDepth::Eight,
            samples: rgb.clone(),
            palette: None,
            transparency,
        };
        assert_both_layouts("truecolour", &truecolour(None), &opaque);
        let transparency = key.iter().flat_map(|&value| [0, value]).collect();
        assert_both_layouts(
            "truecolour with transparency",
            &truecolour(Some(transparency)),
            &keyed,
        );
        let grey_alpha: Vec<Vec<u8>> = (0..PIXELS)
            .map(|index| vec![channel(index, 23), channel(index, 41)])
            .collect();
        assert_both_layouts(
            "greyscale with alpha",
            &Encoded {
                colour: png::ColorType::GrayscaleAlpha,
                depth: png::BitDepth::Eight,
                samples: grey_alpha.clone(),
                palette: None,
                transparency: None,
            },
            &grey_alpha
                .iter()
                .map(|s| [s[0], s[0], s[0], s[1]])
                .collect::<Vec<_>>(),
        );
        let rgba: Vec<Vec<u8>> = (0..PIXELS)
            .map(|index| {
                vec![
                    channel(index, 19),
                    channel(index, 31),
                    channel(index, 47),
                    channel(index, 61),
                ]
            })
            .collect();
        assert_both_layouts(
            "truecolour with alpha",
            &Encoded {
                colour: png::ColorType::Rgba,
                depth: png::BitDepth::Eight,
                samples: rgba.clone(),
                palette: None,
                transparency: None,
            },
            &rgba
                .iter()
                .map(|s| [s[0], s[1], s[2], s[3]])
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn debug_shows_the_size_and_no_texel() {
        let file = match encode_png(2, 1, &[1, 2, 3, 4, 5, 6]) {
            Ok(file) => file,
            Err(error) => panic!("{error}"),
        };
        assert_eq!(
            format!("{:?}", decoded(&file)),
            "Image { width: 2, height: 1, .. }"
        );
    }

    fn test_image() -> Image {
        let pixels: Vec<u8> = (0..8 * 4)
            .flat_map(|index| {
                let index = u8::try_from(index).unwrap_or(0);
                [index * 7, 255 - index * 3, index * 5]
            })
            .collect();
        match encode_png(8, 4, &pixels) {
            Ok(file) => decoded(&file),
            Err(error) => panic!("{error}"),
        }
    }

    fn texel_value(image: &Image, column: usize, row: usize) -> [f64; 3] {
        let [red, green, blue, _] = image.texel(column, row);
        Colour::from_srgb8(red, green, blue).linear()
    }

    const RECTANGLE: [DVec2; 4] = [
        DVec2::new(-1.0, -1.0),
        DVec2::new(3.0, -1.0),
        DVec2::new(3.0, 1.0),
        DVec2::new(-1.0, 1.0),
    ];

    #[test]
    fn placement_centres_the_polygon_and_fits_it_inside_the_margin() {
        let wide = Placement::new(&RECTANGLE, 100, 60);
        assert_eq!(wide.scale(), 24.0625);
        assert_eq!(wide.position(DVec2::new(1.0, 0.0)), DVec2::new(50.0, 30.0));
        assert_eq!(
            wide.position(DVec2::new(-1.0, 1.0)),
            DVec2::new(1.875, 5.9375)
        );
        assert_eq!(
            wide.position(DVec2::new(3.0, -1.0)),
            DVec2::new(98.125, 54.0625)
        );
        let tall = Placement::new(&RECTANGLE, 64, 256);
        assert_eq!(tall.scale(), 15.0);
        assert_eq!(tall.position(DVec2::new(3.0, 1.0)), DVec2::new(62.0, 113.0));
        for placement in [wide, tall] {
            for vertex in RECTANGLE {
                assert_eq!(placement.point(placement.position(vertex)), vertex);
            }
        }
    }

    #[test]
    fn placement_keeps_a_position_on_the_image_and_refuses_one_beyond_it_or_not_finite() {
        let placement = Placement::new(&RECTANGLE, 8, 4);
        for corner in [
            DVec2::new(0.0, 0.0),
            DVec2::new(8.0, 0.0),
            DVec2::new(0.0, 4.0),
            DVec2::new(8.0, 4.0),
        ] {
            assert_eq!(placement.inside(corner), Some(corner));
            for beyond in [
                corner + DVec2::new(if corner.x > 0.0 { 1e-12 } else { -1e-12 }, 0.0),
                corner + DVec2::new(0.0, if corner.y > 0.0 { 1e-12 } else { -1e-12 }),
            ] {
                assert_eq!(placement.inside(beyond), None, "{beyond}");
            }
        }
        for position in [
            DVec2::new(f64::NAN, 1.0),
            DVec2::new(1.0, f64::INFINITY),
            DVec2::new(f64::NEG_INFINITY, 1.0),
        ] {
            assert_eq!(placement.inside(position), None, "{position}");
        }
    }

    #[test]
    fn sample_at_the_edge_of_the_image_takes_the_edge_texels() {
        let image = test_image();
        let linear = decode_table();
        for (position, (column, row)) in [
            (DVec2::new(0.0, 0.0), (0, 0)),
            (DVec2::new(8.0, 0.0), (7, 0)),
            (DVec2::new(0.0, 4.0), (0, 3)),
            (DVec2::new(8.0, 4.0), (7, 3)),
        ] {
            let value = sample(&image, position, [0.5; 3], &linear);
            assert_eq!(value, texel_value(&image, column, row), "{position}");
        }
    }

    #[test]
    fn sample_interpolates_the_four_texels_around_a_position_in_linear_light() {
        let image = test_image();
        let linear = decode_table();
        let value = sample(&image, DVec2::new(2.75, 1.25), [0.0; 3], &linear);
        let expected: Vec<f64> = (0..3)
            .map(|channel| {
                0.75 * 0.25 * texel_value(&image, 2, 0)[channel]
                    + 0.25 * 0.25 * texel_value(&image, 3, 0)[channel]
                    + 0.75 * 0.75 * texel_value(&image, 2, 1)[channel]
                    + 0.25 * 0.75 * texel_value(&image, 3, 1)[channel]
            })
            .collect();
        for (got, want) in value.iter().zip(expected) {
            assert!((got - want).abs() <= 4.0 * f64::EPSILON, "{got} {want}");
        }
    }
}
