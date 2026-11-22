use core::f64::consts::PI;
use std::time::Instant;

use astro_float_num::BigFloat;
use escher_domain::{
    ErrorKind, FundamentalDomain, Orientation, PairingKind, Verification, parameters,
};
use escher_geometry::spherical;
use escher_group::{CosetLimit, enumerate};
use escher_signature::Geometry;
use glam::{DMat3, DVec3};

use crate::common::{
    Kite, LIMIT_ORDER, LargestResiduals, LuneGroup, TWO_MIRROR_LIMIT_ORDER, assert_same_domain,
    built, largest_coordinate_error, limit_kites, lune_test_set, signature, small_kites,
    small_lune_groups, test_set,
};
use crate::reference::{
    Matrix, Model, Reference, Vector, Worst, div, error, int, matrix, max, mul, product, sub,
};
use crate::spherical::{FOLD_BACK_TOLERANCE, Group, LIMIT_IMAGE_FLOOR, fold, sphere_sample};

const VERTEX_BOUND: f64 = 4e-14;
const NORMAL_BOUND: f64 = 2e-14;
const PAIRING_BOUND: f64 = 4e-14;
const LENGTH_BOUND: f64 = 4e-14;
const CYCLE_BOUND: f64 = 2e-13;
const FLAT_BOUND: f64 = 1e-13;
const AREA_BOUND: f64 = 2e-14;

const LUNE_VERTEX_BOUND: f64 = 0.0;
const LUNE_NORMAL_BOUND: f64 = 1e-15;
const LUNE_PAIRING_BOUND: f64 = 2e-15;
const LUNE_LENGTH_BOUND: f64 = 1e-15;
const LUNE_CYCLE_BOUND: f64 = 4e-14;
const LUNE_FLAT_BOUND: f64 = 1e-15;
const LUNE_AREA_BOUND: f64 = 4e-14;

const KITE_IMAGE_FLOOR: f64 = 1e-2;

const MARGIN: f64 = 8.0;

const SEED: u64 = 0x5EED_0011;

const WALL_TOLERANCE: f64 = 1e-9;

const CENSUS_TOLERANCE: f64 = 1e-6;

const SAMPLE_POINTS: u32 = 2000;

fn group_order(text: &str) -> usize {
    signature(text).group_order().map_or(0, |order| {
        usize::try_from(order.get()).unwrap_or(usize::MAX)
    })
}

#[test]
fn spherical_kites_have_the_sides_vertices_and_cycles_of_their_table() {
    for kite in small_kites() {
        let text = kite.text();
        let domain = built(&text);
        assert_eq!(domain.geometry(), Geometry::Spherical, "{text}");
        assert!(domain.verify().passed(), "{text}: {:?}", domain.verify());
        assert!(parameters(&signature(&text)).is_empty(), "{text}");
        assert!(domain.reverses_orientation(), "{text}");
        assert_eq!(domain.sides().len(), 4, "{text}");
        assert_same_domain(&domain, &built(&kite.double()), 2.0 * VERTEX_BOUND, &text);
        for (index, side) in domain.sides().iter().enumerate() {
            let expected = if kite.mirrors().contains(&index) {
                (PairingKind::Mirror, index, Orientation::Reversing)
            } else {
                (PairingKind::Rotation, index ^ 1, Orientation::Preserving)
            };
            assert_eq!(
                (side.kind(), side.partner(), side.orientation()),
                expected,
                "{text}, side {index}"
            );
        }
        let angles = domain.corner_angles();
        for (vertex, angle) in angles.iter().enumerate() {
            let difference = (angle - PI).abs();
            if kite.flat_vertices().contains(&vertex) {
                assert!(
                    difference <= FLAT_BOUND,
                    "{text}, vertex {vertex}: {difference:e}"
                );
            } else {
                assert!(difference > 1e-3, "{text}, vertex {vertex}: {angle}");
            }
        }
        for (order, states) in kite.cycles() {
            let sum: f64 = states.iter().map(|&vertex| angles[vertex]).sum();
            let difference = (sum - 2.0 * PI / f64::from(order)).abs();
            assert!(
                difference <= CYCLE_BOUND,
                "{text}, cycle {states:?}: {difference:e}"
            );
        }
    }
}

const LUNE_VERTICES: [DVec3; 2] = [DVec3::Y, DVec3::NEG_Y];

#[test]
fn spherical_lune_groups_have_the_sides_vertices_and_cycles_of_their_table() {
    for group in small_lune_groups() {
        let text = group.text();
        let domain = built(&text);
        assert_eq!(domain.geometry(), Geometry::Spherical, "{text}");
        assert!(domain.verify().passed(), "{text}: {:?}", domain.verify());
        assert!(parameters(&signature(&text)).is_empty(), "{text}");
        assert_eq!(
            domain.reverses_orientation(),
            group != LuneGroup::Trivial,
            "{text}"
        );
        assert_eq!(domain.centre(), spherical::ORIGIN, "{text}");
        let Some(order) = group.order() else {
            assert!(domain.sides().is_empty(), "{text}");
            assert!(domain.vertices().is_empty(), "{text}");
            assert!(domain.side_lengths().is_empty(), "{text}");
            assert!(domain.corner_angles().is_empty(), "{text}");
            assert_eq!(domain.area().to_bits(), (4.0 * PI).to_bits(), "{text}");
            continue;
        };
        assert_eq!(domain.sides().len(), 2, "{text}");
        for (vertex, expected) in domain.vertices().iter().zip(LUNE_VERTICES) {
            let difference = (*vertex - expected).abs().max_element();
            assert!(
                difference <= 2.0 * LUNE_VERTEX_BOUND,
                "{text}: {difference:e}"
            );
        }
        for (index, side) in domain.sides().iter().enumerate() {
            let expected = if group == LuneGroup::Mirror {
                (PairingKind::Mirror, index, Orientation::Reversing)
            } else {
                (
                    PairingKind::RotaryReflection,
                    1 - index,
                    Orientation::Reversing,
                )
            };
            assert_eq!(
                (side.kind(), side.partner(), side.orientation()),
                expected,
                "{text}, side {index}"
            );
        }
        let angles = domain.corner_angles();
        for (vertex, angle) in angles.iter().enumerate() {
            let difference = (angle - PI).abs();
            if order == 1 {
                assert!(
                    difference <= LUNE_FLAT_BOUND,
                    "{text}, vertex {vertex}: {difference:e}"
                );
            } else {
                assert!(difference > 1e-3, "{text}, vertex {vertex}: {angle}");
            }
        }
        if group != LuneGroup::Mirror {
            let sum = angles[0] + angles[1];
            let difference = (sum - 2.0 * PI / f64::from(order)).abs();
            assert!(difference <= LUNE_CYCLE_BOUND, "{text}: {difference:e}");
        }
    }
}

#[test]
fn spherical_lune_groups_typed_in_another_order_build_the_domain_of_their_canonical_form() {
    for (spelling, canonical) in [
        ("1*", "*"),
        ("*1", "*"),
        ("1*1", "*"),
        ("(1)*", "*"),
        ("1x", "x"),
        ("x1", "x"),
        ("x3", "3x"),
    ] {
        assert_eq!(signature(spelling).to_string(), canonical, "{spelling}");
        assert_eq!(built(spelling), built(canonical), "{spelling}");
    }
}

#[test]
fn spherical_kites_typed_in_another_order_build_the_domain_of_their_canonical_form() {
    for (spelling, canonical) in [("2*1", "2*"), ("3*1", "3*")] {
        assert_eq!(signature(spelling).to_string(), canonical, "{spelling}");
        assert_eq!(built(spelling), built(canonical), "{spelling}");
    }
}

#[test]
fn signatures_of_the_kite_shapes_in_other_geometries_build_no_spherical_kite() {
    for text in ["4*2", "3*3", "2x2", "*x"] {
        assert_eq!(built(text).geometry(), Geometry::Euclidean, "{text}");
    }
    for text in ["4*3", "5*2", "3*4"] {
        let kind = FundamentalDomain::build(&signature(text)).map_err(|error| error.kind());
        assert_eq!(kind.err(), Some(ErrorKind::UnsupportedSignature), "{text}");
    }
}

fn bits(matrix: DMat3) -> [u64; 9] {
    matrix.to_cols_array().map(f64::to_bits)
}

fn vector_bits(vector: DVec3) -> [u64; 3] {
    vector.to_array().map(f64::to_bits)
}

#[test]
fn spherical_kites_keep_the_entries_of_their_doubles_bit_for_bit() {
    let mut largest = (0.0_f64, String::new());
    for kite in small_kites() {
        let text = kite.text();
        let (domain, double, triangle) =
            (built(&text), built(&kite.double()), built(&kite.starred()));
        for (vertex, doubled) in domain.vertices().iter().zip(double.vertices()) {
            assert_eq!(vector_bits(*vertex), vector_bits(*doubled), "{text}");
        }
        assert_eq!(
            vector_bits(domain.centre()),
            vector_bits(double.centre()),
            "{text}"
        );
        for (index, (side, doubled)) in domain.sides().iter().zip(double.sides()).enumerate() {
            assert_eq!(
                vector_bits(side.normal()),
                vector_bits(doubled.normal()),
                "{text}, side {index}"
            );
            assert_eq!(
                vector_bits(side.covector()),
                vector_bits(doubled.covector()),
                "{text}, side {index}"
            );
            let expected = if kite.mirrors().contains(&index) {
                spherical::reflection(side.normal())
            } else {
                doubled.pairing()
            };
            assert_eq!(bits(side.pairing()), bits(expected), "{text}, side {index}");
        }
        let (mirror, source) = match kite {
            Kite::ConeMirror(_) => (3, 2),
            Kite::ThreeMirrorTwo | Kite::TwoMirror(_) => (0, 0),
        };
        assert_eq!(
            bits(domain.sides()[mirror].pairing()),
            bits(triangle.sides()[source].pairing()),
            "{text}, side {mirror}"
        );
        if let Kite::ConeMirror(_) = kite {
            let [first, second] = kite.mirrors().map(|index| domain.sides()[index].normal());
            let difference = (first - second).abs().max_element();
            if difference >= largest.0 {
                largest = (difference, text.clone());
            }
        }
    }
    println!(
        "largest difference between the normals of the two mirror sides of n*: {:e}, at {}",
        largest.0, largest.1
    );
}

fn margins(verification: &Verification) -> [f64; 6] {
    [
        verification.isometries(),
        verification.pairings(),
        verification.walls(),
        verification.relators(),
        verification.angles(),
        verification.area(),
    ]
    .map(|check| check.tolerance() / check.residual())
}

fn assert_margin(verification: &Verification, input: &str) -> f64 {
    let smallest = margins(verification)
        .into_iter()
        .fold(f64::INFINITY, f64::min);
    assert!(
        smallest >= MARGIN,
        "{input}: margin {smallest}: {verification:?}"
    );
    smallest
}

fn split_mix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

fn drawn(state: &mut u64, largest: u32) -> u32 {
    let span = u64::from(largest - 1);
    u32::try_from(split_mix(state) % span).map_or(2, |offset| offset + 2)
}

#[test]
fn spherical_kites_pass_their_checks_with_a_margin() {
    let mut state = SEED;
    let cone = drawn(&mut state, LIMIT_ORDER);
    let two = drawn(&mut state, TWO_MIRROR_LIMIT_ORDER);
    let mut kites = test_set();
    kites.extend([
        Kite::ConeMirror(cone),
        Kite::TwoMirror(two),
        Kite::ConeMirror(LIMIT_ORDER - 1),
        Kite::TwoMirror(TWO_MIRROR_LIMIT_ORDER - 1),
    ]);
    let cross_cap = drawn(&mut state, LIMIT_ORDER);
    println!("random orders: {cone}*, 2*{two}, and {cross_cap}x");
    let mut texts: Vec<String> = kites.iter().map(|kite| kite.text()).collect();
    texts.extend(lune_test_set().into_iter().map(LuneGroup::text));
    texts.extend(
        [
            LuneGroup::ConeCrossCap(cross_cap),
            LuneGroup::ConeCrossCap(LIMIT_ORDER - 1),
        ]
        .map(LuneGroup::text),
    );
    let mut largest = LargestResiduals::new();
    for text in &texts {
        let verification = built(text).verify();
        assert_margin(&verification, text);
        largest.record_verification(&verification, text);
    }
    largest.print_and_assert_samples(texts.len());
}

#[test]
fn spherical_groups_pass_over_their_whole_range() {
    let families: [(&str, Vec<String>); 3] = [
        (
            "n*",
            (2..=LIMIT_ORDER)
                .map(|order| Kite::ConeMirror(order).text())
                .collect(),
        ),
        (
            "nx",
            (2..=LIMIT_ORDER)
                .map(|order| LuneGroup::ConeCrossCap(order).text())
                .collect(),
        ),
        (
            "2*n",
            (2..=TWO_MIRROR_LIMIT_ORDER)
                .map(|order| Kite::TwoMirror(order).text())
                .collect(),
        ),
    ];
    for (family, kites) in families {
        let started = Instant::now();
        let mut largest = LargestResiduals::new();
        let mut smallest = (f64::INFINITY, String::new());
        for text in &kites {
            let verification = built(text).verify();
            let margin = assert_margin(&verification, text);
            if margin < smallest.0 {
                smallest = (margin, text.clone());
            }
            largest.record_verification(&verification, text);
        }
        println!(
            "{family}: smallest margin {:.3e}, at {}, {} domains in {:.2} s",
            smallest.0,
            smallest.1,
            kites.len(),
            started.elapsed().as_secs_f64()
        );
        largest.print_and_assert_samples(kites.len());
    }
}

fn reflection_of(normal: &Vector) -> Matrix {
    [0, 1, 2].map(|row| {
        [0, 1, 2].map(|column| {
            let twice = mul(&int(2), &mul(&normal[row], &normal[column]));
            sub(&int(u32::from(row == column)), &twice)
        })
    })
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

fn exact_pairings(kite: Kite, walls: &[Vector], generating: &[Vector]) -> Vec<Matrix> {
    let [first, second, third] = [0, 1, 2].map(|index| reflection_of(&generating[index]));
    let mut pairings: Vec<Matrix> = walls.iter().map(reflection_of).collect();
    match kite {
        Kite::ConeMirror(_) => {
            pairings[0] = product(&second, &first);
            pairings[1] = product(&first, &second);
        }
        Kite::ThreeMirrorTwo | Kite::TwoMirror(_) => {
            pairings[2] = product(&third, &second);
            pairings[3] = product(&second, &third);
        }
    }
    pairings
}

#[test]
fn spherical_kites_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let pi = reference.pi();
    let kites = test_set();
    let count = kites.len();
    let flats: usize = kites.iter().map(|kite| kite.flat_vertices().len()).sum();
    let mut worst = [
        ("vertex, largest coordinate error", 4 * count, VERTEX_BOUND),
        ("normal, largest coordinate error", 4 * count, NORMAL_BOUND),
        ("pairing, largest entry error", 4 * count, PAIRING_BOUND),
        ("side length, absolute error", 4 * count, LENGTH_BOUND),
        (
            "angle sum of a cycle, absolute error against 2 pi / order",
            3 * count,
            CYCLE_BOUND,
        ),
        ("flat vertex, absolute error against pi", flats, FLAT_BOUND),
        (
            "area, absolute error against the area of the signature",
            count,
            AREA_BOUND,
        ),
    ]
    .map(|(name, samples, bound)| {
        (
            Worst::new(format!("spherical kites, {name}"), samples),
            bound,
        )
    });
    for kite in kites {
        let text = kite.text();
        let input = || text.clone();
        let domain = built(&text);
        let triangle = reference.spherical_triangle(kite.orders());
        let exact = reference.double(
            Model::Spherical,
            &triangle.vertices,
            &triangle.normals,
            &triangle.area,
        );
        let [vertices, normals, pairings, lengths, cycles, flat, area] = &mut worst;
        for (vertex, exact) in domain.vertices().iter().zip(&exact.vertices) {
            vertices
                .0
                .record(&largest_coordinate_error(*vertex, exact), &int(1), input);
        }
        let exact_pairings = exact_pairings(kite, &exact.walls, &triangle.normals);
        for ((side, wall), pairing) in domain.sides().iter().zip(&exact.walls).zip(&exact_pairings)
        {
            normals.0.record(
                &largest_coordinate_error(side.normal(), wall),
                &int(1),
                input,
            );
            pairings.0.record(
                &largest_entry_error(side.pairing(), pairing),
                &int(1),
                input,
            );
        }
        for (length, exact) in domain.side_lengths().into_iter().zip(&exact.lengths) {
            lengths.0.record(&error(length, exact), &int(1), input);
        }
        let angles = domain.corner_angles();
        for (order, states) in kite.cycles() {
            let sum: f64 = states.iter().map(|&vertex| angles[vertex]).sum();
            let expected = div(&mul(&int(2), &pi), &int(order));
            cycles.0.record(&error(sum, &expected), &int(1), input);
        }
        for &vertex in kite.flat_vertices() {
            flat.0.record(&error(angles[vertex], &pi), &int(1), input);
        }
        area.0
            .record(&error(domain.area(), &exact.area), &int(1), input);
        assert_eq!(
            vector_bits(domain.centre()),
            vector_bits(built(&kite.double()).centre()),
            "{text}"
        );
    }
    for (worst, bound) in &worst {
        worst.assert_at_most(&mut reference, *bound);
    }
}

fn half_turn_rows(matrix: &Matrix) -> Matrix {
    let [first, second, third] = matrix.clone();
    [
        first.map(|entry| entry.neg()),
        second.map(|entry| entry.neg()),
        third,
    ]
}

fn exact_lune_pairing(group: LuneGroup, normals: &[Vector], index: usize) -> Matrix {
    let reflection = reflection_of(&normals[index]);
    match group {
        LuneGroup::Mirror => reflection,
        LuneGroup::ConeCrossCap(_) | LuneGroup::CrossCap | LuneGroup::Trivial => {
            half_turn_rows(&reflection)
        }
    }
}

#[test]
fn spherical_lune_groups_hold_their_exact_values() {
    let half_turn = DMat3::from_diagonal(DVec3::new(-1.0, -1.0, 1.0));
    for group in lune_test_set() {
        let text = group.text();
        let domain = built(&text);
        assert_eq!(
            vector_bits(domain.centre()),
            vector_bits(spherical::ORIGIN),
            "{text}"
        );
        let Some(order) = group.order() else {
            assert_eq!(domain.area().to_bits(), (4.0 * PI).to_bits(), "{text}");
            continue;
        };
        for (vertex, expected) in domain.vertices().iter().zip(LUNE_VERTICES) {
            assert_eq!(vector_bits(*vertex), vector_bits(expected), "{text}");
        }
        for side in domain.sides() {
            match group {
                LuneGroup::ConeCrossCap(_) => assert_eq!(
                    side.pairing(),
                    half_turn * spherical::reflection(side.normal()),
                    "{text}"
                ),
                LuneGroup::CrossCap => assert_eq!(side.pairing(), -DMat3::IDENTITY, "{text}"),
                LuneGroup::Mirror => assert_eq!(
                    side.pairing(),
                    DMat3::from_diagonal(DVec3::new(1.0, 1.0, -1.0)),
                    "{text}"
                ),
                LuneGroup::Trivial => {}
            }
            if order == 1 {
                assert_eq!(side.normal(), DVec3::NEG_Z, "{text}");
            }
        }
    }
}

fn lune_worst(groups: &[LuneGroup]) -> [(Worst, f64); 7] {
    let lunes = groups
        .iter()
        .filter(|group| group.order().is_some())
        .count();
    let flats = 2 * groups
        .iter()
        .filter(|group| group.order() == Some(1))
        .count();
    let cycles = groups
        .iter()
        .filter(|group| matches!(group, LuneGroup::ConeCrossCap(_) | LuneGroup::CrossCap))
        .count();
    [
        (
            "vertex, largest coordinate error",
            2 * lunes,
            LUNE_VERTEX_BOUND,
        ),
        (
            "normal, largest coordinate error",
            2 * lunes,
            LUNE_NORMAL_BOUND,
        ),
        (
            "pairing, largest entry error",
            2 * lunes,
            LUNE_PAIRING_BOUND,
        ),
        ("side length, absolute error", 2 * lunes, LUNE_LENGTH_BOUND),
        (
            "angle sum of a cycle, absolute error against 2 pi / order",
            cycles,
            LUNE_CYCLE_BOUND,
        ),
        (
            "flat vertex, absolute error against pi",
            flats,
            LUNE_FLAT_BOUND,
        ),
        (
            "area, absolute error against the area of the signature",
            lunes,
            LUNE_AREA_BOUND,
        ),
    ]
    .map(|(name, samples, bound)| {
        (
            Worst::new(format!("spherical lune groups, {name}"), samples),
            bound,
        )
    })
}

#[test]
fn spherical_lune_groups_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let pi = reference.pi();
    let groups = lune_test_set();
    let mut worst = lune_worst(&groups);
    for group in groups {
        let text = group.text();
        let input = || text.clone();
        let domain = built(&text);
        let Some(order) = group.order() else {
            continue;
        };
        let exact = reference.lune(order);
        let [vertices, normals, pairings, lengths, cycle, flat, area] = &mut worst;
        for (vertex, exact) in domain.vertices().iter().zip(&exact.vertices) {
            vertices
                .0
                .record(&largest_coordinate_error(*vertex, exact), &int(1), input);
        }
        for (index, side) in domain.sides().iter().enumerate() {
            normals.0.record(
                &largest_coordinate_error(side.normal(), &exact.normals[index]),
                &int(1),
                input,
            );
            pairings.0.record(
                &largest_entry_error(
                    side.pairing(),
                    &exact_lune_pairing(group, &exact.normals, index),
                ),
                &int(1),
                input,
            );
        }
        for (length, exact) in domain.side_lengths().into_iter().zip(&exact.lengths) {
            lengths.0.record(&error(length, exact), &int(1), input);
        }
        let angles = domain.corner_angles();
        if group != LuneGroup::Mirror {
            let expected = div(&mul(&int(2), &pi), &int(order));
            cycle
                .0
                .record(&error(angles[0] + angles[1], &expected), &int(1), input);
        }
        if order == 1 {
            for angle in &angles {
                flat.0.record(&error(*angle, &pi), &int(1), input);
            }
        }
        area.0
            .record(&error(domain.area(), &exact.area), &int(1), input);
    }
    for (worst, bound) in &worst {
        worst.assert_at_most(&mut reference, *bound);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Axes {
    order: usize,
    count: usize,
    on_mirrors: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Census {
    order: usize,
    rotations: usize,
    reversing: usize,
    reflections: usize,
    antipodal: bool,
    axes: Vec<Axes>,
}

fn is_identity(matrix: DMat3) -> bool {
    (matrix - DMat3::IDENTITY)
        .to_cols_array()
        .iter()
        .all(|entry| entry.abs() <= CENSUS_TOLERANCE)
}

fn largest_column(matrix: DMat3) -> DVec3 {
    let columns = [matrix.x_axis, matrix.y_axis, matrix.z_axis];
    let longest = columns.into_iter().fold(DVec3::ZERO, |longest, column| {
        if column.length() > longest.length() {
            column
        } else {
            longest
        }
    });
    longest.normalize()
}

fn trace(matrix: DMat3) -> f64 {
    matrix.x_axis.x + matrix.y_axis.y + matrix.z_axis.z
}

fn census(group: &Group) -> Census {
    let (mut rotations, mut reversing, mut antipodal) = (0, 0, false);
    let mut mirrors: Vec<DVec3> = Vec::new();
    let mut lines: Vec<(DVec3, usize)> = Vec::new();
    for &(element, _) in &group.elements {
        if element.determinant() > 0.0 {
            rotations += 1;
            if is_identity(element) {
                continue;
            }
            let fixed = element
                + element.transpose()
                + DMat3::from_diagonal(DVec3::splat(1.0 - trace(element)));
            let direction = largest_column(fixed);
            match lines
                .iter_mut()
                .find(|(other, _)| other.cross(direction).length() <= CENSUS_TOLERANCE)
            {
                Some((_, count)) => *count += 1,
                None => lines.push((direction, 1)),
            }
        } else {
            reversing += 1;
            let trace = trace(element);
            if (trace + 3.0).abs() <= CENSUS_TOLERANCE {
                antipodal = true;
            } else if (trace - 1.0).abs() <= CENSUS_TOLERANCE && is_identity(element * element) {
                mirrors.push(largest_column(DMat3::IDENTITY - element));
            }
        }
    }
    let mut grouped: Vec<Axes> = Vec::new();
    for (direction, count) in lines {
        let order = count + 1;
        let on_mirror = mirrors
            .iter()
            .any(|normal| normal.dot(direction).abs() <= CENSUS_TOLERANCE);
        match grouped.iter_mut().find(|axes| axes.order == order) {
            Some(axes) => {
                axes.count += 1;
                axes.on_mirrors += usize::from(on_mirror);
            }
            None => grouped.push(Axes {
                order,
                count: 1,
                on_mirrors: usize::from(on_mirror),
            }),
        }
    }
    grouped.sort_by(|first, second| second.cmp(first));
    Census {
        order: group.elements.len(),
        rotations,
        reversing,
        reflections: mirrors.len(),
        antipodal,
        axes: grouped,
    }
}

const fn axes(order: usize, count: usize, on_mirrors: usize) -> Axes {
    Axes {
        order,
        count,
        on_mirrors,
    }
}

fn tabled_census(text: &str) -> Option<Census> {
    let row = |order, rotations, reversing, reflections, antipodal, axes: &[Axes]| Census {
        order,
        rotations,
        reversing,
        reflections,
        antipodal,
        axes: axes.to_vec(),
    };
    Some(match text {
        "1" => row(1, 1, 0, 0, false, &[]),
        "*" => row(2, 1, 1, 1, false, &[]),
        "x" => row(2, 1, 1, 0, true, &[]),
        "2x" => row(4, 2, 2, 0, false, &[axes(2, 1, 0)]),
        "3x" => row(6, 3, 3, 0, true, &[axes(3, 1, 0)]),
        "4x" => row(8, 4, 4, 0, false, &[axes(4, 1, 0)]),
        "(100)x" => row(200, 100, 100, 0, false, &[axes(100, 1, 0)]),
        "2*" => row(4, 2, 2, 1, true, &[axes(2, 1, 0)]),
        "3*" => row(6, 3, 3, 1, false, &[axes(3, 1, 0)]),
        "4*" => row(8, 4, 4, 1, true, &[axes(4, 1, 0)]),
        "2*2" => row(8, 4, 4, 2, false, &[axes(2, 3, 1)]),
        "2*3" => row(12, 6, 6, 3, true, &[axes(3, 1, 1), axes(2, 3, 0)]),
        "3*2" => row(24, 12, 12, 3, true, &[axes(3, 4, 0), axes(2, 3, 3)]),
        "(100)*" => row(200, 100, 100, 1, true, &[axes(100, 1, 0)]),
        "2*(50)" => row(200, 100, 100, 50, false, &[axes(50, 1, 1), axes(2, 50, 0)]),
        _ => return None,
    })
}

fn family_census(kite: Kite) -> Census {
    let widen = |order: u32| usize::try_from(order).unwrap_or(usize::MAX);
    match kite {
        Kite::ThreeMirrorTwo => Census {
            order: 24,
            rotations: 12,
            reversing: 12,
            reflections: 3,
            antipodal: true,
            axes: vec![axes(3, 4, 0), axes(2, 3, 3)],
        },
        Kite::ConeMirror(order) => Census {
            order: 2 * widen(order),
            rotations: widen(order),
            reversing: widen(order),
            reflections: 1,
            antipodal: order % 2 == 0,
            axes: vec![axes(widen(order), 1, 0)],
        },
        Kite::TwoMirror(order) => Census {
            order: 4 * widen(order),
            rotations: 2 * widen(order),
            reversing: 2 * widen(order),
            reflections: widen(order),
            antipodal: order % 2 == 1,
            axes: if order == 2 {
                vec![axes(2, 3, 1)]
            } else {
                vec![axes(widen(order), 1, 1), axes(2, widen(order), 0)]
            },
        },
    }
}

fn lune_family_census(group: LuneGroup) -> Census {
    let widen = |order: u32| usize::try_from(order).unwrap_or(usize::MAX);
    match group {
        LuneGroup::ConeCrossCap(order) => Census {
            order: 2 * widen(order),
            rotations: widen(order),
            reversing: widen(order),
            reflections: 0,
            antipodal: order % 2 == 1,
            axes: vec![axes(widen(order), 1, 0)],
        },
        LuneGroup::CrossCap => Census {
            order: 2,
            rotations: 1,
            reversing: 1,
            reflections: 0,
            antipodal: true,
            axes: Vec::new(),
        },
        LuneGroup::Mirror => Census {
            order: 2,
            rotations: 1,
            reversing: 1,
            reflections: 1,
            antipodal: false,
            axes: Vec::new(),
        },
        LuneGroup::Trivial => Census {
            order: 1,
            rotations: 1,
            reversing: 0,
            reflections: 0,
            antipodal: false,
            axes: Vec::new(),
        },
    }
}

fn checked_against_the_table(text: &str, family: Census) -> Census {
    if let Some(tabled) = tabled_census(text) {
        assert_eq!(family, tabled, "{text}");
    }
    family
}

fn expected_census(kite: Kite) -> Census {
    checked_against_the_table(&kite.text(), family_census(kite))
}

fn expected_lune_census(group: LuneGroup) -> Census {
    checked_against_the_table(&group.text(), lune_family_census(group))
}

fn kite_censuses(kites: &[Kite]) -> Vec<(String, Census)> {
    kites
        .iter()
        .map(|&kite| (kite.text(), expected_census(kite)))
        .collect()
}

fn lune_censuses(groups: &[LuneGroup]) -> Vec<(String, Census)> {
    groups
        .iter()
        .map(|&group| (group.text(), expected_lune_census(group)))
        .collect()
}

fn near_points(domain: &FundamentalDomain) -> Vec<DVec3> {
    let mut points = Vec::new();
    let directions = [
        DVec3::X,
        DVec3::Y,
        DVec3::Z,
        -DVec3::X,
        -DVec3::Y,
        -DVec3::Z,
        DVec3::new(0.6, 0.0, 0.8),
    ];
    for &vertex in domain.vertices() {
        for offset in [1e-3, 1e-6] {
            for direction in directions {
                points.push((vertex + direction * offset).normalize());
            }
        }
    }
    let count = domain.vertices().len();
    for (index, side) in domain.sides().iter().enumerate() {
        let (start, end) = (
            domain.vertices()[(index + count - 1) % count],
            domain.vertices()[index],
        );
        let antipodal = (start + end).abs().max_element() <= WALL_TOLERANCE;
        let across = side.normal().cross(end).normalize();
        for step in 1..16 {
            let along = f64::from(step) / 16.0;
            let on_side = if antipodal {
                let angle = PI * along;
                end * angle.cos() + across * angle.sin()
            } else {
                (start * (1.0 - along) + end * along).normalize()
            };
            for offset in [1e-3, -1e-3, 1e-6, -1e-6] {
                points.push((on_side + side.normal() * offset).normalize());
            }
        }
    }
    points
}

fn largest_side_value(domain: &FundamentalDomain, point: DVec3) -> f64 {
    domain
        .sides()
        .iter()
        .map(|side| spherical::side(side.covector(), point))
        .fold(f64::NEG_INFINITY, f64::max)
}

struct Certified {
    closure: usize,
    smallest: f64,
    left_out: usize,
    points: usize,
    census: Census,
}

fn certify(text: &str, expected: &Census, floor: f64) -> Certified {
    let domain = built(text);
    let order = group_order(text);
    let group = Group::enumerate(&domain, order);
    assert_eq!(group.elements.len(), order, "{text}");
    let centre = domain.centre();
    let images: Vec<DVec3> = group
        .elements
        .iter()
        .map(|(element, _)| *element * centre)
        .collect();
    let mut smallest = f64::INFINITY;
    for (index, first) in images.iter().enumerate() {
        for second in &images[index + 1..] {
            smallest = smallest.min(spherical::distance(*first, *second));
        }
        let Some((folded, _)) = fold(&domain, *first) else {
            panic!("{text}: the image {first:?} reaches the fold cap");
        };
        let offset = (folded - centre).abs().max_element();
        assert!(offset <= FOLD_BACK_TOLERANCE, "{text}: {offset:e}");
    }
    assert!(smallest > floor, "{text}: {smallest:e}");
    let mut points = sphere_sample(SAMPLE_POINTS);
    points.extend(near_points(&domain));
    let mut left_out = 0;
    for &point in &points {
        let mut inside = Vec::new();
        let mut near_wall = false;
        for (element, _) in &group.elements {
            let image = *element * point;
            let value = largest_side_value(&domain, image);
            if value < -WALL_TOLERANCE {
                inside.push(image);
            } else if value <= WALL_TOLERANCE {
                near_wall = true;
            }
        }
        if near_wall {
            left_out += 1;
            continue;
        }
        let [image] = inside[..] else {
            panic!("{text}: {point:?} has {} images inside", inside.len());
        };
        let Some((folded, _)) = fold(&domain, point) else {
            panic!("{text}: {point:?} reaches the fold cap");
        };
        let offset = (folded - image).abs().max_element();
        assert!(
            offset <= FOLD_BACK_TOLERANCE,
            "{text}: {point:?}: {offset:e}"
        );
    }
    let census = census(&group);
    assert_eq!(&census, expected, "{text}");
    Certified {
        closure: group.elements.len(),
        smallest,
        left_out,
        points: points.len(),
        census,
    }
}

fn certify_all(groups: &[(String, Census)], floor: f64, name: &str) {
    let mut smallest = (f64::INFINITY, String::new());
    for (text, expected) in groups {
        let certified = certify(text, expected, floor);
        let text = text.clone();
        println!(
            "{text}: closure of {} elements, smallest distance between two images of the centre {:.3e}, {} of {} points left out, census {:?}",
            certified.closure,
            certified.smallest,
            certified.left_out,
            certified.points,
            certified.census
        );
        if certified.smallest < smallest.0 {
            smallest = (certified.smallest, text);
        }
    }
    println!(
        "{name}: smallest distance between two images of the centre {:.3e}, floor {floor:e}, margin {:.1}, at {}",
        smallest.0,
        smallest.0 / floor,
        smallest.1
    );
}

#[test]
fn spherical_kite_groups_have_their_order_tiling_and_census() {
    certify_all(
        &kite_censuses(&small_kites()),
        KITE_IMAGE_FLOOR,
        "spherical kite certificate",
    );
}

#[test]
fn spherical_lune_groups_have_their_order_tiling_and_census() {
    certify_all(
        &lune_censuses(&small_lune_groups()),
        KITE_IMAGE_FLOOR,
        "spherical lune group certificate",
    );
}

#[test]
fn spherical_lune_groups_at_the_limit_have_their_order_tiling_and_census() {
    let started = Instant::now();
    certify_all(
        &lune_censuses(&[LuneGroup::ConeCrossCap(LIMIT_ORDER)]),
        LIMIT_IMAGE_FLOOR,
        "spherical lune group certificate at the limit",
    );
    println!(
        "spherical lune group certificate at the limit in {:.2} s",
        started.elapsed().as_secs_f64()
    );
}

#[test]
fn spherical_kite_groups_at_the_limit_have_their_order_tiling_and_census() {
    let started = Instant::now();
    certify_all(
        &kite_censuses(&limit_kites()),
        LIMIT_IMAGE_FLOOR,
        "spherical kite certificate at the limit",
    );
    println!(
        "spherical kite certificate at the limit in {:.2} s",
        started.elapsed().as_secs_f64()
    );
}

#[test]
fn census_tells_each_wrong_group_from_its_signature() {
    let mut cases: Vec<(String, String, Census)> = [
        ("2*2", Kite::ConeMirror(4)),
        ("2*3", Kite::ConeMirror(6)),
        ("6*", Kite::TwoMirror(3)),
    ]
    .into_iter()
    .map(|(domain_text, label)| (domain_text.to_owned(), label.text(), expected_census(label)))
    .collect();
    for order in [3, 4] {
        let label = LuneGroup::ConeCrossCap(order);
        let double = 2 * order;
        for domain_text in [format!("{double}{double}"), format!("*{order}{order}")] {
            cases.push((domain_text, label.text(), expected_lune_census(label)));
        }
    }
    for (domain_text, label_text, expected) in cases {
        let domain_text = domain_text.as_str();
        let domain = built(domain_text);
        assert!(domain.verify().passed(), "{domain_text}");
        let presentation = match domain.side_presentation() {
            Ok(presentation) => presentation,
            Err(error) => panic!("{domain_text} has no side presentation: {error}"),
        };
        let table = match enumerate(&presentation, &[], CosetLimit::DEFAULT) {
            Ok(table) => table,
            Err(error) => panic!("{domain_text}: {error}"),
        };
        let order = group_order(&label_text);
        assert_eq!(table.index(), order, "{domain_text} as {label_text}");
        let found = census(&Group::enumerate(&domain, order));
        println!(
            "{domain_text} labelled {label_text}: census {found:?}, of the label {expected:?}"
        );
        assert_ne!(found, expected, "{domain_text} as {label_text}");
    }
}
