use core::f64::consts::PI;

use escher_geometry::{euclidean, hyperbolic};
use glam::{DMat3, DVec3};

use crate::domain::{FundamentalDomain, PairingKind, Plane};

const ISOMETRY_TOLERANCE: f64 = 1e-9;
const PAIRING_TOLERANCE: f64 = 1e-9;
const WALL_TOLERANCE: f64 = 1e-9;
const RELATOR_TOLERANCE: f64 = 2e-6;
const ANGLE_TOLERANCE: f64 = 1e-9;
const AREA_TOLERANCE: f64 = 1e-9;

/// The result of one numeric check of a [`FundamentalDomain`], a residual and
/// the tolerance it is compared with.
///
/// The residual is a non-negative number that is 0 for an exact domain. It
/// is infinite when a condition of the check that has no size fails, such as
/// a side whose partner is out of range, and when the computation of the
/// check does not give a finite number, such as for a NaN coordinate, so it
/// is never NaN.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*732".parse::<Signature>()?)?;
/// let check = domain.verify().angles();
/// assert!(check.passed());
/// assert!(check.residual() < check.tolerance());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Check {
    residual: f64,
    tolerance: f64,
}

impl Check {
    /// Returns the residual of the check, which is 0 for an exact domain.
    ///
    /// The residual is infinite when a condition of the check without a size
    /// fails, or when its computation does not give a finite number. It is
    /// never NaN.
    #[must_use]
    pub fn residual(self) -> f64 {
        self.residual
    }

    /// Returns the largest residual for which the check passes.
    #[must_use]
    pub fn tolerance(self) -> f64 {
        self.tolerance
    }

    /// Returns whether the residual is at most the tolerance.
    ///
    /// An infinite residual fails.
    #[must_use]
    pub fn passed(self) -> bool {
        self.residual <= self.tolerance
    }
}

/// The result of the six numeric checks of a [`FundamentalDomain`], one
/// [`Check`] for each.
///
/// Use [`Verification::passed`] to decide whether the domain is fit for use,
/// and the accessor of each check to report which condition failed and by
/// how much. Every check is computed, whether or not another one fails.
///
/// The checks of the pairings, the walls, the relators, the angles, and the
/// area read the vertices, and each requires every vertex to be a point with
/// finite coordinates and a positive last coordinate. Each also requires the
/// domain to have as many vertices and corner orders as sides, and at least
/// one side, and its residual is infinite when a requirement fails.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_signature::Signature;
///
/// let verification = FundamentalDomain::build(&"*544".parse::<Signature>()?)?.verify();
/// let checks = [
///     verification.isometries(),
///     verification.pairings(),
///     verification.walls(),
///     verification.relators(),
///     verification.angles(),
///     verification.area(),
/// ];
/// assert!(checks.iter().all(|check| check.passed()));
/// assert!(verification.passed());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Verification {
    isometries: Check,
    pairings: Check,
    walls: Check,
    relators: Check,
    angles: Check,
    area: Check,
}

impl Verification {
    pub(crate) fn of(domain: &FundamentalDomain) -> Self {
        Self {
            isometries: Check {
                residual: finite_or_infinite(isometries(domain)),
                tolerance: ISOMETRY_TOLERANCE,
            },
            pairings: Check {
                residual: finite_or_infinite(pairings(domain)),
                tolerance: PAIRING_TOLERANCE,
            },
            walls: Check {
                residual: finite_or_infinite(walls(domain)),
                tolerance: WALL_TOLERANCE,
            },
            relators: Check {
                residual: finite_or_infinite(relators(domain)),
                tolerance: RELATOR_TOLERANCE,
            },
            angles: Check {
                residual: finite_or_infinite(angles(domain)),
                tolerance: ANGLE_TOLERANCE,
            },
            area: Check {
                residual: finite_or_infinite(area(domain)),
                tolerance: AREA_TOLERANCE,
            },
        }
    }

    /// Returns the check that each pairing isometry is an isometry, a matrix
    /// that preserves the Minkowski form of the linear model in a hyperbolic
    /// domain, and distances in a Euclidean one.
    ///
    /// The residual is the largest
    /// [`hyperbolic::isometry_defect`](escher_geometry::hyperbolic::isometry_defect)
    /// or
    /// [`euclidean::isometry_defect`](escher_geometry::euclidean::isometry_defect)
    /// of a pairing. It is infinite when an entry of a pairing is not finite,
    /// or, in a hyperbolic domain, when a pairing maps the hyperboloid onto
    /// its other sheet.
    #[must_use]
    pub fn isometries(&self) -> Check {
        self.isometries
    }

    /// Returns the check that the sides pair correctly.
    ///
    /// The residual is infinite unless every vertex is a point with finite
    /// coordinates and a positive last coordinate, the partner of each side is
    /// in range, the partner of the partner of each side is the side itself,
    /// the centre is inside every side, each pairing moves the centre outside
    /// the partner side, and the values below are finite. Otherwise the
    /// residual is the larger of two values. The first is the largest entry of
    /// the product of a pairing and its partner's pairing minus the identity.
    /// The second is the largest coordinate of the difference between an
    /// endpoint of a mirror side and its image under the pairing, divided by
    /// the last coordinate of the endpoint.
    #[must_use]
    pub fn pairings(&self) -> Check {
        self.pairings
    }

    /// Returns the check that each side lies on its wall and that the wall,
    /// its covector, and its pairing agree.
    ///
    /// The residual is the largest of these values. The first is the absolute
    /// value of the side test of an endpoint against its wall, divided by the
    /// last coordinate of the endpoint. In a hyperbolic domain, the second is
    /// the largest entry of a pairing minus the reflection in its normal, and
    /// the third is the largest coordinate of a covector minus the covector of
    /// its normal. In a Euclidean domain, the second is the largest entry of a
    /// pairing minus the reflection in its covector, the third is the largest
    /// coordinate of a normal minus the first two coordinates of its covector,
    /// and the fourth is how far the sum of the squares of those two
    /// coordinates is from 1. The residual is infinite when a vertex is not a
    /// point with finite coordinates and a positive last coordinate, or when
    /// one of these values is not finite.
    #[must_use]
    pub fn walls(&self) -> Check {
        self.walls
    }

    /// Returns the check that the relator of each corner holds, which says
    /// that the pairings close up around the corner.
    ///
    /// At a corner of order `n`, the relator is the product of the pairings
    /// of the two sides that meet there, raised to the power `n`, and it must
    /// be the identity. The residual is the largest entry of the relator minus
    /// the identity, divided by `n`, computed in a hyperbolic domain with the
    /// corner moved to the centre of the disk. The residual is infinite when a
    /// vertex is not a point with finite coordinates and a positive last
    /// coordinate, or when an entry of the relator is not finite. The
    /// tolerance is 2e-6.
    #[must_use]
    pub fn relators(&self) -> Check {
        self.relators
    }

    /// Returns the check that each corner of order `n` has the angle `pi / n`.
    ///
    /// The residual is the largest absolute difference between a measured
    /// angle and `pi / n`. It is infinite when a vertex is not a point with
    /// finite coordinates and a positive last coordinate, when an angle is
    /// not finite, or, in a hyperbolic domain, when the angles sum to `pi` or
    /// more, which no triangle of the hyperbolic plane does.
    #[must_use]
    pub fn angles(&self) -> Check {
        self.angles
    }

    /// Returns the check that the domain agrees with the area of the
    /// signature.
    ///
    /// The area of a hyperbolic orbifold is `-2 pi` times its Euler
    /// characteristic. In a hyperbolic domain, the residual is the absolute
    /// difference between [`FundamentalDomain::area`] and that area. A
    /// Euclidean orbifold has the Euler characteristic 0, so the signature
    /// fixes no area. The residual of a Euclidean domain is instead the
    /// absolute difference between `(m - 2) pi` and the sum of the corner
    /// angles of its `m` corners. The residual is infinite when a vertex is
    /// not a point with finite coordinates and a positive last coordinate, or
    /// when the measured area or angle sum is not finite.
    #[must_use]
    pub fn area(&self) -> Check {
        self.area
    }

    /// Returns whether every check passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        [
            self.isometries,
            self.pairings,
            self.walls,
            self.relators,
            self.angles,
            self.area,
        ]
        .into_iter()
        .all(Check::passed)
    }
}

fn finite_or_infinite(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        f64::INFINITY
    }
}

fn largest(values: impl IntoIterator<Item = f64>) -> f64 {
    values.into_iter().fold(0.0, |largest, value| {
        if largest.is_nan() || value.is_nan() {
            f64::NAN
        } else {
            largest.max(value)
        }
    })
}

fn largest_entry(matrix: DMat3) -> f64 {
    largest(matrix.to_cols_array().map(f64::abs))
}

fn largest_coordinate(vector: DVec3) -> f64 {
    largest(vector.to_array().map(f64::abs))
}

fn vertices_are_points(domain: &FundamentalDomain) -> bool {
    let count = domain.sides.len();
    count > 0
        && domain.vertices.len() == count
        && domain.orders.len() == count
        && domain
            .vertices
            .iter()
            .all(|vertex| vertex.is_finite() && vertex.z > 0.0)
}

fn endpoints(domain: &FundamentalDomain, index: usize) -> [DVec3; 2] {
    let count = domain.vertices.len();
    [
        domain.vertices[(index + count - 1) % count],
        domain.vertices[index],
    ]
}

fn side_value(domain: &FundamentalDomain, covector: DVec3, point: DVec3) -> f64 {
    match domain.plane {
        Plane::Hyperbolic { .. } => hyperbolic::side(covector, point),
        Plane::Euclidean => euclidean::side(covector, point),
    }
}

fn isometries(domain: &FundamentalDomain) -> f64 {
    let defect = match domain.plane {
        Plane::Hyperbolic { .. } => hyperbolic::isometry_defect,
        Plane::Euclidean => euclidean::isometry_defect,
    };
    largest(domain.sides.iter().map(|side| defect(side.pairing)))
}

fn pairings(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    let centre = domain.centre;
    let mut structural = true;
    let mut numeric = Vec::new();
    for (index, current) in domain.sides.iter().enumerate() {
        let Some(partner) = domain.sides.get(current.partner) else {
            structural = false;
            continue;
        };
        let inside = side_value(domain, current.covector, centre) < 0.0;
        let across = side_value(domain, partner.covector, current.pairing * centre) > 0.0;
        structural &= partner.partner == index && inside && across;
        numeric.push(largest_entry(
            current.pairing * partner.pairing - DMat3::IDENTITY,
        ));
        match current.kind {
            PairingKind::Mirror => {
                for endpoint in endpoints(domain, index) {
                    numeric.push(
                        largest_coordinate(current.pairing * endpoint - endpoint) / endpoint.z,
                    );
                }
            }
        }
    }
    if structural {
        largest(numeric)
    } else {
        f64::INFINITY
    }
}

fn walls(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    let mut values = Vec::new();
    for (index, current) in domain.sides.iter().enumerate() {
        for endpoint in endpoints(domain, index) {
            values.push(side_value(domain, current.covector, endpoint).abs() / endpoint.z);
        }
        match domain.plane {
            Plane::Hyperbolic { .. } => {
                values.push(largest_entry(
                    current.pairing - hyperbolic::reflection(current.normal),
                ));
                values.push(largest_coordinate(
                    current.covector - hyperbolic::covector(current.normal),
                ));
            }
            Plane::Euclidean => {
                let line = current.covector;
                values.push(largest_entry(current.pairing - euclidean::reflection(line)));
                values.push(largest_coordinate(
                    current.normal - DVec3::new(line.x, line.y, 0.0),
                ));
                values.push((line.x * line.x + line.y * line.y - 1.0).abs());
            }
        }
    }
    largest(values)
}

fn relators(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    let count = domain.sides.len();
    largest((0..count).map(|corner| {
        let order = domain.orders[corner];
        let (first, second) = (
            domain.sides[corner].pairing,
            domain.sides[(corner + 1) % count].pairing,
        );
        let rotation = match domain.plane {
            Plane::Hyperbolic { .. } => {
                let frame = hyperbolic::translation(domain.vertices[corner]);
                hyperbolic::inverse(frame) * first * second * frame
            }
            Plane::Euclidean => first * second,
        };
        let mut power = rotation;
        for _ in 1..order {
            power *= rotation;
        }
        largest_entry(power - DMat3::IDENTITY) / f64::from(order)
    }))
}

fn angles(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    let measured = domain.angles();
    if let Plane::Hyperbolic { .. } = domain.plane {
        let below_pi = measured.iter().sum::<f64>() < PI;
        if !below_pi {
            return f64::INFINITY;
        }
    }
    largest(
        measured
            .iter()
            .zip(&domain.orders)
            .map(|(angle, &order)| (angle - PI / f64::from(order)).abs()),
    )
}

fn area(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    match domain.plane {
        Plane::Hyperbolic { area } => (domain.area() - area).abs(),
        Plane::Euclidean => {
            let measured = domain.angles();
            let corners = u32::try_from(measured.len()).map_or(f64::INFINITY, f64::from);
            ((corners - 2.0) * PI - measured.iter().sum::<f64>()).abs()
        }
    }
}

#[cfg(test)]
mod tests {
    use escher_geometry::euclidean;
    use escher_geometry::hyperbolic::{covector, reflection, rotation};
    use escher_signature::Signature;

    use super::{Check, Verification};
    use crate::domain::{FundamentalDomain, Plane};
    use crate::triangle::{Supported, construct, supported};

    const BASES: [[u32; 3]; 3] = [[7, 3, 2], [4, 4, 4], [1000, 3, 2]];

    const NAMES: [&str; 6] = [
        "isometries",
        "pairings",
        "walls",
        "relators",
        "angles",
        "area",
    ];

    fn text(orders: [u32; 3]) -> String {
        let corners: String = orders
            .iter()
            .map(|order| {
                if *order < 10 {
                    order.to_string()
                } else {
                    format!("({order})")
                }
            })
            .collect();
        format!("*{corners}")
    }

    fn labelled(orders: [u32; 3], label: [u32; 3]) -> FundamentalDomain {
        let signature: Signature = match text(label).parse() {
            Ok(signature) => signature,
            Err(error) => panic!("{label:?} is rejected: {error}"),
        };
        let Some(Supported::HyperbolicTriangle {
            orders: label_orders,
            area,
        }) = supported(&signature)
        else {
            panic!("{signature} is not a supported hyperbolic triangle");
        };
        let triangle = match construct(orders) {
            Ok(triangle) => triangle,
            Err(error) => panic!("{orders:?} has no triangle: {error:?}"),
        };
        FundamentalDomain {
            plane: Plane::Hyperbolic { area },
            orders: label_orders.to_vec(),
            sides: triangle.sides.to_vec(),
            vertices: triangle.vertices.to_vec(),
            centre: triangle.centre,
        }
    }

    fn built(text: &str) -> FundamentalDomain {
        let signature: Signature = match text.parse() {
            Ok(signature) => signature,
            Err(error) => panic!("{text:?} is rejected: {error}"),
        };
        match FundamentalDomain::build(&signature) {
            Ok(domain) => domain,
            Err(error) => panic!("{text} has no domain: {error}"),
        }
    }

    fn euclidean_domains() -> [FundamentalDomain; 2] {
        [built("*632"), built("*2222")]
    }

    fn domain(orders: [u32; 3]) -> FundamentalDomain {
        labelled(orders, orders)
    }

    fn failed(verification: &Verification) -> Vec<&'static str> {
        let checks = [
            verification.isometries(),
            verification.pairings(),
            verification.walls(),
            verification.relators(),
            verification.angles(),
            verification.area(),
        ];
        NAMES
            .into_iter()
            .zip(checks)
            .filter(|(_, check)| !check.passed())
            .map(|(name, _)| name)
            .collect()
    }

    fn assert_failed(domain: &FundamentalDomain, expected: &[&str], defect: &str) {
        let verification = domain.verify();
        assert_eq!(
            failed(&verification),
            expected,
            "{defect} on {:?}: {verification:?}",
            domain.orders
        );
        assert!(!verification.passed(), "{defect} on {:?}", domain.orders);
    }

    fn tilt(domain: &mut FundamentalDomain, angle: f64) {
        let side = &mut domain.sides[0];
        let normal = rotation(angle) * side.normal;
        side.normal = normal;
        side.covector = covector(normal);
        side.pairing = reflection(normal);
    }

    #[test]
    fn verify_passes_on_built_domains() {
        for orders in BASES {
            let verification = domain(orders).verify();
            assert!(
                failed(&verification).is_empty(),
                "{orders:?}: {verification:?}"
            );
            assert!(verification.passed());
        }
    }

    #[test]
    fn verify_fails_on_a_scaled_pairing() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            corrupted.sides[0].pairing *= 1.0 + 1e-6;
            assert_failed(
                &corrupted,
                &["isometries", "pairings", "walls"],
                "scaled pairing",
            );
        }
    }

    #[test]
    fn verify_fails_on_a_wrong_partner() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            corrupted.sides[0].partner = 1;
            assert_failed(&corrupted, &["pairings"], "wrong partner");
        }
    }

    #[test]
    fn verify_fails_on_an_out_of_range_partner() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            corrupted.sides[2].partner = 3;
            assert_failed(&corrupted, &["pairings"], "out-of-range partner");
        }
    }

    #[test]
    fn verify_fails_on_swapped_sides() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            corrupted.sides.swap(0, 1);
            let expected: &[&str] = if orders == [4, 4, 4] {
                &["pairings", "walls"]
            } else {
                &["pairings", "walls", "relators"]
            };
            assert_failed(&corrupted, expected, "swapped sides");
        }
    }

    #[test]
    fn verify_fails_on_a_tilted_wall_with_its_pairing() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            tilt(&mut corrupted, 1e-6);
            assert_failed(
                &corrupted,
                &["pairings", "walls", "relators"],
                "wall and pairing tilted by 1e-6",
            );
        }
    }

    #[test]
    fn verify_fails_on_a_tilted_wall_without_its_pairing() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            let pairing = corrupted.sides[0].pairing;
            tilt(&mut corrupted, 1e-3);
            corrupted.sides[0].pairing = pairing;
            assert_failed(
                &corrupted,
                &["walls"],
                "wall tilted by 1e-3 without its pairing",
            );
        }
    }

    #[test]
    fn verify_fails_on_a_triangle_with_the_wrong_signature() {
        assert_failed(
            &labelled([7, 3, 3], [7, 3, 2]),
            &["relators", "angles", "area"],
            "(7, 3, 3) labelled *732",
        );
    }

    #[test]
    fn verify_fails_on_the_next_order_above_the_limit_labelled_as_the_limit() {
        assert_failed(
            &labelled([1001, 3, 2], [1000, 3, 2]),
            &["relators", "angles", "area"],
            "(1001, 3, 2) labelled *(1000)32",
        );
    }

    fn assert_corner_order_off_by_one_fails(base: [u32; 3], corner: usize) {
        let mut orders = base;
        orders[corner] += 1;
        assert_failed(
            &labelled(orders, base),
            &["relators", "angles", "area"],
            &format!("{orders:?} labelled with the order of corner {corner} one less"),
        );
    }

    #[test]
    fn verify_fails_on_a_triangle_whose_corner_0_has_the_wrong_order() {
        for base in [[7, 3, 2], [4, 4, 4]] {
            assert_corner_order_off_by_one_fails(base, 0);
        }
    }

    #[test]
    fn verify_fails_on_a_triangle_whose_corner_1_has_the_wrong_order() {
        for base in [[7, 3, 2], [4, 4, 4]] {
            assert_corner_order_off_by_one_fails(base, 1);
        }
    }

    #[test]
    fn verify_fails_on_a_triangle_whose_corner_2_has_the_wrong_order() {
        for base in [[7, 3, 2], [4, 4, 4]] {
            assert_corner_order_off_by_one_fails(base, 2);
        }
    }

    #[test]
    fn verify_fails_on_a_triangle_labelled_with_euclidean_orders() {
        let mut corrupted = domain([6, 4, 3]);
        corrupted.orders = vec![6, 3, 2];
        corrupted.plane = Plane::Hyperbolic { area: 0.0 };
        assert_failed(
            &corrupted,
            &["relators", "angles", "area"],
            "(6, 4, 3) labelled with the orders and area of *632",
        );
    }

    fn assert_infinite_failures(domain: &FundamentalDomain, expected: &[&str], defect: &str) {
        assert_failed(domain, expected, defect);
        let verification = domain.verify();
        let checks = [
            verification.isometries(),
            verification.pairings(),
            verification.walls(),
            verification.relators(),
            verification.angles(),
            verification.area(),
        ];
        for (name, check) in NAMES.into_iter().zip(checks) {
            assert!(!check.residual().is_nan(), "{defect}: {name} is NaN");
            if expected.contains(&name) {
                assert_eq!(
                    check.residual(),
                    f64::INFINITY,
                    "{defect}: {name}: {verification:?}"
                );
            }
        }
    }

    #[test]
    fn verify_reports_infinite_residuals_for_a_nan_vertex() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            corrupted.vertices[1].x = f64::NAN;
            assert_infinite_failures(
                &corrupted,
                &["pairings", "walls", "relators", "angles", "area"],
                "NaN vertex",
            );
        }
    }

    #[test]
    fn verify_reports_infinite_residuals_for_a_nan_normal() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            corrupted.sides[1].normal.y = f64::NAN;
            assert_infinite_failures(&corrupted, &["walls"], "NaN normal");
        }
    }

    #[test]
    fn verify_reports_infinite_residuals_for_a_nan_pairing_entry() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            corrupted.sides[2].pairing.y_axis.x = f64::NAN;
            assert_infinite_failures(
                &corrupted,
                &["isometries", "pairings", "walls", "relators"],
                "NaN pairing entry",
            );
        }
    }

    #[test]
    fn verify_fails_on_a_vertex_on_the_other_sheet() {
        for orders in BASES {
            let mut corrupted = domain(orders);
            corrupted.vertices[0] = -corrupted.vertices[0];
            assert_infinite_failures(
                &corrupted,
                &["pairings", "walls", "relators", "angles", "area"],
                "vertex 0 negated",
            );
        }
    }

    #[test]
    fn verify_passes_on_built_euclidean_domains() {
        for domain in euclidean_domains() {
            let verification = domain.verify();
            assert!(
                failed(&verification).is_empty(),
                "{:?}: {verification:?}",
                domain.orders
            );
            assert!(verification.passed());
        }
    }

    #[test]
    fn verify_fails_on_the_euclidean_triangle_of_442_labelled_632() {
        let mut corrupted = built("*442");
        corrupted.orders = vec![6, 3, 2];
        assert_failed(
            &corrupted,
            &["relators", "angles"],
            "*442 labelled with the orders of *632",
        );
    }

    #[test]
    fn verify_fails_on_a_moved_euclidean_line_without_its_pairing() {
        for mut corrupted in euclidean_domains() {
            corrupted.sides[0].covector.z -= 1e-6;
            assert_failed(&corrupted, &["walls"], "offset moved without its pairing");
        }
    }

    #[test]
    fn verify_fails_on_a_moved_euclidean_line_with_its_pairing() {
        for mut corrupted in euclidean_domains() {
            let side = &mut corrupted.sides[0];
            side.covector.z -= 1e-6;
            side.pairing = euclidean::reflection(side.covector);
            assert_failed(
                &corrupted,
                &["pairings", "walls"],
                "offset moved with its pairing",
            );
        }
    }

    #[test]
    fn verify_fails_on_a_scaled_euclidean_pairing() {
        for mut corrupted in euclidean_domains() {
            corrupted.sides[0].pairing *= 1.0 + 1e-6;
            assert_failed(
                &corrupted,
                &["isometries", "pairings", "walls"],
                "scaled pairing",
            );
        }
    }

    #[test]
    fn verify_fails_on_a_wrong_euclidean_partner() {
        for mut corrupted in euclidean_domains() {
            corrupted.sides[0].partner = 1;
            assert_failed(&corrupted, &["pairings"], "wrong partner");
        }
    }

    #[test]
    fn verify_reports_infinite_residuals_for_a_nan_euclidean_vertex() {
        for mut corrupted in euclidean_domains() {
            corrupted.vertices[1].x = f64::NAN;
            assert_infinite_failures(
                &corrupted,
                &["pairings", "walls", "relators", "angles", "area"],
                "NaN vertex",
            );
        }
    }

    #[test]
    fn verify_fails_on_a_euclidean_pairing_with_a_wrong_last_row() {
        for mut corrupted in euclidean_domains() {
            corrupted.sides[0].pairing.x_axis.z = 1e-6;
            assert_failed(
                &corrupted,
                &["isometries", "pairings", "walls"],
                "last row of the pairing set to (1e-6, 0, 1)",
            );
        }
    }

    const READ_THE_COUNTS: [&str; 5] = ["pairings", "walls", "relators", "angles", "area"];

    #[test]
    fn verify_fails_on_a_rectangle_labelled_with_three_orders() {
        let mut corrupted = built("*2222");
        corrupted.orders = vec![6, 3, 2];
        assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "rectangle with three orders");
    }

    #[test]
    fn verify_fails_on_a_domain_with_a_vertex_removed() {
        for mut corrupted in euclidean_domains() {
            corrupted.vertices.pop();
            assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "vertex removed");
        }
        let mut corrupted = domain([7, 3, 2]);
        corrupted.vertices.pop();
        assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "vertex removed");
    }

    #[test]
    fn verify_fails_on_a_domain_without_vertices() {
        for mut corrupted in euclidean_domains().into_iter().chain([domain([7, 3, 2])]) {
            corrupted.vertices.clear();
            assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "no vertices");
        }
    }

    #[test]
    fn check_with_a_nan_residual_fails() {
        let check = Check {
            residual: f64::NAN,
            tolerance: 1e-9,
        };
        assert!(!check.passed());
    }

    #[test]
    fn check_with_a_residual_equal_to_the_tolerance_passes() {
        let check = Check {
            residual: 1e-9,
            tolerance: 1e-9,
        };
        assert!(check.passed());
    }

    #[test]
    fn check_with_an_infinite_residual_fails() {
        let check = Check {
            residual: f64::INFINITY,
            tolerance: 1e-9,
        };
        assert!(!check.passed());
    }
}
