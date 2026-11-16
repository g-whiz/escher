use png::{BitDepth, ColorType, Encoder, SrgbRenderingIntent};

use crate::error::{Error, Result};

/// Encodes an image of 8-bit sRGB pixels as a PNG file in memory.
///
/// Use it on the pixels of [`Renderer::render`](crate::Renderer::render), or
/// on the bands of [`Renderer::render_rows`](crate::Renderer::render_rows)
/// joined in order. The buffer holds 3 bytes per pixel, red, green, and blue,
/// row by row from the top. The file has the colour type RGB, the bit depth
/// 8, and the sRGB chunk with the perceptual rendering intent, which tells a
/// viewer that the samples are in the sRGB colour space. It has no `gAMA` or
/// `cHRM` chunk. The function touches no file and returns the bytes of the PNG
/// file.
///
/// # Examples
///
/// ```
/// use escher_render::encode_png;
///
/// let file = encode_png(2, 1, &[255, 0, 0, 0, 0, 255])?;
/// assert_eq!(file.get(1..4), Some(&b"PNG"[..]));
/// # Ok::<(), escher_render::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error of kind
/// [`ErrorKind::InvalidBuffer`](crate::ErrorKind::InvalidBuffer) when the
/// width or the height is 0 or the length of the buffer is not 3 times the
/// number of pixels, and of kind
/// [`ErrorKind::Encoding`](crate::ErrorKind::Encoding) when the PNG encoder
/// fails.
pub fn encode_png(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>> {
    let expected = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|count| count.checked_mul(3));
    let length = u64::try_from(pixels.len()).ok();
    if width == 0 || height == 0 || expected.is_none() || expected != length {
        return Err(Error::wrong_image(pixels.len(), width, height));
    }
    let mut file = Vec::new();
    let mut encoder = Encoder::new(&mut file, width, height);
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_source_srgb(SrgbRenderingIntent::Perceptual);
    let mut writer = encoder.write_header().map_err(Error::encoding)?;
    writer.write_image_data(pixels).map_err(Error::encoding)?;
    writer.finish().map_err(Error::encoding)?;
    Ok(file)
}
