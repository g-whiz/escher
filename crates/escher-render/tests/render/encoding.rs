use escher_render::{ErrorKind, encode_png};

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
