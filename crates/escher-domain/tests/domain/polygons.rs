use core::f64::consts::PI;
use std::collections::VecDeque;

use astro_float_num::BigFloat;
use escher_domain::{ErrorKind, FundamentalDomain, ParameterSpec, ShapeParameters, parameters};
use escher_geometry::hyperbolic;
use escher_signature::{Geometry, Signature};
use proptest::prelude::{prop_assert, prop_assert_eq, prop_oneof, proptest};
use proptest::sample::select;
use proptest::strategy::Strategy;

use crate::common::{proptest_config_with_miri_cases, signature};
use crate::reference::{Reference, add, div, int, mul, num, sub, vector};
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

#[derive(Clone, Copy)]
enum Kind {
    Corner(usize),
    Length(usize),
}

#[derive(Clone, Copy)]
struct Feature {
    kind: Kind,
    first: usize,
    last: usize,
}

fn reference_side_zero(reference: &mut Reference, orders: &[u32], values: &[f64]) -> BigFloat {
    let count = orders.len();
    let mut list = VecDeque::new();
    let mut pairs = 0;
    let mut corner = 0;
    while corner < count {
        let paired = orders[corner] == 2 && corner + 1 < count && orders[corner + 1] == 2;
        if paired {
            list.push_back(Feature {
                kind: Kind::Length(pairs),
                first: corner,
                last: corner + 1,
            });
            pairs += 1;
            corner += 2;
        } else {
            list.push_back(Feature {
                kind: Kind::Corner(corner),
                first: corner,
                last: corner,
            });
            corner += 1;
        }
    }
    let mut sheets = Vec::new();
    let mut slices = 0;
    while list.len() > 3 {
        let (Some(x), Some(y)) = (list.pop_back(), list.pop_back()) else {
            panic!("{orders:?}: the list is shorter than it says");
        };
        let kind = Kind::Length(pairs + slices);
        slices += 1;
        sheets.push([
            y,
            x,
            Feature {
                kind,
                first: (x.last + 1) % count,
                last: (y.first + count - 1) % count,
            },
        ]);
        list.push_front(Feature {
            kind,
            first: y.first,
            last: x.last,
        });
    }
    sheets.push([list[0], list[1], list[2]]);
    let pi = reference.pi();
    let mut trig = |feature: Feature| match feature.kind {
        Kind::Corner(corner) => {
            let angle = div(&pi, &int(orders[corner]));
            (true, reference.cos(&angle), reference.sin(&angle))
        }
        Kind::Length(index) => {
            let length = num(values[index]);
            (false, reference.cosh(&length), reference.sinh(&length))
        }
    };
    let mut gaps = Vec::new();
    for sheet in &sheets {
        for start in 0..3 {
            let (f, g, h) = (sheet[start], sheet[(start + 1) % 3], sheet[(start + 2) % 3]);
            assert_eq!((f.last + 1) % count, g.first, "{orders:?}");
            let ((f_corner, c_f, s_f), (g_corner, c_g, s_g), (_, c_h, _)) =
                (trig(f), trig(g), trig(h));
            let argument = div(&add(&mul(&c_f, &c_g), &c_h), &mul(&s_f, &s_g));
            if g.first == 0 {
                gaps.push((f_corner == g_corner, argument));
            }
        }
    }
    let mut total = num(0.0);
    for (same, argument) in gaps {
        let gap = if same {
            reference.acosh(&argument)
        } else {
            reference.asinh(&argument)
        };
        total = add(&total, &gap);
    }
    total
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
            let expected = reference_side_zero(&mut reference, &orders, &values);
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
