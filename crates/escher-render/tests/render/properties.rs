use core::f64::consts::TAU;
use std::sync::LazyLock;

use escher_domain::{FundamentalDomain, Orientation, PairingKind};
use escher_geometry::hyperbolic::{from_disk, side, to_disk};
use escher_geometry::spherical;
use escher_group::{Colouring, IndexLimit, Letter, SearchBudget, colourings};
use escher_render::fold;
use glam::{DVec2, DVec3};
use proptest::prelude::{any, prop_assert, prop_assert_eq, prop_assume, proptest};
use proptest::sample::Index;

use crate::common::{
    DEFAULT_CAP, EUCLIDEAN_DOMAINS, EUCLIDEAN_RANGE, EUCLIDEAN_REPLAY_BOUNDS, FOLD_TRIPLES,
    Measure, REPLAY_BOUNDS, ROTATION_EUCLIDEAN_REPLAY_BOUNDS, ROTATION_REPLAY_BOUNDS,
    SPHERICAL_DOMAINS, TOLERANCE, domain, euclidean_band, euclidean_domain, proptest_config,
    rotation_domain, spherical_domain, text, wallpaper_bounds,
};
use crate::golden_images::{POLYGON_DOMAINS, WALLPAPER_DOMAINS, domain_with_aspect};
use crate::reference::{self, Reference, num, vector};

const WALL_CLEARANCE: f64 = 1e-6;

fn point_in_range(exponent: f64, angle: f64) -> DVec3 {
    let radius = 1.0 - 10.0_f64.powf(-exponent);
    let position = DVec2::new(radius * angle.cos(), radius * angle.sin());
    from_disk(position).unwrap_or_else(|| panic!("{position:?} is outside the disk"))
}

fn plane_point(distance: f64, angle: f64) -> DVec3 {
    DVec3::new(distance * angle.cos(), distance * angle.sin(), 1.0)
}

fn band(exponent: f64) -> usize {
    let band = exponent.ceil().clamp(1.0, 6.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the band is a whole number from 1 to 6"
    )]
    let band = band as usize;
    band - 1
}

fn sphere_point(height: f64, angle: f64, exponent: i32) -> DVec3 {
    let radius = (1.0 - height * height).sqrt();
    DVec3::new(radius * angle.cos(), radius * angle.sin(), height) * power_of_two(exponent)
}

fn power_of_two(exponent: i32) -> f64 {
    let step = if exponent < 0 { 0.5 } else { 2.0 };
    (0..exponent.unsigned_abs()).fold(1.0, |factor, _| factor * step)
}

fn fold_counting_reversing_sides(
    domain: &FundamentalDomain,
    point: DVec3,
    cap: u32,
) -> (escher_render::Fold, u32) {
    let sides = domain.sides();
    let mut reversing = 0;
    let result = fold(domain, point, cap, |index| {
        if sides[index].orientation() == Orientation::Reversing {
            reversing += 1;
        }
    });
    (result, reversing)
}

const ROTATION_DOMAINS: usize =
    FOLD_TRIPLES.len() + EUCLIDEAN_DOMAINS.len() + SPHERICAL_DOMAINS.len();

struct RotationDomain {
    domain: FundamentalDomain,
    measure: Measure,
    index: usize,
}

fn rotation_case(index: usize) -> RotationDomain {
    let euclidean = index.checked_sub(FOLD_TRIPLES.len());
    let spherical = euclidean.and_then(|index| index.checked_sub(EUCLIDEAN_DOMAINS.len()));
    match (euclidean, spherical) {
        (None, _) => RotationDomain {
            domain: rotation_domain(&text(FOLD_TRIPLES[index]), None),
            measure: Measure::Hyperbolic,
            index,
        },
        (Some(index), None) => {
            let (starred, aspect) = EUCLIDEAN_DOMAINS[index];
            RotationDomain {
                domain: rotation_domain(starred, aspect),
                measure: Measure::Euclidean,
                index,
            }
        }
        (Some(_), Some(index)) => RotationDomain {
            domain: rotation_domain(SPHERICAL_DOMAINS[index], None),
            measure: Measure::Spherical,
            index,
        },
    }
}

impl RotationDomain {
    fn may_stop_at_the_cap(&self, result: escher_render::Fold) -> bool {
        self.measure == Measure::Hyperbolic
            && FOLD_TRIPLES[self.index] == [1000, 1000, 1000]
            && !result.converged()
            && result.steps() == DEFAULT_CAP
    }

    fn point(&self, unit: f64, angle: f64, exponent: i32) -> DVec3 {
        match self.measure {
            Measure::Hyperbolic => point_in_range(6.0 * unit, angle),
            Measure::Euclidean => plane_point(EUCLIDEAN_RANGE * unit, angle),
            Measure::Spherical => sphere_point(2.0 * unit - 1.0, angle, exponent),
        }
    }

    fn size(&self, point: DVec3) -> f64 {
        match self.measure {
            Measure::Hyperbolic | Measure::Euclidean => point.z,
            Measure::Spherical => point.abs().max_element(),
        }
    }

    fn bound(&self, point: DVec3, image: DVec3) -> Option<f64> {
        match self.measure {
            Measure::Hyperbolic => {
                let exponent = |point: DVec3| -(1.0 - to_disk(point).length()).log10();
                let outer = exponent(point).max(exponent(image));
                (outer <= 6.0).then(|| ROTATION_REPLAY_BOUNDS[self.index][band(outer)])
            }
            Measure::Euclidean => {
                let outer = point.truncate().length().max(image.truncate().length());
                euclidean_band(outer).map(|band| ROTATION_EUCLIDEAN_REPLAY_BOUNDS[self.index][band])
            }
            Measure::Spherical => Some(SPHERICAL_PAIRING_BOUNDS[self.index]),
        }
    }
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn rotation_fold_converges_inside_every_side(
        index in 0..ROTATION_DOMAINS,
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let case = rotation_case(index);
        let result = fold(&case.domain, case.point(unit, angle, exponent), DEFAULT_CAP, |_| {});
        prop_assume!(!case.may_stop_at_the_cap(result));
        prop_assert!(result.converged());
        prop_assert_eq!(result.reversals(), 0);
        let point = result.point();
        if case.measure == Measure::Euclidean {
            prop_assert_eq!(point.z.to_bits(), 1.0_f64.to_bits());
        }
        for wall in case.domain.sides() {
            let covector = wall.covector();
            let threshold = TOLERANCE * covector.abs().element_sum() * case.size(point);
            prop_assert!(covector.dot(point) <= threshold);
        }
    }

    #[test]
    fn rotation_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in 0..ROTATION_DOMAINS,
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let case = rotation_case(index);
        let first = fold(&case.domain, case.point(unit, angle, exponent), DEFAULT_CAP, |_| {});
        prop_assume!(first.converged());
        let second = fold(&case.domain, first.point(), DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        prop_assert_eq!(second.steps(), 0);
        prop_assert_eq!(second.point().to_array().map(f64::to_bits), first.point().to_array().map(f64::to_bits));
    }

    #[test]
    fn rotation_fold_of_a_paired_point_keeps_its_parity_and_its_folded_point(
        index in 0..ROTATION_DOMAINS,
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        paired in 0..6_usize,
    ) {
        let case = rotation_case(index);
        let point = case.point(unit, angle, 0);
        let first = fold(&case.domain, point, DEFAULT_CAP, |_| {});
        prop_assume!(!case.may_stop_at_the_cap(first));
        prop_assert!(first.converged());
        let sides = case.domain.sides();
        let image = sides[paired % sides.len()].pairing() * point;
        let bound = case.bound(point, image);
        prop_assume!(bound.is_some());
        let bound = bound.unwrap_or(0.0);
        let clearance = num(WALL_CLEARANCE.max(2.0 * bound));
        let mut reference = Reference::new();
        let folded = vector(first.point().to_array());
        for wall in sides {
            let distance = case.measure.wall_distance(&mut reference, &vector(wall.covector().to_array()), &folded);
            prop_assume!(distance >= clearance);
        }
        let second = fold(&case.domain, image, DEFAULT_CAP, |_| {});
        prop_assume!(!case.may_stop_at_the_cap(second));
        prop_assert!(second.converged());
        prop_assert_eq!(first.reversals() % 2, second.reversals() % 2);
        let distance = case.measure.distance(&mut reference, &folded, &vector(second.point().to_array()));
        prop_assert!(distance <= num(bound), "distance {}", reference.approximate(&distance));
    }

    #[test]
    fn rotation_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in 0..ROTATION_DOMAINS,
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let case = rotation_case(index);
        let point = case.point(unit, angle, exponent);
        let mut sides = Vec::new();
        let unscaled = fold(&case.domain, point, DEFAULT_CAP, |index| sides.push(index));
        for power in -40..=40 {
            let factor = power_of_two(power);
            let mut scaled_sides = Vec::new();
            let scaled = fold(&case.domain, point * factor, DEFAULT_CAP, |index| scaled_sides.push(index));
            prop_assert_eq!(&scaled_sides, &sides);
            prop_assert_eq!(scaled.steps(), unscaled.steps());
            prop_assert_eq!(scaled.reversals(), unscaled.reversals());
            prop_assert_eq!(scaled.converged(), unscaled.converged());
            prop_assert_eq!(scaled.point(), unscaled.point() * factor);
        }
    }

    #[test]
    fn rotation_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_point(
        index in 0..ROTATION_DOMAINS,
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let case = rotation_case(index);
        let point = DVec3::from_array(bits.map(f64::from_bits));
        let result = fold(&case.domain, point, cap, |_| {});
        prop_assert!(result.steps() <= cap);
        prop_assert_eq!(result.reversals(), 0);
        prop_assert!(!result.converged() || (result.point().is_finite() && case.size(result.point()) > 0.0));
        let positive = case.size(point) > 0.0;
        if !positive {
            prop_assert!(!result.converged());
            prop_assert_eq!(result.steps(), 0);
        }
    }

    #[test]
    fn fold_converges_inside_every_side(
        triple in 0..FOLD_TRIPLES.len(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        let domain = domain(FOLD_TRIPLES[triple]);
        let result = fold(&domain, point_in_range(exponent, angle), DEFAULT_CAP, |_| {});
        prop_assert!(result.converged());
        let point = result.point();
        for wall in domain.sides() {
            let covector = wall.covector();
            let threshold = TOLERANCE * covector.abs().element_sum() * point.z;
            prop_assert!(side(covector, point) <= threshold);
        }
    }

    #[test]
    fn fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        triple in 0..FOLD_TRIPLES.len(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        let domain = domain(FOLD_TRIPLES[triple]);
        let first = fold(&domain, point_in_range(exponent, angle), DEFAULT_CAP, |_| {});
        prop_assume!(first.converged());
        let second = fold(&domain, first.point(), DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        prop_assert_eq!(second.steps(), 0);
        prop_assert_eq!(second.point().to_array().map(f64::to_bits), first.point().to_array().map(f64::to_bits));
    }

    #[test]
    fn fold_of_a_paired_point_takes_one_step_more_or_less_with_the_other_parity(
        triple in 0..FOLD_TRIPLES.len(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
        paired in 0..3_usize,
    ) {
        let domain = domain(FOLD_TRIPLES[triple]);
        let point = point_in_range(exponent, angle);
        let first = fold(&domain, point, DEFAULT_CAP, |_| {});
        prop_assert!(first.converged());
        let image = domain.sides()[paired].pairing() * point;
        let image_exponent = -(1.0 - to_disk(image).length()).log10();
        prop_assume!(image_exponent <= 6.0);
        let bound = REPLAY_BOUNDS[triple][band(exponent.max(image_exponent))];
        let clearance = WALL_CLEARANCE.max(2.0 * bound);
        let mut reference = Reference::new();
        let folded = vector(first.point().to_array());
        for wall in domain.sides() {
            let distance = reference.wall_distance(&vector(wall.covector().to_array()), &folded);
            prop_assume!(distance >= num(clearance));
        }
        let second = fold(&domain, image, DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        prop_assert_eq!(first.steps().abs_diff(second.steps()), 1);
        prop_assert_eq!(first.reversals() % 2 + second.reversals() % 2, 1);
        let distance = reference.distance(&folded, &vector(second.point().to_array()));
        prop_assert!(distance <= num(bound), "distance {}", reference.approximate(&distance));
    }

    #[test]
    fn fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        triple in 0..FOLD_TRIPLES.len(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        let domain = domain(FOLD_TRIPLES[triple]);
        let point = point_in_range(exponent, angle);
        let mut sides = Vec::new();
        let unscaled = fold(&domain, point, DEFAULT_CAP, |index| sides.push(index));
        for power in -40..=40 {
            let factor = power_of_two(power);
            let mut scaled_sides = Vec::new();
            let scaled = fold(&domain, point * factor, DEFAULT_CAP, |index| scaled_sides.push(index));
            prop_assert_eq!(&scaled_sides, &sides);
            prop_assert_eq!(scaled.steps(), unscaled.steps());
            prop_assert_eq!(scaled.reversals(), unscaled.reversals());
            prop_assert_eq!(scaled.converged(), unscaled.converged());
            prop_assert_eq!(scaled.point(), unscaled.point() * factor);
        }
    }

    #[test]
    fn fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_on_the_upper_sheet(
        triple in 0..FOLD_TRIPLES.len(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let domain = domain(FOLD_TRIPLES[triple]);
        let point = DVec3::from_array(bits.map(f64::from_bits));
        let result = fold(&domain, point, cap, |_| {});
        prop_assert!(result.steps() <= cap);
        prop_assert!(result.reversals() <= result.steps());
        prop_assert!(!result.converged() || (result.point().is_finite() && result.point().z > 0.0));
        let on_upper_sheet = point.z > 0.0;
        if !on_upper_sheet {
            prop_assert!(!result.converged());
            prop_assert_eq!(result.steps(), 0);
        }
    }

    #[test]
    fn euclidean_fold_converges_inside_every_side_with_a_last_coordinate_of_one(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        let domain = euclidean_domain(index);
        let result = fold(&domain, plane_point(distance, angle), DEFAULT_CAP, |_| {});
        prop_assert!(result.converged());
        let point = result.point();
        prop_assert_eq!(point.z.to_bits(), 1.0_f64.to_bits());
        for wall in domain.sides() {
            let covector = wall.covector();
            let threshold = TOLERANCE * covector.abs().element_sum() * point.z;
            prop_assert!(side(covector, point) <= threshold);
        }
    }

    #[test]
    fn euclidean_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        let domain = euclidean_domain(index);
        let first = fold(&domain, plane_point(distance, angle), DEFAULT_CAP, |_| {});
        prop_assume!(first.converged());
        let second = fold(&domain, first.point(), DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        prop_assert_eq!(second.steps(), 0);
        prop_assert_eq!(second.point().to_array().map(f64::to_bits), first.point().to_array().map(f64::to_bits));
    }

    #[test]
    fn euclidean_fold_of_a_paired_point_takes_one_step_more_or_less_with_the_other_parity(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
        paired in 0..4_usize,
    ) {
        let domain = euclidean_domain(index);
        let point = plane_point(distance, angle);
        let first = fold(&domain, point, DEFAULT_CAP, |_| {});
        prop_assert!(first.converged());
        let pairing = domain.sides()[paired % domain.sides().len()].pairing();
        let image = pairing * point;
        let outer = euclidean_band(distance.max(image.truncate().length()));
        prop_assume!(outer.is_some());
        let bound = outer.map_or(0.0, |outer| EUCLIDEAN_REPLAY_BOUNDS[index][outer]);
        let clearance = WALL_CLEARANCE.max(2.0 * bound);
        let folded = vector(first.point().to_array());
        for wall in domain.sides() {
            let distance = reference::plane_wall_distance(&vector(wall.covector().to_array()), &folded);
            prop_assume!(distance >= num(clearance));
        }
        let second = fold(&domain, image, DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        prop_assert_eq!(first.steps().abs_diff(second.steps()), 1);
        prop_assert_eq!(first.reversals() % 2 + second.reversals() % 2, 1);
        let distance = reference::plane_distance(&folded, &vector(second.point().to_array()));
        let mut reference = Reference::new();
        prop_assert!(distance <= num(bound), "distance {}", reference.approximate(&distance));
    }

    #[test]
    fn euclidean_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        let domain = euclidean_domain(index);
        let point = plane_point(distance, angle);
        let mut sides = Vec::new();
        let unscaled = fold(&domain, point, DEFAULT_CAP, |index| sides.push(index));
        for power in -40..=40 {
            let factor = power_of_two(power);
            let mut scaled_sides = Vec::new();
            let scaled = fold(&domain, point * factor, DEFAULT_CAP, |index| scaled_sides.push(index));
            prop_assert_eq!(&scaled_sides, &sides);
            prop_assert_eq!(scaled.steps(), unscaled.steps());
            prop_assert_eq!(scaled.reversals(), unscaled.reversals());
            prop_assert_eq!(scaled.converged(), unscaled.converged());
            prop_assert_eq!(scaled.point(), unscaled.point() * factor);
        }
    }

    #[test]
    fn euclidean_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_with_a_positive_last_coordinate(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let domain = euclidean_domain(index);
        let point = DVec3::from_array(bits.map(f64::from_bits));
        let result = fold(&domain, point, cap, |_| {});
        prop_assert!(result.steps() <= cap);
        prop_assert!(result.reversals() <= result.steps());
        prop_assert!(!result.converged() || (result.point().is_finite() && result.point().z > 0.0));
        let positive = point.z > 0.0;
        if !positive {
            prop_assert!(!result.converged());
            prop_assert_eq!(result.steps(), 0);
        }
    }

    #[test]
    fn spherical_fold_converges_inside_every_side(
        index in 0..SPHERICAL_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let domain = spherical_domain(index);
        let result = fold(&domain, sphere_point(height, angle, exponent), DEFAULT_CAP, |_| {});
        prop_assert!(result.converged());
        let point = result.point();
        for wall in domain.sides() {
            let covector = wall.covector();
            let threshold = TOLERANCE * covector.abs().element_sum() * point.abs().max_element();
            prop_assert!(spherical::side(covector, point) <= threshold);
        }
    }

    #[test]
    fn spherical_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in 0..SPHERICAL_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let domain = spherical_domain(index);
        let first = fold(&domain, sphere_point(height, angle, exponent), DEFAULT_CAP, |_| {});
        prop_assume!(first.converged());
        let second = fold(&domain, first.point(), DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        prop_assert_eq!(second.steps(), 0);
        prop_assert_eq!(second.point().to_array().map(f64::to_bits), first.point().to_array().map(f64::to_bits));
    }

    #[test]
    fn spherical_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in 0..SPHERICAL_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let domain = spherical_domain(index);
        let point = sphere_point(height, angle, exponent);
        let mut sides = Vec::new();
        let unscaled = fold(&domain, point, DEFAULT_CAP, |index| sides.push(index));
        for power in -40..=40 {
            let factor = power_of_two(power);
            let mut scaled_sides = Vec::new();
            let scaled = fold(&domain, point * factor, DEFAULT_CAP, |index| scaled_sides.push(index));
            prop_assert_eq!(&scaled_sides, &sides);
            prop_assert_eq!(scaled.steps(), unscaled.steps());
            prop_assert_eq!(scaled.reversals(), unscaled.reversals());
            prop_assert_eq!(scaled.converged(), unscaled.converged());
            prop_assert_eq!(scaled.point(), unscaled.point() * factor);
        }
    }

    #[test]
    fn spherical_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_that_is_not_zero(
        index in 0..SPHERICAL_DOMAINS.len(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
        negative in any::<[bool; 3]>(),
    ) {
        let domain = spherical_domain(index);
        let point = DVec3::from_array(bits.map(f64::from_bits));
        let result = fold(&domain, point, cap, |_| {});
        prop_assert!(result.steps() <= cap);
        prop_assert!(result.reversals() <= result.steps());
        prop_assert!(!result.converged() || (result.point().is_finite() && result.point() != DVec3::ZERO));
        let zero = DVec3::from_array(negative.map(|negative| if negative { -0.0 } else { 0.0 }));
        let from_zero = fold(&domain, zero, cap, |_| {});
        prop_assert!(!from_zero.converged());
        prop_assert_eq!(from_zero.steps(), 0);
    }
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn polygon_fold_converges_inside_every_side(
        index in any::<Index>(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        let domain = POLYGON_DOMAINS[index.index(POLYGON_DOMAINS.len())].domain();
        let (result, reversing) = fold_counting_reversing_sides(&domain, point_in_range(exponent, angle), DEFAULT_CAP);
        prop_assert!(result.converged());
        prop_assert!(result.reversals() <= result.steps());
        prop_assert_eq!(result.reversals(), reversing);
        let point = result.point();
        for wall in domain.sides() {
            let covector = wall.covector();
            let threshold = TOLERANCE * covector.abs().element_sum() * point.z;
            prop_assert!(side(covector, point) <= threshold);
        }
    }

    #[test]
    fn polygon_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in any::<Index>(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        let domain = POLYGON_DOMAINS[index.index(POLYGON_DOMAINS.len())].domain();
        let first = fold(&domain, point_in_range(exponent, angle), DEFAULT_CAP, |_| {});
        prop_assert!(first.converged());
        let second = fold(&domain, first.point(), DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        prop_assert_eq!(second.steps(), 0);
        prop_assert_eq!(second.point().to_array().map(f64::to_bits), first.point().to_array().map(f64::to_bits));
    }

    #[test]
    fn polygon_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in any::<Index>(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        let domain = POLYGON_DOMAINS[index.index(POLYGON_DOMAINS.len())].domain();
        let point = point_in_range(exponent, angle);
        let mut sides = Vec::new();
        let unscaled = fold(&domain, point, DEFAULT_CAP, |index| sides.push(index));
        for power in -40..=40 {
            let factor = power_of_two(power);
            let mut scaled_sides = Vec::new();
            let scaled = fold(&domain, point * factor, DEFAULT_CAP, |index| scaled_sides.push(index));
            prop_assert_eq!(&scaled_sides, &sides);
            prop_assert_eq!(scaled.steps(), unscaled.steps());
            prop_assert_eq!(scaled.reversals(), unscaled.reversals());
            prop_assert_eq!(scaled.converged(), unscaled.converged());
            prop_assert_eq!(scaled.point(), unscaled.point() * factor);
        }
    }

    #[test]
    fn polygon_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_with_a_positive_last_coordinate(
        index in any::<Index>(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let domain = POLYGON_DOMAINS[index.index(POLYGON_DOMAINS.len())].domain();
        let point = DVec3::from_array(bits.map(f64::from_bits));
        let (result, reversing) = fold_counting_reversing_sides(&domain, point, cap);
        prop_assert!(result.steps() <= cap);
        prop_assert!(result.reversals() <= result.steps());
        prop_assert_eq!(result.reversals(), reversing);
        prop_assert!(!result.converged() || (result.point().is_finite() && result.point().z > 0.0));
        let positive = point.z > 0.0;
        if !positive {
            prop_assert!(!result.converged());
            prop_assert_eq!(result.steps(), 0);
        }
    }
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn wallpaper_fold_converges_inside_every_side_with_a_last_coordinate_of_one(
        index in any::<Index>(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        let domain = WALLPAPER_DOMAINS[index.index(WALLPAPER_DOMAINS.len())].domain();
        let (result, reversing) = fold_counting_reversing_sides(&domain, plane_point(distance, angle), DEFAULT_CAP);
        prop_assert!(result.converged());
        prop_assert!(result.reversals() <= result.steps());
        prop_assert_eq!(result.reversals(), reversing);
        let point = result.point();
        prop_assert_eq!(point.z.to_bits(), 1.0_f64.to_bits());
        for wall in domain.sides() {
            let covector = wall.covector();
            let threshold = TOLERANCE * covector.abs().element_sum() * point.z;
            prop_assert!(side(covector, point) <= threshold);
        }
    }

    #[test]
    fn wallpaper_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in any::<Index>(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        let domain = WALLPAPER_DOMAINS[index.index(WALLPAPER_DOMAINS.len())].domain();
        let first = fold(&domain, plane_point(distance, angle), DEFAULT_CAP, |_| {});
        prop_assert!(first.converged());
        let second = fold(&domain, first.point(), DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        prop_assert_eq!(second.steps(), 0);
        prop_assert_eq!(second.point().to_array().map(f64::to_bits), first.point().to_array().map(f64::to_bits));
    }

    #[test]
    fn wallpaper_fold_of_a_paired_point_changes_its_parity_exactly_when_the_side_reverses_orientation(
        index in any::<Index>(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
        paired in any::<Index>(),
    ) {
        let shape = &WALLPAPER_DOMAINS[index.index(WALLPAPER_DOMAINS.len())];
        let domain = shape.domain();
        let point = plane_point(distance, angle);
        let first = fold(&domain, point, DEFAULT_CAP, |_| {});
        prop_assert!(first.converged());
        let side = domain.sides()[paired.index(domain.sides().len())];
        let image = side.pairing() * point;
        let outer = euclidean_band(distance.max(image.truncate().length()));
        prop_assume!(outer.is_some());
        let bound = outer.map_or(0.0, |outer| wallpaper_bounds(shape).replay[outer]);
        let clearance = WALL_CLEARANCE.max(2.0 * bound);
        let folded = vector(first.point().to_array());
        for wall in domain.sides() {
            let distance = reference::plane_wall_distance(&vector(wall.covector().to_array()), &folded);
            prop_assume!(distance >= num(clearance));
        }
        let second = fold(&domain, image, DEFAULT_CAP, |_| {});
        prop_assert!(second.converged());
        let reverses = matches!(side.kind(), PairingKind::Mirror | PairingKind::GlideReflection);
        prop_assert_eq!((first.reversals() + second.reversals()) % 2, u32::from(reverses));
        let distance = reference::plane_distance(&folded, &vector(second.point().to_array()));
        let mut reference = Reference::new();
        prop_assert!(distance <= num(bound), "distance {}", reference.approximate(&distance));
    }

    #[test]
    fn wallpaper_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in any::<Index>(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        let domain = WALLPAPER_DOMAINS[index.index(WALLPAPER_DOMAINS.len())].domain();
        let point = plane_point(distance, angle);
        let mut sides = Vec::new();
        let unscaled = fold(&domain, point, DEFAULT_CAP, |index| sides.push(index));
        for power in -40..=40 {
            let factor = power_of_two(power);
            let mut scaled_sides = Vec::new();
            let scaled = fold(&domain, point * factor, DEFAULT_CAP, |index| scaled_sides.push(index));
            prop_assert_eq!(&scaled_sides, &sides);
            prop_assert_eq!(scaled.steps(), unscaled.steps());
            prop_assert_eq!(scaled.reversals(), unscaled.reversals());
            prop_assert_eq!(scaled.converged(), unscaled.converged());
            prop_assert_eq!(scaled.point(), unscaled.point() * factor);
        }
    }

    #[test]
    fn wallpaper_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_with_a_positive_last_coordinate(
        index in any::<Index>(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let domain = WALLPAPER_DOMAINS[index.index(WALLPAPER_DOMAINS.len())].domain();
        let point = DVec3::from_array(bits.map(f64::from_bits));
        let (result, reversing) = fold_counting_reversing_sides(&domain, point, cap);
        prop_assert!(result.steps() <= cap);
        prop_assert!(result.reversals() <= result.steps());
        prop_assert_eq!(result.reversals(), reversing);
        prop_assert!(!result.converged() || (result.point().is_finite() && result.point().z > 0.0));
        let positive = point.z > 0.0;
        if !positive {
            prop_assert!(!result.converged());
            prop_assert_eq!(result.steps(), 0);
        }
    }
}

const SPHERICAL_PAIRING_BOUNDS: [f64; 17] = [
    2e-14, 3e-14, 5e-14, 9e-15, 2e-14, 2e-14, 3e-14, 3e-12, 3e-10, 3e-10, 8e-15, 2e-14, 5e-15,
    6e-15, 9e-15, 2e-14, 3e-14,
];

const PAIRING_GRID: u32 = 32;

#[test]
fn rotation_folds_of_paired_points_on_the_sphere_stay_within_the_pairing_bound() {
    let mut reference = Reference::new();
    for (index, (&name, &bound)) in SPHERICAL_DOMAINS
        .iter()
        .zip(&SPHERICAL_PAIRING_BOUNDS)
        .enumerate()
    {
        let case = rotation_case(FOLD_TRIPLES.len() + EUCLIDEAN_DOMAINS.len() + index);
        let sides = case.domain.sides();
        let clearance = num(WALL_CLEARANCE.max(2.0 * bound));
        let mut worst = reference::Worst::new(
            format!(
                "folded points of a point and its image under a pairing, {}",
                &name[1..]
            ),
            1000,
        );
        for height in 0..=PAIRING_GRID {
            for turn in 0..PAIRING_GRID {
                let unit = f64::from(height) / f64::from(PAIRING_GRID);
                let angle = TAU * f64::from(turn) / f64::from(PAIRING_GRID);
                let point = case.point(unit, angle, 0);
                let first = fold(&case.domain, point, DEFAULT_CAP, |_| {});
                assert!(first.converged(), "{name} at {unit}, {turn}");
                let folded = vector(first.point().to_array());
                let clear = sides.iter().all(|wall| {
                    case.measure.wall_distance(
                        &mut reference,
                        &vector(wall.covector().to_array()),
                        &folded,
                    ) >= clearance
                });
                if !clear {
                    continue;
                }
                for (paired, side) in sides.iter().enumerate() {
                    let second = fold(&case.domain, side.pairing() * point, DEFAULT_CAP, |_| {});
                    assert!(
                        second.converged(),
                        "{name} at {unit}, {turn}, side {paired}"
                    );
                    let distance = case.measure.distance(
                        &mut reference,
                        &folded,
                        &vector(second.point().to_array()),
                    );
                    worst.record(&distance, &reference::int(1), || {
                        format!("height {height} and turn {turn} of {PAIRING_GRID}, side {paired}")
                    });
                }
            }
        }
        worst.assert_at_most(&mut reference, bound);
    }
}

struct ColouredDomain {
    name: &'static str,
    domain: FundamentalDomain,
    measure: Measure,
    colourings: Vec<Colouring>,
}

const COLOURED_DOMAINS: [(&str, Measure, u32); 13] = [
    ("*732", Measure::Hyperbolic, 9),
    ("*642", Measure::Hyperbolic, 6),
    ("*2222", Measure::Euclidean, 6),
    ("532", Measure::Spherical, 6),
    ("432", Measure::Spherical, 6),
    ("2222", Measure::Euclidean, 6),
    ("732", Measure::Hyperbolic, 9),
    ("*3333", Measure::Hyperbolic, 4),
    ("*22222", Measure::Hyperbolic, 4),
    ("3333", Measure::Hyperbolic, 4),
    ("22x", Measure::Euclidean, 6),
    ("*x", Measure::Euclidean, 6),
    ("3*3", Measure::Euclidean, 6),
];

static COLOURED: LazyLock<Vec<ColouredDomain>> = LazyLock::new(|| {
    COLOURED_DOMAINS
        .iter()
        .map(|&(name, measure, colours)| {
            let domain = domain_with_aspect(name, None);
            let presentation = domain
                .side_presentation()
                .unwrap_or_else(|error| panic!("{name} has no side presentation: {error}"));
            let limit = IndexLimit::new(colours).unwrap_or_else(|error| panic!("{error}"));
            let found = colourings(&presentation, limit, SearchBudget::DEFAULT)
                .unwrap_or_else(|error| panic!("the search of {name} fails: {error}"));
            assert_eq!(found.complete_to(), colours, "{name}");
            ColouredDomain {
                name,
                domain,
                measure,
                colourings: found.list().to_vec(),
            }
        })
        .collect()
});

const LARGEST_DISK_RADIUS: f64 = 1.0 - 1e-3;

fn coloured_point(measure: Measure, unit: f64, angle: f64) -> DVec3 {
    match measure {
        Measure::Hyperbolic => point_in_range(3.0 * unit, angle),
        Measure::Euclidean => plane_point(EUCLIDEAN_RANGE * unit, angle),
        Measure::Spherical => sphere_point(2.0 * unit - 1.0, angle, 0),
    }
}

fn fold_sides(
    coloured: &ColouredDomain,
    reference: &mut Reference,
    point: DVec3,
) -> Option<Vec<u8>> {
    let mut sides = Vec::new();
    let result = fold(&coloured.domain, point, DEFAULT_CAP, |side| {
        sides.push(u8::try_from(side).unwrap_or(u8::MAX));
    });
    if !result.converged() {
        return None;
    }
    let folded = vector(result.point().to_array());
    let clear = coloured.domain.sides().iter().all(|wall| {
        let covector = vector(wall.covector().to_array());
        coloured
            .measure
            .wall_distance(reference, &covector, &folded)
            >= num(WALL_CLEARANCE)
    });
    clear.then_some(sides)
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn each_side_moves_the_coset_colour_by_the_inverse_column_of_its_generator(
        index in 0..COLOURED_DOMAINS.len(),
        chosen in any::<Index>(),
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        side in any::<Index>(),
    ) {
        let coloured = &COLOURED[index];
        let colouring = chosen.get(&coloured.colourings);
        let side = side.index(coloured.domain.sides().len());
        let point = coloured_point(coloured.measure, unit, angle);
        let partner = coloured.domain.sides()[side].partner();
        let image = coloured.domain.sides()[partner].pairing() * point;
        if coloured.measure == Measure::Hyperbolic {
            prop_assume!(to_disk(image).length() <= LARGEST_DISK_RADIUS);
        }
        let mut reference = Reference::new();
        let sides = fold_sides(coloured, &mut reference, point);
        let image_sides = fold_sides(coloured, &mut reference, image);
        prop_assume!(sides.is_some() && image_sides.is_some());
        let colour = sides.and_then(|sides| colouring.colour_of(&sides));
        let image_colour = image_sides.and_then(|sides| colouring.colour_of(&sides));
        prop_assert!(colour.is_some());
        let letter = Letter::inverse_of(u8::try_from(side).unwrap_or(u8::MAX));
        let expected = colour.and_then(|colour| colouring.table().image(colour, letter));
        prop_assert_eq!(
            image_colour,
            expected,
            "{} {} side {}",
            coloured.name,
            colouring.id(),
            side
        );
    }
}
