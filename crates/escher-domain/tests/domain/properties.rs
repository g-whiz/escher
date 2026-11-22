use astro_float_num::BigFloat;
use escher_domain::FundamentalDomain;
use escher_geometry::hyperbolic::{ORIGIN, inverse, side, to_disk, translation};
use escher_signature::Signature;
use proptest::array::uniform3;
use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, prop_assert, prop_oneof, proptest};
use proptest::sample::select;

use crate::common::{cone_text, domain, is_hyperbolic, kite, order_text, text, triangles};
use crate::reference::{largest_entry_from_identity, power};
use escher_reference::{Matrix, Reference, Worst, div, error, int, matrix, max, mul, num, product};

const INSIDE_TOLERANCE: f64 = 1e-9;
const CENTRE_TOLERANCE: f64 = 1e-9;
const CHECK_AGREEMENT_TOLERANCE: f64 = 1e-9;
const FRAME_AGREEMENT_TOLERANCE: f64 = 2.0;
const NAMED_TRIPLES: usize = 30;

fn hyperbolic_triple() -> impl Strategy<Value = [u32; 3]> {
    prop_oneof![uniform3(2_u32..=100), uniform3(90_u32..=100)]
        .prop_filter("the triangle is hyperbolic", |orders| {
            is_hyperbolic(*orders)
        })
}

fn order() -> impl Strategy<Value = String> {
    prop_oneof![
        4 => (2_u32..=9).prop_map(order_text),
        2 => (10_u32..=110).prop_map(order_text),
                1 => Just("(inf)".to_owned()),
    ]
}

fn structured_text() -> impl Strategy<Value = String> {
    (
        0_usize..=1,
        0_usize..=1,
        vec(order(), 0..=4),
        vec(vec(order(), 0..=4), 0..=2),
    )
        .prop_map(|(handles, crosscaps, cones, boundaries)| {
            let mut text = "o".repeat(handles);
            text.push_str(&cones.concat());
            for corners in boundaries {
                text.push('*');
                text.push_str(&corners.concat());
            }
            text.push_str(&"x".repeat(crosscaps));
            text
        })
}

fn triangle_text() -> impl Strategy<Value = String> {
    vec(order(), 3).prop_map(|corners| format!("*{}", corners.concat()))
}

fn loose_text() -> impl Strategy<Value = String> {
    vec(
        select(vec![
            '*', 'x', 'o', '1', '2', '3', '7', '9', '0', '(', ')', 'i', 'n', 'f',
        ]),
        0..=12,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

fn any_text() -> impl Strategy<Value = String> {
    prop_oneof![structured_text(), triangle_text(), loose_text()]
}

fn counterclockwise(built: &FundamentalDomain) -> bool {
    let [first, second, third] = [0, 1, 2].map(|index| to_disk(built.vertices()[index]));
    (second - first).perp_dot(third - first) > 0.0
}

proptest! {
    #[test]
    fn random_triples_build_and_pass_their_checks(orders in hyperbolic_triple()) {
        for (built, input) in [(domain(orders), text(orders)), (kite(orders), cone_text(orders))] {
            let verification = built.verify();
            prop_assert!(verification.passed(), "{input}: {verification:?}");
            prop_assert!(counterclockwise(&built), "{input}");
            for wall in built.sides() {
                for vertex in built.vertices() {
                    let outside = side(wall.covector(), *vertex) / vertex.z;
                    prop_assert!(outside <= INSIDE_TOLERANCE, "{input}: {outside:e}");
                }
            }
            let offset = (built.centre() - ORIGIN).abs().max_element();
            prop_assert!(offset <= CENTRE_TOLERANCE, "{input}: {offset:e}");
        }
    }

    #[test]
    fn any_signature_builds_without_a_panic_and_a_domain_passes(text in any_text()) {
        if let Ok(signature) = text.parse::<Signature>()
            && let Ok(built) = FundamentalDomain::build(&signature)
        {
            prop_assert!(built.verify().passed(), "{text}: {:?}", built.verify());
        }
    }
}

fn exact_relator(rotation: &Matrix, order: u32) -> BigFloat {
    div(
        &largest_entry_from_identity(&power(rotation, order)),
        &int(order),
    )
}

struct Frames {
    domain: BigFloat,
    corner: BigFloat,
    conditioning: BigFloat,
}

fn exact_frames(built: &FundamentalDomain, orders: [u32; 3]) -> [Frames; 3] {
    let sides = built.sides();
    [0, 1, 2].map(|corner| {
        let pairings = [sides[corner].pairing(), sides[(corner + 1) % 3].pairing()]
            .map(|pairing| matrix(pairing.to_cols_array()));
        let rotation = product(&pairings[0], &pairings[1]);
        let frame = translation(built.vertices()[corner]);
        let conjugate = product(
            &product(&matrix(inverse(frame).to_cols_array()), &rotation),
            &matrix(frame.to_cols_array()),
        );
        let largest_frame_entry = frame
            .to_cols_array()
            .into_iter()
            .map(f64::abs)
            .fold(0.0, f64::max);
        Frames {
            domain: exact_relator(&rotation, orders[corner]),
            corner: exact_relator(&conjugate, orders[corner]),
            conditioning: num(9.0 * largest_frame_entry * largest_frame_entry),
        }
    })
}

#[test]
fn relator_check_agrees_with_the_exact_powers_in_both_frames() {
    let mut reference = Reference::new();
    let mut check_agreement = Worst::new(
        "relator check against the exact power in the corner frame, absolute",
        NAMED_TRIPLES,
    );
    let mut frame_agreement = Worst::new(
        "exact power in one frame over the other times the conditioning of the frame",
        2 * 3 * NAMED_TRIPLES,
    );
    for orders in triangles() {
        let built = domain(orders);
        let mut corner_frame_largest = int(0);
        for (corner, frames) in exact_frames(&built, orders).iter().enumerate() {
            let input = || format!("corner {corner} of {}", text(orders));
            frame_agreement.record(
                &frames.domain,
                &mul(&frames.conditioning, &frames.corner),
                input,
            );
            frame_agreement.record(
                &frames.corner,
                &mul(&frames.conditioning, &frames.domain),
                input,
            );
            corner_frame_largest = max(&corner_frame_largest, &frames.corner);
        }
        let residual = built.verify().relators().residual();
        check_agreement.record(&error(residual, &corner_frame_largest), &int(1), || {
            text(orders)
        });
    }
    check_agreement.assert_at_most(&mut reference, CHECK_AGREEMENT_TOLERANCE);
    frame_agreement.assert_at_most(&mut reference, FRAME_AGREEMENT_TOLERANCE);
}
