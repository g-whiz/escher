use escher_signature::{Order, Signature};

use crate::error::{Detail, Error, Limit, Result};
use crate::presentation::{Presentation, Relator};
use crate::word::{Letter, Word};

/// Returns the presentation that Conway's rules read from the orbifold
/// [`Signature`] `signature`.
///
/// The features are taken in the order handles, cone points, boundaries,
/// and crosscaps, and each kind of generator is numbered from 1 across the
/// whole signature:
///
/// - Handle `i` gives the generators `x<i>`, `y<i>`, and `a<i>`, and the
///   relator `x<i>^-1 y<i>^-1 x<i> y<i> a<i>^-1`.
/// - Cone point `i` of order `A` gives the generator `g<i>` and the relator
///   `g<i>^A`.
/// - Boundary `i` with corners gives one reflection more than it has corners,
///   numbered on from the reflections of earlier boundaries, then the
///   generator `l<i>`. Its relators are the square of each reflection, with a
///   corner relator between the squares of each two neighbouring reflections
///   `P` and `Q`, which is `(P Q)^a` for the corner order `a` between them,
///   then `l<i>^-1 P l<i> S^-1` for its first reflection `P` and its last
///   reflection `S`. A boundary without corners has one reflection, which is
///   both its first and its last.
/// - Crosscap `i` gives the generators `z<i>` and `w<i>`, and the relator
///   `z<i> z<i> w<i>^-1`.
///
/// The last relator is the product of the generators `a<i>`, `g<i>`,
/// `l<i>`, and `w<i>` in the order of the features. The trivial signature
/// `1` has none of them and gives the presentation with no generator and no
/// relator.
///
/// # Examples
///
/// ```
/// use escher_group::conway_presentation;
/// use escher_signature::Signature;
///
/// let signature: Signature = "3*2".parse()?;
/// let presentation = conway_presentation(&signature)?;
/// assert_eq!(presentation.generator_names(), ["g1", "r1", "r2", "l1"]);
/// let relators: Vec<String> = presentation
///     .relators()
///     .iter()
///     .filter_map(|relator| presentation.display_relator(relator))
///     .map(|text| text.to_string())
///     .collect();
/// assert_eq!(relators, ["g1^3", "r1^2", "(r1 r2)^2", "r2^2", "l1^-1 r1 l1 r2^-1", "g1 l1"]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an [`Error`] of kind
/// [`ErrorKind::UnsupportedSignature`](crate::ErrorKind::UnsupportedSignature)
/// when the signature has a cone point or a corner of infinite order.
pub fn conway_presentation(signature: &Signature) -> Result<Presentation> {
    let unsupported = || Error::build(Detail::UnsupportedSignature(signature.to_string()));
    let finite = |order: &Order| order.finite().ok_or_else(unsupported);
    let cones = signature
        .cone_orders()
        .iter()
        .map(finite)
        .collect::<Result<Vec<u32>>>()?;
    let boundaries = signature
        .boundaries()
        .iter()
        .map(|boundary| {
            boundary
                .corner_orders()
                .iter()
                .map(finite)
                .collect::<Result<Vec<u32>>>()
        })
        .collect::<Result<Vec<Vec<u32>>>>()?;

    let mut builder = Builder::default();
    let mut greek = Vec::new();
    for handle in 1..=signature.handles() {
        let x = builder.generator(format!("x{handle}"))?;
        let y = builder.generator(format!("y{handle}"))?;
        let a = builder.generator(format!("a{handle}"))?;
        builder.relator(&[x.inverse(), y.inverse(), x, y, a.inverse()], 1)?;
        greek.push(a);
    }
    for (cone, order) in (1..).zip(cones) {
        let g = builder.generator(format!("g{cone}"))?;
        builder.relator(&[g], order)?;
        greek.push(g);
    }
    let mut reflections = 0_usize;
    for (boundary, corners) in (1..).zip(boundaries) {
        let mut mirrors = Vec::with_capacity(corners.len() + 1);
        for _ in 0..=corners.len() {
            reflections += 1;
            mirrors.push(builder.generator(format!("r{reflections}"))?);
        }
        let l = builder.generator(format!("l{boundary}"))?;
        for (position, &mirror) in mirrors.iter().enumerate() {
            builder.relator(&[mirror], 2)?;
            if let (Some(&order), Some(&next)) = (corners.get(position), mirrors.get(position + 1))
            {
                builder.relator(&[mirror, next], order)?;
            }
        }
        if let (Some(&first), Some(&last)) = (mirrors.first(), mirrors.last()) {
            builder.relator(&[l.inverse(), first, l, last.inverse()], 1)?;
        }
        greek.push(l);
    }
    for crosscap in 1..=signature.crosscaps() {
        let z = builder.generator(format!("z{crosscap}"))?;
        let w = builder.generator(format!("w{crosscap}"))?;
        builder.relator(&[z, z, w.inverse()], 1)?;
        greek.push(w);
    }
    if !greek.is_empty() {
        builder.relator(&greek, 1)?;
    }
    Presentation::new(builder.names, builder.relators)
}

#[derive(Default)]
struct Builder {
    names: Vec<String>,
    relators: Vec<Relator>,
}

impl Builder {
    fn generator(&mut self, name: String) -> Result<Letter> {
        let index = u8::try_from(self.names.len())
            .map_err(|_too_many| Error::build(Detail::LimitExceeded(Limit::Generators)))?;
        self.names.push(name);
        Ok(Letter::generator(index))
    }

    fn relator(&mut self, letters: &[Letter], exponent: u32) -> Result<()> {
        let base: Word = letters.iter().copied().collect();
        self.relators.push(Relator::new(base, exponent)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::conway_presentation;
    use crate::common::{conway, signature};
    use crate::error::ErrorKind;

    fn texts(text: &str) -> (String, String) {
        let presentation = conway(text);
        let relators: Vec<String> = presentation
            .relators()
            .iter()
            .map(|relator| {
                presentation
                    .display_relator(relator)
                    .expect("the relator uses the generators of the presentation")
                    .to_string()
            })
            .collect();
        (
            presentation.generator_names().join(", "),
            relators.join(", "),
        )
    }

    #[test]
    fn conway_presentation_writes_the_generators_and_relators_of_each_example() {
        let cases = [
            (
                "*532",
                "r1, r2, r3, r4, l1",
                "r1^2, (r1 r2)^5, r2^2, (r2 r3)^3, r3^2, (r3 r4)^2, r4^2, l1^-1 r1 l1 r4^-1, l1",
            ),
            ("532", "g1, g2, g3", "g1^5, g2^3, g3^2, g1 g2 g3"),
            (
                "3*2",
                "g1, r1, r2, l1",
                "g1^3, r1^2, (r1 r2)^2, r2^2, l1^-1 r1 l1 r2^-1, g1 l1",
            ),
            ("x", "z1, w1", "z1 z1 w1^-1, w1"),
            ("*", "r1, l1", "r1^2, l1^-1 r1 l1 r1^-1, l1"),
            (
                "o3*22",
                "x1, y1, a1, g1, r1, r2, r3, l1",
                "x1^-1 y1^-1 x1 y1 a1^-1, g1^3, r1^2, (r1 r2)^2, r2^2, (r2 r3)^2, r3^2, l1^-1 r1 l1 r3^-1, a1 g1 l1",
            ),
            (
                "3*22x",
                "g1, r1, r2, r3, l1, z1, w1",
                "g1^3, r1^2, (r1 r2)^2, r2^2, (r2 r3)^2, r3^2, l1^-1 r1 l1 r3^-1, z1 z1 w1^-1, g1 l1 w1",
            ),
            ("1", "", ""),
        ];
        for (signature, generators, relators) in cases {
            assert_eq!(
                texts(signature),
                (generators.to_owned(), relators.to_owned()),
                "{signature}"
            );
        }
    }

    #[test]
    fn conway_presentation_numbers_reflections_across_boundaries() {
        assert_eq!(
            texts("**"),
            (
                "r1, l1, r2, l2".to_owned(),
                "r1^2, l1^-1 r1 l1 r1^-1, r2^2, l2^-1 r2 l2 r2^-1, l1 l2".to_owned()
            )
        );
    }

    #[test]
    fn conway_presentation_rejects_an_infinite_order() {
        for (text, shown) in [("*(inf)22", "*(inf)22"), ("(inf)(inf)", "(inf)(inf)")] {
            let error = conway_presentation(&signature(text)).err();
            assert_eq!(
                error.map(|error| (error.kind(), error.to_string())),
                Some((
                    ErrorKind::UnsupportedSignature,
                    format!(
                        "cannot build presentation: the signature {shown} has an infinite order"
                    )
                ))
            );
        }
    }
}
