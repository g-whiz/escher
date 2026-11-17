use escher_geometry::hyperbolic::{
    ORIGIN, boost, from_disk, normalize_point, rotation, side, translation,
};
use escher_geometry::spherical;
use escher_render::{Fill, RenderOptions, Renderer, fold};
use escher_signature::{Geometry, Signature};
use glam::{DMat3, DVec3};

use crate::common::{
    BANDS, DEFAULT_CAP, EUCLIDEAN_DISTANCES, EUCLIDEAN_DOMAINS, EUCLIDEAN_REPLAY_BOUNDS,
    FOLD_TRIPLES, REPLAY_BOUNDS, RotationCase, SPHERICAL_DECISION_COUNTS, SPHERICAL_DOMAINS,
    SPHERICAL_IMAGE_BOUNDS, SPHERICAL_REPLAY_BOUNDS, TOLERANCE, VERTEX_TRIPLES, describe,
    describe_plane, describe_sphere, domain, euclidean_domain, euclidean_fold_set, euclidean_name,
    euclidean_rotation_cases, fold_set, hyperbolic_rotation_cases, one_fill, options,
    pixel_position, reference_walls, rim_set, rotation_domain, rotation_text, spherical_domain,
    spherical_fold_set, spherical_rotation_cases, text,
};
use crate::golden_images::domain_with_aspect;
use crate::reference::{self, Reference, Worst, count_bound, num, vector};

const DECISION_COUNTS: [usize; 6] = [0, 0, 0, 0, 0, 6];

const EUCLIDEAN_DECISION_COUNTS: [usize; 4] = [0, 0, 0, 0];

const VERTEX_ASPECTS: u32 = 64;

const CENTRE_ORDERS: [u32; 7] = [2, 3, 5, 12, 100, 999, 1000];

const SPHERICAL_LUNES: usize = 10;

const NEAR_VERTEX_DOMAINS: [(&str, f64); 4] = [
    ("*(1000)(1000)(1000)", 1e-7),
    ("*(1000)(1000)(999)", 2e-7),
    ("*(998)(981)(961)", 1e-11),
    ("*(1000)32", 1e-11),
];

const NEAR_VERTEX_DISTANCES: [f64; 13] = [
    1e-5, 3e-6, 1e-6, 6e-7, 3e-7, 2e-7, 1e-7, 3e-8, 2e-8, 1e-8, 1e-9, 1e-10, 1e-11,
];

const NEAR_VERTEX_DIRECTIONS: u32 = 256;

fn sides_of(
    domain: &escher_domain::FundamentalDomain,
    point: DVec3,
    cap: u32,
) -> (escher_render::Fold, Vec<usize>) {
    let mut sides = Vec::new();
    let result = fold(domain, point, cap, |index| sides.push(index));
    (result, sides)
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn fold_replay_stays_within_the_bound_of_each_band_and_triple() {
    let mut reference = Reference::new();
    for (triple, bounds) in FOLD_TRIPLES.iter().zip(REPLAY_BOUNDS) {
        let domain = domain(*triple);
        let walls = reference_walls(&domain);
        let mut worst: Vec<Worst> = BANDS
            .iter()
            .map(|band| {
                Worst::new(
                    format!(
                        "folded point, hyperbolic distance from the replay, {} band {band}",
                        text(*triple)
                    ),
                    512,
                )
            })
            .collect();
        for sample in fold_set(*triple) {
            let (result, sides) = sides_of(&domain, sample.point, DEFAULT_CAP);
            assert!(
                result.converged(),
                "{} does not converge",
                describe(&sample)
            );
            let replayed = reference::replay(&walls, sample.point.to_array(), &sides);
            let distance = reference.distance(&vector(result.point().to_array()), &replayed);
            worst[sample.band].record(&distance, &reference::int(1), || describe(&sample));
        }
        for (worst, bound) in worst.iter().zip(bounds) {
            worst.assert_at_most(&mut reference, bound);
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn fold_decisions_differ_from_the_exact_fold_within_the_count_of_each_band() {
    let mut differing = [0_usize; 6];
    for triple in FOLD_TRIPLES {
        let domain = domain(triple);
        let walls = reference_walls(&domain);
        let mut per_band = [0_usize; 6];
        for sample in fold_set(triple) {
            let (_, sides) = sides_of(&domain, sample.point, DEFAULT_CAP);
            let exact = reference::fold(&walls, sample.point.to_array(), DEFAULT_CAP);
            assert!(
                exact.converged,
                "the exact fold of {} does not converge",
                describe(&sample)
            );
            if exact.sides != sides {
                let first = sides
                    .iter()
                    .zip(&exact.sides)
                    .take_while(|(left, right)| left == right)
                    .count();
                println!(
                    "{}: the side sequences first differ at step {first}",
                    describe(&sample)
                );
                per_band[sample.band] += 1;
            }
        }
        println!(
            "{}: folds whose side sequence differs, by band: {per_band:?}",
            text(triple)
        );
        for (total, count) in differing.iter_mut().zip(per_band) {
            *total += count;
        }
    }
    for ((band, count), measured) in BANDS.iter().zip(differing).zip(DECISION_COUNTS) {
        let bound = count_bound(measured);
        println!(
            "band {band}: {count} of {} folds differ, bound {bound}",
            512 * FOLD_TRIPLES.len()
        );
        assert!(
            count <= bound,
            "band {band}: {count} folds differ, more than {bound}"
        );
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn every_fold_converges_within_the_default_cap() {
    for triple in FOLD_TRIPLES {
        let domain = domain(triple);
        let mut steps = [(0_u64, 0_u32); 6];
        for sample in fold_set(triple) {
            let result = fold(&domain, sample.point, DEFAULT_CAP, |_| {});
            assert!(
                result.converged(),
                "{} does not converge",
                describe(&sample)
            );
            let (total, largest) = &mut steps[sample.band];
            *total += u64::from(result.steps());
            *largest = (*largest).max(result.steps());
        }
        for (band, (total, largest)) in BANDS.iter().zip(steps) {
            #[expect(
                clippy::cast_precision_loss,
                reason = "the total is far below 2 to the power 52"
            )]
            let mean = total as f64 / 512.0;
            println!(
                "{} band {band}: mean steps {mean:.1}, largest {largest}",
                text(triple)
            );
        }
    }
    let options = options(RenderOptions::builder(one_fill()));
    let (_, stats) = Renderer::new(&domain([1000, 3, 2]), options).render();
    let inside = stats.samples() - stats.outside();
    #[expect(
        clippy::cast_precision_loss,
        reason = "the counts are far below 2 to the power 52"
    )]
    let mean = stats.folds() as f64 / inside as f64;
    println!(
        "*(1000)32 at 512 by 512 with 4 by 4 samples: {inside} samples inside the disk, mean steps {mean:.1}, largest {}, {} unconverged",
        stats.largest_fold(),
        stats.unconverged()
    );
    assert_eq!(stats.unconverged(), 0);
}

fn largest_vertex_ratio(
    domains: impl IntoIterator<Item = (String, escher_domain::FundamentalDomain)>,
) -> (f64, String) {
    let mut largest = (0.0_f64, String::new());
    for (name, domain) in domains {
        for (index, &vertex) in domain.vertices().iter().enumerate() {
            for (side_index, wall) in domain.sides().iter().enumerate() {
                let covector = wall.covector();
                let (value, size) = match domain.geometry() {
                    Geometry::Spherical => (
                        spherical::side(covector, vertex),
                        vertex.abs().max_element(),
                    ),
                    _ => (side(covector, vertex), vertex.z),
                };
                let ratio = value / (covector.abs().element_sum() * size);
                if ratio > largest.0 {
                    largest = (ratio, format!("{name} vertex {index} side {side_index}"));
                }
            }
            let result = fold(&domain, vertex, DEFAULT_CAP, |_| {});
            assert!(result.converged(), "{name} vertex {index}");
            assert_eq!(result.steps(), 0, "{name} vertex {index}");
            assert_eq!(result.point(), vertex);
        }
    }
    largest
}

fn vertex_aspects() -> impl Iterator<Item = f64> {
    (0..VERTEX_ASPECTS).map(|index| {
        if index + 1 == VERTEX_ASPECTS {
            8.0
        } else {
            0.125 * 64.0_f64.powf(f64::from(index) / f64::from(VERTEX_ASPECTS - 1))
        }
    })
}

#[test]
fn every_vertex_folds_in_zero_steps() {
    let hyperbolic = FOLD_TRIPLES
        .into_iter()
        .chain(VERTEX_TRIPLES)
        .map(|triple| (text(triple), domain(triple)));
    let euclidean = (0..EUCLIDEAN_DOMAINS.len())
        .map(|index| (euclidean_name(index), euclidean_domain(index)))
        .chain(core::iter::once(4.0).chain(vertex_aspects()).map(|aspect| {
            (
                format!("*2222 aspect {aspect}"),
                domain_with_aspect("*2222", Some(aspect)),
            )
        }));
    let spherical = (0..SPHERICAL_DOMAINS.len())
        .map(|index| (SPHERICAL_DOMAINS[index].to_owned(), spherical_domain(index)));
    for (set, domains) in [
        ("", largest_vertex_ratio(hyperbolic)),
        ("Euclidean ", largest_vertex_ratio(euclidean)),
        ("spherical ", largest_vertex_ratio(spherical)),
    ] {
        let (ratio, at) = domains;
        println!(
            "largest side value at a {set}vertex over the tolerance scale: {ratio:.3e} at {at}, tolerance {TOLERANCE:e}, margin {:.1}",
            TOLERANCE / ratio
        );
        assert!(8.0 * ratio <= TOLERANCE, "{ratio:e} at {at}");
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn euclidean_fold_replay_and_decisions_stay_within_their_bounds_and_converge_within_the_default_cap()
 {
    let mut reference = Reference::new();
    let mut differing = [0_usize; 4];
    for (index, bounds) in EUCLIDEAN_REPLAY_BOUNDS.iter().enumerate() {
        let domain = euclidean_domain(index);
        let walls = reference_walls(&domain);
        let mut worst: Vec<Worst> = EUCLIDEAN_DISTANCES
            .iter()
            .map(|distance| {
                Worst::new(
                    format!(
                        "folded point, plane distance from the replay, {} at distance {distance}",
                        euclidean_name(index)
                    ),
                    512,
                )
            })
            .collect();
        let mut steps = [(0_u64, 0_u32); 4];
        for sample in euclidean_fold_set(index) {
            let (result, sides) = sides_of(&domain, sample.point, DEFAULT_CAP);
            assert!(
                result.converged(),
                "{} does not converge",
                describe_plane(&sample)
            );
            assert_eq!(result.point().z, 1.0, "{}", describe_plane(&sample));
            let (total, largest) = &mut steps[sample.distance];
            *total += u64::from(result.steps());
            *largest = (*largest).max(result.steps());
            let replayed = reference::replay(&walls, sample.point.to_array(), &sides);
            let distance = reference::plane_distance(&vector(result.point().to_array()), &replayed);
            worst[sample.distance]
                .record(&distance, &reference::int(1), || describe_plane(&sample));
            let exact = reference::fold(&walls, sample.point.to_array(), DEFAULT_CAP);
            assert!(
                exact.converged,
                "the exact fold of {} does not converge",
                describe_plane(&sample)
            );
            if exact.sides != sides {
                println!("{}: the side sequences differ", describe_plane(&sample));
                differing[sample.distance] += 1;
            }
        }
        for (worst, &bound) in worst.iter().zip(bounds) {
            worst.assert_at_most(&mut reference, bound);
        }
        for (distance, (total, largest)) in EUCLIDEAN_DISTANCES.iter().zip(steps) {
            #[expect(
                clippy::cast_precision_loss,
                reason = "the total is far below 2 to the power 52"
            )]
            let mean = total as f64 / 512.0;
            println!(
                "{} at distance {distance}: mean steps {mean:.1}, largest {largest}",
                euclidean_name(index)
            );
        }
    }
    for ((distance, count), measured) in EUCLIDEAN_DISTANCES
        .iter()
        .zip(differing)
        .zip(EUCLIDEAN_DECISION_COUNTS)
    {
        let bound = count_bound(measured);
        println!(
            "distance {distance}: {count} of {} folds differ, bound {bound}",
            512 * EUCLIDEAN_DOMAINS.len()
        );
        assert!(
            count <= bound,
            "distance {distance}: {count} folds differ, more than {bound}"
        );
    }
    for index in 0..EUCLIDEAN_DOMAINS.len() {
        let options = options(RenderOptions::builder(one_fill()));
        let (_, stats) = Renderer::new(&euclidean_domain(index), options).render();
        #[expect(
            clippy::cast_precision_loss,
            reason = "the counts are far below 2 to the power 52"
        )]
        let mean = stats.folds() as f64 / stats.samples() as f64;
        println!(
            "{} at 512 by 512 with 4 by 4 samples: mean steps {mean:.2}, largest {}, {} unconverged, {} outside",
            euclidean_name(index),
            stats.largest_fold(),
            stats.unconverged(),
            stats.outside()
        );
        assert_eq!(stats.unconverged(), 0);
        assert_eq!(stats.outside(), 0);
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn spherical_fold_replay_and_decisions_stay_within_their_bounds_and_converge_within_the_default_cap()
 {
    let mut reference = Reference::new();
    for (index, (&name, &bound)) in SPHERICAL_DOMAINS
        .iter()
        .zip(&SPHERICAL_REPLAY_BOUNDS)
        .enumerate()
    {
        let domain = spherical_domain(index);
        let walls = reference_walls(&domain);
        let set = spherical_fold_set(index);
        let mut worst = Worst::new(
            format!("folded point, angle from the replay, {name}"),
            set.len(),
        );
        let mut differing = 0_usize;
        let mut total = 0_u64;
        let mut largest = 0_u32;
        for sample in &set {
            let (result, sides) = sides_of(&domain, sample.point, DEFAULT_CAP);
            assert!(
                result.converged(),
                "{} does not converge",
                describe_sphere(sample)
            );
            total += u64::from(result.steps());
            largest = largest.max(result.steps());
            let replayed = reference::replay(&walls, sample.point.to_array(), &sides);
            let angle = reference.angle(&vector(result.point().to_array()), &replayed);
            worst.record(&angle, &reference::int(1), || describe_sphere(sample));
            let exact = reference::fold(&walls, sample.point.to_array(), DEFAULT_CAP);
            assert!(
                exact.converged,
                "the exact fold of {} does not converge",
                describe_sphere(sample)
            );
            if exact.sides != sides {
                println!("{}: the side sequences differ", describe_sphere(sample));
                differing += 1;
            }
        }
        worst.assert_at_most(&mut reference, bound);
        #[expect(
            clippy::cast_precision_loss,
            reason = "the total is far below 2 to the power 52"
        )]
        let mean = total as f64 / set.len() as f64;
        let allowed = count_bound(SPHERICAL_DECISION_COUNTS[index]);
        println!(
            "{name}: mean steps {mean:.1}, largest {largest}, {differing} of {} folds differ, bound {allowed}",
            set.len()
        );
        assert!(
            differing <= allowed,
            "{name}: {differing} folds differ, more than {allowed}"
        );
    }
}

struct GroupElement {
    approximate: DMat3,
    exact: reference::Matrix,
    depth: u32,
}

fn enumerate_group(domain: &escher_domain::FundamentalDomain, order: u64) -> Vec<GroupElement> {
    let generators: Vec<(DMat3, reference::Matrix)> = domain
        .sides()
        .iter()
        .map(|side| {
            let pairing = side.pairing();
            (pairing, reference::matrix(pairing.to_cols_array()))
        })
        .collect();
    let mut elements = vec![GroupElement {
        approximate: DMat3::IDENTITY,
        exact: reference::matrix(DMat3::IDENTITY.to_cols_array()),
        depth: 0,
    }];
    let mut next = 0;
    while next < elements.len() {
        let depth = elements[next].depth;
        for (generator, exact) in &generators {
            let product = elements[next].approximate * *generator;
            if !elements
                .iter()
                .any(|known| known.approximate.abs_diff_eq(product, 1e-9))
            {
                let exact = reference::product(&elements[next].exact, exact);
                elements.push(GroupElement {
                    approximate: product,
                    exact,
                    depth: depth + 1,
                });
                assert!(
                    u64::try_from(elements.len()).is_ok_and(|count| count <= order),
                    "the enumeration found more than {order} elements"
                );
            }
        }
        next += 1;
    }
    elements
}

fn image_bound(largest: usize, first: usize, order: u32) -> f64 {
    if order == 1000 {
        return SPHERICAL_IMAGE_BOUNDS[largest];
    }
    let position = CENTRE_ORDERS
        .iter()
        .position(|&known| known >= order)
        .unwrap_or_else(|| panic!("{order} is above every order of the set"));
    SPHERICAL_REPLAY_BOUNDS[first + position]
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration of the group runs loops too long for the interpreter"
)]
fn images_of_the_centre_under_the_group_fold_back_in_as_many_steps_as_their_depth() {
    let mut reference = Reference::new();
    let signatures = [("*332", 0), ("*432", 1), ("*532", 2)]
        .into_iter()
        .map(|(name, index)| (name.to_owned(), SPHERICAL_REPLAY_BOUNDS[index]))
        .chain((2..=12).chain([1000]).flat_map(|order| {
            let corner = if order < 10 {
                order.to_string()
            } else {
                format!("({order})")
            };
            [
                (format!("*{corner}22"), image_bound(0, 3, order)),
                (format!("*{corner}{corner}"), image_bound(1, 10, order)),
            ]
        }));
    for (name, bound) in signatures {
        let signature: Signature = name
            .parse()
            .unwrap_or_else(|error| panic!("{name:?} is rejected: {error}"));
        let order = signature
            .group_order()
            .unwrap_or_else(|| panic!("{name} has no group order"))
            .get();
        let domain = domain_with_aspect(&name, None);
        let elements = enumerate_group(&domain, order);
        assert_eq!(elements.len() as u64, order, "{name}");
        let centre = domain.centre();
        let mut largest_depth = 0;
        let mut largest = 0.0_f64;
        let exact_centre = vector(centre.to_array());
        for element in &elements {
            let depth = element.depth;
            let exact_image = reference::apply(&element.exact, &exact_centre);
            let image =
                DVec3::from_array([0, 1, 2].map(|row| reference.approximate(&exact_image[row])));
            let result = fold(&domain, image, DEFAULT_CAP, |_| {});
            assert!(result.converged(), "{name} at depth {depth}");
            assert_eq!(result.steps(), depth, "{name} at {:?}", element.approximate);
            let angle = reference.angle(
                &vector(result.point().to_array()),
                &vector(centre.to_array()),
            );
            let angle = reference.approximate(&angle);
            largest_depth = largest_depth.max(depth);
            largest = largest.max(angle);
            assert!(
                angle <= bound,
                "{name} at depth {depth}: angle {angle:e} exceeds {bound:e}"
            );
        }
        println!(
            "{name}: {} elements, largest depth {largest_depth}, largest angle {largest:.3e}, bound {bound:e}",
            elements.len()
        );
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "folding thousands of points of a large group runs loops too long for the interpreter"
)]
fn fold_near_a_vertex_of_the_largest_order_and_its_antipode_returns_within_the_cap() {
    let domain = domain_with_aspect("*(1000)22", None);
    let target = core::f64::consts::PI / 1000.0;
    let (index, _) = domain
        .corner_angles()
        .into_iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| (left - target).abs().total_cmp(&(right - target).abs()))
        .unwrap_or_else(|| panic!("the domain has no corner"));
    let vertex = domain.vertices()[index];
    let first = vertex.any_orthonormal_vector();
    let second = vertex.cross(first).normalize();
    for (name, centre) in [("vertex", vertex), ("antipode", -vertex)] {
        let mut unconverged = 0;
        for direction in 0..1000_u32 {
            let heading = core::f64::consts::TAU * f64::from(direction) / 1000.0;
            let angle = 1e-14 * f64::from(direction + 1) / 1000.0;
            let tangent = first * heading.cos() + second * heading.sin();
            let point = centre * angle.cos() + tangent * angle.sin();
            let result = fold(&domain, point, DEFAULT_CAP, |_| {});
            assert!(
                result.steps() <= DEFAULT_CAP,
                "{name} direction {direction}"
            );
            if !result.converged() {
                unconverged += 1;
            }
        }
        println!(
            "*(1000)22 near its {name} of order 1000: {unconverged} of 1000 points unconverged"
        );
    }
}

#[test]
fn spherical_fold_of_wrong_inputs_stops_as_the_cap_and_the_point_size_require() {
    let domain = spherical_domain(2);
    for vector in [DVec3::ZERO, -DVec3::ZERO] {
        let result = fold(&domain, vector, DEFAULT_CAP, |_| {});
        assert!(!result.converged(), "{vector:?}");
        assert_eq!(result.steps(), 0, "{vector:?}");
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for point in [
            DVec3::new(value, 0.0, 1.0),
            DVec3::new(0.0, value, 1.0),
            DVec3::new(0.0, 0.0, value),
        ] {
            let result = fold(&domain, point, DEFAULT_CAP, |_| {});
            assert!(!result.converged(), "{point:?}");
            assert_eq!(result.steps(), 0, "{point:?}");
        }
    }
    let point = DVec3::new(0.0, 0.0, -1.0);
    let needed = fold(&domain, point, DEFAULT_CAP, |_| {});
    println!(
        "*532 at (0, 0, -1): converged {}, {} steps",
        needed.converged(),
        needed.steps()
    );
    assert!(needed.converged());
    assert!(needed.steps() > 1);
    let none = fold(&domain, point, 0, |_| {});
    assert!(!none.converged());
    assert_eq!(none.steps(), 0);
    assert_eq!(none.point(), point);
    let capped = fold(&domain, point, needed.steps() - 1, |_| {});
    assert!(!capped.converged());
    assert_eq!(capped.steps(), needed.steps() - 1);
}

#[test]
fn fold_of_a_point_with_a_coordinate_that_is_not_finite_does_not_converge() {
    let domain = domain([7, 3, 2]);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for point in [
            DVec3::new(value, 0.0, 1.0),
            DVec3::new(0.0, value, 1.0),
            DVec3::new(0.0, 0.0, value),
        ] {
            let result = fold(&domain, point, DEFAULT_CAP, |_| {});
            assert!(!result.converged(), "{point:?}");
            assert_eq!(result.steps(), 0, "{point:?}");
        }
    }
}

#[test]
fn fold_of_a_vector_with_a_last_coordinate_that_is_not_positive_stops_unconverged_at_once() {
    let domain = domain([7, 3, 2]);
    for vector in [
        DVec3::ZERO,
        DVec3::new(0.5, 0.25, 0.0),
        DVec3::new(0.5, 0.25, -0.0),
        DVec3::X,
        -ORIGIN,
    ] {
        let result = fold(&domain, vector, DEFAULT_CAP, |_| {});
        assert!(!result.converged(), "{vector:?}");
        assert_eq!(result.steps(), 0, "{vector:?}");
    }
}

#[test]
fn fold_of_a_spacelike_vector_with_a_positive_last_coordinate_returns_unconverged() {
    let domain = domain([7, 3, 2]);
    let vector = DVec3::new(3.0, -2.0, 0.5);
    let result = fold(&domain, vector, DEFAULT_CAP, |_| {});
    assert!(!result.converged(), "{result:?}");
    assert!(result.steps() <= DEFAULT_CAP, "{result:?}");
}

#[test]
fn fold_with_a_cap_of_zero_leaves_a_point_outside_the_domain_unconverged() {
    let domain = domain([7, 3, 2]);
    let Some(point) = from_disk(glam::DVec2::new(0.5, 0.25)) else {
        panic!("the position is inside the disk");
    };
    let result = fold(&domain, point, 0, |_| {});
    assert!(!result.converged());
    assert_eq!(result.steps(), 0);
    assert_eq!(result.point(), point);
}

#[test]
fn fold_with_a_cap_one_below_the_steps_needed_stops_at_the_cap() {
    let domain = domain([7, 3, 2]);
    let Some(point) = from_disk(glam::DVec2::new(0.6, -0.3)) else {
        panic!("the position is inside the disk");
    };
    let needed = fold(&domain, point, DEFAULT_CAP, |_| {});
    assert!(needed.converged());
    assert!(needed.steps() > 1);
    let capped = fold(&domain, point, needed.steps() - 1, |_| {});
    assert!(!capped.converged());
    assert_eq!(capped.steps(), needed.steps() - 1);
}

#[test]
fn fold_of_a_point_that_overflows_during_the_fold_does_not_converge() {
    for triple in [[7, 3, 2], [1000, 3, 2]] {
        let domain = domain(triple);
        let result = fold(
            &domain,
            DVec3::new(f64::MAX, 0.0, f64::MAX),
            DEFAULT_CAP,
            |_| {},
        );
        assert!(!result.converged(), "{} gives {result:?}", text(triple));
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn fold_and_render_beyond_the_range_of_accuracy_return_without_a_panic() {
    let mut converged = 0;
    let mut unconverged = 0;
    let mut outside = 0;
    for triple in FOLD_TRIPLES {
        let domain = domain(triple);
        for position in rim_set() {
            let Some(point) = from_disk(position) else {
                outside += 1;
                continue;
            };
            let result = fold(&domain, point, DEFAULT_CAP, |_| {});
            if result.converged() {
                converged += 1;
                assert!(
                    result.point().is_finite(),
                    "{} at {position:?}",
                    text(triple)
                );
                for wall in domain.sides() {
                    assert!(side(wall.covector(), result.point()).is_finite());
                }
            } else {
                unconverged += 1;
            }
        }
    }
    println!(
        "rim set: {converged} converged, {unconverged} unconverged, {outside} outside the disk"
    );
    let fill = one_fill();
    let size = 4096;
    let options = options(RenderOptions::builder(fill).size(size, size).samples(1));
    let renderer = Renderer::new(&domain([7, 3, 2]), options);
    let Fill::One(colour) = fill else {
        panic!("the rim test uses one fill colour");
    };
    let allowed = [
        colour,
        escher_render::Colour::BLACK,
        escher_render::Colour::WHITE,
    ]
    .map(escher_render::Colour::to_srgb8);
    let mut beyond = 0;
    for row in 0..size {
        let edge = pixel_position(size, 0, row).y;
        let half_chord = (1.0 - edge * edge).max(0.0).sqrt();
        let pixel = 2.0 / (15.0 / 16.0 * f64::from(size));
        let near_edge = |x: f64| x / pixel + f64::from(size) / 2.0 - 0.5;
        let mut found = false;
        for centre in [near_edge(-half_chord), near_edge(half_chord)] {
            for offset in -1..=1 {
                let column = centre.floor() + f64::from(offset);
                if !(0.0..f64::from(size)).contains(&column) {
                    continue;
                }
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "the column is a whole number inside the image"
                )]
                let column = column as u32;
                let position = pixel_position(size, column, row);
                if from_disk(position).is_some() && 1.0 - position.length() < 1e-6 {
                    found = true;
                }
            }
        }
        if !found {
            continue;
        }
        let mut band = vec![0; 3 * 4096];
        let stats = renderer
            .render_rows(row, &mut band)
            .unwrap_or_else(|error| panic!("row {row}: {error}"));
        beyond += 1;
        println!(
            "*732 at {size} by {size}, row {row} with a sample beyond the range of accuracy: {stats:?}"
        );
        for pixel in band.as_chunks::<3>().0 {
            assert!(allowed.contains(pixel), "row {row}: {pixel:?}");
        }
    }
    assert!(
        beyond > 0,
        "no row of the image has a sample beyond the range of accuracy"
    );
}

fn assert_rotation_replay(cases: &[RotationCase]) {
    let mut reference = Reference::new();
    for case in cases {
        let walls = reference_walls(&case.domain);
        let groups = case.groups.len();
        let mut worst: Vec<Worst> = case
            .groups
            .iter()
            .map(|group| {
                Worst::new(
                    format!("folded point of {} from the replay, {group}", case.name),
                    0,
                )
            })
            .collect();
        let mut differing = vec![0_usize; groups];
        let mut unconverged = vec![0_usize; groups];
        let mut steps = vec![(0_u64, 0_u32, 0_u64); groups];
        for sample in &case.samples {
            let (result, sides) = sides_of(&case.domain, sample.point, DEFAULT_CAP);
            let (total, largest, count) = &mut steps[sample.group];
            *total += u64::from(result.steps());
            *largest = (*largest).max(result.steps());
            *count += 1;
            if !result.converged() {
                println!("{} does not converge", sample.name);
                unconverged[sample.group] += 1;
                continue;
            }
            let replayed = reference::replay(&walls, sample.point.to_array(), &sides);
            let distance = case.measure.distance(
                &mut reference,
                &vector(result.point().to_array()),
                &replayed,
            );
            worst[sample.group].record(&distance, &reference::int(1), || sample.name.clone());
            let exact = reference::fold(&walls, sample.point.to_array(), DEFAULT_CAP);
            if !exact.converged || exact.sides != sides {
                println!(
                    "{}: the side sequence differs from the exact fold, which converged {}",
                    sample.name, exact.converged
                );
                differing[sample.group] += 1;
            }
        }
        for (group, label) in case.groups.iter().enumerate() {
            let (total, largest, count) = steps[group];
            #[expect(
                clippy::cast_precision_loss,
                reason = "the counts are far below 2 to the power 52"
            )]
            let mean = total as f64 / count as f64;
            let allowed = count_bound(case.decisions[group]);
            println!(
                "{} {label}: mean steps {mean:.1}, largest {largest}, {} unconverged, {} of {count} folds differ, bound {allowed}",
                case.name, unconverged[group], differing[group]
            );
            if group < case.range {
                assert_eq!(unconverged[group], 0, "{} {label}", case.name);
                worst[group].assert_at_most(&mut reference, case.bounds[group]);
                assert!(
                    differing[group] <= allowed,
                    "{} {label}: {} folds differ, more than {allowed}",
                    case.name,
                    differing[group]
                );
            } else {
                println!("{} {label} lies beyond the range of accuracy", case.name);
            }
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn rotation_fold_replay_of_hyperbolic_domains_stays_within_its_bounds() {
    assert_rotation_replay(&hyperbolic_rotation_cases());
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn rotation_fold_replay_of_euclidean_domains_stays_within_its_bounds() {
    assert_rotation_replay(&euclidean_rotation_cases());
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn rotation_fold_replay_of_spherical_kites_stays_within_its_bounds() {
    assert_rotation_replay(&spherical_rotation_cases(0..SPHERICAL_LUNES));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn rotation_fold_replay_of_spherical_lunes_stays_within_its_bounds() {
    assert_rotation_replay(&spherical_rotation_cases(
        SPHERICAL_LUNES..SPHERICAL_DOMAINS.len(),
    ));
}

fn assert_double_oracle(cases: &[RotationCase]) {
    let mut reference = Reference::new();
    for case in cases {
        let reflection_walls = reference_walls(&case.reflection);
        let covectors: Vec<reference::Vector> = case
            .reflection
            .sides()
            .iter()
            .map(|side| vector(side.covector().to_array()))
            .collect();
        let doubled = &reflection_walls[1].pairing;
        let groups = case.groups.len();
        let mut kept = vec![0_usize; groups];
        let mut failed = vec![0_usize; groups];
        for sample in &case.samples {
            if sample.group >= case.range {
                continue;
            }
            let tolerance = case.reflection_bounds[sample.group] + case.bounds[sample.group];
            let clearance = num(1e-6_f64.max(2.0 * tolerance));
            let reflected = fold(&case.reflection, sample.point, DEFAULT_CAP, |_| {});
            if !reflected.converged() {
                continue;
            }
            let folded = vector(reflected.point().to_array());
            let clear = covectors.iter().all(|covector| {
                case.measure
                    .wall_distance(&mut reference, covector, &folded)
                    >= clearance
            });
            if !clear {
                continue;
            }
            kept[sample.group] += 1;
            let target = if reflected.reversals().is_multiple_of(2) {
                folded
            } else {
                reference::apply(doubled, &folded)
            };
            let rotated = fold(&case.domain, sample.point, DEFAULT_CAP, |_| {});
            let agrees = rotated.converged()
                && case.measure.distance(
                    &mut reference,
                    &vector(rotated.point().to_array()),
                    &target,
                ) <= num(tolerance);
            if !agrees {
                println!(
                    "{}: the rotation fold, converged {}, lands farther than {tolerance:e} from the reflection fold of parity {}",
                    sample.name,
                    rotated.converged(),
                    reflected.reversals() % 2
                );
                failed[sample.group] += 1;
            }
        }
        assert!(
            kept.iter().sum::<usize>() > 0,
            "{}: no point is kept",
            case.name
        );
        for (group, label) in case.groups.iter().enumerate().take(case.range) {
            let allowed = count_bound(case.oracle[group]);
            println!(
                "{} {label}: {} of {} kept points disagree with the reflection fold, bound {allowed}",
                case.name, failed[group], kept[group]
            );
            assert!(
                failed[group] <= allowed,
                "{} {label}: {} points disagree, more than {allowed}",
                case.name,
                failed[group]
            );
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn rotation_folds_of_hyperbolic_domains_agree_with_the_folds_of_their_reflection_domains() {
    assert_double_oracle(&hyperbolic_rotation_cases());
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn rotation_folds_of_euclidean_domains_agree_with_the_folds_of_their_reflection_domains() {
    assert_double_oracle(&euclidean_rotation_cases());
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn rotation_folds_of_spherical_kites_agree_with_the_folds_of_their_reflection_domains() {
    assert_double_oracle(&spherical_rotation_cases(0..SPHERICAL_LUNES));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn rotation_folds_of_spherical_lunes_agree_with_the_folds_of_their_reflection_domains() {
    assert_double_oracle(&spherical_rotation_cases(
        SPHERICAL_LUNES..SPHERICAL_DOMAINS.len(),
    ));
}

#[test]
fn every_vertex_of_a_rotation_domain_folds_in_zero_steps() {
    let hyperbolic = FOLD_TRIPLES
        .into_iter()
        .chain(VERTEX_TRIPLES)
        .chain([[868, 416, 5]])
        .map(|triple| {
            let starred = text(triple);
            let name = rotation_text(&starred).to_owned();
            let domain = rotation_domain(&starred, None);
            (name, domain)
        });
    let euclidean = EUCLIDEAN_DOMAINS
        .into_iter()
        .chain(
            core::iter::once(4.0)
                .chain(vertex_aspects())
                .map(|aspect| ("*2222", Some(aspect))),
        )
        .map(|(starred, aspect)| {
            (
                format!("{} aspect {aspect:?}", rotation_text(starred)),
                rotation_domain(starred, aspect),
            )
        });
    let spherical = SPHERICAL_DOMAINS.into_iter().map(|starred| {
        (
            rotation_text(starred).to_owned(),
            rotation_domain(starred, None),
        )
    });
    let (ratio, at) = largest_vertex_ratio(hyperbolic.chain(euclidean).chain(spherical));
    println!(
        "largest side value at a vertex of a rotation domain over the tolerance scale: {ratio:.3e} at {at}, tolerance {TOLERANCE:e}, margin {:.2}",
        TOLERANCE / ratio
    );
    assert!(ratio <= TOLERANCE, "{ratio:e} at {at}");
}

fn assert_rotation_image_converges(text: &str) {
    let options = options(RenderOptions::builder(one_fill()));
    let renderer = Renderer::new(&domain_with_aspect(text, None), options);
    let band_rows = 64;
    let band_bytes = 3 * 512 * band_rows;
    let mut pixels = vec![0; band_bytes * 8];
    let stats = std::thread::scope(|scope| {
        let bands: Vec<_> = pixels
            .chunks_mut(band_bytes)
            .zip((0..512).step_by(64))
            .map(|(band, first)| {
                let renderer = &renderer;
                scope.spawn(move || renderer.render_rows(first, band))
            })
            .collect();
        bands.into_iter().fold(
            escher_render::RenderStats::default(),
            |stats, band| match band.join().expect("a band of rows returns") {
                Ok(band) => stats.merged(band),
                Err(error) => panic!("{text}: {error}"),
            },
        )
    });
    let inside = stats.samples() - stats.outside();
    #[expect(
        clippy::cast_precision_loss,
        reason = "the counts are far below 2 to the power 52"
    )]
    let mean = stats.folds() as f64 / inside as f64;
    println!(
        "{text} at 512 by 512 with 4 by 4 samples: {inside} samples inside the view, mean steps {mean:.2}, largest {}, {} unconverged",
        stats.largest_fold(),
        stats.unconverged()
    );
    assert_eq!(stats.unconverged(), 0, "{text}");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn rotation_triangle_of_order_1000_converges_at_every_sample() {
    assert_rotation_image_converges("(1000)32");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn rotation_triangle_of_three_orders_1000_converges_at_every_sample() {
    assert_rotation_image_converges("(1000)(1000)(1000)");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn rotation_kite_of_order_1000_on_the_sphere_converges_at_every_sample() {
    assert_rotation_image_converges("(1000)22");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn rotation_lune_of_order_1000_converges_at_every_sample() {
    assert_rotation_image_converges("(1000)(1000)");
}

const ROTATION_CENTRE_DOMAINS: [&str; 27] = [
    "332",
    "432",
    "532",
    "222",
    "322",
    "422",
    "522",
    "622",
    "722",
    "822",
    "922",
    "(10)22",
    "(11)22",
    "(12)22",
    "(1000)22",
    "22",
    "33",
    "44",
    "55",
    "66",
    "77",
    "88",
    "99",
    "(10)(10)",
    "(11)(11)",
    "(12)(12)",
    "(1000)(1000)",
];

const ROTATION_CENTRE_BOUNDS: [f64; 27] = [
    7e-15, 2e-14, 3e-14, 6e-15, 7e-15, 1e-14, 9e-15, 2e-14, 5e-15, 2e-14, 3e-14, 3e-14, 2e-14,
    3e-14, 3e-10, 8e-15, 9e-16, 6e-15, 3e-15, 8e-15, 3e-15, 6e-15, 4e-15, 4e-15, 2e-15, 4e-15,
    3e-13,
];

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration of the group runs loops too long for the interpreter"
)]
fn images_of_the_centre_under_a_rotation_group_fold_back_in_at_least_as_many_steps_as_their_depth()
{
    let mut reference = Reference::new();
    for (name, bound) in ROTATION_CENTRE_DOMAINS
        .into_iter()
        .zip(ROTATION_CENTRE_BOUNDS)
    {
        let signature: Signature = name
            .parse()
            .unwrap_or_else(|error| panic!("{name:?} is rejected: {error}"));
        let order = signature
            .group_order()
            .unwrap_or_else(|| panic!("{name} has no group order"))
            .get();
        let domain = domain_with_aspect(name, None);
        let elements = enumerate_group(&domain, order);
        assert_eq!(elements.len() as u64, order, "{name}");
        let centre = domain.centre();
        let exact_centre = vector(centre.to_array());
        let mut largest = (0.0_f64, 0_u32);
        let mut largest_depth = 0;
        let mut most_steps = 0;
        for element in &elements {
            let depth = element.depth;
            let exact_image = reference::apply(&element.exact, &exact_centre);
            let image =
                DVec3::from_array([0, 1, 2].map(|row| reference.approximate(&exact_image[row])));
            let result = fold(&domain, image, DEFAULT_CAP, |_| {});
            assert!(result.converged(), "{name} at depth {depth}");
            assert!(
                result.steps() >= depth,
                "{name} at depth {depth}: {} steps",
                result.steps()
            );
            let angle = reference.angle(&vector(result.point().to_array()), &exact_centre);
            let angle = reference.approximate(&angle);
            largest_depth = largest_depth.max(depth);
            most_steps = most_steps.max(result.steps());
            if angle > largest.0 {
                largest = (angle, depth);
            }
        }
        println!(
            "{name}: {} elements, largest depth {largest_depth}, most steps {most_steps}, largest angle {:.3e} at depth {}, bound {bound:e}, margin {:.1}",
            elements.len(),
            largest.0,
            largest.1,
            bound / largest.0
        );
        assert!(
            largest.0 <= bound,
            "{name}: angle {:e} exceeds {bound:e}",
            largest.0
        );
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "folding thousands of points of a large group runs loops too long for the interpreter"
)]
fn fold_near_a_vertex_of_a_rotation_domain_converges_beyond_the_measured_distance() {
    for (starred, clear) in NEAR_VERTEX_DOMAINS {
        let name = rotation_text(starred);
        let domain = rotation_domain(starred, None);
        let reflection = domain_with_aspect(starred, None);
        for (index, &vertex) in domain.vertices().iter().enumerate() {
            let vertex = normalize_point(vertex)
                .unwrap_or_else(|error| panic!("{name} vertex {index}: {error}"));
            let to_vertex = translation(vertex);
            let mut unconverged = [[0_u32; NEAR_VERTEX_DISTANCES.len()]; 2];
            for (slot, &distance) in NEAR_VERTEX_DISTANCES.iter().enumerate() {
                for direction in 0..NEAR_VERTEX_DIRECTIONS {
                    let heading = core::f64::consts::TAU * f64::from(direction)
                        / f64::from(NEAR_VERTEX_DIRECTIONS);
                    let point = to_vertex * rotation(heading) * boost(distance) * ORIGIN;
                    for (counts, folded) in unconverged.iter_mut().zip([&domain, &reflection]) {
                        let result = fold(folded, point, DEFAULT_CAP, |_| {});
                        assert!(
                            result.steps() <= DEFAULT_CAP,
                            "{name} vertex {index} direction {direction}"
                        );
                        if !result.converged() {
                            counts[slot] += 1;
                        }
                    }
                }
                assert!(
                    distance < clear || unconverged[0][slot] == 0,
                    "{name} vertex {index}: {} of {NEAR_VERTEX_DIRECTIONS} folds unconverged at distance {distance:e}, at or beyond {clear:e}",
                    unconverged[0][slot]
                );
            }
            println!(
                "{name} vertex {index}: unconverged of {NEAR_VERTEX_DIRECTIONS} at distances {NEAR_VERTEX_DISTANCES:?}: {:?}, and of the same points in {starred}: {:?}",
                unconverged[0], unconverged[1]
            );
        }
    }
}
