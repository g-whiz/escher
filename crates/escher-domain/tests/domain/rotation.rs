use core::f64::consts::PI;

use astro_float_num::BigFloat;
use escher_domain::{FundamentalDomain, Orientation, PairingKind, parameters};
use escher_geometry::hyperbolic::{inverse, translation};
use escher_geometry::{euclidean, hyperbolic, spherical};
use escher_signature::Geometry;
use glam::{DMat3, DVec3};
use proptest::prelude::{prop_assert, proptest};

use crate::common::{
    EUCLIDEAN_TRIANGLES, EXAMPLE_TRIPLES, LargestResiduals, all_triangles, aspects,
    assert_same_domain, built, cone_text, domain, kite, largest_coordinate_error,
    named_rotation_polygons, order_text, signature, starred, triangles, with_aspect,
};
use crate::polygons::signature_area;
use crate::reference::{Double, Model, ReferenceExt, largest_entry_from_identity, power};
use crate::shared::corner_orders;
use escher_reference::{
    Matrix, Reference, Worst, add, div, error, int, largest_magnitude, matrix, max, mul, num,
    product, sub,
};

#[derive(Clone, Copy)]
struct Samples {
    domains: usize,
    image_vertices: usize,
    image_walls: usize,
    sides: usize,
    flats: usize,
}

#[derive(Clone, Copy)]
struct Bounds {
    image_vertex: f64,
    image_wall: f64,
    pairing: f64,
    isometry: f64,
    length: f64,
    angle: f64,
    flat: Option<f64>,
    area: f64,
}

const HYPERBOLIC_BOUNDS: Bounds = Bounds {
    image_vertex: 2e-11,
    image_wall: 4e-11,
    pairing: 2e-14,
    isometry: 2e-12,
    length: 1e-11,
    angle: 1e-12,
    flat: None,
    area: 1e-12,
};

const EUCLIDEAN_BOUNDS: Bounds = Bounds {
    image_vertex: 1e-14,
    image_wall: 1e-14,
    pairing: 1e-15,
    isometry: 4e-15,
    length: 1e-14,
    angle: 4e-15,
    flat: Some(1e-15),
    area: 2e-15,
};

const SPHERICAL_BOUNDS: Bounds = Bounds {
    image_vertex: 4e-14,
    image_wall: 2e-14,
    pairing: 1e-15,
    isometry: 2e-14,
    length: 4e-14,
    angle: 1e-13,
    flat: Some(1e-13),
    area: 4e-14,
};

const HYPERBOLIC_SAMPLES: Samples = Samples {
    domains: 294,
    image_vertices: 294,
    image_walls: 588,
    sides: 1176,
    flats: 0,
};

const EUCLIDEAN_SAMPLES: Samples = Samples {
    domains: 72,
    image_vertices: 141,
    image_walls: 213,
    sides: 426,
    flats: 138,
};

const SPHERICAL_SAMPLES: Samples = Samples {
    domains: 16,
    image_vertices: 10,
    image_walls: 26,
    sides: 52,
    flats: 11,
};

const MEASURED_ORDERS: [u32; 6] = [2, 3, 5, 12, 99, 100];

const CHECK_AGREEMENT_TOLERANCE: f64 = 1e-9;

const NAMED_TRIPLES: usize = 30;

const KITE_PAIRS: &[(usize, usize, PairingKind)] =
    &[(0, 1, PairingKind::Rotation), (2, 3, PairingKind::Rotation)];

const KITE_COPIES: &[u32] = &[2, 1, 2, 1];

struct Expected {
    text: &'static str,
    geometry: Geometry,
    orders: &'static [u32],
    copies: &'static [u32],
    pairs: &'static [(usize, usize, PairingKind)],
    flats: &'static [usize],
}

const fn kite_row(
    text: &'static str,
    geometry: Geometry,
    orders: &'static [u32],
    flats: &'static [usize],
) -> Expected {
    Expected {
        text,
        geometry,
        orders,
        copies: KITE_COPIES,
        pairs: KITE_PAIRS,
        flats,
    }
}

const TABLE: [Expected; 16] = [
    kite_row("732", Geometry::Hyperbolic, &[7, 2, 3, 2], &[]),
    kite_row("542", Geometry::Hyperbolic, &[5, 2, 4, 2], &[]),
    kite_row("444", Geometry::Hyperbolic, &[4, 4, 4, 4], &[]),
    kite_row("433", Geometry::Hyperbolic, &[4, 3, 3, 3], &[]),
    kite_row("(12)32", Geometry::Hyperbolic, &[12, 2, 3, 2], &[]),
    kite_row("632", Geometry::Euclidean, &[6, 2, 3, 2], &[]),
    kite_row("442", Geometry::Euclidean, &[4, 2, 4, 2], &[]),
    kite_row("333", Geometry::Euclidean, &[3, 3, 3, 3], &[]),
    Expected {
        text: "2222",
        geometry: Geometry::Euclidean,
        orders: &[2, 2, 2, 2, 2, 2],
        copies: &[2, 1, 1, 2, 1, 1],
        pairs: &[
            (0, 1, PairingKind::Rotation),
            (2, 5, PairingKind::Translation),
            (3, 4, PairingKind::Rotation),
        ],
        flats: &[0, 3],
    },
    kite_row("332", Geometry::Spherical, &[3, 2, 3, 2], &[]),
    kite_row("432", Geometry::Spherical, &[4, 2, 3, 2], &[]),
    kite_row("532", Geometry::Spherical, &[5, 2, 3, 2], &[]),
    kite_row("222", Geometry::Spherical, &[2, 2, 2, 2], &[0, 2]),
    kite_row("722", Geometry::Spherical, &[7, 2, 2, 2], &[2]),
    Expected {
        text: "22",
        geometry: Geometry::Spherical,
        orders: &[2, 2],
        copies: &[2, 2],
        pairs: &[(0, 1, PairingKind::Rotation)],
        flats: &[0, 1],
    },
    Expected {
        text: "55",
        geometry: Geometry::Spherical,
        orders: &[5, 5],
        copies: &[2, 2],
        pairs: &[(0, 1, PairingKind::Rotation)],
        flats: &[],
    },
];

const SPELLINGS: [(&str, &str); 6] = [
    ("237", "732"),
    ("372", "732"),
    ("236", "632"),
    ("244", "442"),
    ("235", "532"),
    ("227", "722"),
];

fn bounds(geometry: Geometry) -> Bounds {
    match geometry {
        Geometry::Hyperbolic => HYPERBOLIC_BOUNDS,
        Geometry::Euclidean => EUCLIDEAN_BOUNDS,
        Geometry::Spherical => SPHERICAL_BOUNDS,
    }
}

fn source(sides: usize, index: usize) -> (usize, bool) {
    let count = sides / 2 + 1;
    if index == 0 {
        (0, false)
    } else if index < count {
        ((count + 1 - index) % count, true)
    } else {
        (index + 2 - count, false)
    }
}

fn image_of(sides: usize, index: usize) -> Option<usize> {
    let count = sides / 2 + 1;
    (index > 0 && index + 1 < count).then(|| count - index)
}

fn original_of(sides: usize, index: usize) -> Option<usize> {
    let count = sides / 2 + 1;
    match index {
        0 => Some(0),
        _ if index + 1 == count => Some(1),
        _ if index >= count => Some(index + 2 - count),
        _ => None,
    }
}

fn to_klein(point: DVec3) -> glam::DVec2 {
    point.truncate() / point.z
}

fn counterclockwise(domain: &FundamentalDomain) -> bool {
    let [first, second, third] = [0, 1, 2].map(|index| domain.vertices()[index]);
    match domain.geometry() {
        Geometry::Hyperbolic => {
            let [first, second, third] = [first, second, third].map(to_klein);
            (second - first).perp_dot(third - first) > 0.0
        }
        Geometry::Euclidean => {
            (second - first)
                .truncate()
                .perp_dot((third - first).truncate())
                > 0.0
        }
        Geometry::Spherical => first.cross(second).dot(third) > 0.0,
    }
}

fn assert_row(expected: &Expected) {
    let text = expected.text;
    let double = built(text);
    let generating = built(&starred(text));
    let verification = double.verify();
    assert!(verification.passed(), "{text}: {verification:?}");
    assert!(!double.reverses_orientation(), "{text}");
    assert_eq!(double.geometry(), expected.geometry, "{text}");
    let count = expected.orders.len();
    assert_eq!(double.sides().len(), count, "{text}");
    assert_eq!(double.vertices().len(), count, "{text}");
    let mut partners = vec![None; count];
    for &(first, second, kind) in expected.pairs {
        partners[first] = Some((second, kind));
        partners[second] = Some((first, kind));
    }
    for (index, side) in double.sides().iter().enumerate() {
        assert_eq!(side.orientation(), Orientation::Preserving, "{text}");
        assert_eq!(
            Some((side.partner(), side.kind())),
            partners[index],
            "{text}: side {index}"
        );
        assert_ne!(side.partner(), index, "{text}: side {index}");
    }
    let bound = bounds(expected.geometry);
    let mut flats = Vec::new();
    for (index, angle) in double.corner_angles().into_iter().enumerate() {
        let exact = f64::from(expected.copies[index]) * PI / f64::from(expected.orders[index]);
        let difference = (angle - exact).abs();
        assert!(
            difference <= bound.angle,
            "{text}: vertex {index}: {difference:e}"
        );
        if (angle - PI).abs() <= bound.angle {
            flats.push(index);
        }
    }
    assert_eq!(flats, expected.flats, "{text}");
    if count >= 3 {
        assert!(counterclockwise(&double), "{text}");
    }
    let wall = generating.sides()[1].pairing();
    for index in 0..count {
        if let Some(image) = image_of(count, index) {
            let exact = wall * generating.vertices()[image];
            let difference = (double.vertices()[index] - exact).abs().max_element();
            let scale = exact.abs().max_element();
            assert!(
                difference <= 2.0 * bound.image_vertex * scale.max(1.0),
                "{text}: vertex {index}: {difference:e}"
            );
        }
    }
}

#[test]
fn rotation_signatures_build_their_doubles_with_the_table_of_pairs() {
    for expected in &TABLE {
        assert_row(expected);
    }
    for (spelling, canonical) in SPELLINGS {
        let first = built(spelling);
        let tolerance = 2.0 * bounds(first.geometry()).image_vertex;
        assert_same_domain(&first, &built(canonical), tolerance, spelling);
    }
    let reflections = EXAMPLE_TRIPLES
        .into_iter()
        .map(domain)
        .chain(["*632", "*442", "*333", "*2222", "*532", "*55"].map(built));
    for reflection in reflections {
        assert!(reflection.reverses_orientation());
    }
}

fn assert_bits(first: DVec3, second: DVec3, input: &str) {
    assert_eq!(
        first.to_array().map(f64::to_bits),
        second.to_array().map(f64::to_bits),
        "{input}"
    );
}

#[test]
fn doubles_keep_their_generating_domain_bit_for_bit() {
    let doubles = TABLE
        .iter()
        .map(|expected| {
            (
                expected.text.to_owned(),
                built(expected.text),
                built(&starred(expected.text)),
            )
        })
        .chain([0.125, 1.618, 8.0].map(|aspect| {
            (
                format!("2222 at aspect {aspect}"),
                with_aspect("2222", aspect),
                with_aspect("*2222", aspect),
            )
        }));
    for (text, double, generating) in doubles {
        let count = double.sides().len();
        for index in 0..count {
            let (original, image) = source(count, index);
            if !image {
                let (side, kept) = (double.sides()[index], generating.sides()[original]);
                let input = format!("{text}: side {index}");
                assert_bits(side.normal(), kept.normal(), &input);
                assert_bits(side.covector(), kept.covector(), &input);
            }
            if let Some(original) = original_of(count, index) {
                assert_bits(
                    double.vertices()[index],
                    generating.vertices()[original],
                    &format!("{text}: vertex {index}"),
                );
            }
        }
        assert_bits(double.centre(), generating.centre(), &text);
    }
}

struct Case {
    text: String,
    double: FundamentalDomain,
    generating: FundamentalDomain,
    orders: Vec<u32>,
    exact: Double,
    model: Model,
}

fn corners(orders: &[u32]) -> Vec<(u32, u32)> {
    let count = orders.len();
    let mut corners = vec![(orders[0], 2)];
    corners.extend((1..count - 1).map(|index| (orders[count - index], 1)));
    corners.push((orders[1], 2));
    corners.extend((1..count - 1).map(|index| (orders[1 + index], 1)));
    corners
}

fn hyperbolic_case(reference: &mut Reference, orders: [u32; 3]) -> Case {
    let exact = reference.triangle(orders);
    let double = reference.double(
        Model::Hyperbolic,
        &exact.vertices,
        &exact.normals,
        &exact.area,
    );
    Case {
        text: cone_text(orders),
        double: kite(orders),
        generating: domain(orders),
        orders: orders.to_vec(),
        exact: double,
        model: Model::Hyperbolic,
    }
}

fn euclidean_cases(reference: &mut Reference) -> Vec<Case> {
    let mut cases = Vec::new();
    for orders in EUCLIDEAN_TRIANGLES {
        let exact = reference.euclidean_triangle(orders);
        cases.push(Case {
            text: cone_text(orders),
            double: kite(orders),
            generating: domain(orders),
            orders: orders.to_vec(),
            exact: reference.double(
                Model::Euclidean,
                &exact.vertices,
                &exact.covectors,
                &exact.area,
            ),
            model: Model::Euclidean,
        });
    }
    for aspect in aspects() {
        let exact = reference.rectangle(aspect);
        cases.push(Case {
            text: format!("2222 at aspect {aspect}"),
            double: with_aspect("2222", aspect),
            generating: with_aspect("*2222", aspect),
            orders: vec![2; 4],
            exact: reference.double(
                Model::Euclidean,
                &exact.vertices,
                &exact.covectors,
                &exact.area,
            ),
            model: Model::Euclidean,
        });
    }
    cases
}

fn spherical_cases(reference: &mut Reference) -> Vec<Case> {
    let mut triples = vec![[3, 3, 2], [4, 3, 2], [5, 3, 2], [2, 2, 2]];
    triples.extend(MEASURED_ORDERS.map(|order| [order, 2, 2]));
    let mut cases = Vec::new();
    for orders in triples {
        let exact = reference.spherical_triangle(orders);
        let text = cone_text(orders);
        cases.push(Case {
            double: built(&text),
            generating: built(&starred(&text)),
            text,
            orders: orders.to_vec(),
            exact: reference.double(
                Model::Spherical,
                &exact.vertices,
                &exact.normals,
                &exact.area,
            ),
            model: Model::Spherical,
        });
    }
    for order in MEASURED_ORDERS {
        let exact = reference.lune(order);
        let text = format!("{}{}", order_text(order), order_text(order));
        cases.push(Case {
            double: built(&text),
            generating: built(&starred(&text)),
            text,
            orders: vec![order; 2],
            exact: reference.double(
                Model::Spherical,
                &exact.vertices,
                &exact.normals,
                &exact.area,
            ),
            model: Model::Spherical,
        });
    }
    cases
}

struct Quantities {
    label: &'static str,
    bounds: Bounds,
    samples: Samples,
    image_vertex: Worst,
    image_wall: Worst,
    length: Worst,
    angle: Worst,
    flat: Worst,
    area: Worst,
    residuals: LargestResiduals,
}

impl Quantities {
    fn new(model: Model) -> Self {
        let label = label(model);
        let samples = model_samples(model);
        let worst =
            |name: &str, minimum: usize| Worst::new(format!("{label} doubles, {name}"), minimum);
        Self {
            label,
            bounds: model_bounds(model),
            samples,
            image_vertex: worst(
                "image vertex, largest coordinate error",
                samples.image_vertices,
            ),
            image_wall: worst(
                "image normal or covector, largest coordinate error",
                samples.image_walls,
            ),
            length: worst("side length, absolute error", samples.sides),
            angle: worst(
                "corner angle, absolute error against copies pi / n",
                samples.sides,
            ),
            flat: worst("flat vertex, absolute error against pi", samples.flats),
            area: worst("area, absolute error", samples.domains),
            residuals: LargestResiduals::new(),
        }
    }

    fn record(&mut self, reference: &mut Reference, case: &Case) {
        let input = || case.text.clone();
        let (double, exact) = (&case.double, &case.exact);
        let count = double.vertices().len();
        for index in 0..count {
            if image_of(count, index).is_some() {
                let scale = match case.model {
                    Model::Hyperbolic => largest_magnitude(&exact.vertices[index]),
                    Model::Euclidean | Model::Spherical => int(1),
                };
                self.image_vertex.record(
                    &largest_coordinate_error(double.vertices()[index], &exact.vertices[index]),
                    &scale,
                    input,
                );
            }
            if source(count, index).1 {
                let side = double.sides()[index];
                let wall = match case.model {
                    Model::Euclidean => side.covector(),
                    Model::Hyperbolic | Model::Spherical => side.normal(),
                };
                self.image_wall.record(
                    &largest_coordinate_error(wall, &exact.walls[index]),
                    &int(1),
                    input,
                );
            }
        }
        for (length, exact) in double.side_lengths().into_iter().zip(&exact.lengths) {
            self.length.record(&error(length, exact), &int(1), input);
        }
        let pi = reference.pi();
        for (angle, (order, copies)) in double
            .corner_angles()
            .into_iter()
            .zip(corners(&case.orders))
        {
            let expected = div(&mul(&int(copies), &pi), &int(order));
            let difference = error(angle, &expected);
            self.angle.record(&difference, &int(1), input);
            if copies == order {
                self.flat.record(&difference, &int(1), input);
            }
        }
        self.area
            .record(&error(double.area(), &exact.area), &int(1), input);
        self.residuals.record(double, &case.text);
    }

    fn assert_within_bounds(&self, reference: &mut Reference) {
        println!("{} doubles:", self.label);
        self.image_vertex
            .assert_at_most(reference, self.bounds.image_vertex);
        self.image_wall
            .assert_at_most(reference, self.bounds.image_wall);
        self.length.assert_at_most(reference, self.bounds.length);
        self.angle.assert_at_most(reference, self.bounds.angle);
        match self.bounds.flat {
            Some(bound) => self.flat.assert_at_most(reference, bound),
            None => assert_eq!(self.flat.count(), 0, "{} doubles", self.label),
        }
        self.area.assert_at_most(reference, self.bounds.area);
        self.residuals
            .print_and_assert_samples(self.samples.domains);
    }
}

fn all_cases(reference: &mut Reference) -> Vec<Case> {
    let mut cases: Vec<Case> = all_triangles()
        .into_iter()
        .map(|orders| hyperbolic_case(reference, orders))
        .collect();
    cases.extend(euclidean_cases(reference));
    cases.extend(spherical_cases(reference));
    cases
}

fn label(model: Model) -> &'static str {
    match model {
        Model::Hyperbolic => "hyperbolic",
        Model::Euclidean => "euclidean",
        Model::Spherical => "spherical",
    }
}

fn model_samples(model: Model) -> Samples {
    match model {
        Model::Hyperbolic => HYPERBOLIC_SAMPLES,
        Model::Euclidean => EUCLIDEAN_SAMPLES,
        Model::Spherical => SPHERICAL_SAMPLES,
    }
}

fn model_bounds(model: Model) -> Bounds {
    match model {
        Model::Hyperbolic => HYPERBOLIC_BOUNDS,
        Model::Euclidean => EUCLIDEAN_BOUNDS,
        Model::Spherical => SPHERICAL_BOUNDS,
    }
}

const MODELS: [Model; 3] = [Model::Hyperbolic, Model::Euclidean, Model::Spherical];

#[test]
fn doubles_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let cases = all_cases(&mut reference);
    for model in MODELS {
        let mut quantities = Quantities::new(model);
        for case in cases.iter().filter(|case| case.model == model) {
            quantities.record(&mut reference, case);
        }
        quantities.assert_within_bounds(&mut reference);
    }
}

fn isometry_defect(model: Model, pairing: DMat3) -> f64 {
    match model {
        Model::Hyperbolic => hyperbolic::isometry_defect(pairing),
        Model::Euclidean => euclidean::isometry_defect(pairing),
        Model::Spherical => spherical::isometry_defect(pairing),
    }
}

fn exact_product(first: DMat3, second: DMat3) -> Matrix {
    product(
        &matrix(first.to_cols_array()),
        &matrix(second.to_cols_array()),
    )
}

fn largest_entry_error(computed: DMat3, exact: &Matrix) -> BigFloat {
    let computed = matrix(computed.to_cols_array());
    let mut largest = int(0);
    for (computed_row, exact_row) in computed.iter().zip(exact) {
        for (value, exact) in computed_row.iter().zip(exact_row) {
            largest = max(&largest, &sub(value, exact).abs());
        }
    }
    largest
}

#[test]
fn pairings_are_accurately_rounded_products_of_the_stored_reflections() {
    let mut reference = Reference::new();
    let cases = all_cases(&mut reference);
    for model in MODELS {
        let bound = model_bounds(model);
        let name = label(model);
        let sides = model_samples(model).sides;
        let mut pairing = Worst::new(
            format!("{name} doubles, pairing, largest entry error"),
            sides,
        );
        let mut defect = Worst::new(
            format!("{name} doubles, isometry defect of a pairing"),
            sides,
        );
        let mut entries = 0;
        let mut misrounded = Vec::new();
        let mut plain_misrounded = 0;
        for case in cases.iter().filter(|case| case.model == model) {
            let reflections: Vec<DMat3> = case
                .generating
                .sides()
                .iter()
                .map(|side| side.pairing())
                .collect();
            let wall = reflections[1];
            let count = case.double.sides().len();
            for (index, side) in case.double.sides().iter().enumerate() {
                let (original, image) = source(count, index);
                let (first, second) = if image {
                    (reflections[original], wall)
                } else {
                    (wall, reflections[original])
                };
                let input = || format!("side {index} of {}", case.text);
                let exact = exact_product(first, second);
                let (computed, plain) = (side.pairing(), first * second);
                for column in 0..3 {
                    for row in 0..3 {
                        let exact = &exact[row][column];
                        entries += 1;
                        if !is_nearest(computed.col(column)[row], exact) {
                            misrounded.push(format!("entry ({row}, {column}) of {}", input()));
                        }
                        if !is_nearest(plain.col(column)[row], exact) {
                            plain_misrounded += 1;
                        }
                    }
                }
                pairing.record(&largest_entry_error(computed, &exact), &int(1), input);
                defect.record(&num(isometry_defect(model, side.pairing())), &int(1), input);
            }
        }
        println!(
            "{name} doubles: {} of {entries} entries of the pairings are not the exact product rounded to nearest, against {plain_misrounded} for the plain product",
            misrounded.len()
        );
        assert!(entries >= 9 * sides, "{name}: {entries} entries");
        assert!(misrounded.is_empty(), "{name}: {misrounded:?}");
        pairing.assert_at_most(&mut reference, bound.pairing);
        defect.assert_at_most(&mut reference, bound.isometry);
    }
}

fn is_nearest(value: f64, exact: &BigFloat) -> bool {
    let below = div(&add(&num(value.next_down()), &num(value)), &int(2));
    let above = div(&add(&num(value), &num(value.next_up())), &int(2));
    if *exact > below && *exact < above {
        true
    } else if *exact == below || *exact == above {
        value.to_bits().is_multiple_of(2)
    } else {
        false
    }
}

fn cycles(domain: &FundamentalDomain) -> Vec<(usize, DMat3)> {
    let sides = domain.sides();
    let count = sides.len();
    let mut visited = vec![false; count];
    let mut found = Vec::new();
    for start in 0..count {
        if visited[start] {
            continue;
        }
        let (mut vertex, mut transformation) = (start, DMat3::IDENTITY);
        loop {
            visited[vertex] = true;
            let side = sides[(vertex + 1) % count];
            transformation = side.pairing() * transformation;
            vertex = side.partner();
            if vertex == start {
                break;
            }
        }
        found.push((start, transformation));
    }
    found
}

fn exact_cycle_residual(
    domain: &FundamentalDomain,
    start: usize,
    cycle: DMat3,
    order: u32,
) -> BigFloat {
    let frame = translation(domain.vertices()[start]);
    let conjugate = product(
        &product(
            &matrix(inverse(frame).to_cols_array()),
            &matrix(cycle.to_cols_array()),
        ),
        &matrix(frame.to_cols_array()),
    );
    div(
        &largest_entry_from_identity(&power(&conjugate, order)),
        &int(order),
    )
}

#[test]
fn cycle_relator_check_agrees_with_the_exact_powers() {
    let mut reference = Reference::new();
    let mut agreement = Worst::new(
        "relator check of a kite against the exact powers of its cycles, absolute",
        NAMED_TRIPLES,
    );
    for orders in triangles() {
        let double = kite(orders);
        let corner_orders = [orders[0], orders[2], orders[1], orders[2]];
        let mut largest = int(0);
        let found = cycles(&double);
        assert_eq!(
            found.iter().map(|(start, _)| *start).collect::<Vec<_>>(),
            [0, 1, 2],
            "{}",
            cone_text(orders)
        );
        for (start, cycle) in found {
            largest = max(
                &largest,
                &exact_cycle_residual(&double, start, cycle, corner_orders[start]),
            );
        }
        let residual = double.verify().relators().residual();
        agreement.record(&error(residual, &largest), &int(1), || cone_text(orders));
    }
    agreement.assert_at_most(&mut reference, CHECK_AGREEMENT_TOLERANCE);
}

proptest! {
    #[test]
    fn doubled_rectangles_pass_their_checks_with_their_angles_and_area(aspect in 0.125..=8.0) {
        let double = with_aspect("2222", aspect);
        prop_assert!(double.verify().passed(), "{aspect}: {:?}", double.verify());
        for (index, angle) in double.corner_angles().into_iter().enumerate() {
            let exact = if index == 0 || index == 3 { PI } else { PI / 2.0 };
            let difference = (angle - exact).abs();
            prop_assert!(difference <= EUCLIDEAN_BOUNDS.angle, "{aspect}: {index}: {difference:e}");
        }
        let difference = (double.area() - 2.0).abs();
        prop_assert!(difference <= EUCLIDEAN_BOUNDS.area, "{aspect}: {difference:e}");
    }
}

pub(crate) fn two_vertex_cycle_residual(double: &FundamentalDomain, order: u32) -> f64 {
    let sides = double.sides();
    let vertex = double.vertices()[1];
    let cycle = sides[0].pairing() * sides[2].pairing();
    let fixed = (cycle * vertex - vertex).abs().max_element() / vertex.z;
    let frame = translation(vertex);
    let rotation = inverse(frame) * cycle * frame;
    let mut power = rotation;
    for _ in 1..order {
        power *= rotation;
    }
    let relator = (power - DMat3::IDENTITY)
        .to_cols_array()
        .into_iter()
        .map(f64::abs)
        .fold(0.0, f64::max)
        / f64::from(order);
    fixed.max(relator)
}

const SMALL_ROTATION_POLYGONS: [&str; 4] = ["3222", "3333", "5432", "22222"];

const MISROUNDED_ENTRY_BOUND: f64 = 1e-30;

fn cycle_sets(domain: &FundamentalDomain) -> Vec<Vec<usize>> {
    let sides = domain.sides();
    let count = sides.len();
    let mut visited = vec![false; count];
    let mut found = Vec::new();
    for start in 0..count {
        if visited[start] {
            continue;
        }
        let (mut vertex, mut cycle) = (start, Vec::new());
        loop {
            visited[vertex] = true;
            cycle.push(vertex);
            vertex = sides[(vertex + 1) % count].partner();
            if vertex == start || cycle.len() > count {
                break;
            }
        }
        cycle.sort_unstable();
        found.push(cycle);
    }
    found.sort();
    found
}

fn expected_cycle_sets(corners: usize) -> Vec<Vec<usize>> {
    let mut expected = vec![vec![0], vec![corners - 1]];
    expected.extend((2..corners).map(|j| vec![corners - j, corners + j - 2]));
    expected.sort();
    expected
}

fn assert_rotation_polygon(text: &str) {
    let double = built(text);
    let generating = built(&starred(text));
    let verification = double.verify();
    assert!(verification.passed(), "{text}: {verification:?}");
    assert!(!double.reverses_orientation(), "{text}");
    assert!(generating.reverses_orientation(), "{text}");
    assert!(parameters(&signature(text)).is_empty(), "{text}");
    let orders = corner_orders(&signature(&starred(text)));
    let corner_count = orders.len();
    let count = 2 * corner_count - 2;
    assert_eq!(double.geometry(), Geometry::Hyperbolic, "{text}");
    assert_eq!(double.sides().len(), count, "{text}");
    assert_eq!(double.vertices().len(), count, "{text}");
    for (index, side) in double.sides().iter().enumerate() {
        assert_eq!(side.orientation(), Orientation::Preserving, "{text}");
        let (original, _) = source(count, index);
        let expected = if original == 0 || original == 2 {
            PairingKind::Rotation
        } else {
            PairingKind::Translation
        };
        assert_eq!(side.kind(), expected, "{text}: side {index}");
        assert_eq!(
            side.partner(),
            (2 * corner_count - 1 - index) % count,
            "{text}: side {index}"
        );
    }
    let tolerance = verification.angles().tolerance();
    let mut flats = Vec::new();
    for (index, (angle, (order, copies))) in double
        .corner_angles()
        .into_iter()
        .zip(corners(&orders))
        .enumerate()
    {
        let exact = f64::from(copies) * PI / f64::from(order);
        let difference = (angle - exact).abs();
        assert!(
            difference <= tolerance,
            "{text}: vertex {index}: {difference:e}"
        );
        if (angle - PI).abs() <= tolerance {
            flats.push(index);
        }
    }
    let mut expected_flats = Vec::new();
    if orders.iter().all(|&order| order == 2) {
        expected_flats.push(0);
    }
    if orders[1] == 2 {
        expected_flats.push(corner_count - 1);
    }
    assert_eq!(flats, expected_flats, "{text}");
    assert_eq!(
        cycle_sets(&double),
        expected_cycle_sets(corner_count),
        "{text}"
    );
    assert!(counterclockwise(&double), "{text}");
    let wall = generating.sides()[1].pairing();
    for index in 0..count {
        if let Some(image) = image_of(count, index) {
            assert_bits(
                double.vertices()[index],
                wall * generating.vertices()[image],
                &format!("{text}: image vertex {index}"),
            );
        }
        let (original, image) = source(count, index);
        if !image {
            let (side, kept) = (double.sides()[index], generating.sides()[original]);
            let input = format!("{text}: side {index}");
            assert_bits(side.normal(), kept.normal(), &input);
            assert_bits(side.covector(), kept.covector(), &input);
        }
        if let Some(original) = original_of(count, index) {
            assert_bits(
                double.vertices()[index],
                generating.vertices()[original],
                &format!("{text}: vertex {index}"),
            );
        }
    }
    assert_bits(double.centre(), generating.centre(), text);
}

#[test]
fn small_rotation_polygons_build_their_doubles_with_the_table_of_pairs() {
    for text in SMALL_ROTATION_POLYGONS {
        assert_rotation_polygon(text);
    }
}

#[test]
fn named_rotation_polygons_build_their_doubles() {
    for text in named_rotation_polygons() {
        assert_rotation_polygon(&text);
    }
}

#[test]
fn pairings_of_rotation_polygons_are_accurately_rounded_products() {
    let mut entries = 0;
    let mut misrounded: Vec<String> = Vec::new();
    let mut largest = int(0);
    for text in named_rotation_polygons() {
        let double = built(&text);
        let generating = built(&starred(&text));
        let reflections: Vec<DMat3> = generating
            .sides()
            .iter()
            .map(|side| side.pairing())
            .collect();
        let wall = reflections[1];
        let count = double.sides().len();
        for (index, side) in double.sides().iter().enumerate() {
            let (original, image) = source(count, index);
            let (first, second) = if image {
                (reflections[original], wall)
            } else {
                (wall, reflections[original])
            };
            let exact = exact_product(first, second);
            for (row, exact_row) in exact.iter().enumerate() {
                for (column, exact) in exact_row.iter().enumerate() {
                    entries += 1;
                    let value = side.pairing().col(column)[row];
                    let input = || format!("entry ({row}, {column}) of side {index} of {text}");
                    if !is_nearest(value, exact) {
                        misrounded.push(input());
                        largest = max(&largest, &error(value, exact));
                    }
                }
            }
        }
    }
    println!(
        "doubles of the named polygons: {} of {entries} entries of the pairings are not the exact product rounded to nearest: {misrounded:?}",
        misrounded.len()
    );
    let mut reference = Reference::new();
    let largest = reference.approximate(&largest);
    println!(
        "largest absolute error of those entries: {largest:e}, bound {MISROUNDED_ENTRY_BOUND:e}"
    );
    assert!(entries > 0);
    assert!(largest <= MISROUNDED_ENTRY_BOUND, "{largest:e}");
}

const ROTATION_POLYGON_BOUNDS: [f64; 7] = [4e-13, 1e-11, 1e-11, 1e-15, 4e-12, 1e-13, 1e-11];

const ROTATION_POLYGON_QUANTITIES: [&str; 7] = [
    "doubled polygon vertex, largest coordinate error over the largest coordinate",
    "doubled polygon normal, largest coordinate error",
    "doubled polygon side length, absolute error",
    "doubled polygon centre, largest coordinate distance from the origin",
    "doubled polygon corner angle, absolute error against copies pi / n",
    "doubled polygon flat vertex, absolute error against pi",
    "doubled polygon area, absolute error against twice the area of the starred signature",
];

fn record_rotation_polygon(
    reference: &mut Reference,
    worst: &mut [Worst],
    text: &str,
    double: &FundamentalDomain,
) {
    let starred_text = starred(text);
    let polygon = signature(&starred_text);
    let generating = built(&starred_text);
    assert_eq!(double.centre(), generating.centre(), "{text}");
    let orders = corner_orders(&polygon);
    let values: Vec<f64> = parameters(&polygon)
        .iter()
        .map(escher_domain::ParameterSpec::default_value)
        .collect();
    let exact = reference.reflection_polygon(&orders, &values);
    let area = signature_area(reference, &polygon);
    let exact = reference.double(Model::Hyperbolic, &exact.vertices, &exact.normals, &area);
    let input = || text.to_owned();
    let [vertices, normals, lengths, centre, angles, flats, areas] = worst else {
        panic!("the double test has seven quantities");
    };
    for (vertex, exact_vertex) in double.vertices().iter().zip(&exact.vertices) {
        vertices.record(
            &largest_coordinate_error(*vertex, exact_vertex),
            &largest_magnitude(exact_vertex),
            input,
        );
    }
    for (side, exact_wall) in double.sides().iter().zip(&exact.walls) {
        normals.record(
            &largest_coordinate_error(side.normal(), exact_wall),
            &int(1),
            input,
        );
    }
    for (length, exact_length) in double.side_lengths().into_iter().zip(&exact.lengths) {
        lengths.record(&error(length, exact_length), &int(1), input);
    }
    centre.record(
        &largest_coordinate_error(double.centre(), &crate::common::big(hyperbolic::ORIGIN)),
        &int(1),
        input,
    );
    let pi = reference.pi();
    for (angle, (order, copies)) in double.corner_angles().into_iter().zip(corners(&orders)) {
        let expected = div(&mul(&int(copies), &pi), &int(order));
        let difference = error(angle, &expected);
        angles.record(&difference, &int(1), input);
        if copies == order {
            flats.record(&difference, &int(1), input);
        }
    }
    areas.record(&error(double.area(), &exact.area), &int(1), input);
}

fn rotation_quantities(label: &str) -> Vec<Worst> {
    ROTATION_POLYGON_QUANTITIES
        .iter()
        .map(|name| Worst::new(format!("{name}, {label}"), 1))
        .collect()
}

#[test]
fn rotation_polygons_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = rotation_quantities("named doubles");
    for text in named_rotation_polygons() {
        record_rotation_polygon(&mut reference, &mut worst, &text, &built(&text));
    }
    for (quantity, bound) in worst.iter().zip(ROTATION_POLYGON_BOUNDS) {
        quantity.assert_at_most(&mut reference, bound);
    }
}
