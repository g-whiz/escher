use core::f64::consts::TAU;
use std::sync::LazyLock;

use escher_domain::{FundamentalDomain, Orientation};
use escher_geometry::hyperbolic::{from_disk, side, to_disk};
use escher_geometry::spherical;
use escher_group::{Colouring, ColouringSet, IndexLimit, Letter, SearchBudget, colourings};
use escher_render::fold;
use glam::{DVec2, DVec3};
use proptest::prelude::{any, prop_assert, prop_assert_eq, prop_assume, proptest};
use proptest::sample::Index;
use proptest::test_runner::TestCaseError;

use crate::common::{
    DEFAULT_CAP, EUCLIDEAN_DOMAINS, EUCLIDEAN_RANGE, EUCLIDEAN_REPLAY_BOUNDS, FOLD_TRIPLES,
    Measure, REPLAY_BOUNDS, ROTATION_EUCLIDEAN_REPLAY_BOUNDS, ROTATION_REPLAY_BOUNDS,
    SPHERICAL_BOUNDS, SPHERICAL_DOMAINS, SPHERICAL_GROUP_DOMAINS, TOLERANCE, domain,
    euclidean_band, euclidean_domain, rotation_domain, spherical_domain, spherical_group_bounds,
    text, wallpaper_bounds,
};
use crate::golden_images::{
    POLYGON_DOMAINS, PolygonShape, WALLPAPER_DOMAINS, domain_with_aspect, domain_with_settings,
};
use escher_reference::{Reference, Worst, int, num, vector};

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

fn reverses(side: &escher_domain::Side) -> bool {
    side.pairing().determinant() < 0.0
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

fn size(measure: Measure, point: DVec3) -> f64 {
    match measure {
        Measure::Hyperbolic | Measure::Euclidean => point.z,
        Measure::Spherical => point.abs().max_element(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reversals {
    None,
    Uncounted,
    Counted,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FirstFold {
    Assumed,
    Asserted,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Parity {
    OtherAfterOneStepMoreOrLess,
    Kept,
    ChangedExactlyWhenTheSideReverses,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rejected {
    NotPositive,
    SignedZero([bool; 3]),
}

struct Family {
    domain: FundamentalDomain,
    measure: Measure,
    side: fn(DVec3, DVec3) -> f64,
    reversals: Reversals,
    first_fold: FirstFold,
    may_stop_at_the_cap: bool,
}

impl Family {
    fn stops_at_the_cap(&self, result: escher_render::Fold) -> bool {
        self.may_stop_at_the_cap && !result.converged() && result.steps() == DEFAULT_CAP
    }
}

fn hyperbolic(triple: usize) -> Family {
    Family {
        domain: domain(FOLD_TRIPLES[triple]),
        measure: Measure::Hyperbolic,
        side,
        reversals: Reversals::Uncounted,
        first_fold: FirstFold::Assumed,
        may_stop_at_the_cap: false,
    }
}

fn euclidean(index: usize) -> Family {
    Family {
        domain: euclidean_domain(index),
        measure: Measure::Euclidean,
        side,
        reversals: Reversals::Uncounted,
        first_fold: FirstFold::Assumed,
        may_stop_at_the_cap: false,
    }
}

fn spherical(index: usize) -> Family {
    Family {
        domain: spherical_domain(index),
        measure: Measure::Spherical,
        side: spherical::side,
        reversals: Reversals::Uncounted,
        first_fold: FirstFold::Assumed,
        may_stop_at_the_cap: false,
    }
}

fn polygon(shape: &PolygonShape) -> Family {
    Family {
        domain: shape.domain(),
        measure: Measure::Hyperbolic,
        side,
        reversals: Reversals::Counted,
        first_fold: FirstFold::Asserted,
        may_stop_at_the_cap: false,
    }
}

fn wallpaper(shape: &PolygonShape) -> Family {
    Family {
        domain: shape.domain(),
        measure: Measure::Euclidean,
        side,
        reversals: Reversals::Counted,
        first_fold: FirstFold::Asserted,
        may_stop_at_the_cap: false,
    }
}

fn spherical_group(index: usize) -> Family {
    Family {
        domain: group_domain(index),
        measure: Measure::Spherical,
        side: spherical::side,
        reversals: Reversals::Counted,
        first_fold: FirstFold::Assumed,
        may_stop_at_the_cap: false,
    }
}

const ROTATION_DOMAINS: usize =
    FOLD_TRIPLES.len() + EUCLIDEAN_DOMAINS.len() + SPHERICAL_DOMAINS.len();

struct RotationDomain {
    family: Family,
    index: usize,
}

fn rotation_case(index: usize) -> RotationDomain {
    let euclidean = index.checked_sub(FOLD_TRIPLES.len());
    let spherical = euclidean.and_then(|index| index.checked_sub(EUCLIDEAN_DOMAINS.len()));
    let (domain, measure, index) = match (euclidean, spherical) {
        (None, _) => (
            rotation_domain(&text(FOLD_TRIPLES[index]), None),
            Measure::Hyperbolic,
            index,
        ),
        (Some(index), None) => {
            let (starred, aspect) = EUCLIDEAN_DOMAINS[index];
            (rotation_domain(starred, aspect), Measure::Euclidean, index)
        }
        (Some(_), Some(index)) => (
            rotation_domain(SPHERICAL_DOMAINS[index], None),
            Measure::Spherical,
            index,
        ),
    };
    RotationDomain {
        family: Family {
            domain,
            measure,
            side: DVec3::dot,
            reversals: Reversals::None,
            first_fold: FirstFold::Assumed,
            may_stop_at_the_cap: measure == Measure::Hyperbolic
                && FOLD_TRIPLES[index] == [100, 100, 100],
        },
        index,
    }
}

impl RotationDomain {
    fn point(&self, unit: f64, angle: f64, exponent: i32) -> DVec3 {
        match self.family.measure {
            Measure::Hyperbolic => point_in_range(6.0 * unit, angle),
            Measure::Euclidean => plane_point(EUCLIDEAN_RANGE * unit, angle),
            Measure::Spherical => sphere_point(2.0 * unit - 1.0, angle, exponent),
        }
    }

    fn bound(&self, point: DVec3, image: DVec3) -> Option<f64> {
        match self.family.measure {
            Measure::Hyperbolic => {
                let exponent = |point: DVec3| -(1.0 - to_disk(point).length()).log10();
                let outer = exponent(point).max(exponent(image));
                (outer <= 6.0).then(|| ROTATION_REPLAY_BOUNDS[self.index][band(outer)])
            }
            Measure::Euclidean => {
                let outer = point.truncate().length().max(image.truncate().length());
                euclidean_band(outer).map(|band| ROTATION_EUCLIDEAN_REPLAY_BOUNDS[self.index][band])
            }
            Measure::Spherical => Some(SPHERICAL_BOUNDS[self.index].pairing),
        }
    }
}

fn converges_inside_every_side(family: &Family, point: DVec3) -> Result<(), TestCaseError> {
    let (result, reversing) = fold_counting_reversing_sides(&family.domain, point, DEFAULT_CAP);
    prop_assume!(!family.stops_at_the_cap(result));
    prop_assert!(result.converged());
    match family.reversals {
        Reversals::None => prop_assert_eq!(result.reversals(), 0),
        Reversals::Uncounted => {}
        Reversals::Counted => {
            prop_assert!(result.reversals() <= result.steps());
            prop_assert_eq!(result.reversals(), reversing);
        }
    }
    let folded = result.point();
    if family.measure == Measure::Euclidean {
        prop_assert_eq!(folded.z.to_bits(), 1.0_f64.to_bits());
    }
    for wall in family.domain.sides() {
        let covector = wall.covector();
        let threshold = TOLERANCE * covector.abs().element_sum() * size(family.measure, folded);
        prop_assert!((family.side)(covector, folded) <= threshold);
    }
    if family.domain.sides().is_empty() {
        prop_assert_eq!(result.steps(), 0);
        prop_assert_eq!(
            folded.to_array().map(f64::to_bits),
            point.to_array().map(f64::to_bits)
        );
    }
    Ok(())
}

fn fold_of_a_folded_point_takes_zero_steps(
    family: &Family,
    point: DVec3,
) -> Result<(), TestCaseError> {
    let first = fold(&family.domain, point, DEFAULT_CAP, |_| {});
    match family.first_fold {
        FirstFold::Assumed => prop_assume!(first.converged()),
        FirstFold::Asserted => prop_assert!(first.converged()),
    }
    let second = fold(&family.domain, first.point(), DEFAULT_CAP, |_| {});
    prop_assert!(second.converged());
    prop_assert_eq!(second.steps(), 0);
    prop_assert_eq!(
        second.point().to_array().map(f64::to_bits),
        first.point().to_array().map(f64::to_bits)
    );
    Ok(())
}

fn fold_of_a_paired_point_keeps_the_folded_point(
    family: &Family,
    point: DVec3,
    paired: impl FnOnce(usize) -> usize,
    bound: impl FnOnce(DVec3) -> Option<f64>,
    parity: Parity,
) -> Result<(), TestCaseError> {
    let first = fold(&family.domain, point, DEFAULT_CAP, |_| {});
    prop_assume!(!family.stops_at_the_cap(first));
    prop_assert!(first.converged());
    let sides = family.domain.sides();
    let side = sides[paired(sides.len())];
    let image = side.pairing() * point;
    let bound = bound(image);
    prop_assume!(bound.is_some());
    let bound = bound.unwrap_or(0.0);
    let clearance = num(WALL_CLEARANCE.max(2.0 * bound));
    let mut reference = Reference::new();
    let folded = vector(first.point().to_array());
    for wall in sides {
        let covector = vector(wall.covector().to_array());
        let distance = family
            .measure
            .wall_distance(&mut reference, &covector, &folded);
        prop_assume!(distance >= clearance);
    }
    let second = fold(&family.domain, image, DEFAULT_CAP, |_| {});
    prop_assume!(!family.stops_at_the_cap(second));
    prop_assert!(second.converged());
    match parity {
        Parity::OtherAfterOneStepMoreOrLess => {
            prop_assert_eq!(first.steps().abs_diff(second.steps()), 1);
            prop_assert_eq!(first.reversals() % 2 + second.reversals() % 2, 1);
        }
        Parity::Kept => prop_assert_eq!(first.reversals() % 2, second.reversals() % 2),
        Parity::ChangedExactlyWhenTheSideReverses => prop_assert_eq!(
            (first.reversals() + second.reversals()) % 2,
            u32::from(reverses(&side))
        ),
    }
    let distance =
        family
            .measure
            .distance(&mut reference, &folded, &vector(second.point().to_array()));
    prop_assert!(
        distance <= num(bound),
        "distance {}",
        reference.approximate(&distance)
    );
    Ok(())
}

fn fold_takes_the_same_decisions_for_the_scaled_point(
    family: &Family,
    point: DVec3,
) -> Result<(), TestCaseError> {
    let mut sides = Vec::new();
    let unscaled = fold(&family.domain, point, DEFAULT_CAP, |index| {
        sides.push(index);
    });
    for power in -40..=40 {
        let factor = power_of_two(power);
        let mut scaled_sides = Vec::new();
        let scaled = fold(&family.domain, point * factor, DEFAULT_CAP, |index| {
            scaled_sides.push(index);
        });
        prop_assert_eq!(&scaled_sides, &sides);
        prop_assert_eq!(scaled.steps(), unscaled.steps());
        prop_assert_eq!(scaled.reversals(), unscaled.reversals());
        prop_assert_eq!(scaled.converged(), unscaled.converged());
        prop_assert_eq!(scaled.point(), unscaled.point() * factor);
    }
    Ok(())
}

fn fold_of_any_bit_pattern_returns_within_the_cap(
    family: &Family,
    point: DVec3,
    cap: u32,
    rejected: Rejected,
) -> Result<(), TestCaseError> {
    let (result, reversing) = fold_counting_reversing_sides(&family.domain, point, cap);
    prop_assert!(result.steps() <= cap);
    match family.reversals {
        Reversals::None => prop_assert_eq!(result.reversals(), 0),
        Reversals::Uncounted => prop_assert!(result.reversals() <= result.steps()),
        Reversals::Counted => {
            prop_assert!(result.reversals() <= result.steps());
            prop_assert_eq!(result.reversals(), reversing);
        }
    }
    match rejected {
        Rejected::NotPositive => {
            prop_assert!(
                !result.converged()
                    || (result.point().is_finite() && size(family.measure, result.point()) > 0.0)
            );
            let positive = size(family.measure, point) > 0.0;
            if !positive {
                prop_assert!(!result.converged());
                prop_assert_eq!(result.steps(), 0);
            }
        }
        Rejected::SignedZero(negative) => {
            prop_assert!(
                !result.converged()
                    || (result.point().is_finite() && result.point() != DVec3::ZERO)
            );
            let zero =
                DVec3::from_array(negative.map(|negative| if negative { -0.0 } else { 0.0 }));
            let from_zero = fold(&family.domain, zero, cap, |_| {});
            prop_assert!(!from_zero.converged());
            prop_assert_eq!(from_zero.steps(), 0);
        }
    }
    Ok(())
}

proptest! {
    #[test]
    fn rotation_fold_converges_inside_every_side(
        index in 0..ROTATION_DOMAINS,
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let rotation = rotation_case(index);
        converges_inside_every_side(&rotation.family, rotation.point(unit, angle, exponent))?;
    }

    #[test]
    fn rotation_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in 0..ROTATION_DOMAINS,
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let rotation = rotation_case(index);
        fold_of_a_folded_point_takes_zero_steps(&rotation.family, rotation.point(unit, angle, exponent))?;
    }

    #[test]
    fn rotation_fold_of_a_paired_point_keeps_its_parity_and_its_folded_point(
        index in 0..ROTATION_DOMAINS,
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        paired in 0..6_usize,
    ) {
        let rotation = rotation_case(index);
        let point = rotation.point(unit, angle, 0);
        fold_of_a_paired_point_keeps_the_folded_point(
            &rotation.family,
            point,
            |sides| paired % sides,
            |image| rotation.bound(point, image),
            Parity::Kept,
        )?;
    }

    #[test]
    fn rotation_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in 0..ROTATION_DOMAINS,
        unit in 0.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        let rotation = rotation_case(index);
        fold_takes_the_same_decisions_for_the_scaled_point(&rotation.family, rotation.point(unit, angle, exponent))?;
    }

    #[test]
    fn rotation_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_point(
        index in 0..ROTATION_DOMAINS,
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let point = DVec3::from_array(bits.map(f64::from_bits));
        fold_of_any_bit_pattern_returns_within_the_cap(&rotation_case(index).family, point, cap, Rejected::NotPositive)?;
    }

    #[test]
    fn fold_converges_inside_every_side(
        triple in 0..FOLD_TRIPLES.len(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        converges_inside_every_side(&hyperbolic(triple), point_in_range(exponent, angle))?;
    }

    #[test]
    fn fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        triple in 0..FOLD_TRIPLES.len(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        fold_of_a_folded_point_takes_zero_steps(&hyperbolic(triple), point_in_range(exponent, angle))?;
    }

    #[test]
    fn fold_of_a_paired_point_takes_one_step_more_or_less_with_the_other_parity(
        triple in 0..FOLD_TRIPLES.len(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
        paired in 0..3_usize,
    ) {
        fold_of_a_paired_point_keeps_the_folded_point(
            &hyperbolic(triple),
            point_in_range(exponent, angle),
            |_| paired,
            |image| {
                let image_exponent = -(1.0 - to_disk(image).length()).log10();
                (image_exponent <= 6.0).then(|| REPLAY_BOUNDS[triple][band(exponent.max(image_exponent))])
            },
            Parity::OtherAfterOneStepMoreOrLess,
        )?;
    }

    #[test]
    fn fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        triple in 0..FOLD_TRIPLES.len(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        fold_takes_the_same_decisions_for_the_scaled_point(&hyperbolic(triple), point_in_range(exponent, angle))?;
    }

    #[test]
    fn fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_on_the_upper_sheet(
        triple in 0..FOLD_TRIPLES.len(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let point = DVec3::from_array(bits.map(f64::from_bits));
        fold_of_any_bit_pattern_returns_within_the_cap(&hyperbolic(triple), point, cap, Rejected::NotPositive)?;
    }

    #[test]
    fn euclidean_fold_converges_inside_every_side_with_a_last_coordinate_of_one(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        converges_inside_every_side(&euclidean(index), plane_point(distance, angle))?;
    }

    #[test]
    fn euclidean_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        fold_of_a_folded_point_takes_zero_steps(&euclidean(index), plane_point(distance, angle))?;
    }

    #[test]
    fn euclidean_fold_of_a_paired_point_takes_one_step_more_or_less_with_the_other_parity(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
        paired in 0..4_usize,
    ) {
        fold_of_a_paired_point_keeps_the_folded_point(
            &euclidean(index),
            plane_point(distance, angle),
            |sides| paired % sides,
            |image| {
                euclidean_band(distance.max(image.truncate().length()))
                    .map(|outer| EUCLIDEAN_REPLAY_BOUNDS[index][outer])
            },
            Parity::OtherAfterOneStepMoreOrLess,
        )?;
    }

    #[test]
    fn euclidean_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        fold_takes_the_same_decisions_for_the_scaled_point(&euclidean(index), plane_point(distance, angle))?;
    }

    #[test]
    fn euclidean_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_with_a_positive_last_coordinate(
        index in 0..EUCLIDEAN_DOMAINS.len(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let point = DVec3::from_array(bits.map(f64::from_bits));
        fold_of_any_bit_pattern_returns_within_the_cap(&euclidean(index), point, cap, Rejected::NotPositive)?;
    }

    #[test]
    fn spherical_fold_converges_inside_every_side(
        index in 0..SPHERICAL_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        converges_inside_every_side(&spherical(index), sphere_point(height, angle, exponent))?;
    }

    #[test]
    fn spherical_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in 0..SPHERICAL_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        fold_of_a_folded_point_takes_zero_steps(&spherical(index), sphere_point(height, angle, exponent))?;
    }

    #[test]
    fn spherical_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in 0..SPHERICAL_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        fold_takes_the_same_decisions_for_the_scaled_point(&spherical(index), sphere_point(height, angle, exponent))?;
    }

    #[test]
    fn spherical_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_that_is_not_zero(
        index in 0..SPHERICAL_DOMAINS.len(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
        negative in any::<[bool; 3]>(),
    ) {
        let point = DVec3::from_array(bits.map(f64::from_bits));
        fold_of_any_bit_pattern_returns_within_the_cap(&spherical(index), point, cap, Rejected::SignedZero(negative))?;
    }
}

proptest! {
    #[test]
    fn polygon_fold_converges_inside_every_side(
        index in any::<Index>(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        converges_inside_every_side(&polygon(index.get(&POLYGON_DOMAINS)), point_in_range(exponent, angle))?;
    }

    #[test]
    fn polygon_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in any::<Index>(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        fold_of_a_folded_point_takes_zero_steps(&polygon(index.get(&POLYGON_DOMAINS)), point_in_range(exponent, angle))?;
    }

    #[test]
    fn polygon_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in any::<Index>(),
        exponent in 0.0..=6.0_f64,
        angle in 0.0..TAU,
    ) {
        fold_takes_the_same_decisions_for_the_scaled_point(&polygon(index.get(&POLYGON_DOMAINS)), point_in_range(exponent, angle))?;
    }

    #[test]
    fn polygon_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_with_a_positive_last_coordinate(
        index in any::<Index>(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let point = DVec3::from_array(bits.map(f64::from_bits));
        fold_of_any_bit_pattern_returns_within_the_cap(&polygon(index.get(&POLYGON_DOMAINS)), point, cap, Rejected::NotPositive)?;
    }
}

proptest! {
    #[test]
    fn wallpaper_fold_converges_inside_every_side_with_a_last_coordinate_of_one(
        index in any::<Index>(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        converges_inside_every_side(&wallpaper(index.get(&WALLPAPER_DOMAINS)), plane_point(distance, angle))?;
    }

    #[test]
    fn wallpaper_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in any::<Index>(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        fold_of_a_folded_point_takes_zero_steps(&wallpaper(index.get(&WALLPAPER_DOMAINS)), plane_point(distance, angle))?;
    }

    #[test]
    fn wallpaper_fold_of_a_paired_point_changes_its_parity_exactly_when_the_side_reverses_orientation(
        index in any::<Index>(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
        paired in any::<Index>(),
    ) {
        let shape = index.get(&WALLPAPER_DOMAINS);
        fold_of_a_paired_point_keeps_the_folded_point(
            &wallpaper(shape),
            plane_point(distance, angle),
            |sides| paired.index(sides),
            |image| {
                euclidean_band(distance.max(image.truncate().length()))
                    .map(|outer| wallpaper_bounds(shape).replay[outer])
            },
            Parity::ChangedExactlyWhenTheSideReverses,
        )?;
    }

    #[test]
    fn wallpaper_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in any::<Index>(),
        distance in 0.0..=EUCLIDEAN_RANGE,
        angle in 0.0..TAU,
    ) {
        fold_takes_the_same_decisions_for_the_scaled_point(&wallpaper(index.get(&WALLPAPER_DOMAINS)), plane_point(distance, angle))?;
    }

    #[test]
    fn wallpaper_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_with_a_positive_last_coordinate(
        index in any::<Index>(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
    ) {
        let point = DVec3::from_array(bits.map(f64::from_bits));
        fold_of_any_bit_pattern_returns_within_the_cap(&wallpaper(index.get(&WALLPAPER_DOMAINS)), point, cap, Rejected::NotPositive)?;
    }
}

const PAIRING_GRID: u32 = 32;

#[test]
fn rotation_folds_of_paired_points_on_the_sphere_stay_within_the_pairing_bound() {
    let mut reference = Reference::new();
    for (index, row) in SPHERICAL_BOUNDS.iter().enumerate() {
        let (name, bound) = (row.signature, row.pairing);
        let rotation = rotation_case(FOLD_TRIPLES.len() + EUCLIDEAN_DOMAINS.len() + index);
        let sides = rotation.family.domain.sides();
        let clearance = num(WALL_CLEARANCE.max(2.0 * bound));
        let mut worst = Worst::new(
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
                let point = rotation.point(unit, angle, 0);
                let first = fold(&rotation.family.domain, point, DEFAULT_CAP, |_| {});
                assert!(first.converged(), "{name} at {unit}, {turn}");
                let folded = vector(first.point().to_array());
                let clear = sides.iter().all(|wall| {
                    rotation.family.measure.wall_distance(
                        &mut reference,
                        &vector(wall.covector().to_array()),
                        &folded,
                    ) >= clearance
                });
                if !clear {
                    continue;
                }
                for (paired, side) in sides.iter().enumerate() {
                    let second = fold(
                        &rotation.family.domain,
                        side.pairing() * point,
                        DEFAULT_CAP,
                        |_| {},
                    );
                    assert!(
                        second.converged(),
                        "{name} at {unit}, {turn}, side {paired}"
                    );
                    let distance = rotation.family.measure.distance(
                        &mut reference,
                        &folded,
                        &vector(second.point().to_array()),
                    );
                    worst.record(&distance, &int(1), || {
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

const COLOURED_DOMAINS: [(&str, Measure, u32); 16] = [
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
    ("3*2", Measure::Spherical, 6),
    ("5x", Measure::Spherical, 6),
    ("2*3", Measure::Spherical, 6),
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
            let found = colourings(
                &presentation,
                ColouringSet::All,
                limit,
                SearchBudget::DEFAULT,
            )
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

fn group_domain(index: usize) -> FundamentalDomain {
    domain_with_settings(SPHERICAL_GROUP_DOMAINS[index], &[])
}

proptest! {
    #[test]
    fn spherical_group_fold_converges_inside_every_side(
        index in 0..SPHERICAL_GROUP_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        converges_inside_every_side(&spherical_group(index), sphere_point(height, angle, exponent))?;
    }

    #[test]
    fn spherical_group_fold_of_a_folded_point_takes_zero_steps_and_returns_the_same_bits(
        index in 0..SPHERICAL_GROUP_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        fold_of_a_folded_point_takes_zero_steps(&spherical_group(index), sphere_point(height, angle, exponent))?;
    }

    #[test]
    fn spherical_group_fold_takes_the_same_decisions_for_the_point_scaled_by_a_power_of_two(
        index in 0..SPHERICAL_GROUP_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        exponent in -100..=100_i32,
    ) {
        fold_takes_the_same_decisions_for_the_scaled_point(&spherical_group(index), sphere_point(height, angle, exponent))?;
    }

    #[test]
    fn spherical_group_fold_of_any_bit_pattern_returns_within_the_cap_and_converges_only_at_a_finite_point_that_is_not_zero(
        index in 0..SPHERICAL_GROUP_DOMAINS.len(),
        bits in any::<[u64; 3]>(),
        cap in 0..=2000_u32,
        negative in any::<[bool; 3]>(),
    ) {
        let point = DVec3::from_array(bits.map(f64::from_bits));
        fold_of_any_bit_pattern_returns_within_the_cap(&spherical_group(index), point, cap, Rejected::SignedZero(negative))?;
    }

    #[test]
    fn spherical_group_fold_of_a_paired_point_changes_its_parity_exactly_when_the_side_reverses_orientation(
        index in 0..SPHERICAL_GROUP_DOMAINS.len(),
        height in -1.0..=1.0_f64,
        angle in 0.0..TAU,
        paired in any::<Index>(),
    ) {
        let name = SPHERICAL_GROUP_DOMAINS[index];
        let domain = group_domain(index);
        prop_assume!(!domain.sides().is_empty());
        let bound = spherical_group_bounds(name).pairing;
        let mut reference = Reference::new();
        let outcome = paired_fold(&mut reference, &domain, sphere_point(height, angle, 0), paired.index(domain.sides().len()), bound);
        let Some((parity_changes, side_reverses, distance)) = outcome else {
            return Err(TestCaseError::reject("the folded point lies too close to a wall"));
        };
        prop_assert_eq!(parity_changes, side_reverses, "{}", name);
        prop_assert!(distance <= num(bound), "{} distance {}", name, reference.approximate(&distance));
    }
}

fn paired_fold(
    reference: &mut Reference,
    domain: &FundamentalDomain,
    point: DVec3,
    paired: usize,
    bound: f64,
) -> Option<(bool, bool, astro_float_num::BigFloat)> {
    let first = fold(domain, point, DEFAULT_CAP, |_| {});
    assert!(first.converged(), "{point:?}");
    let folded = vector(first.point().to_array());
    let clearance = num(WALL_CLEARANCE.max(2.0 * bound));
    let clear = domain.sides().iter().all(|wall| {
        Measure::Spherical.wall_distance(reference, &vector(wall.covector().to_array()), &folded)
            >= clearance
    });
    if !clear {
        return None;
    }
    let side = domain.sides()[paired];
    let second = fold(domain, side.pairing() * point, DEFAULT_CAP, |_| {});
    assert!(second.converged(), "{point:?} across side {paired}");
    let distance =
        Measure::Spherical.distance(reference, &folded, &vector(second.point().to_array()));
    Some((
        (first.reversals() + second.reversals()) % 2 == 1,
        reverses(&side),
        distance,
    ))
}

const GROUP_PAIRING_GRID: u32 = 16;

#[test]
fn spherical_group_folds_of_paired_points_stay_within_the_pairing_bound_with_the_parity_of_the_side()
 {
    let mut reference = Reference::new();
    for (index, &name) in SPHERICAL_GROUP_DOMAINS.iter().enumerate() {
        let domain = group_domain(index);
        if domain.sides().is_empty() {
            continue;
        }
        let bound = spherical_group_bounds(name).pairing;
        let mut worst = Worst::new(
            format!("folded points of a point and its image under a pairing, {name}"),
            500,
        );
        for height in 0..=GROUP_PAIRING_GRID {
            for turn in 0..GROUP_PAIRING_GRID {
                let unit = f64::from(height) / f64::from(GROUP_PAIRING_GRID);
                let angle = TAU * f64::from(turn) / f64::from(GROUP_PAIRING_GRID);
                let point = sphere_point(2.0 * unit - 1.0, angle, 0);
                for paired in 0..domain.sides().len() {
                    let Some((changes, reverses, distance)) =
                        paired_fold(&mut reference, &domain, point, paired, bound)
                    else {
                        continue;
                    };
                    assert_eq!(
                        changes, reverses,
                        "{name} at {height}, {turn}, side {paired}"
                    );
                    worst.record(&distance, &int(1), || {
                        format!(
                            "height {height} and turn {turn} of {GROUP_PAIRING_GRID}, side {paired}"
                        )
                    });
                }
            }
        }
        worst.assert_at_most(&mut reference, bound);
    }
}
