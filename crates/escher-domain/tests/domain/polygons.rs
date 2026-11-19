use core::f64::consts::PI;
use std::time::Instant;

use astro_float_num::BigFloat;
use escher_domain::{ErrorKind, FundamentalDomain, ParameterSpec, ShapeParameters, parameters};
use escher_geometry::hyperbolic;
use escher_group::{IndexLimit, SearchBudget, conway_presentation, count_colourings};
use escher_signature::{Geometry, Signature};
use proptest::prelude::{prop_assert, prop_assert_eq, prop_oneof, proptest};
use proptest::sample::select;
use proptest::strategy::Strategy;

use crate::common::{big, largest_coordinate_error, proptest_config_with_miri_cases, signature};
use crate::reference::{
    Cuts, FeatureKind, Reference, ReflectionPolygon, Worst, add, div, error, form, int,
    largest_magnitude, mul, num, sub, vector,
};
use crate::shared::{NAMED_POLYGONS, corner_orders, small_polygons};

const RANDOM_POLYGON_MIRI_CASES: u32 = 3;

const VERTEX_BOUND: f64 = 4e-15;

const CLOSING_SIDE_BOUND: f64 = 4e-12;

const SETTING_VALUES: [f64; 5] = [0.5, 1.0, 2.0, 3.0, 3.5];

const RANDOM_SETTINGS: usize = 32;

const GUARD_TEXT: &str =
    "cannot build fundamental domain: the shape is too extreme to build accurately";

const REFUSED_SHARES: [(usize, u32); 5] = [(4, 0), (5, 0), (6, 0), (7, 8), (8, 22)];

const DRAWS_PER_COUNT: u32 = 300;

struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: u32) -> u32 {
        let bound = u64::from(bound);
        u32::try_from(self.next() % bound).unwrap_or(0)
    }

    fn length(&mut self) -> f64 {
        let fraction = f64::from(self.below(1 << 30)) / f64::from(1_u32 << 30);
        0.5 + 3.0 * fraction
    }
}

fn polygon_text(orders: &[u32]) -> String {
    let corners: String = orders
        .iter()
        .map(|&order| {
            if order < 10 {
                order.to_string()
            } else {
                format!("({order})")
            }
        })
        .collect();
    format!("*{corners}")
}

fn settings_of(names: &[String], values: &[f64]) -> ShapeParameters {
    let mut settings = ShapeParameters::new();
    for (name, &value) in names.iter().zip(values) {
        settings.set(name.clone(), value);
    }
    settings
}

fn polygon_settings(
    signature: &Signature,
    random: &mut Random,
) -> Vec<(ShapeParameters, Vec<f64>)> {
    let specs = parameters(signature);
    let names: Vec<String> = specs.iter().map(|spec| spec.name().to_owned()).collect();
    let defaults: Vec<f64> = specs.iter().map(ParameterSpec::default_value).collect();
    let mut all = vec![(ShapeParameters::new(), defaults.clone())];
    for index in 0..names.len() {
        for value in SETTING_VALUES {
            let mut values = defaults.clone();
            values[index] = value;
            all.push((settings_of(&names, &values), values));
        }
    }
    for _ in 0..RANDOM_SETTINGS {
        let values: Vec<f64> = names.iter().map(|_| random.length()).collect();
        all.push((settings_of(&names, &values), values));
    }
    all
}

fn assert_canonical_corners(domain: &FundamentalDomain, signature: &Signature, input: &str) {
    let orders = corner_orders(signature);
    let vertices = domain.vertices();
    assert_eq!(vertices.len(), orders.len(), "{input}");
    let angles = domain.corner_angles();
    for (corner, (&order, angle)) in orders.iter().zip(angles).enumerate() {
        let expected = PI / f64::from(order);
        assert!(
            (angle - expected).abs() <= 1e-9,
            "{input}: corner {corner} has {angle}, not pi / {order}"
        );
    }
    let count = vertices.len();
    for index in 0..count {
        let [first, second, third] =
            [index, (index + 1) % count, (index + 2) % count].map(|vertex| vertices[vertex]);
        let turn = first.dot(second.cross(third));
        assert!(
            turn > 0.0,
            "{input}: vertices {index} to {} turn clockwise",
            index + 2
        );
    }
    let centre = domain.centre();
    let offset = (centre - hyperbolic::ORIGIN).abs().max_element();
    assert!(offset <= 1e-9, "{input}: centre {centre:?}");
    let first = vertices[0];
    assert!(first.x > 0.0, "{input}: vertex 0 {first:?}");
    assert!(
        first.y.abs() <= 1e-9 * first.z,
        "{input}: vertex 0 {first:?}"
    );
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the sweep of the small polygons runs loops too long for the interpreter"
)]
fn every_polygon_has_its_corners_in_the_canonical_cyclic_order() {
    let mut random = Random(0x9e37_79b9_7f4a_7c15);
    let named = NAMED_POLYGONS.map(signature);
    let small = small_polygons();
    let mut built = 0;
    let mut refused = 0;
    for polygon in named.iter().chain(&small) {
        let text = polygon.to_string();
        let mut settings = vec![(ShapeParameters::new(), Vec::new())];
        if named.contains(polygon) {
            settings = polygon_settings(polygon, &mut random);
        }
        for (setting, _) in settings {
            match FundamentalDomain::build_with(polygon, &setting) {
                Ok(domain) => {
                    assert!(domain.verify().passed(), "{text} {setting:?}");
                    assert_canonical_corners(&domain, polygon, &format!("{text} {setting:?}"));
                    built += 1;
                }
                Err(error) => {
                    assert_eq!(error.to_string(), GUARD_TEXT, "{text} {setting:?}");
                    refused += 1;
                }
            }
        }
    }
    println!("{built} polygons built, {refused} settings refused by the guard");
    assert!(built > 5000, "{built}");
}

#[test]
fn equal_signatures_give_equal_polygons_and_parameter_names() {
    for (canonical, spellings) in [
        ("*3322", ["*2233", "*2332"].as_slice()),
        ("*3222", ["*2223"].as_slice()),
    ] {
        let expected = FundamentalDomain::build(&signature(canonical));
        let Ok(expected) = expected else {
            panic!("{canonical} has no domain");
        };
        for spelling in spellings {
            let typed = signature(spelling);
            assert_eq!(typed.to_string(), canonical, "{spelling}");
            assert_eq!(
                parameters(&typed),
                parameters(&signature(canonical)),
                "{spelling}"
            );
            let Ok(built) = FundamentalDomain::build(&typed) else {
                panic!("{spelling} has no domain");
            };
            assert_eq!(built.vertices().len(), expected.vertices().len());
            for (one, other) in built.vertices().iter().zip(expected.vertices()) {
                let difference = (*one - *other).abs().max_element();
                let scale = one.abs().max_element();
                assert!(
                    difference <= 2.0 * VERTEX_BOUND * scale,
                    "{spelling}: {difference:e}"
                );
            }
        }
    }
    assert_eq!(
        parameters(&signature("*2223"))[0].name(),
        "cornerpair1.length"
    );
}

#[test]
fn rotations_and_reversals_give_one_canonical_signature() {
    for text in ["*5432", "*22323"] {
        let orders = corner_orders(&signature(text));
        let canonical = signature(text).to_string();
        let names: Vec<String> = parameters(&signature(text))
            .iter()
            .map(|spec| spec.name().to_owned())
            .collect();
        for reversed in [false, true] {
            for shift in 0..orders.len() {
                let mut turned = orders.clone();
                turned.rotate_left(shift);
                if reversed {
                    turned.reverse();
                }
                let typed = signature(&polygon_text(&turned));
                assert_eq!(typed.to_string(), canonical, "{turned:?}");
                let typed_names: Vec<String> = parameters(&typed)
                    .iter()
                    .map(|spec| spec.name().to_owned())
                    .collect();
                assert_eq!(typed_names, names, "{turned:?}");
            }
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn closing_side_has_the_length_of_its_gaps() {
    let mut reference = Reference::new();
    let mut random = Random(0x2545_f491_4f6c_dd1d);
    let mut largest = (0.0, String::new());
    let mut samples = 0;
    for text in NAMED_POLYGONS {
        let polygon = signature(text);
        let orders = corner_orders(&polygon);
        for (setting, values) in polygon_settings(&polygon, &mut random) {
            let Ok(domain) = FundamentalDomain::build_with(&polygon, &setting) else {
                continue;
            };
            let expected = reference.gap_sums(&orders, &values).swap_remove(0);
            let vertices = domain.vertices();
            let measured = reference.distance(
                &vector(vertices[vertices.len() - 1].to_array()),
                &vector(vertices[0].to_array()),
            );
            let error = reference.approximate(&sub(&measured, &expected).abs());
            if error > largest.0 {
                largest = (error, format!("{text} {values:?}"));
            }
            assert!(error <= CLOSING_SIDE_BOUND, "{text} {values:?}: {error:e}");
            samples += 1;
        }
    }
    println!(
        "closing side: largest error {:.3e} at {}, bound {CLOSING_SIDE_BOUND:e}, margin {:.1}, {samples} samples",
        largest.0,
        largest.1,
        CLOSING_SIDE_BOUND / largest.0
    );
    assert!(samples > 500, "{samples}");
}

fn random_polygon(random: &mut Random, count: usize) -> Signature {
    loop {
        let orders: Vec<u32> = (0..count).map(|_| 2 + random.below(11)).collect();
        let drawn = signature(&polygon_text(&orders));
        if drawn.geometry() == Geometry::Hyperbolic {
            return drawn;
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the checks of the largest orders run loops too long for the interpreter"
)]
fn guard_refuses_few_random_settings() {
    let mut random = Random(0x1234_5678_9abc_def1);
    for (count, recorded) in REFUSED_SHARES {
        let mut refused = 0;
        for _ in 0..DRAWS_PER_COUNT {
            let polygon = random_polygon(&mut random, count);
            let names: Vec<String> = parameters(&polygon)
                .iter()
                .map(|spec| spec.name().to_owned())
                .collect();
            let values: Vec<f64> = names.iter().map(|_| random.length()).collect();
            match FundamentalDomain::build_with(&polygon, &settings_of(&names, &values)) {
                Ok(domain) => assert!(domain.verify().passed(), "{polygon} {values:?}"),
                Err(error) => {
                    assert_eq!(error.to_string(), GUARD_TEXT, "{polygon} {values:?}");
                    refused += 1;
                }
            }
        }
        println!(
            "{count} corners: {refused} of {DRAWS_PER_COUNT} settings refused, {:.1}%",
            100.0 * f64::from(refused) / f64::from(DRAWS_PER_COUNT)
        );
        assert!(
            refused <= 2 * recorded,
            "{count} corners: {refused} refused"
        );
    }
}

fn polygon_orders() -> impl Strategy<Value = Vec<u32>> {
    let order = prop_oneof![2_u32..=12, select(vec![900_u32, 950, 1000])];
    (4_usize..=8).prop_flat_map(move |count| proptest::collection::vec(order.clone(), count))
}

proptest! {
    #![proptest_config(proptest_config_with_miri_cases(RANDOM_POLYGON_MIRI_CASES))]

    #[test]
    fn random_polygons_build_and_pass_or_are_refused(
        orders in polygon_orders(),
        fractions in proptest::collection::vec(0.0_f64..=1.0, 5),
        pick in select(vec![0.5_f64, 3.5]),
        ends in proptest::bool::ANY,
    ) {
        let polygon = signature(&polygon_text(&orders));
        let specs = parameters(&polygon);
        let mut settings = ShapeParameters::new();
        for (index, (spec, fraction)) in specs.iter().zip(&fractions).enumerate() {
            let value = if ends && index == 0 { pick } else { 0.5 + 3.0 * fraction };
            settings.set(spec.name(), value);
        }
        match FundamentalDomain::build_with(&polygon, &settings) {
            Ok(domain) => {
                prop_assert!(domain.verify().passed());
                prop_assert_eq!(domain.vertices().len(), orders.len());
            }
            Err(error) => {
                prop_assert_eq!(error.kind(), ErrorKind::Numerical);
                prop_assert_eq!(error.to_string(), GUARD_TEXT);
            }
        }
    }
}

const BINDING_BOUND: f64 = 4e-14;

#[test]
fn each_length_parameter_sets_the_length_that_it_names() {
    let distance_of_walls = |domain: &FundamentalDomain, first: usize, second: usize| {
        let sides = domain.sides();
        hyperbolic::form(sides[first].normal(), sides[second].normal())
            .abs()
            .acosh()
    };
    let mut largest: f64 = 0.0;
    for (text, settings, pair_sides, slice_walls) in [
        (
            "*33222",
            [("cornerpair1.length", 1.3), ("slice1.length", 2.1)].as_slice(),
            [3].as_slice(),
            [(2, 0)].as_slice(),
        ),
        (
            "*33222",
            [("cornerpair1.length", 2.1), ("slice1.length", 1.3)].as_slice(),
            [3].as_slice(),
            [(2, 0)].as_slice(),
        ),
        (
            "*2222222",
            [
                ("cornerpair1.length", 1.1),
                ("cornerpair2.length", 1.7),
                ("cornerpair3.length", 2.3),
                ("slice1.length", 2.9),
            ]
            .as_slice(),
            [1, 3, 5].as_slice(),
            [(4, 0)].as_slice(),
        ),
    ] {
        let mut setting = ShapeParameters::new();
        for &(name, value) in settings {
            setting.set(name, value);
        }
        let Ok(domain) = FundamentalDomain::build_with(&signature(text), &setting) else {
            panic!("{text} {settings:?} has no domain");
        };
        let lengths = domain.side_lengths();
        let measured = pair_sides.iter().map(|&side| lengths[side]).chain(
            slice_walls
                .iter()
                .map(|&(first, second)| distance_of_walls(&domain, first, second)),
        );
        for (&(name, value), measured) in settings.iter().zip(measured) {
            let error = (measured - value).abs();
            assert!(
                error <= BINDING_BOUND,
                "{text} {name}: {measured} for {value}"
            );
            largest = largest.max(error);
        }
    }
    println!(
        "largest error of a parameter against its length {largest:.3e}, bound {BINDING_BOUND:e}"
    );
}

const POLYGON_BOUNDS: [f64; 6] = [4e-10, 2e-7, 2e-8, 1e-15, 1e-10, 1e-10];

const SMALL_ORDER_POLYGON_BOUNDS: [f64; 6] = [1e-13, 4e-13, 4e-13, 1e-15, 1e-13, 1e-13];

const SMALL_ORDER: u32 = 12;

const POLYGON_QUANTITIES: [&str; 6] = [
    "polygon vertex, largest coordinate error over the largest coordinate",
    "polygon normal, largest coordinate error",
    "polygon side length, absolute error",
    "polygon centre, largest coordinate distance from the origin",
    "polygon corner angle, absolute error against pi / n",
    "polygon area, absolute error against the area of the signature",
];

const IDENTITY_TOLERANCE: &str = "1e-50";

const TANGENCY_TOLERANCE: &str = "1e-60";

const DEFAULT_ROUNDING: f64 = 5.0e-4;

const UNROUNDED_DEFAULT_BOUND: f64 = 1e-12;

const SMALL_POLYGON_STRIDE: usize = 256;

pub(crate) fn signature_area(reference: &mut Reference, polygon: &Signature) -> BigFloat {
    let Some(area) = polygon.area_over_pi() else {
        panic!("{polygon} has no area");
    };
    let (Ok(numerator), Ok(denominator)) = (
        i32::try_from(area.numerator()),
        u32::try_from(area.denominator().get()),
    ) else {
        panic!("{polygon} has an area too large for the test");
    };
    div(
        &mul(&reference.pi(), &num(f64::from(numerator))),
        &int(denominator),
    )
}

struct Identity {
    name: String,
    worst: Worst,
    tolerance: BigFloat,
}

impl Identity {
    fn new(reference: &mut Reference, name: &str, tolerance: &str) -> Self {
        Self {
            name: name.to_owned(),
            worst: Worst::new(name, 1),
            tolerance: reference.parse(tolerance),
        }
    }

    fn record(&mut self, difference: &BigFloat, input: impl Fn() -> String) {
        let difference = difference.abs();
        assert!(
            difference <= self.tolerance,
            "{}: {difference} at {}",
            self.name,
            input()
        );
        self.worst.record(&difference, &int(1), input);
    }

    fn print(&self, reference: &mut Reference) {
        let bound = reference.approximate(&self.tolerance);
        self.worst.assert_at_most(reference, bound);
    }
}

struct PolygonColumn {
    bounds: [f64; 6],
    worst: Vec<Worst>,
}

impl PolygonColumn {
    fn new(label: &str, bounds: [f64; 6]) -> Self {
        Self {
            bounds,
            worst: POLYGON_QUANTITIES
                .iter()
                .map(|name| Worst::new(format!("{name}, {label}"), 1))
                .collect(),
        }
    }

    fn record(
        &mut self,
        domain: &FundamentalDomain,
        exact: &ReflectionPolygon,
        expected: (&[BigFloat], &BigFloat),
        input: impl Fn() -> String + Copy,
    ) {
        let (expected_angles, expected_area) = expected;
        let [vertices, normals, lengths, centre, angles, area] = &mut self.worst[..] else {
            panic!("the polygon test has six quantities");
        };
        for (vertex, exact_vertex) in domain.vertices().iter().zip(&exact.vertices) {
            vertices.record(
                &largest_coordinate_error(*vertex, exact_vertex),
                &largest_magnitude(exact_vertex),
                input,
            );
        }
        for (side, exact_normal) in domain.sides().iter().zip(&exact.normals) {
            normals.record(
                &largest_coordinate_error(side.normal(), exact_normal),
                &int(1),
                input,
            );
        }
        for (length, exact_length) in domain.side_lengths().into_iter().zip(&exact.lengths) {
            lengths.record(&error(length, exact_length), &int(1), input);
        }
        centre.record(
            &largest_coordinate_error(domain.centre(), &big(hyperbolic::ORIGIN)),
            &int(1),
            input,
        );
        for (angle, expected) in domain.corner_angles().into_iter().zip(expected_angles) {
            angles.record(&error(angle, expected), &int(1), input);
        }
        area.record(&error(domain.area(), expected_area), &int(1), input);
    }

    fn assert_within_bounds(&self, reference: &mut Reference) {
        for (quantity, bound) in self.worst.iter().zip(self.bounds) {
            quantity.assert_at_most(reference, bound);
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn polygons_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut random = Random(0x6a09_e667_f3bc_c908);
    let mut every_setting = PolygonColumn::new("named polygons at their settings", POLYGON_BOUNDS);
    let mut small_orders = PolygonColumn::new(
        "named polygons with orders up to 12 at the default shape",
        SMALL_ORDER_POLYGON_BOUNDS,
    );
    let mut angles = Identity::new(
        &mut reference,
        "reference polygon, angle against pi / n",
        IDENTITY_TOLERANCE,
    );
    let mut areas = Identity::new(
        &mut reference,
        "reference polygon, area of its angles against the area of the signature",
        IDENTITY_TOLERANCE,
    );
    let (mut built, mut refused) = (0, 0);
    let started = Instant::now();
    for text in NAMED_POLYGONS {
        let polygon = signature(text);
        let orders = corner_orders(&polygon);
        let expected_area = signature_area(&mut reference, &polygon);
        let pi = reference.pi();
        let expected_angles: Vec<BigFloat> =
            orders.iter().map(|&order| div(&pi, &int(order))).collect();
        let Ok(sides) = u32::try_from(orders.len()) else {
            panic!("{text} has too many corners");
        };
        let small = orders.iter().all(|&order| order <= SMALL_ORDER);
        for (index, (setting, values)) in polygon_settings(&polygon, &mut random)
            .into_iter()
            .enumerate()
        {
            let domain = match FundamentalDomain::build_with(&polygon, &setting) {
                Ok(domain) => domain,
                Err(error) => {
                    assert_eq!(
                        (error.kind(), error.to_string()),
                        (ErrorKind::Numerical, GUARD_TEXT.to_owned()),
                        "{text} {values:?}"
                    );
                    refused += 1;
                    continue;
                }
            };
            built += 1;
            let input = || format!("{text} {values:?}");
            let exact = reference.reflection_polygon(&orders, &values);
            let expected = (expected_angles.as_slice(), &expected_area);
            every_setting.record(&domain, &exact, expected, input);
            if small && index == 0 {
                small_orders.record(&domain, &exact, expected, input);
            }
            let mut angle_total = int(0);
            for (exact_angle, expected) in exact.angles.iter().zip(&expected_angles) {
                angles.record(&sub(exact_angle, expected), input);
                angle_total = add(&angle_total, exact_angle);
            }
            let measured_area = sub(&mul(&int(sides - 2), &pi), &angle_total);
            areas.record(&sub(&measured_area, &expected_area), input);
        }
    }
    every_setting.assert_within_bounds(&mut reference);
    small_orders.assert_within_bounds(&mut reference);
    angles.print(&mut reference);
    areas.print(&mut reference);
    println!(
        "{built} settings built, {refused} settings refused by the guard, in {:.1} s",
        started.elapsed().as_secs_f64()
    );
    assert!(built > 900, "{built}");
}

fn slice_walls(
    vertices: &[crate::reference::Vector],
    side: usize,
) -> [&crate::reference::Vector; 2] {
    let count = vertices.len();
    [&vertices[(side + count - 1) % count], &vertices[side]]
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn each_cut_has_the_length_of_its_parameter_in_the_reference() {
    let mut reference = Reference::new();
    let mut random = Random(0xbb67_ae85_84ca_a73b);
    let mut pairs = Identity::new(
        &mut reference,
        "corner pair side against its parameter",
        IDENTITY_TOLERANCE,
    );
    let mut slices = Identity::new(
        &mut reference,
        "slice walls against their parameter",
        IDENTITY_TOLERANCE,
    );
    let started = Instant::now();
    for text in NAMED_POLYGONS {
        let polygon = signature(text);
        let orders = corner_orders(&polygon);
        let cuts = Cuts::of(&orders);
        for (_, values) in polygon_settings(&polygon, &mut random) {
            let walk = reference.reflection_polygon(&orders, &values).walk;
            let input = || format!("{text} {values:?}");
            for sheet in &cuts.sheets {
                for feature in sheet {
                    if let FeatureKind::Length(index) = feature.kind
                        && index < cuts.pairs
                    {
                        let side = reference.distance(&walk[feature.first], &walk[feature.last]);
                        pairs.record(&sub(&side, &num(values[index])), input);
                    }
                }
            }
            for (slice, &(first, second)) in cuts.slice_walls.iter().enumerate() {
                let distance = reference
                    .distance_between_walls(slice_walls(&walk, first), slice_walls(&walk, second));
                slices.record(&sub(&distance, &num(values[cuts.pairs + slice])), input);
            }
        }
    }
    pairs.print(&mut reference);
    slices.print(&mut reference);
    println!("cut meaning in {:.1} s", started.elapsed().as_secs_f64());
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn walk_agrees_with_the_welded_sheets_in_the_reference() {
    let mut reference = Reference::new();
    let mut distances = Identity::new(
        &mut reference,
        "distance between two vertices, weld against walk",
        IDENTITY_TOLERANCE,
    );
    let mut ends = Identity::new(
        &mut reference,
        "angle at an end of a welded slice against pi",
        IDENTITY_TOLERANCE,
    );
    let mut closures = Identity::new(
        &mut reference,
        "closure of a laid-out sheet",
        IDENTITY_TOLERANCE,
    );
    let started = Instant::now();
    for text in NAMED_POLYGONS {
        let polygon = signature(text);
        let orders = corner_orders(&polygon);
        let values: Vec<f64> = parameters(&polygon)
            .iter()
            .map(ParameterSpec::default_value)
            .collect();
        let walk = reference.reflection_polygon(&orders, &values).walk;
        let weld = reference.weld(&orders, &values);
        for first in 0..orders.len() {
            for second in first + 1..orders.len() {
                let walked = reference.distance(&walk[first], &walk[second]);
                let welded = reference.distance(&weld.vertices[first], &weld.vertices[second]);
                distances.record(&sub(&walked, &welded), || {
                    format!("{text} vertices {first} and {second}")
                });
            }
        }
        for end in &weld.slice_ends {
            ends.record(end, || text.to_owned());
        }
        for closure in &weld.closures {
            closures.record(closure, || text.to_owned());
        }
    }
    distances.print(&mut reference);
    ends.print(&mut reference);
    closures.print(&mut reference);
    println!(
        "walk against weld in {:.1} s",
        started.elapsed().as_secs_f64()
    );
}

fn default_shape_of(reference: &mut Reference, polygon: &Signature, record: &mut DefaultShape) {
    let text = polygon.to_string();
    let orders = corner_orders(polygon);
    let count = orders.len();
    let inscribed = reference.inscribed(&orders);
    let pi = reference.pi();
    let vertices = &inscribed.vertices;
    for index in 0..count {
        let angle = reference.angle(
            &vertices[(index + count - 1) % count],
            &vertices[index],
            &vertices[(index + 1) % count],
        );
        record
            .angles
            .record(&sub(&angle, &div(&pi, &int(orders[index]))), || {
                format!("{text} vertex {index}")
            });
        let origin = [int(0), int(0), int(1)];
        let distance = reference.distance_to_segment(
            &origin,
            &vertices[(index + count - 1) % count],
            &vertices[index],
        );
        record
            .tangency
            .record(&sub(&distance, &inscribed.radius), || {
                format!("{text} side {index}")
            });
    }
    let specs = parameters(polygon);
    let exact = exact_defaults(reference, &orders, vertices);
    let cuts = Cuts::of(&orders);
    for (slice, &(first, second)) in cuts.slice_walls.iter().enumerate() {
        let distance = reference
            .distance_between_walls(slice_walls(vertices, first), slice_walls(vertices, second));
        record
            .walls
            .record(&sub(&distance, &exact[cuts.pairs + slice]), || {
                format!("{text} slice {}", slice + 1)
            });
    }
    let (shortest, longest) = (num(0.5), num(3.5));
    for (spec, exact) in specs.iter().zip(&exact) {
        assert!(
            *exact > shortest && *exact < longest,
            "{text} {}: {exact}",
            spec.name()
        );
        let difference = error(spec.default_value(), exact);
        record
            .defaults
            .record(&difference, &int(1), || format!("{text} {}", spec.name()));
        let thousandths = mul(exact, &int(1000));
        let fraction = sub(&thousandths, &thousandths.floor());
        let boundary = div(&sub(&fraction, &div(&int(1), &int(2))).abs(), &int(1000));
        if boundary < record.boundary.0 {
            record.boundary = (boundary, format!("{text} {}", spec.name()));
        }
    }
}

pub(crate) fn exact_defaults(
    reference: &mut Reference,
    orders: &[u32],
    vertices: &[crate::reference::Vector],
) -> Vec<BigFloat> {
    let cuts = Cuts::of(orders);
    let mut exact = vec![int(0); cuts.pairs + cuts.slice_walls.len()];
    for sheet in &cuts.sheets {
        for feature in sheet {
            if let FeatureKind::Length(index) = feature.kind
                && index < cuts.pairs
            {
                exact[index] =
                    reference.distance(&vertices[feature.first], &vertices[feature.last]);
            }
        }
    }
    for (slice, &(first, second)) in cuts.slice_walls.iter().enumerate() {
        let [a, b] = [first, second].map(|side| {
            let [start, end] = slice_walls(vertices, side);
            crate::reference::hyperbolic_normal(start, end)
        });
        exact[cuts.pairs + slice] = reference.acosh(&form(&a, &b).abs());
    }
    exact
}

struct DefaultShape {
    angles: Identity,
    tangency: Identity,
    walls: Identity,
    defaults: Worst,
    boundary: (BigFloat, String),
}

const BOUNDARY_POLYGONS: [&str; 2] = ["*422322", "*432232"];

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn default_shapes_are_the_inscribed_polygons_of_the_reference() {
    let mut reference = Reference::new();
    let mut record = DefaultShape {
        angles: Identity::new(
            &mut reference,
            "inscribed polygon, angle against pi / n",
            TANGENCY_TOLERANCE,
        ),
        tangency: Identity::new(
            &mut reference,
            "inscribed polygon, distance of a side from the centre against the inradius",
            TANGENCY_TOLERANCE,
        ),
        walls: Identity::new(
            &mut reference,
            "inscribed polygon, slice by the common perpendicular against the form of the normals",
            IDENTITY_TOLERANCE,
        ),
        defaults: Worst::new("default against its unrounded reference value", 1),
        boundary: (int(1), String::new()),
    };
    let started = Instant::now();
    for text in NAMED_POLYGONS {
        default_shape_of(&mut reference, &signature(text), &mut record);
    }
    println!("named polygons in {:.1} s", started.elapsed().as_secs_f64());
    for polygon in small_polygons().iter().step_by(SMALL_POLYGON_STRIDE) {
        default_shape_of(&mut reference, polygon, &mut record);
    }
    for text in BOUNDARY_POLYGONS {
        let polygon = signature(text);
        assert_eq!(polygon.to_string(), text);
        default_shape_of(&mut reference, &polygon, &mut record);
    }
    record.angles.print(&mut reference);
    record.tangency.print(&mut reference);
    record.walls.print(&mut reference);
    record
        .defaults
        .assert_at_most(&mut reference, DEFAULT_ROUNDING + UNROUNDED_DEFAULT_BOUND);
    println!(
        "smallest distance of an unrounded default from a boundary of the rounding: {:.3e} at {}",
        reference.approximate(&record.boundary.0),
        record.boundary.1
    );
    println!("default shapes in {:.1} s", started.elapsed().as_secs_f64());
}

const COLOUR_LIMIT: u32 = 4;

#[test]
#[cfg_attr(
    miri,
    ignore = "the colouring search runs loops too long for the interpreter"
)]
fn side_and_conway_presentations_give_equal_colouring_counts() {
    let Ok(limit) = IndexLimit::new(COLOUR_LIMIT) else {
        panic!("{COLOUR_LIMIT} colours are out of range");
    };
    for text in ["*3333", "*5432", "*3222", "*22222"] {
        let polygon = signature(text);
        let side = match crate::common::built(text).side_presentation() {
            Ok(presentation) => presentation,
            Err(error) => panic!("{text} has no side presentation: {error}"),
        };
        let conway = match conway_presentation(&polygon) {
            Ok(presentation) => presentation,
            Err(error) => panic!("{text} has no Conway presentation: {error}"),
        };
        let counts = [&side, &conway].map(|presentation| {
            match count_colourings(presentation, limit, SearchBudget::DEFAULT) {
                Ok(counts) => counts,
                Err(error) => panic!("{text}: the search fails: {error}"),
            }
        });
        let found: Vec<[Option<u64>; 2]> = (1..=COLOUR_LIMIT)
            .map(|colours| counts.clone().map(|count| count.colourings(colours)))
            .collect();
        println!("{text}: colourings with 1 to {COLOUR_LIMIT} colours, side and Conway: {found:?}");
        for (colours, [by_sides, by_conway]) in (1..=COLOUR_LIMIT).zip(found) {
            assert!(by_sides.is_some(), "{text}: {colours} colours not counted");
            assert_eq!(by_sides, by_conway, "{text}: {colours} colours");
        }
    }
}
