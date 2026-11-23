use escher_render::{ErrorKind, Image, encode_png};
use proptest::prelude::{any, prop_assert, proptest};

use crate::common::decode_png;

#[test]
fn encode_png_writes_an_rgb_image_of_the_requested_size_with_the_srgb_chunk() {
    let pixels: Vec<u8> = (0..16 * 32 * 3)
        .map(|index: u32| (index % 251).to_le_bytes()[0])
        .collect();
    let file = match encode_png(16, 32, &pixels) {
        Ok(file) => file,
        Err(error) => panic!("{error}"),
    };
    let decoded = decode_png(&file).unwrap_or_else(|error| panic!("{error}"));
    let (output, info) = (decoded.output, decoded.info);
    assert_eq!((output.width, output.height), (16, 32));
    assert_eq!(output.color_type, png::ColorType::Rgb);
    assert_eq!(output.bit_depth, png::BitDepth::Eight);
    assert_eq!(info.srgb, Some(png::SrgbRenderingIntent::Perceptual));
    assert_eq!(info.gama_chunk, None);
    assert_eq!(info.chrm_chunk, None);
    assert_eq!(decoded.pixels, pixels);
}

#[test]
fn encode_png_rejects_an_empty_size_or_a_buffer_of_the_wrong_length() {
    let cases: [(u32, u32, usize, &str); 5] = [
        (
            0,
            32,
            0,
            "cannot encode image: the buffer of 0 bytes does not hold an image of 0x32 pixels",
        ),
        (
            16,
            0,
            0,
            "cannot encode image: the buffer of 0 bytes does not hold an image of 16x0 pixels",
        ),
        (
            16,
            32,
            1535,
            "cannot encode image: the buffer of 1535 bytes does not hold an image of 16x32 pixels",
        ),
        (
            16,
            32,
            1537,
            "cannot encode image: the buffer of 1537 bytes does not hold an image of 16x32 pixels",
        ),
        (
            u32::MAX,
            u32::MAX,
            3,
            "cannot encode image: the buffer of 3 bytes does not hold an image of 4294967295x4294967295 pixels",
        ),
    ];
    for (width, height, length, text) in cases {
        match encode_png(width, height, &vec![0; length]) {
            Ok(_) => panic!("{width}x{height} with {length} bytes is accepted"),
            Err(error) => {
                assert_eq!(error.kind(), ErrorKind::InvalidBuffer);
                assert_eq!(error.to_string(), text);
            }
        }
    }
}

const NOT_PNG: &str = "cannot decode image: the data is not a valid PNG image";

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

fn crc(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(u32::MAX, |crc, &byte| {
        (0..8).fold(crc ^ u32::from(byte), |crc, _| {
            if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            }
        })
    })
}

fn header_only(width: u32, height: u32, depth: u8) -> Vec<u8> {
    let mut chunk = b"IHDR".to_vec();
    chunk.extend(width.to_be_bytes());
    chunk.extend(height.to_be_bytes());
    chunk.extend([depth, 2, 0, 0, 0]);
    let mut stream = SIGNATURE.to_vec();
    stream.extend(13_u32.to_be_bytes());
    stream.extend(&chunk);
    stream.extend(crc(&chunk).to_be_bytes());
    stream
}

fn image_file(width: u32, height: u32) -> Vec<u8> {
    let pixels: Vec<u8> = (0..width * height * 3)
        .map(|index| (index % 251).to_le_bytes()[0])
        .collect();
    encode_png(width, height, &pixels).unwrap_or_else(|error| panic!("{error}"))
}

const DECODER_BYTES: usize = 1 << 24;

const MEMORY_LIMIT: &str = "cannot decode image: the PNG decoder reached its memory limit";

fn with_chunk(width: u16, kind: [u8; 4], length: usize) -> Vec<u8> {
    let mut file = Vec::new();
    let mut encoder = png::Encoder::new(&mut file, u32::from(width), 1);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| {
            writer
                .write_chunk(png::chunk::ChunkType(kind), &vec![0; length])
                .and_then(|()| writer.write_image_data(&vec![7; 3 * usize::from(width)]))
                .and_then(|()| writer.finish())
        })
        .unwrap_or_else(|error| panic!("{error}"));
    file
}

fn refusal(stream: &[u8]) -> (ErrorKind, String) {
    match Image::decode_png(stream) {
        Ok(image) => panic!("the stream decodes to {image:?}"),
        Err(error) => (error.kind(), error.to_string()),
    }
}

#[test]
fn decoding_checks_the_size_and_the_bit_depth_from_the_header_before_the_image_data() {
    let cases: [(Vec<u8>, ErrorKind, &str); 8] = [
        (
            header_only(16384, 16384, 8),
            ErrorKind::LimitExceeded,
            "cannot decode image: the image size 16384x16384 has more than 67108864 pixels",
        ),
        (
            header_only(16384, 4097, 8),
            ErrorKind::LimitExceeded,
            "cannot decode image: the image size 16384x4097 has more than 67108864 pixels",
        ),
        (
            header_only(16385, 1, 8),
            ErrorKind::LimitExceeded,
            "cannot decode image: the image size 16385x1 is larger than 16384 pixels on a side",
        ),
        (
            header_only(1, 16385, 16),
            ErrorKind::LimitExceeded,
            "cannot decode image: the image size 1x16385 is larger than 16384 pixels on a side",
        ),
        (
            header_only(4, 4, 16),
            ErrorKind::Decoding,
            "cannot decode image: the image has 16 bits per channel, and at most 8 are supported",
        ),
        (header_only(16384, 4096, 8), ErrorKind::Decoding, NOT_PNG),
        (header_only(0, 4, 8), ErrorKind::Decoding, NOT_PNG),
        (header_only(4, 0, 8), ErrorKind::Decoding, NOT_PNG),
    ];
    for (stream, kind, text) in cases {
        assert_eq!(refusal(&stream), (kind, text.to_owned()), "{text}");
    }
}

#[test]
fn decoding_refuses_an_image_with_16_bits_per_channel() {
    let mut file = Vec::new();
    let mut encoder = png::Encoder::new(&mut file, 2, 2);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Sixteen);
    encoder
        .write_header()
        .and_then(|mut writer| {
            writer
                .write_image_data(&[7; 24])
                .and_then(|()| writer.finish())
        })
        .unwrap_or_else(|error| panic!("{error}"));
    let (kind, text) = refusal(&file);
    assert_eq!(kind, ErrorKind::Decoding);
    assert_eq!(
        text,
        "cannot decode image: the image has 16 bits per channel, and at most 8 are supported"
    );
}

#[test]
fn decoding_refuses_an_empty_stream_a_text_a_bad_checksum_and_every_cut_of_an_image() {
    let file = image_file(5, 3);
    let image = Image::decode_png(file.as_slice()).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!((image.width(), image.height()), (5, 3));
    let not_png = (ErrorKind::Decoding, NOT_PNG.to_owned());
    assert_eq!(refusal(&[]), not_png);
    assert_eq!(refusal(b"an image\n"), not_png);
    let mut corrupted = file.clone();
    let last = corrupted.len() - 1;
    corrupted[last] ^= 1;
    assert_eq!(refusal(&corrupted), not_png);
    for length in 0..file.len() {
        assert_eq!(
            refusal(&file[..length]),
            not_png,
            "cut after {length} bytes"
        );
    }
}

struct Failing;

impl std::io::Read for Failing {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("the disk is gone"))
    }
}

#[test]
fn decoding_reports_a_reader_that_returns_an_error_and_keeps_it_as_the_source() {
    match Image::decode_png(Failing) {
        Ok(image) => panic!("the stream decodes to {image:?}"),
        Err(error) => {
            assert_eq!(error.kind(), ErrorKind::Decoding);
            assert_eq!(
                error.to_string(),
                "cannot decode image: the image cannot be read"
            );
            let source = core::error::Error::source(&error).map(ToString::to_string);
            assert!(source.is_some_and(|text| text.contains("the disk is gone")));
        }
    }
}

proptest! {
    #[test]
    fn decoding_an_image_with_one_changed_byte_returns_an_image_or_a_documented_error(
        index in any::<proptest::sample::Index>(),
        byte in any::<u8>(),
    ) {
        let mut file = image_file(5, 3);
        let at = index.index(file.len());
        file[at] = byte;
        match Image::decode_png(file.as_slice()) {
            Ok(image) => prop_assert!(u64::from(image.width()) * u64::from(image.height()) <= 1 << 26),
            Err(error) => {
                let text = error.to_string();
                let documented = match error.kind() {
                    ErrorKind::Decoding => text == NOT_PNG || text.contains("16 bits per channel"),
                    ErrorKind::LimitExceeded => text.contains("pixels") || text.contains("memory limit"),
                    _ => false,
                };
                prop_assert!(documented, "{text}");
            }
        }
    }
}

fn full_palette_file(width: u32, height: u32) -> Vec<u8> {
    let palette: Vec<u8> = (0..=255_u8)
        .flat_map(|entry| [entry, !entry, entry / 2])
        .collect();
    let transparency: Vec<u8> = (0..=255_u8).rev().collect();
    let indices: Vec<u8> = (0..width * height)
        .map(|index| (index % 256).to_le_bytes()[0])
        .collect();
    let mut file = Vec::new();
    let mut encoder = png::Encoder::new(&mut file, width, height);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(palette);
    encoder.set_trns(transparency);
    encoder
        .write_header()
        .and_then(|mut writer| {
            writer
                .write_image_data(&indices)
                .and_then(|()| writer.finish())
        })
        .unwrap_or_else(|error| panic!("{error}"));
    file
}

#[test]
fn decoding_accepts_a_full_palette_with_transparency() {
    let file = full_palette_file(4, 4);
    let image = Image::decode_png(file.as_slice()).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!((image.width(), image.height()), (4, 4));
}

#[test]
fn decoding_accepts_a_full_palette_image_of_the_largest_width() {
    let file = full_palette_file(16384, 1);
    let image = Image::decode_png(file.as_slice()).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!((image.width(), image.height()), (16384, 1));
}

#[test]
fn decoding_keeps_an_exif_chunk_within_the_memory_limit_and_refuses_a_longer_one() {
    for (width, length) in [(2, DECODER_BYTES - 1024), (16384, DECODER_BYTES / 2)] {
        let file = with_chunk(width, *b"eXIf", length);
        let image = Image::decode_png(file.as_slice()).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!((image.width(), image.height()), (u32::from(width), 1));
    }
    assert_eq!(
        refusal(&with_chunk(2, *b"eXIf", DECODER_BYTES + 1024)),
        (ErrorKind::LimitExceeded, MEMORY_LIMIT.to_owned())
    );
}

#[test]
fn decoding_skips_a_text_chunk_and_a_colour_profile_chunk_longer_than_the_memory_limit() {
    for kind in [*b"tEXt", *b"iCCP"] {
        let file = with_chunk(2, kind, DECODER_BYTES + 1024);
        let image = Image::decode_png(file.as_slice()).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!((image.width(), image.height()), (2, 1));
    }
}
