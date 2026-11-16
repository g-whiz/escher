/// An opaque colour, stored in linear light.
///
/// Use it for every colour of an image: the fills, the border, and the
/// background. The only constructor takes the three 8-bit channels of sRGB,
/// the colour space of most image files and displays, and the colour stores
/// them as linear light, the values that are proportional to light
/// intensity. The renderer averages the samples of a pixel in linear light.
/// Each stored channel is finite and lies between 0 and 1.
///
/// # Examples
///
/// ```
/// use escher_render::Colour;
///
/// let crimson = Colour::from_srgb8(220, 20, 60);
/// assert_eq!(crimson.to_srgb8(), [220, 20, 60]);
/// assert_eq!(Colour::from_srgb8(0, 0, 0), Colour::BLACK);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colour {
    linear: [f64; 3],
}

impl Colour {
    /// Black, equal to `Colour::from_srgb8(0, 0, 0)`.
    pub const BLACK: Self = Self { linear: [0.0; 3] };

    /// White, equal to `Colour::from_srgb8(255, 255, 255)`.
    pub const WHITE: Self = Self { linear: [1.0; 3] };

    /// Returns the colour of three 8-bit sRGB channels.
    ///
    /// Each byte is divided by 255 and decoded to linear light by the sRGB
    /// transfer function.
    #[must_use]
    pub fn from_srgb8(red: u8, green: u8, blue: u8) -> Self {
        Self {
            linear: [red, green, blue].map(decode),
        }
    }

    /// Returns the three 8-bit sRGB channels of the colour, red first.
    ///
    /// Each channel is encoded by the sRGB transfer function, multiplied by
    /// 255, and rounded to the nearest integer, so that
    /// `Colour::from_srgb8(r, g, b).to_srgb8()` is `[r, g, b]` for every byte
    /// value.
    #[must_use]
    pub fn to_srgb8(self) -> [u8; 3] {
        self.linear.map(encode)
    }

    pub(crate) fn linear(self) -> [f64; 3] {
        self.linear
    }
}

/// The colours that fill the tiles of an image.
///
/// The enum is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
///
/// # Examples
///
/// ```
/// use escher_render::{Colour, Fill};
///
/// let fill = Fill::Parity([Colour::WHITE, Colour::from_srgb8(70, 130, 180)]);
/// assert_ne!(fill, Fill::One(Colour::WHITE));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Fill {
    /// Every tile takes the one colour.
    One(Colour),
    /// A tile takes the first colour when an even number of
    /// orientation-reversing pairings maps it onto the fundamental domain,
    /// and the second colour when an odd number does, so that two tiles that
    /// share a mirror side differ. The fundamental domain takes the first
    /// colour.
    Parity([Colour; 2]),
}

const DECODE_THRESHOLD: f64 = 0.04045;
const ENCODE_THRESHOLD: f64 = 0.003_130_8;
const LINEAR_SLOPE: f64 = 12.92;
const OFFSET: f64 = 0.055;
const SCALE: f64 = 1.055;
const EXPONENT: f64 = 2.4;

fn decode(byte: u8) -> f64 {
    decode_unit(f64::from(byte) / 255.0)
}

fn decode_unit(encoded: f64) -> f64 {
    if encoded <= DECODE_THRESHOLD {
        encoded / LINEAR_SLOPE
    } else {
        ((encoded + OFFSET) / SCALE).powf(EXPONENT)
    }
}

pub(crate) fn encode(linear: f64) -> u8 {
    to_byte((255.0 * encode_unit(linear)).round().clamp(0.0, 255.0))
}

fn encode_unit(linear: f64) -> f64 {
    let linear = linear.clamp(0.0, 1.0);
    if linear <= ENCODE_THRESHOLD {
        LINEAR_SLOPE * linear
    } else {
        SCALE * linear.powf(1.0 / EXPONENT) - OFFSET
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the caller rounds the value and clamps it to the range of a byte, and a NaN becomes 0"
)]
fn to_byte(value: f64) -> u8 {
    value as u8
}

#[cfg(test)]
mod tests {
    use super::{Colour, decode, decode_unit, encode, encode_unit};
    use crate::reference::{Reference, Worst, int, mul, num, sub};

    #[test]
    fn black_and_white_equal_their_constructors() {
        assert_eq!(Colour::from_srgb8(0, 0, 0), Colour::BLACK);
        assert_eq!(Colour::from_srgb8(255, 255, 255), Colour::WHITE);
    }

    #[test]
    fn to_srgb8_inverts_from_srgb8_for_every_byte() {
        for byte in 0..=u8::MAX {
            let colour = Colour::from_srgb8(byte, 0, u8::MAX - byte);
            assert_eq!(colour.to_srgb8(), [byte, 0, u8::MAX - byte]);
        }
    }

    #[test]
    fn encode_clamps_values_outside_the_unit_interval() {
        assert_eq!(encode(-1.0), 0);
        assert_eq!(encode(2.0), 255);
        assert_eq!(encode(f64::NAN), 0);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
    fn transfer_function_agrees_with_the_reference_for_every_byte() {
        let mut reference = Reference::new();
        let mut decoding = Worst::new("decoding, absolute error in linear light", 256);
        let mut encoding = Worst::new("encoding, absolute error in encoded units", 256);
        let mut rounding = Worst::new(
            "encoding, distance of 255 times the encoded value from the byte, in units of one byte",
            256,
        );
        for byte in 0..=u8::MAX {
            let exact = reference.srgb_decode(byte);
            let input = || format!("byte {byte}");
            decoding.record(
                &crate::reference::error(decode(byte), &exact),
                &int(1),
                input,
            );
            let stored = num(decode(byte));
            let exact_encoded = reference.srgb_encode(&stored);
            encoding.record(
                &crate::reference::error(encode_unit(decode(byte)), &exact_encoded),
                &int(1),
                input,
            );
            let encoded = mul(&int(255), &exact_encoded);
            rounding.record(&sub(&encoded, &int(u32::from(byte))).abs(), &int(1), input);
            assert_eq!(encode(decode(byte)), byte);
        }
        decoding.assert_at_most(&mut reference, 4.0e-15);
        encoding.assert_at_most(&mut reference, 2.0e-15);
        rounding.assert_at_most(&mut reference, 5.0e-13);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
    fn transfer_function_agrees_with_the_reference_between_the_current_and_the_obsolete_thresholds()
    {
        const STEPS: u32 = 64;
        let mut reference = Reference::new();
        let mut decoding = Worst::new(
            "decoding between the thresholds 0.03928 and 0.04045, absolute error in linear light",
            usize::try_from(STEPS + 1).unwrap_or(usize::MAX),
        );
        let mut encoding = Worst::new(
            "encoding between the thresholds 0.00304 and 0.0031308, absolute error in encoded units",
            usize::try_from(STEPS + 1).unwrap_or(usize::MAX),
        );
        let between = |low: f64, high: f64, step: u32| {
            low + (high - low) * f64::from(step) / f64::from(STEPS)
        };
        for step in 0..=STEPS {
            let encoded = between(0.03928, 0.04045, step);
            let exact = reference.srgb_decode_unit(&num(encoded));
            decoding.record(
                &crate::reference::error(decode_unit(encoded), &exact),
                &int(1),
                || format!("encoded value {encoded}"),
            );
            let linear = between(0.00304, 0.003_130_8, step);
            let exact = reference.srgb_encode(&num(linear));
            encoding.record(
                &crate::reference::error(encode_unit(linear), &exact),
                &int(1),
                || format!("linear value {linear}"),
            );
        }
        decoding.assert_at_most(&mut reference, 4.0e-15);
        encoding.assert_at_most(&mut reference, 2.0e-15);
    }
}
