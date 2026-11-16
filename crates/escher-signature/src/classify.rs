use core::cmp::Ordering;
use core::num::NonZeroU64;

use crate::error::{Detail, Value};
use crate::order::Order;
use crate::parse::Features;
use crate::rational::{Rational, gcd};

/// The geometry of constant curvature in which the group of a signature
/// acts: the sphere, the Euclidean plane, or the hyperbolic plane.
///
/// The sign of the Euler characteristic decides it: positive for the sphere,
/// zero for the Euclidean plane, and negative for the hyperbolic plane.
///
/// # Examples
///
/// ```
/// use escher_signature::{Geometry, Signature};
///
/// let geometries = ["*532", "*632", "*732"]
///     .map(|text| text.parse::<Signature>().map(|signature| signature.geometry()));
/// assert_eq!(
///     geometries,
///     [Ok(Geometry::Spherical), Ok(Geometry::Euclidean), Ok(Geometry::Hyperbolic)],
/// );
/// ```
#[expect(
    clippy::exhaustive_enums,
    reason = "the three geometries of constant curvature are a closed set"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Geometry {
    /// The sphere, for a positive Euler characteristic. The group is finite.
    Spherical,
    /// The Euclidean plane, for an Euler characteristic of 0.
    Euclidean,
    /// The hyperbolic plane, for a negative Euler characteristic.
    Hyperbolic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Derived {
    pub(crate) chi: Rational,
    pub(crate) area_over_pi: Option<Rational>,
    pub(crate) group_order: Option<NonZeroU64>,
    pub(crate) deformation_dimension: Option<u32>,
}

pub(crate) fn derive(features: &Features) -> Result<Derived, Detail> {
    let (chi, twice_chi) =
        characteristic(features).ok_or(Detail::Overflow(Value::EulerCharacteristic))?;
    if is_bad_orbifold(features) {
        return Err(Detail::BadOrbifold);
    }
    let compact = !has_infinite_order(features);
    if !compact && chi.numerator() > 0 {
        return Err(Detail::PositiveNonCompact);
    }
    let geometry = geometry(chi);
    let group_order = match geometry {
        Geometry::Spherical => Some(group_order(chi).ok_or(Detail::Overflow(Value::GroupOrder))?),
        Geometry::Euclidean | Geometry::Hyperbolic => None,
    };
    let deformation_dimension = match geometry {
        Geometry::Hyperbolic if compact => Some(
            deformation_dimension(features).ok_or(Detail::Overflow(Value::DeformationDimension))?,
        ),
        Geometry::Spherical | Geometry::Euclidean | Geometry::Hyperbolic => None,
    };
    let area_over_pi = match geometry {
        Geometry::Spherical | Geometry::Hyperbolic => Some(
            twice_chi
                .abs()
                .ok_or(Detail::Overflow(Value::EulerCharacteristic))?,
        ),
        Geometry::Euclidean => None,
    };
    Ok(Derived {
        chi,
        area_over_pi,
        group_order,
        deformation_dimension,
    })
}

pub(crate) fn geometry(chi: Rational) -> Geometry {
    match chi.numerator().cmp(&0) {
        Ordering::Greater => Geometry::Spherical,
        Ordering::Equal => Geometry::Euclidean,
        Ordering::Less => Geometry::Hyperbolic,
    }
}

fn characteristic(features: &Features) -> Option<(Rational, Rational)> {
    let corners = || {
        features
            .boundaries
            .iter()
            .flat_map(|boundary| boundary.corners.iter())
    };
    let finite_cones = features
        .cone_orders
        .iter()
        .filter_map(|order| order.finite());
    let finite_corners = corners().filter_map(|order| order.finite());

    let mut lcm: i128 = 1;
    for order in finite_cones.clone().chain(finite_corners.clone()) {
        let order = i128::from(order);
        let divisor = i128::try_from(gcd(lcm.unsigned_abs(), order.unsigned_abs())).ok()?;
        lcm = lcm.checked_div(divisor)?.checked_mul(order)?;
    }
    let denominator = lcm.checked_mul(2)?;

    let cone_count = i128::try_from(features.cone_orders.len()).ok()?;
    let corner_count = i128::try_from(corners().count()).ok()?;
    let boundary_count = i128::try_from(features.boundaries.len()).ok()?;
    let surface = 2_i128
        .checked_sub(i128::from(features.handles).checked_mul(2)?)?
        .checked_sub(i128::from(features.crosscaps))?
        .checked_sub(boundary_count)?
        .checked_sub(cone_count)?;

    let mut numerator = denominator
        .checked_mul(surface)?
        .checked_sub(lcm.checked_mul(corner_count)?)?;
    for order in finite_cones {
        numerator = numerator.checked_add(denominator.checked_div(i128::from(order))?)?;
    }
    for order in finite_corners {
        numerator = numerator.checked_add(lcm.checked_div(i128::from(order))?)?;
    }
    narrow(numerator, denominator, lcm)
}

fn narrow(numerator: i128, denominator: i128, lcm: i128) -> Option<(Rational, Rational)> {
    let chi = Rational::new(numerator, denominator)?;
    let twice_chi = Rational::new(numerator, lcm)?;
    let absolute_values_fit = chi.numerator() != i64::MIN && twice_chi.numerator() != i64::MIN;
    absolute_values_fit.then_some((chi, twice_chi))
}

fn group_order(chi: Rational) -> Option<NonZeroU64> {
    let numerator = i128::from(chi.numerator());
    let doubled = i128::from(chi.denominator().get()).checked_mul(2)?;
    if doubled.checked_rem(numerator)? != 0 {
        return None;
    }
    NonZeroU64::new(u64::try_from(doubled.checked_div(numerator)?).ok()?)
}

fn deformation_dimension(features: &Features) -> Option<u32> {
    let cones = i64::try_from(features.cone_orders.len()).ok()?;
    let boundaries = i64::try_from(features.boundaries.len()).ok()?;
    let corners = features
        .boundaries
        .iter()
        .try_fold(0_i64, |count, boundary| {
            count.checked_add(i64::try_from(boundary.corners.len()).ok()?)
        })?;
    let dimension = i64::from(features.handles)
        .checked_mul(6)?
        .checked_add(i64::from(features.crosscaps).checked_mul(3)?)?
        .checked_add(boundaries.checked_mul(3)?)?
        .checked_add(cones.checked_mul(2)?)?
        .checked_add(corners)?
        .checked_sub(6)?;
    u32::try_from(dimension).ok()
}

pub(crate) fn is_general_non_compact(features: &Features, chi: Rational) -> bool {
    let triangle_with_ideal_corners = matches!(
        (features.cone_orders.as_slice(), features.boundaries.as_slice()),
        ([], [boundary]) if boundary.corners.len() == 3
    );
    let rotation_triangle = features.cone_orders.len() == 3 && features.boundaries.is_empty();
    let triangle = features.handles == 0
        && features.crosscaps == 0
        && (triangle_with_ideal_corners || rotation_triangle);
    has_infinite_order(features) && chi.numerator() < 0 && !triangle
}

pub(crate) fn has_infinite_order(features: &Features) -> bool {
    features
        .cone_orders
        .iter()
        .chain(
            features
                .boundaries
                .iter()
                .flat_map(|boundary| boundary.corners.iter()),
        )
        .any(|order| order.is_infinite())
}

pub(crate) fn is_bad_orbifold(features: &Features) -> bool {
    if features.handles > 0 || features.crosscaps > 0 || has_infinite_order(features) {
        return false;
    }
    match (
        features.cone_orders.as_slice(),
        features.boundaries.as_slice(),
    ) {
        (orders, []) => is_bad_family(orders),
        ([], [boundary]) => is_bad_family(&boundary.corners),
        _ => false,
    }
}

fn is_bad_family(orders: &[Order]) -> bool {
    match orders {
        [_] => true,
        [first, second] => first != second,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroU64;

    use super::{characteristic, deformation_dimension, group_order, is_bad_orbifold, narrow};
    use crate::order::Order;
    use crate::parse::Features;
    use crate::rational::Rational;
    use crate::signature::Boundary;

    fn order(value: u32) -> Order {
        Order::from_value(value).expect("the test orders are at least 2")
    }

    #[test]
    fn euler_characteristic_of_the_trivial_signature_is_two() {
        let (chi, _) = characteristic(&Features::default()).expect("fits");
        assert_eq!((chi.numerator(), chi.denominator().get()), (2, 1));
    }

    #[test]
    fn euler_characteristic_counts_infinite_orders_without_a_denominator() {
        let features = Features {
            cone_orders: vec![Order::INFINITE, Order::INFINITE],
            ..Features::default()
        };
        let (chi, _) = characteristic(&features).expect("fits");
        assert_eq!((chi.numerator(), chi.denominator().get()), (0, 1));
    }

    #[test]
    fn is_bad_orbifold_matches_the_four_families_only() {
        let cones = |orders: Vec<Order>| Features {
            cone_orders: orders,
            ..Features::default()
        };
        let boundary = |orders: Vec<Order>| Features {
            boundaries: vec![Boundary { corners: orders }],
            ..Features::default()
        };
        assert!(is_bad_orbifold(&cones(vec![order(5)])));
        assert!(is_bad_orbifold(&cones(vec![order(5), order(3)])));
        assert!(is_bad_orbifold(&boundary(vec![order(5)])));
        assert!(is_bad_orbifold(&boundary(vec![order(5), order(3)])));
        assert!(!is_bad_orbifold(&cones(vec![order(5), order(5)])));
        assert!(!is_bad_orbifold(&boundary(vec![order(5), order(5)])));
        assert!(!is_bad_orbifold(&cones(vec![Order::INFINITE])));
        assert!(!is_bad_orbifold(&Features::default()));
        assert!(!is_bad_orbifold(&boundary(Vec::new())));
        let mixed = Features {
            cone_orders: vec![order(2)],
            boundaries: vec![Boundary {
                corners: Vec::new(),
            }],
            ..Features::default()
        };
        assert!(!is_bad_orbifold(&mixed));
    }

    #[test]
    fn narrow_rejects_a_reduced_numerator_whose_absolute_value_does_not_fit() {
        let smallest = i128::from(i64::MIN);
        assert_eq!(narrow(smallest, 2, 1), None);
        assert_eq!(narrow(smallest, 1, 2), None);
        let (chi, twice_chi) = narrow(smallest, 4, 2).expect("both halves fit");
        assert_eq!(
            (chi.numerator(), twice_chi.numerator()),
            (i64::MIN / 4, i64::MIN / 2)
        );
        let largest = i128::from(i64::MAX);
        assert!(narrow(largest, 2, 1).is_some());
    }

    #[test]
    fn group_order_is_two_over_the_characteristic_when_whole_and_in_range() {
        let order = |numerator, denominator| {
            Rational::new(numerator, denominator)
                .and_then(group_order)
                .map(NonZeroU64::get)
        };
        assert_eq!(order(1, 60), Some(120));
        assert_eq!(order(2, 1), Some(1));
        assert_eq!(order(3, 5), None);
        assert_eq!(order(1, i128::from(u64::MAX)), None);
    }

    #[test]
    fn deformation_dimension_rejects_a_negative_value() {
        let cones = |values: &[u32]| Features {
            cone_orders: values.iter().map(|&value| order(value)).collect(),
            ..Features::default()
        };
        assert_eq!(deformation_dimension(&cones(&[7, 3, 2])), Some(0));
        assert_eq!(deformation_dimension(&cones(&[2, 2, 2, 3])), Some(2));
        assert_eq!(deformation_dimension(&cones(&[2, 2])), None);
        assert_eq!(deformation_dimension(&Features::default()), None);
    }
}
