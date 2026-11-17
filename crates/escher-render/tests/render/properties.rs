use core::f64::consts::TAU;

use escher_geometry::hyperbolic::{from_disk, side, to_disk};
use escher_render::fold;
use glam::{DVec2, DVec3};
use proptest::prelude::{any, prop_assert, prop_assert_eq, prop_assume, proptest};

use crate::common::{
    DEFAULT_CAP, EUCLIDEAN_DOMAINS, EUCLIDEAN_RANGE, EUCLIDEAN_REPLAY_BOUNDS, FOLD_TRIPLES,
    REPLAY_BOUNDS, TOLERANCE, domain, euclidean_band, euclidean_domain, proptest_config,
};
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

fn power_of_two(exponent: i32) -> f64 {
    let step = if exponent < 0 { 0.5 } else { 2.0 };
    (0..exponent.unsigned_abs()).fold(1.0, |factor, _| factor * step)
}

proptest! {
    #![proptest_config(proptest_config())]

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
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
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
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
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
}
