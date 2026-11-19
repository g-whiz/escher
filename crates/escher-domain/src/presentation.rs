use escher_group::{Letter, Presentation, Relator, Word};

use crate::domain::{FundamentalDomain, PairingKind};
use crate::error::{Error, PairingDefect, Result};
use crate::verify::{CycleDefect, SideKinds, corner_pairs, side_kinds, vertex_cycles};

pub(crate) const MAX_SIDES: usize = 256;

pub(crate) fn side_presentation(domain: &FundamentalDomain) -> Result<Presentation> {
    let count = domain.sides.len();
    if domain.corners.len() != count {
        return Err(Error::invalid_pairing(PairingDefect::CornerCount));
    }
    if count > MAX_SIDES {
        return Err(Error::invalid_pairing(PairingDefect::TooManySides));
    }
    let relators = match side_kinds(domain) {
        SideKinds::Mirrors => mirror_relators(domain)?,
        SideKinds::Paired | SideKinds::Mixed => paired_relators(domain)?,
    };
    let names = (0..count).map(|side| format!("s{side}")).collect();
    Presentation::new(names, relators).map_err(Error::rejected_presentation)
}

fn mirror_relators(domain: &FundamentalDomain) -> Result<Vec<Relator>> {
    let pairs =
        corner_pairs(domain).ok_or_else(|| Error::invalid_pairing(PairingDefect::CornerCount))?;
    let mut relators = Vec::with_capacity(2 * pairs.len());
    for side in 0..domain.sides.len() {
        relators.push(relator(&[side], 2)?);
    }
    for pair in pairs {
        relators.push(relator(&pair.sides, pair.order)?);
    }
    Ok(relators)
}

fn paired_relators(domain: &FundamentalDomain) -> Result<Vec<Relator>> {
    let cycles = vertex_cycles(domain).map_err(|defect| {
        Error::invalid_pairing(match defect {
            CycleDefect::Open => PairingDefect::OpenCycle,
            CycleDefect::Orders => PairingDefect::CycleOrders,
        })
    })?;
    let mut relators = Vec::new();
    for (index, side) in domain.sides.iter().enumerate() {
        match side.kind {
            PairingKind::Mirror => relators.push(relator(&[index], 2)?),
            PairingKind::Rotation | PairingKind::Translation | PairingKind::GlideReflection => {
                if side.partner > index {
                    relators.push(relator(&[index, side.partner], 1)?);
                }
            }
        }
    }
    for cycle in cycles {
        relators.push(relator(&cycle.sides, cycle.order)?);
    }
    Ok(relators)
}

fn relator(sides: &[usize], exponent: u32) -> Result<Relator> {
    let base = sides
        .iter()
        .map(|&side| {
            u8::try_from(side)
                .ok()
                .map(Letter::generator)
                .ok_or_else(|| Error::invalid_pairing(PairingDefect::TooManySides))
        })
        .collect::<Result<Word>>()?;
    Relator::new(base, exponent).map_err(Error::rejected_presentation)
}

#[cfg(test)]
mod tests {
    use escher_geometry::hyperbolic;
    use glam::{DMat3, DVec3};

    use crate::domain::{Closure, Corner, FundamentalDomain, Orientation, PairingKind, Side};
    use crate::error::ErrorKind;
    use crate::fixtures::two_two_mirror;
    use crate::shared::{built, built_with_aspect, relator_texts};
    use crate::verify::{framed_power_residual, power_residual};

    const RELATOR_TOLERANCE: f64 = 2e-6;

    const CUT: f64 = 0.37;

    fn point(vector: DVec3) -> DVec3 {
        match hyperbolic::normalize_point(vector) {
            Ok(point) => point,
            Err(error) => panic!("{vector:?} is not a point: {error}"),
        }
    }

    fn hexagon() -> FundamentalDomain {
        let kite = built("732");
        let [w0, w1, w2, w3] = kite.vertices[..] else {
            panic!("the kite of 732 has {} vertices", kite.vertices.len());
        };
        let about_w0 = kite.sides[0].pairing;
        let about_w2 = kite.sides[3].pairing;
        let x = point(w2 * (1.0 - CUT) + w3 * CUT);
        let vertices = vec![w0, about_w0 * x, w1, about_w2 * x, w2, x];
        let about_w1 = about_w2 * hyperbolic::inverse(about_w0);
        let pairings = [
            about_w0,
            hyperbolic::inverse(about_w0),
            about_w1,
            hyperbolic::inverse(about_w1),
            hyperbolic::inverse(about_w2),
            about_w2,
        ];
        let sides = pairings
            .into_iter()
            .enumerate()
            .map(|(side, pairing)| {
                let normal =
                    match hyperbolic::wall_through(vertices[(side + 5) % 6], vertices[side]) {
                        Ok(normal) => normal,
                        Err(error) => panic!("side {side} has no wall: {error}"),
                    };
                Side {
                    normal,
                    covector: hyperbolic::covector(normal),
                    pairing,
                    partner: side ^ 1,
                    kind: PairingKind::Rotation,
                    orientation: Orientation::Preserving,
                }
            })
            .collect();
        let corner = |order, copies| Corner { order, copies };
        FundamentalDomain {
            plane: kite.plane,
            corners: vec![
                corner(7, 2),
                corner(1, 1),
                corner(2, 2),
                corner(1, 1),
                corner(3, 2),
                corner(1, 0),
            ],
            sides,
            vertices,
            centre: kite.centre,
            closure: Closure::PerVertex,
        }
    }

    #[derive(Clone, Copy)]
    enum Convention {
        Inverse,
        Pairing,
    }

    fn residual(
        domain: &FundamentalDomain,
        sides: &[usize],
        exponent: u32,
        convention: Convention,
    ) -> f64 {
        let product = sides.iter().fold(DMat3::IDENTITY, |product, &side| {
            let pairing = domain.sides[side].pairing;
            product
                * match convention {
                    Convention::Inverse => hyperbolic::inverse(pairing),
                    Convention::Pairing => pairing,
                }
        });
        let start = (sides[0] + domain.sides.len() - 1) % domain.sides.len();
        framed_power_residual(domain.plane, domain.vertices[start], &[product], exponent)
    }

    #[test]
    fn long_cycle_word_holds_only_in_walk_order_with_the_inverse_pairings() {
        let domain = hexagon();
        assert!(domain.verify().relators().passed(), "{:?}", domain.verify());
        let presentation = match domain.side_presentation() {
            Ok(presentation) => presentation,
            Err(error) => panic!("the hexagon has no side presentation: {error}"),
        };
        assert_eq!(
            relator_texts(&presentation),
            [
                "s0 s1", "s2 s3", "s4 s5", "s1^7", "s2 s4 s0", "s3^2", "s5^3"
            ]
        );
        let cycle = &presentation.relators()[4];
        let sides: Vec<usize> = cycle
            .base()
            .letters()
            .iter()
            .map(|letter| usize::from(letter.index()))
            .collect();
        let exponent = cycle.exponent();
        let walk = residual(&domain, &sides, exponent, Convention::Inverse);
        let reversed_sides: Vec<usize> = sides.iter().rev().copied().collect();
        let reversed = residual(&domain, &reversed_sides, exponent, Convention::Inverse);
        let opposite = residual(&domain, &sides, exponent, Convention::Pairing);
        println!(
            "long cycle word: walk {walk:e}, reversed {reversed:e}, opposite convention {opposite:e}, tolerance {RELATOR_TOLERANCE:e}"
        );
        assert!(walk <= RELATOR_TOLERANCE, "{walk:e}");
        assert!(reversed > 8.0 * RELATOR_TOLERANCE, "{reversed:e}");
        assert!(opposite > 8.0 * RELATOR_TOLERANCE, "{opposite:e}");
        for pair in &presentation.relators()[..3] {
            let sides: Vec<usize> = pair
                .base()
                .letters()
                .iter()
                .map(|letter| usize::from(letter.index()))
                .collect();
            let product = sides.iter().fold(DMat3::IDENTITY, |product, &side| {
                product * hyperbolic::inverse(domain.sides[side].pairing)
            });
            let largest = power_residual(product, pair.exponent());
            assert!(largest <= 1e-9, "{sides:?}: {largest:e}");
        }
    }

    fn assert_invalid(domain: &FundamentalDomain, text: &str, case: &str) {
        match domain.side_presentation() {
            Ok(presentation) => panic!("{case}: gives {presentation:?}"),
            Err(error) => {
                assert_eq!(error.kind(), ErrorKind::InvalidPairing, "{case}");
                assert_eq!(
                    error.to_string(),
                    format!("cannot derive side presentation: {text}"),
                    "{case}"
                );
            }
        }
    }

    #[test]
    fn reflection_triangle_with_a_paired_side_gives_invalid_pairing() {
        let mut domain = built("*732");
        domain.sides[0].kind = PairingKind::Rotation;
        domain.sides[0].orientation = Orientation::Preserving;
        domain.sides[0].partner = 1;
        assert_invalid(
            &domain,
            "the side pairings do not close a vertex cycle",
            "a rotation side in *732",
        );
    }

    #[test]
    fn double_whose_walk_does_not_return_gives_invalid_pairing() {
        for text in ["732", "532", "2222", "22"] {
            let mut domain = built(text);
            domain.sides[1].partner = 1;
            assert_invalid(
                &domain,
                "the side pairings do not close a vertex cycle",
                &format!("{text} with side 1 as its own partner"),
            );
        }
        let mut domain = built("732");
        domain.sides[1].partner = 7;
        assert_invalid(
            &domain,
            "the side pairings do not close a vertex cycle",
            "732 with a partner out of range",
        );
    }

    #[test]
    fn double_with_unequal_orders_in_a_cycle_gives_invalid_pairing() {
        let mut domain = built("732");
        domain.corners[3].order += 1;
        assert_invalid(
            &domain,
            "a vertex cycle joins corners of different orders",
            "732 with vertex 3 of order 3",
        );
    }

    #[test]
    fn double_whose_cycle_copies_do_not_sum_to_two_gives_invalid_pairing() {
        let mut domain = built("732");
        domain.corners[3].copies = 2;
        assert_invalid(
            &domain,
            "a vertex cycle joins corners of different orders",
            "732 with three copies in the cycle of vertices 1 and 3",
        );
    }

    #[test]
    fn domain_with_a_corner_missing_gives_invalid_pairing() {
        for text in ["*732", "732"] {
            let mut domain = built(text);
            domain.corners.pop();
            assert_invalid(
                &domain,
                "the domain has a different number of corners than of sides",
                &format!("{text} with its last corner removed"),
            );
        }
    }

    #[test]
    fn corner_of_order_zero_gives_invalid_pairing_with_the_group_error() {
        for text in ["*732", "732"] {
            let mut domain = built(text);
            domain.corners[0].order = 0;
            let error = match domain.side_presentation() {
                Ok(presentation) => panic!("{text}: gives {presentation:?}"),
                Err(error) => error,
            };
            assert_eq!(error.kind(), ErrorKind::InvalidPairing, "{text}");
            assert_eq!(
                error.to_string(),
                "cannot derive side presentation: the side relators do not form a presentation",
                "{text}"
            );
            let source = core::error::Error::source(&error).map(ToString::to_string);
            assert_eq!(
                source.as_deref(),
                Some("cannot build presentation: the relator exponent is 0"),
                "{text}"
            );
        }
    }

    #[test]
    fn domain_with_more_than_256_sides_gives_invalid_pairing() {
        let mut domain = built("*732");
        let (side, corner) = (domain.sides[0], domain.corners[0]);
        domain.sides.resize(256, side);
        domain.corners.resize(256, corner);
        match domain.side_presentation() {
            Ok(presentation) => assert_eq!(presentation.generator_count(), 256),
            Err(error) => panic!("256 sides give {error}"),
        }
        domain.sides.push(side);
        domain.corners.push(corner);
        assert_invalid(&domain, "the domain has more than 256 sides", "257 sides");
        assert!(
            domain
                .side_presentation()
                .err()
                .is_some_and(|error| core::error::Error::source(&error).is_none())
        );
    }

    #[test]
    fn causes_are_checked_in_order() {
        let mut domain = built("*732");
        domain.sides[0].kind = PairingKind::Rotation;
        domain.corners.pop();
        assert_invalid(
            &domain,
            "the domain has a different number of corners than of sides",
            "mixed sides and a corner removed",
        );
        let mut domain = built("*732");
        let (side, corner) = (domain.sides[0], domain.corners[0]);
        domain.sides.resize(300, side);
        domain.corners.resize(299, corner);
        assert_invalid(
            &domain,
            "the domain has a different number of corners than of sides",
            "300 sides and 299 corners",
        );
        let mut domain = built("22");
        let (side, corner) = (domain.sides[0], domain.corners[0]);
        domain.sides.resize(300, side);
        domain.corners.resize(300, corner);
        domain.sides[1].partner = 1;
        assert_invalid(
            &domain,
            "the domain has more than 256 sides",
            "300 paired sides and an open walk",
        );
        let mut domain = built("732");
        domain.sides[1].partner = 1;
        domain.corners[0].order = 0;
        assert_invalid(
            &domain,
            "the side pairings do not close a vertex cycle",
            "an open walk and a corner of order 0",
        );
    }

    fn presentation_texts(domain: &FundamentalDomain, case: &str) -> Vec<String> {
        match domain.side_presentation() {
            Ok(presentation) => relator_texts(&presentation),
            Err(error) => panic!("{case} has no side presentation: {error}"),
        }
    }

    #[test]
    fn hand_built_domain_of_22_mirror_gives_its_pairing_and_cycle_relators() {
        for aspect in [1.0, 0.125, 8.0] {
            assert_eq!(
                presentation_texts(&two_two_mirror(aspect), "22*"),
                [
                    "s0 s1",
                    "s2^2",
                    "s3 s4",
                    "s5^2",
                    "s1^2",
                    "s2 s1 s5 s0",
                    "s3 s5 s4 s2",
                    "s4^2",
                ],
                "22* at aspect {aspect}"
            );
        }
    }

    fn letters(relator: &escher_group::Relator) -> Vec<usize> {
        relator
            .base()
            .letters()
            .iter()
            .map(|letter| usize::from(letter.index()))
            .collect()
    }

    fn euclidean_residual(
        domain: &FundamentalDomain,
        sides: &[usize],
        exponent: u32,
        convention: Convention,
    ) -> f64 {
        let product = sides.iter().fold(DMat3::IDENTITY, |product, &side| {
            let pairing = domain.sides[side].pairing;
            product
                * match convention {
                    Convention::Inverse => pairing.inverse(),
                    Convention::Pairing => pairing,
                }
        });
        power_residual(product, exponent)
    }

    #[test]
    fn relators_of_domains_with_glides_and_mirrors_evaluate_to_the_identity() {
        for name in ["**", "xx", "*x", "22x", "22*"] {
            for aspect in [1.0, 0.125, 8.0] {
                let domain = if name == "22*" {
                    two_two_mirror(aspect)
                } else {
                    built_with_aspect(name, aspect, None)
                };
                let presentation = match domain.side_presentation() {
                    Ok(presentation) => presentation,
                    Err(error) => panic!("{name} has no side presentation: {error}"),
                };
                for relator in presentation.relators() {
                    let sides = letters(relator);
                    let largest = euclidean_residual(
                        &domain,
                        &sides,
                        relator.exponent(),
                        Convention::Inverse,
                    );
                    assert!(
                        largest <= 1e-9,
                        "{name} at aspect {aspect}, {sides:?}: {largest:e}"
                    );
                }
            }
        }
    }

    #[test]
    fn long_cycle_word_of_a_chain_holds_in_walk_order_with_the_inverse_pairings() {
        let domain = two_two_mirror(1.0);
        let presentation = match domain.side_presentation() {
            Ok(presentation) => presentation,
            Err(error) => panic!("22* has no side presentation: {error}"),
        };
        let chain = &presentation.relators()[5];
        let sides = letters(chain);
        assert_eq!(sides, [2, 1, 5, 0]);
        let exponent = chain.exponent();
        let walk = euclidean_residual(&domain, &sides, exponent, Convention::Inverse);
        let reversed_sides: Vec<usize> = sides.iter().rev().copied().collect();
        let reversed = euclidean_residual(&domain, &reversed_sides, exponent, Convention::Inverse);
        let opposite = euclidean_residual(&domain, &sides, exponent, Convention::Pairing);
        println!(
            "chain of 22*: walk {walk:e}, reversed {reversed:e}, opposite convention {opposite:e}, tolerance {RELATOR_TOLERANCE:e}"
        );
        assert!(walk <= RELATOR_TOLERANCE, "{walk:e}");
    }

    #[test]
    fn domain_with_mirrors_beside_paired_sides_whose_walk_does_not_close_gives_invalid_pairing() {
        let mut domain = built_with_aspect("*x", 1.0, None);
        domain.sides[0].partner = 0;
        assert_invalid(
            &domain,
            "the side pairings do not close a vertex cycle",
            "*x with side 0 as its own partner",
        );
        let mut domain = built_with_aspect("22x", 1.0, None);
        domain.corners[0].order = 1;
        assert_invalid(
            &domain,
            "a vertex cycle joins corners of different orders",
            "22x with vertex 0 alone of order 1",
        );
    }
}
