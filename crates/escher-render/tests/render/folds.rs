use escher_geometry::hyperbolic::{ORIGIN, from_disk, side};
use escher_render::{Fill, RenderOptions, Renderer, fold};
use glam::DVec3;

use crate::common::{
    BANDS, DEFAULT_CAP, FOLD_TRIPLES, REPLAY_BOUNDS, TOLERANCE, VERTEX_TRIPLES, describe, domain,
    fold_set, one_fill, options, pixel_position, reference_walls, rim_set, text,
};
use crate::reference::{self, Reference, Worst, count_bound, vector};

const DECISION_COUNTS: [usize; 6] = [0, 0, 0, 0, 0, 6];

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

#[test]
fn every_vertex_folds_in_zero_steps() {
    let mut largest = (0.0_f64, String::new());
    for triple in FOLD_TRIPLES.into_iter().chain(VERTEX_TRIPLES) {
        let domain = domain(triple);
        for (index, &vertex) in domain.vertices().iter().enumerate() {
            for (side_index, wall) in domain.sides().iter().enumerate() {
                let covector = wall.covector();
                let ratio = side(covector, vertex) / (covector.abs().element_sum() * vertex.z);
                if ratio > largest.0 {
                    largest = (
                        ratio,
                        format!("{} vertex {index} side {side_index}", text(triple)),
                    );
                }
            }
            let result = fold(&domain, vertex, DEFAULT_CAP, |_| {});
            assert!(result.converged(), "{} vertex {index}", text(triple));
            assert_eq!(result.steps(), 0, "{} vertex {index}", text(triple));
            assert_eq!(result.point(), vertex);
        }
    }
    let (ratio, at) = largest;
    println!(
        "largest side value at a vertex over the tolerance scale: {ratio:.3e} at {at}, tolerance {TOLERANCE:e}, margin {:.1}",
        TOLERANCE / ratio
    );
    assert!(8.0 * ratio <= TOLERANCE, "{ratio:e} at {at}");
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
