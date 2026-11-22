use core::f64::consts::PI;

use astro_float_num::BigFloat;
use escher_domain::{FundamentalDomain, Orientation, PairingKind};
use escher_signature::Geometry;
use glam::DVec3;
use proptest::prelude::{prop_assert, proptest};

use crate::common::{
    EUCLIDEAN_TRIANGLES, LargestResiduals, aspects, assert_same_domain, big, domain,
    largest_coordinate_error, rectangle, signature, text,
};
use crate::reference::{Polygon, ReferenceExt};
use escher_reference::{
    Matrix, Reference, Vector, Worst, add, div, error, int, matrix, max, mul, num, sub,
};

pub(crate) const VERTEX_BOUND: f64 = 2e-15;
const COVECTOR_BOUND: f64 = 2e-15;
const PAIRING_BOUND: f64 = 4e-15;
const LENGTH_BOUND: f64 = 4e-15;
const ANGLE_BOUND: f64 = 2e-15;
const AREA_BOUND: f64 = 2e-15;
const CENTRE_BOUND: f64 = 1e-15;
const ASPECT_BOUND: f64 = 4e-15;
const SYMMHUB_BOUND: f64 = 2e-15;

const EUCLIDEAN_DOMAINS: usize = 72;

const EUCLIDEAN_SIDES: usize = 3 * 3 + 4 * 69;

const SPELLINGS: [(&str, &str); 7] = [
    ("*236", "*632"),
    ("*263", "*632"),
    ("*326", "*632"),
    ("*362", "*632"),
    ("*623", "*632"),
    ("*424", "*442"),
    ("*244", "*442"),
];

fn largest_entry_error(computed: glam::DMat3, exact: &Matrix) -> BigFloat {
    let computed = matrix(computed.to_cols_array());
    let mut largest = int(0);
    for (computed_row, exact_row) in computed.iter().zip(exact) {
        for (value, exact) in computed_row.iter().zip(exact_row) {
            largest = max(&largest, &sub(value, exact).abs());
        }
    }
    largest
}

struct Quantities {
    worst: [Worst; 7],
}

impl Quantities {
    fn new() -> Self {
        let names = [
            "vertex, largest coordinate error",
            "covector, largest coordinate error",
            "pairing, largest entry error",
            "side length, absolute error",
            "corner angle, absolute error",
            "area, absolute error",
            "centre, largest coordinate distance from the origin",
        ];
        let samples = [
            EUCLIDEAN_SIDES,
            EUCLIDEAN_SIDES,
            EUCLIDEAN_SIDES,
            EUCLIDEAN_SIDES,
            EUCLIDEAN_SIDES,
            EUCLIDEAN_DOMAINS,
            EUCLIDEAN_DOMAINS,
        ];
        let mut worst = names
            .into_iter()
            .zip(samples)
            .map(|(name, samples)| Worst::new(format!("euclidean {name}"), samples));
        Self {
            worst: [(); 7].map(|()| worst.next().expect("seven quantities have names")),
        }
    }

    fn record(&mut self, built: &FundamentalDomain, exact: &Polygon, input: &str) {
        let input = || input.to_owned();
        let [vertices, covectors, pairings, lengths, angles, area, centre] = &mut self.worst;
        for (vertex, exact) in built.vertices().iter().zip(&exact.vertices) {
            vertices.record(&largest_coordinate_error(*vertex, exact), &int(1), input);
        }
        for (side, (covector, pairing)) in built
            .sides()
            .iter()
            .zip(exact.covectors.iter().zip(&exact.pairings))
        {
            covectors.record(
                &largest_coordinate_error(side.covector(), covector),
                &int(1),
                input,
            );
            pairings.record(
                &largest_entry_error(side.pairing(), pairing),
                &int(1),
                input,
            );
        }
        for (length, exact) in built.side_lengths().into_iter().zip(&exact.lengths) {
            lengths.record(&error(length, exact), &int(1), input);
        }
        for (angle, exact) in built.corner_angles().into_iter().zip(&exact.angles) {
            angles.record(&error(angle, exact), &int(1), input);
        }
        area.record(&error(built.area(), &exact.area), &int(1), input);
        centre.record(
            &largest_coordinate_error(built.centre(), &big(DVec3::Z)),
            &int(1),
            input,
        );
    }

    fn assert_within_bounds(&self, reference: &mut Reference) {
        let bounds = [
            VERTEX_BOUND,
            COVECTOR_BOUND,
            PAIRING_BOUND,
            LENGTH_BOUND,
            ANGLE_BOUND,
            AREA_BOUND,
            CENTRE_BOUND,
        ];
        for (worst, bound) in self.worst.iter().zip(bounds) {
            worst.assert_at_most(reference, bound);
        }
    }
}

fn aspect_error(built: &FundamentalDomain, aspect: f64) -> f64 {
    let lengths = built.side_lengths();
    (lengths[0] / lengths[1] / aspect - 1.0).abs()
}

#[test]
fn euclidean_signatures_build_with_their_sides_and_angles() {
    let cases: [(&str, &[u32]); 4] = [
        ("*632", &[6, 3, 2]),
        ("*442", &[4, 4, 2]),
        ("*333", &[3, 3, 3]),
        ("*2222", &[2, 2, 2, 2]),
    ];
    for (text, orders) in cases {
        let built = match FundamentalDomain::build(&signature(text)) {
            Ok(built) => built,
            Err(error) => panic!("{text} has no domain: {error}"),
        };
        assert_eq!(built.geometry(), Geometry::Euclidean, "{text}");
        assert!(built.verify().passed(), "{text}: {:?}", built.verify());
        assert!(built.reverses_orientation(), "{text}");
        assert_eq!(built.sides().len(), orders.len(), "{text}");
        assert_eq!(built.vertices().len(), orders.len(), "{text}");
        for (index, side) in built.sides().iter().enumerate() {
            assert_eq!(side.partner(), index, "{text}");
            assert_eq!(side.kind(), PairingKind::Mirror, "{text}");
            assert_eq!(side.orientation(), Orientation::Reversing, "{text}");
        }
        for (angle, &order) in built.corner_angles().into_iter().zip(orders) {
            let difference = (angle - PI / f64::from(order)).abs();
            assert!(difference <= ANGLE_BOUND, "{text}: {difference:e}");
        }
    }
    for (spelling, canonical) in SPELLINGS {
        let (first, second) = (
            FundamentalDomain::build(&signature(spelling)),
            FundamentalDomain::build(&signature(canonical)),
        );
        let (Ok(first), Ok(second)) = (first, second) else {
            panic!("{spelling} or {canonical} has no domain");
        };
        assert_same_domain(&first, &second, 2.0 * VERTEX_BOUND, spelling);
    }
}

#[test]
fn euclidean_domains_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut quantities = Quantities::new();
    let mut residuals = LargestResiduals::new();
    let mut ratio = Worst::new(
        "rectangle, side 0 over side 1 against aspect, relative error",
        69,
    );
    for orders in EUCLIDEAN_TRIANGLES {
        let input = text(orders);
        let built = domain(orders);
        quantities.record(&built, &reference.euclidean_triangle(orders), &input);
        residuals.record(&built, &input);
    }
    for aspect in aspects() {
        let input = format!("*2222 at aspect {aspect}");
        let built = rectangle(aspect);
        quantities.record(&built, &reference.rectangle(aspect), &input);
        residuals.record(&built, &input);
        ratio.record(&num(aspect_error(&built, aspect)), &int(1), || {
            input.clone()
        });
    }
    quantities.assert_within_bounds(&mut reference);
    ratio.assert_at_most(&mut reference, ASPECT_BOUND);
    residuals.print_and_assert_samples(EUCLIDEAN_DOMAINS);
}

fn symmhub_lines(reference: &mut Reference, orders: [u32; 3], exact: &Polygon) -> [Vector; 3] {
    let pi = reference.pi();
    let [first_angle, second_angle] = [orders[2], orders[0]].map(|order| div(&pi, &int(order)));
    let offset = reference.sin(&second_angle);
    let lines = [
        [int(0), int(1), int(0)],
        [
            reference.sin(&first_angle).neg(),
            reference.cos(&first_angle),
            int(0),
        ],
        [
            reference.sin(&second_angle),
            reference.cos(&second_angle),
            offset.neg(),
        ],
    ];
    lines.map(|line| {
        let shift = add(
            &mul(&line[0], &exact.mean[0]),
            &mul(&line[1], &exact.mean[1]),
        );
        [line[0].clone(), line[1].clone(), add(&line[2], &shift)]
    })
}

#[test]
fn euclidean_triangle_walls_are_the_lines_of_symmhub() {
    let mut reference = Reference::new();
    let mut worst = Worst::new("wall against the line of SymmHub, up to sign", 9);
    for orders in EUCLIDEAN_TRIANGLES {
        let exact = reference.euclidean_triangle(orders);
        let lines = symmhub_lines(&mut reference, orders, &exact);
        let built = domain(orders);
        let mut matched = Vec::new();
        for (index, side) in built.sides().iter().enumerate() {
            let covector = side.covector();
            let mut best: Option<(usize, BigFloat)> = None;
            for (line_index, line) in lines.iter().enumerate() {
                let negated = [line[0].neg(), line[1].neg(), line[2].neg()];
                let direct = largest_coordinate_error(covector, line);
                let opposite = largest_coordinate_error(covector, &negated);
                let distance = if opposite < direct { opposite } else { direct };
                if best
                    .as_ref()
                    .is_none_or(|(_, smallest)| distance < *smallest)
                {
                    best = Some((line_index, distance));
                }
            }
            let Some((line_index, distance)) = best else {
                panic!("SymmHub gives no line");
            };
            worst.record(&distance, &int(1), || {
                format!("side {index} and line {line_index} of {}", text(orders))
            });
            matched.push(line_index);
        }
        println!(
            "{}: sides 0, 1, and 2 match the lines {matched:?}",
            text(orders)
        );
        let mut sorted = matched.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, [0, 1, 2], "{}: {matched:?}", text(orders));
    }
    worst.assert_at_most(&mut reference, SYMMHUB_BOUND);
}

proptest! {
    #[test]
    fn rectangles_pass_their_checks_with_right_angles_and_their_aspect(aspect in 0.125..=8.0) {
        let built = rectangle(aspect);
        prop_assert!(built.verify().passed(), "{aspect}: {:?}", built.verify());
        for angle in built.corner_angles() {
            let difference = (angle - PI / 2.0).abs();
            prop_assert!(difference <= ANGLE_BOUND, "{aspect}: {difference:e}");
        }
        let ratio = aspect_error(&built, aspect);
        prop_assert!(ratio <= ASPECT_BOUND, "{aspect}: {ratio:e}");
    }
}
