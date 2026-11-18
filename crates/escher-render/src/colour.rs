use escher_domain::FundamentalDomain;
use escher_group::{Colouring, CosetTable, Presentation};

use crate::error::{Error, Result};

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
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Fill {
    /// Every tile takes the one colour.
    One(Colour),
    /// A tile takes the first colour when an even number of
    /// orientation-reversing pairings maps it onto the fundamental domain,
    /// and the second colour when an odd number does, so that two tiles that
    /// share a mirror side differ. The fundamental domain takes the first
    /// colour. When no side pairing of the domain reverses orientation, as for
    /// the domain of `732`, every tile takes the first colour, so the image
    /// equals the image with [`Fill::One`] of that colour.
    Parity([Colour; 2]),
    /// A tile takes the colour of its coset in the colouring of a
    /// [`CosetFill`].
    Cosets(CosetFill),
}

/// A colouring of a tile by its coset of a subgroup of finite index, with one
/// colour for each coset.
///
/// Build it with [`CosetFill::new`] from the domain whose side presentation
/// gave the colouring, and render it as [`Fill::Cosets`] with a domain that
/// has the same side presentation. For each sample, the renderer records the
/// sides that the fold crossed, in order. The sample takes the colour at the
/// index that [`Colouring::colour_of`](escher_group::Colouring::colour_of)
/// returns for those sides, so the fundamental domain takes the first colour.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_group::{IndexLimit, SearchBudget, colourings_of_index};
/// use escher_render::{Colour, CosetFill, Fill, RenderOptions, Renderer};
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*642".parse::<Signature>()?)?;
/// let presentation = domain.side_presentation()?;
/// let found = colourings_of_index(&presentation, IndexLimit::new(3)?, SearchBudget::DEFAULT)?;
/// let colouring = found.find("3a".parse()?).ok_or("3a is missing")?.clone();
/// let colours = vec![
///     Colour::from_srgb8(245, 222, 179),
///     Colour::from_srgb8(70, 130, 180),
///     Colour::from_srgb8(220, 20, 60),
/// ];
/// let fill = Fill::Cosets(CosetFill::new(&domain, colouring, colours)?);
/// let options = RenderOptions::builder(fill).size(32, 32).build()?;
/// let (_, stats) = Renderer::new(&domain, options)?.render();
/// assert_eq!(stats.unconverged(), 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CosetFill {
    presentation: Presentation,
    colouring: Colouring,
    colours: Vec<Colour>,
}

impl CosetFill {
    /// Returns the fill that gives coset `c` of the colouring the colour at
    /// index `c` of `colours`.
    ///
    /// The fill keeps the side presentation of the domain, so that
    /// [`Renderer::new`](crate::Renderer::new) can refuse a domain with
    /// another one, and keeps no reference to the domain. It accepts the
    /// colouring only when the colouring is a colouring of that side
    /// presentation: its table has one generator for each side, and tracing
    /// each relator from each coset returns to that coset. The check traces
    /// the base of each relator once from each coset and then follows the
    /// cycles of that permutation of the cosets, so for each relator it reads
    /// at most the number of cosets times the length of the base, plus the
    /// square of the number of cosets. A colouring has at most 64 cosets, and
    /// for every domain that the crate `escher_domain` builds the whole check
    /// reads a few thousand table entries at most.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidOption`](crate::ErrorKind::InvalidOption), for the
    /// first of these causes: the domain has no side presentation, the number
    /// of generators of the colouring differs from the number of sides of the
    /// domain, a relator of the side presentation does not return some coset
    /// of the colouring to itself, or the number of colours differs from the
    /// number of colours of the colouring.
    pub fn new(
        domain: &FundamentalDomain,
        colouring: Colouring,
        colours: Vec<Colour>,
    ) -> Result<Self> {
        let presentation = domain.side_presentation().map_err(Error::no_presentation)?;
        let generators = colouring.table().generator_count();
        let sides = presentation.generator_count();
        if generators != sides {
            return Err(Error::generator_count(generators, sides));
        }
        if let Some(relator) = broken_relator(&presentation, colouring.table()) {
            return Err(Error::broken_relator(relator));
        }
        let needed = colouring.table().index();
        if needed != colours.len() {
            return Err(Error::colour_count(needed, colours.len()));
        }
        Ok(Self {
            presentation,
            colouring,
            colours,
        })
    }

    pub(crate) fn presentation(&self) -> &Presentation {
        &self.presentation
    }

    #[expect(
        clippy::expect_used,
        reason = "the renderer uses a coset fill only with a domain of the side presentation that the fill checked, whose sides are the generators of the table, and the table has one colour for each coset"
    )]
    pub(crate) fn colour(&self, sides: &[u8]) -> Colour {
        self.colouring
            .colour_of(sides)
            .and_then(|coset| self.colours.get(usize::try_from(coset).ok()?))
            .copied()
            .expect(
                "internal error: every side sequence of the domain of a coset fill has a colour",
            )
    }
}

fn broken_relator(presentation: &Presentation, table: &CosetTable) -> Option<usize> {
    let cosets = u32::try_from(table.index()).unwrap_or(u32::MAX);
    presentation.relators().iter().position(|relator| {
        let images: Option<Vec<u32>> = (0..cosets)
            .map(|coset| table.trace(coset, relator.base()))
            .collect();
        let Some(images) = images else {
            return true;
        };
        (0..cosets).any(|start| {
            let mut length: u32 = 1;
            let mut coset = start;
            loop {
                match images.get(usize::try_from(coset).unwrap_or(usize::MAX)) {
                    Some(&next) if next == start => break,
                    Some(&next) if length < cosets => {
                        coset = next;
                        length += 1;
                    }
                    _ => return true,
                }
            }
            !relator.exponent().is_multiple_of(length)
        })
    })
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
    use escher_domain::FundamentalDomain;
    use escher_group::{Colouring, Letter, Presentation, Relator, Word};

    use super::{Colour, CosetFill, decode, decode_unit, encode, encode_unit};
    use crate::ErrorKind;
    use crate::golden_images::{colouring, colourings_of, domain_with_aspect};
    use crate::reference::{Reference, Worst, int, mul, num, sub};

    fn rejection(domain: &FundamentalDomain, colouring: &Colouring, count: u8) -> String {
        let colours = (0..count)
            .map(|value| Colour::from_srgb8(value, 0, 0))
            .collect();
        match CosetFill::new(domain, colouring.clone(), colours) {
            Ok(_) => panic!("{count} colours are accepted"),
            Err(error) => {
                assert_eq!(error.kind(), ErrorKind::InvalidOption);
                error.to_string()
            }
        }
    }

    #[test]
    fn coset_fill_refuses_another_number_of_generators_before_another_number_of_colours() {
        let colouring = colouring(&domain_with_aspect("22", None), "2a");
        let triangle = domain_with_aspect("*732", None);
        for count in [1, 2, 3] {
            assert_eq!(
                rejection(&triangle, &colouring, count),
                "cannot build coset fill: the colouring has 2 generators, and the domain has 3 sides",
            );
        }
    }

    #[test]
    fn coset_fill_refuses_another_number_of_colours_and_keeps_the_colours_in_order() {
        let lune = domain_with_aspect("22", None);
        let colouring = colouring(&lune, "2a");
        assert_eq!(
            rejection(&lune, &colouring, 1),
            "cannot build coset fill: the colouring needs 2 colours, and 1 was given",
        );
        assert_eq!(
            rejection(&lune, &colouring, 3),
            "cannot build coset fill: the colouring needs 2 colours, and 3 were given",
        );
        let colours = vec![Colour::WHITE, Colour::BLACK];
        match CosetFill::new(&lune, colouring, colours.clone()) {
            Ok(fill) => assert_eq!(fill.colours, colours),
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the search of the colourings of a hyperbolic triangle runs loops too long for the interpreter"
    )]
    fn coset_fill_refuses_a_colouring_of_another_triangle_with_as_many_sides() {
        let colouring = colouring(&domain_with_aspect("*642", None), "3a");
        let triangle = domain_with_aspect("*732", None);
        for count in [2, 3] {
            assert_eq!(
                rejection(&triangle, &colouring, count),
                "cannot build coset fill: relator 3 of the side presentation of the domain moves a coset of the colouring",
            );
        }
    }

    fn renumbered(presentation: &Presentation, order: &[u8]) -> Presentation {
        let letter = |letter: Letter| {
            let index = order[usize::from(letter.index())];
            if letter.is_inverse() {
                Letter::inverse_of(index)
            } else {
                Letter::generator(index)
            }
        };
        let relators = presentation
            .relators()
            .iter()
            .map(|relator| {
                let base: Word = relator
                    .base()
                    .letters()
                    .iter()
                    .map(|&each| letter(each))
                    .collect();
                match Relator::new(base, relator.exponent()) {
                    Ok(relator) => relator,
                    Err(error) => panic!("{error}"),
                }
            })
            .collect();
        match Presentation::new(presentation.generator_names().to_vec(), relators) {
            Ok(presentation) => presentation,
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the search of the colourings of a hyperbolic triangle runs loops too long for the interpreter"
    )]
    fn coset_fill_refuses_a_colouring_of_the_side_presentation_with_its_sides_renumbered() {
        let triangle = domain_with_aspect("*642", None);
        let presentation = match triangle.side_presentation() {
            Ok(presentation) => presentation,
            Err(error) => panic!("{error}"),
        };
        let swapped = renumbered(&presentation, &[0, 2, 1]);
        let limit = match escher_group::IndexLimit::new(3) {
            Ok(limit) => limit,
            Err(error) => panic!("{error}"),
        };
        let found = match escher_group::colourings_of_index(
            &swapped,
            limit,
            escher_group::SearchBudget::DEFAULT,
        ) {
            Ok(found) => found,
            Err(error) => panic!("{error}"),
        };
        assert!(found.is_complete());
        let own = colourings_of(&triangle, 3);
        let foreign: Vec<&Colouring> = found
            .list()
            .iter()
            .filter(|colouring| !own.iter().any(|mine| mine.table() == colouring.table()))
            .collect();
        assert_ne!(foreign.len(), 0);
        for colouring in foreign {
            let text = rejection(&triangle, colouring, 3);
            assert!(
                text.starts_with("cannot build coset fill: relator "),
                "{}: {text}",
                colouring.id()
            );
        }
        for colouring in &own {
            let colours = vec![Colour::WHITE; 3];
            assert!(CosetFill::new(&triangle, colouring.clone(), colours).is_ok());
        }
    }

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
