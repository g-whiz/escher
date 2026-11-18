use core::f64::consts::PI;

use escher_geometry::{euclidean, hyperbolic, spherical};
use glam::{DMat3, DVec3};

use crate::domain::{FundamentalDomain, Orientation, PairingKind, Plane, polygon_angle_limit};

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
/// area read the vertices, and each requires every vertex to be a point: a
/// vector with finite coordinates and, in a hyperbolic or Euclidean domain, a
/// positive last coordinate, or, in a spherical domain, not the zero vector.
/// Each also requires the domain to have as many vertices and corner orders
/// as sides, and at least one side, and its residual is infinite when a
/// requirement fails.
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
    /// domain, and distances in a Euclidean or spherical one.
    ///
    /// The residual is the largest
    /// [`hyperbolic::isometry_defect`](escher_geometry::hyperbolic::isometry_defect),
    /// [`euclidean::isometry_defect`](escher_geometry::euclidean::isometry_defect),
    /// or
    /// [`spherical::isometry_defect`](escher_geometry::spherical::isometry_defect)
    /// of a pairing. It is infinite when an entry of a pairing is not finite,
    /// or, in a hyperbolic domain, when a pairing maps the hyperboloid onto
    /// its other sheet.
    #[must_use]
    pub fn isometries(&self) -> Check {
        self.isometries
    }

    /// Returns the check that the sides pair correctly.
    ///
    /// The residual is infinite unless every vertex is a point, the partner of
    /// each side is in range, the partner of the partner of each side is the
    /// side itself, the centre is inside every side, each pairing moves the
    /// centre outside the partner side, and the values below are finite. It is
    /// also infinite unless the kind of each side agrees with its partner and
    /// its orientation: a [mirror](crate::PairingKind::Mirror) is its own
    /// partner and reverses orientation, and a side of another kind has
    /// another side as its partner and preserves orientation. The determinant
    /// of each pairing must also be positive for a pairing that preserves
    /// orientation and negative for one that reverses it, and a side has the
    /// kind [`Translation`](crate::PairingKind::Translation) exactly when the
    /// domain is Euclidean and the linear part of its pairing is the identity
    /// within the tolerance. Otherwise the residual is the largest of three
    /// values, and the third applies only to a side that is not a mirror. The
    /// first is the largest entry of the product of a pairing and its partner's pairing minus the
    /// identity. The second is the largest coordinate of a difference between
    /// two points. For a mirror side, the points are each endpoint of the side
    /// and its image under the pairing. For a side of another kind, the points
    /// are the image of each endpoint and the endpoint of the partner side that
    /// it must map to. Side `j` has the endpoints vertex `j - 1` and vertex
    /// `j`, so the pairing must map vertex `j - 1` onto the second endpoint of
    /// the partner side, and vertex `j` onto its first endpoint. The difference
    /// is divided by the last coordinate of the endpoint in a hyperbolic or
    /// Euclidean domain. The third is the largest coordinate of the sum of the
    /// covector of the side and the covector of the partner multiplied by the
    /// transpose of the pairing, which is 0 when the pairing carries the wall
    /// of the side onto the wall of the partner with the domain on opposite
    /// sides.
    #[must_use]
    pub fn pairings(&self) -> Check {
        self.pairings
    }

    /// Returns the check that each side lies on its wall and that the wall,
    /// its covector, and, for a mirror, its pairing agree.
    ///
    /// The residual is the largest of these values. The first is the absolute
    /// value of the side test of an endpoint against its wall, divided by the
    /// last coordinate of the endpoint in a hyperbolic or Euclidean domain. In
    /// a hyperbolic domain, the second is the largest entry of a pairing minus
    /// the reflection in its normal, and the third is the largest coordinate
    /// of a covector minus the covector of its normal. In a Euclidean domain,
    /// the second is the largest entry of a pairing minus the reflection in
    /// its covector, the third is the largest coordinate of a normal minus the
    /// first two coordinates of its covector, and the fourth is how far the
    /// sum of the squares of those two coordinates is from 1. In a spherical
    /// domain, the second is the largest entry of a pairing minus the
    /// reflection in its normal, the third is the largest coordinate of a
    /// covector minus its normal, and the fourth is how far the dot product of
    /// the normal with itself is from 1. The second value applies to a
    /// [mirror](crate::PairingKind::Mirror) only. The last value is the largest
    /// side test of a vertex outside a wall, divided by the last coordinate of
    /// the vertex in a hyperbolic or Euclidean domain, over every vertex and
    /// every wall, and is 0 for a convex domain. A spherical domain adds how
    /// far the dot product of each vertex with itself is from 1, and for a
    /// domain with two vertices, which must be antipodal, the largest
    /// coordinate of their sum. The residual is infinite when a vertex is not
    /// a point, or when one of these values is not finite.
    #[must_use]
    pub fn walls(&self) -> Check {
        self.walls
    }

    /// Returns the check that the relator of each corner holds, which says
    /// that the pairings close up around the corner.
    ///
    /// For a domain whose sides are all mirrors, at a corner of order `n`, the
    /// relator is the product of the pairings of the two sides that meet
    /// there, raised to the power `n`, and it must be the identity.
    ///
    /// For a domain without a mirror, the pairing of the side that has a
    /// vertex as its first endpoint carries that vertex onto a vertex of the
    /// domain, possibly itself. The vertices that this map takes into one
    /// another form a vertex cycle. The product of the pairings along a cycle
    /// must fix its first vertex, and raised to the order `n` of the cycle it
    /// must be the identity. Every vertex of a cycle must have that order. A
    /// vertex of a double holds one corner of the domain that it doubles, or
    /// two when it lies on the doubled wall, and the vertices of a cycle must
    /// hold two corners in all.
    ///
    /// The residual is the largest entry of the relator minus the identity,
    /// divided by `n`, where a hyperbolic relator is computed with the corner,
    /// or the first vertex of the cycle, moved to the centre of the disk. For a
    /// cycle, the residual also covers the largest coordinate of the
    /// difference between its first vertex and the image of that vertex under
    /// the product, divided by its last coordinate in a hyperbolic or
    /// Euclidean domain. The residual is infinite when a vertex is not a
    /// point, when an entry of a relator is not finite, when a cycle breaks
    /// one of its conditions, and for a domain with both mirrors and sides of
    /// another kind. The tolerance is 2e-6.
    #[must_use]
    pub fn relators(&self) -> Check {
        self.relators
    }

    /// Returns the check that each corner of order `n` has the angle `pi / n`.
    ///
    /// A vertex of a double where two corners of the domain that it doubles
    /// meet has twice that angle. The residual is the largest absolute
    /// difference between an angle that
    /// [`FundamentalDomain::corner_angles`] measures and `pi / n`, or
    /// `2 pi / n` at such a vertex.
    ///
    /// The residual is infinite in these cases:
    ///
    /// - A vertex is not a point, or an angle is not finite.
    /// - In a hyperbolic domain, the angles of a triangle sum to `pi` or more,
    ///   which no triangle of the hyperbolic plane does.
    /// - In a hyperbolic domain with `m` vertices, five or more, the corner
    ///   angles sum to `(m - 2) pi` or more, which no polygon of the
    ///   hyperbolic plane does.
    /// - In a spherical domain, the angles of a triangle sum to `pi` or less,
    ///   which no triangle of the sphere does.
    #[must_use]
    pub fn angles(&self) -> Check {
        self.angles
    }

    /// Returns the check that the area of the domain agrees with the
    /// signature.
    ///
    /// The area of a hyperbolic or spherical orbifold is `2 pi` times the
    /// absolute value of its Euler characteristic. In a hyperbolic or
    /// spherical domain, the residual is the absolute difference between
    /// [`FundamentalDomain::area`] and that area. A Euclidean orbifold has the
    /// Euler characteristic 0, so the signature fixes no area. The residual of
    /// a Euclidean domain is instead the absolute difference between
    /// `(m - 2) pi` and the sum of the corner angles of its `m` corners. The
    /// residual is infinite when a vertex is not a point, or when the measured
    /// area or angle sum is not finite.
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
    let is_point = |vertex: &DVec3| match domain.plane {
        Plane::Hyperbolic { .. } | Plane::Euclidean => vertex.is_finite() && vertex.z > 0.0,
        Plane::Spherical { .. } => vertex.is_finite() && *vertex != DVec3::ZERO,
    };
    count > 0
        && domain.vertices.len() == count
        && domain.corners.len() == count
        && domain.vertices.iter().all(is_point)
}

fn endpoint_scale(domain: &FundamentalDomain, endpoint: DVec3) -> f64 {
    match domain.plane {
        Plane::Hyperbolic { .. } | Plane::Euclidean => endpoint.z,
        Plane::Spherical { .. } => 1.0,
    }
}

fn endpoints(domain: &FundamentalDomain, index: usize) -> [DVec3; 2] {
    let count = domain.vertices.len();
    [
        domain.vertices[(index + count - 1) % count],
        domain.vertices[index],
    ]
}

fn side_value(domain: &FundamentalDomain, covector: DVec3, point: DVec3) -> f64 {
    domain.plane.side(covector, point)
}

fn isometries(domain: &FundamentalDomain) -> f64 {
    let defect = match domain.plane {
        Plane::Hyperbolic { .. } => hyperbolic::isometry_defect,
        Plane::Euclidean => euclidean::isometry_defect,
        Plane::Spherical { .. } => spherical::isometry_defect,
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
        structural &= kind_agrees(current.kind, current.partner == index, current.orientation);
        structural &= (current.kind == PairingKind::Translation)
            == is_translation(domain.plane, current.pairing);
        let determinant = current.pairing.determinant();
        structural &= match current.orientation {
            Orientation::Preserving => determinant > 0.0,
            Orientation::Reversing => determinant < 0.0,
        };
        numeric.push(largest_entry(
            current.pairing * partner.pairing - DMat3::IDENTITY,
        ));
        match current.kind {
            PairingKind::Mirror => {
                for endpoint in endpoints(domain, index) {
                    numeric.push(
                        largest_coordinate(current.pairing * endpoint - endpoint)
                            / endpoint_scale(domain, endpoint),
                    );
                }
            }
            PairingKind::Rotation | PairingKind::Translation => {
                let [start, end] = endpoints(domain, index);
                let [partner_start, partner_end] = endpoints(domain, current.partner);
                for (image, target) in [(start, partner_end), (end, partner_start)] {
                    numeric.push(
                        largest_coordinate(current.pairing * image - target)
                            / endpoint_scale(domain, target),
                    );
                }
                numeric.push(largest_coordinate(
                    current.pairing.transpose() * partner.covector + current.covector,
                ));
            }
        }
    }
    if structural {
        largest(numeric)
    } else {
        f64::INFINITY
    }
}

pub(crate) fn is_translation(plane: Plane, pairing: DMat3) -> bool {
    let linear = largest(
        [
            pairing.x_axis.x - 1.0,
            pairing.x_axis.y,
            pairing.y_axis.x,
            pairing.y_axis.y - 1.0,
        ]
        .map(f64::abs),
    );
    matches!(plane, Plane::Euclidean) && linear <= PAIRING_TOLERANCE
}

fn kind_agrees(kind: PairingKind, own_partner: bool, orientation: Orientation) -> bool {
    match kind {
        PairingKind::Mirror => own_partner && orientation == Orientation::Reversing,
        PairingKind::Rotation | PairingKind::Translation => {
            !own_partner && orientation == Orientation::Preserving
        }
    }
}

fn walls(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    let mut values = Vec::new();
    for (index, current) in domain.sides.iter().enumerate() {
        for endpoint in endpoints(domain, index) {
            values.push(
                side_value(domain, current.covector, endpoint).abs()
                    / endpoint_scale(domain, endpoint),
            );
        }
        let mirror = match current.kind {
            PairingKind::Mirror => true,
            PairingKind::Rotation | PairingKind::Translation => false,
        };
        match domain.plane {
            Plane::Hyperbolic { .. } => {
                if mirror {
                    values.push(largest_entry(
                        current.pairing - hyperbolic::reflection(current.normal),
                    ));
                }
                values.push(largest_coordinate(
                    current.covector - hyperbolic::covector(current.normal),
                ));
            }
            Plane::Euclidean => {
                let line = current.covector;
                if mirror {
                    values.push(largest_entry(current.pairing - euclidean::reflection(line)));
                }
                values.push(largest_coordinate(
                    current.normal - DVec3::new(line.x, line.y, 0.0),
                ));
                values.push((line.x * line.x + line.y * line.y - 1.0).abs());
            }
            Plane::Spherical { .. } => {
                let normal = current.normal;
                if mirror {
                    values.push(largest_entry(
                        current.pairing - spherical::reflection(normal),
                    ));
                }
                values.push(largest_coordinate(current.covector - normal));
                values.push((normal.dot(normal) - 1.0).abs());
            }
        }
        for &vertex in &domain.vertices {
            values.push(
                side_value(domain, current.covector, vertex).max(0.0)
                    / endpoint_scale(domain, vertex),
            );
        }
    }
    if let Plane::Spherical { .. } = domain.plane {
        for vertex in &domain.vertices {
            values.push((vertex.dot(*vertex) - 1.0).abs());
        }
        if let [first, second] = domain.vertices[..] {
            values.push(largest_coordinate(first + second));
        }
    }
    largest(values)
}

fn relators(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    match side_kinds(domain) {
        SideKinds::Mirrors => corner_relators(domain),
        SideKinds::Paired => cycle_relators(domain),
        SideKinds::Mixed => f64::INFINITY,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SideKinds {
    Mirrors,
    Paired,
    Mixed,
}

pub(crate) fn side_kinds(domain: &FundamentalDomain) -> SideKinds {
    let mirrors = domain
        .sides
        .iter()
        .filter(|side| match side.kind {
            PairingKind::Mirror => true,
            PairingKind::Rotation | PairingKind::Translation => false,
        })
        .count();
    if mirrors == domain.sides.len() {
        SideKinds::Mirrors
    } else if mirrors == 0 {
        SideKinds::Paired
    } else {
        SideKinds::Mixed
    }
}

pub(crate) struct CornerPair {
    pub(crate) corner: usize,
    pub(crate) sides: [usize; 2],
    pub(crate) order: u32,
}

pub(crate) fn corner_pairs(domain: &FundamentalDomain) -> Option<Vec<CornerPair>> {
    let count = domain.sides.len();
    if domain.corners.len() != count {
        return None;
    }
    Some(
        domain
            .corners
            .iter()
            .enumerate()
            .map(|(corner, details)| CornerPair {
                corner,
                sides: [corner, (corner + 1) % count],
                order: details.order,
            })
            .collect(),
    )
}

fn corner_relators(domain: &FundamentalDomain) -> f64 {
    let Some(pairs) = corner_pairs(domain) else {
        return f64::INFINITY;
    };
    let residuals: Option<Vec<f64>> = pairs
        .iter()
        .map(|pair| {
            let [first, second] = pair.sides.map(|side| domain.sides[side].pairing);
            let vertex = *domain.vertices.get(pair.corner)?;
            Some(framed_power_residual(
                domain.plane,
                vertex,
                &[first, second],
                pair.order,
            ))
        })
        .collect();
    residuals.map_or(f64::INFINITY, largest)
}

pub(crate) fn framed_power_residual(
    plane: Plane,
    vertex: DVec3,
    factors: &[DMat3],
    order: u32,
) -> f64 {
    let rotation = match plane {
        Plane::Hyperbolic { .. } => {
            let frame = hyperbolic::translation(vertex);
            factors
                .iter()
                .fold(hyperbolic::inverse(frame), |product, factor| {
                    product * *factor
                })
                * frame
        }
        Plane::Euclidean | Plane::Spherical { .. } => factors
            .iter()
            .fold(DMat3::IDENTITY, |product, factor| product * *factor),
    };
    power_residual(rotation, order)
}

pub(crate) fn power_residual(rotation: DMat3, order: u32) -> f64 {
    let mut power = rotation;
    for _ in 1..order {
        power *= rotation;
    }
    largest_entry(power - DMat3::IDENTITY) / f64::from(order)
}

fn cycle_relators(domain: &FundamentalDomain) -> f64 {
    let Ok(cycles) = vertex_cycles(domain) else {
        return f64::INFINITY;
    };
    let mut numeric = Vec::new();
    for cycle in cycles {
        let vertex = domain.vertices[cycle.start];
        numeric.push(
            largest_coordinate(cycle.transformation * vertex - vertex)
                / endpoint_scale(domain, vertex),
        );
        numeric.push(framed_power_residual(
            domain.plane,
            vertex,
            &[cycle.transformation],
            cycle.order,
        ));
    }
    largest(numeric)
}

pub(crate) struct Cycle {
    pub(crate) start: usize,
    pub(crate) order: u32,
    pub(crate) sides: Vec<usize>,
    pub(crate) transformation: DMat3,
}

struct Walk {
    vertices: Vec<usize>,
    sides: Vec<usize>,
    transformation: DMat3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CycleDefect {
    Open,
    Orders,
}

pub(crate) fn vertex_cycles(domain: &FundamentalDomain) -> Result<Vec<Cycle>, CycleDefect> {
    let count = domain.sides.len();
    let mut visited = vec![false; count];
    let mut cycles = Vec::new();
    for start in 0..count {
        if visited[start] {
            continue;
        }
        let walk = vertex_cycle(domain, start).ok_or(CycleDefect::Open)?;
        let order = domain.corners.get(start).ok_or(CycleDefect::Orders)?.order;
        let mut copies = 0_u32;
        for &vertex in &walk.vertices {
            visited[vertex] = true;
            let corner = domain.corners.get(vertex).ok_or(CycleDefect::Orders)?;
            if corner.order != order {
                return Err(CycleDefect::Orders);
            }
            copies = copies.saturating_add(corner.copies);
        }
        if copies != 2 {
            return Err(CycleDefect::Orders);
        }
        cycles.push(Cycle {
            start,
            order,
            sides: walk.sides,
            transformation: walk.transformation,
        });
    }
    Ok(cycles)
}

fn vertex_cycle(domain: &FundamentalDomain, start: usize) -> Option<Walk> {
    let count = domain.sides.len();
    let mut vertices = Vec::new();
    let mut sides = Vec::new();
    let mut transformation = DMat3::IDENTITY;
    let mut vertex = start;
    for _ in 0..count {
        vertices.push(vertex);
        let index = (vertex + 1) % count;
        let side = domain.sides[index];
        sides.push(index);
        transformation = side.pairing * transformation;
        vertex = side.partner;
        if vertex >= count {
            return None;
        }
        if vertex == start {
            return Some(Walk {
                vertices,
                sides,
                transformation,
            });
        }
    }
    None
}

fn angles(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    let sum_fits = domain.triangles().iter().all(|triangle| {
        let sum = triangle.iter().sum::<f64>();
        match domain.plane {
            Plane::Hyperbolic { .. } => sum < PI,
            Plane::Euclidean => true,
            Plane::Spherical { .. } => sum > PI,
        }
    });
    let polygon = domain.polygon_angles();
    let polygon_fits = polygon
        .as_deref()
        .is_none_or(|angles| angles.iter().sum::<f64>() < polygon_angle_limit(angles.len()));
    if !(sum_fits && polygon_fits) {
        return f64::INFINITY;
    }
    largest(
        polygon
            .unwrap_or_else(|| domain.angles())
            .iter()
            .zip(&domain.corners)
            .map(|(angle, corner)| {
                (angle - f64::from(corner.copies) * PI / f64::from(corner.order)).abs()
            }),
    )
}

fn area(domain: &FundamentalDomain) -> f64 {
    if !vertices_are_points(domain) {
        return f64::INFINITY;
    }
    match domain.plane {
        Plane::Hyperbolic { area } | Plane::Spherical { area } => (domain.area() - area).abs(),
        Plane::Euclidean => {
            let measured = domain.angles();
            let corners = u32::try_from(measured.len()).map_or(f64::INFINITY, f64::from);
            ((corners - 2.0) * PI - measured.iter().sum::<f64>()).abs()
        }
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;

    use astro_float_num::BigFloat;
    use escher_geometry::hyperbolic::{
        covector, distance, from_disk, normalize_point, reflection, rotation, side, wall_through,
    };
    use escher_geometry::{euclidean, spherical};
    use escher_signature::Signature;
    use glam::{DVec2, DVec3};

    use super::{ANGLE_TOLERANCE, AREA_TOLERANCE, Check, Verification, vertex_cycle};
    use crate::domain::{
        Corner, FundamentalDomain, Orientation, PairingKind, Plane, Side, triangle_angles,
    };
    use crate::double::double;
    use crate::polygon::Polygon;
    use crate::reference::{Reference, Worst, add, div, error, int, mul, num, sqrt, sub};
    use crate::shared::{built, signature};
    use crate::triangle::{Shape, Supported, construct, supported};

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
        let Some(Supported::Reflection(Shape::HyperbolicTriangle {
            orders: label_orders,
            area,
        })) = supported(&signature).ok()
        else {
            panic!("{signature} is not a supported hyperbolic triangle");
        };
        let triangle = match construct(orders) {
            Ok(triangle) => triangle,
            Err(error) => panic!("{orders:?} has no triangle: {error:?}"),
        };
        FundamentalDomain {
            plane: Plane::Hyperbolic { area },
            corners: label_orders.map(Corner::single).to_vec(),
            sides: triangle.sides.to_vec(),
            vertices: triangle.vertices.to_vec(),
            centre: triangle.centre,
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
            domain.corners
        );
        assert!(!verification.passed(), "{defect} on {:?}", domain.corners);
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
        corrupted.corners = [6, 3, 2].map(Corner::single).to_vec();
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
                domain.corners
            );
            assert!(verification.passed());
        }
    }

    #[test]
    fn verify_fails_on_the_euclidean_triangle_of_442_labelled_632() {
        let mut corrupted = built("*442");
        corrupted.corners = [6, 3, 2].map(Corner::single).to_vec();
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
        corrupted.corners = [6, 3, 2].map(Corner::single).to_vec();
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
    fn verify_fails_on_a_domain_with_a_corner_removed() {
        for text in ["*732", "*2222", "*532", "732", "2222", "22"] {
            let mut corrupted = built(text);
            corrupted.corners.pop();
            assert_infinite_failures(
                &corrupted,
                &READ_THE_COUNTS,
                &format!("{text} corner removed"),
            );
        }
    }

    #[test]
    fn verify_fails_on_a_domain_without_vertices() {
        for mut corrupted in euclidean_domains().into_iter().chain([domain([7, 3, 2])]) {
            corrupted.vertices.clear();
            assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "no vertices");
        }
    }

    fn spherical_domains() -> [FundamentalDomain; 3] {
        [built("*532"), built("*722"), built("*55")]
    }

    fn spherical(
        polygon: Result<Polygon, Option<escher_geometry::Error>>,
        orders: Vec<u32>,
        area: f64,
    ) -> FundamentalDomain {
        let polygon = match polygon {
            Ok(polygon) => polygon,
            Err(error) => panic!("{orders:?} has no spherical domain: {error:?}"),
        };
        FundamentalDomain {
            plane: Plane::Spherical { area },
            corners: orders.into_iter().map(Corner::single).collect(),
            sides: polygon.sides,
            vertices: polygon.vertices,
            centre: polygon.centre,
        }
    }

    #[test]
    fn verify_passes_on_built_spherical_domains() {
        for domain in spherical_domains() {
            let verification = domain.verify();
            assert!(
                failed(&verification).is_empty(),
                "{:?}: {verification:?}",
                domain.corners
            );
            assert!(verification.passed());
        }
    }

    #[test]
    fn verify_fails_on_a_tilted_spherical_wall_with_its_pairing() {
        let expected: [&[&str]; 3] = [
            &["pairings", "walls"],
            &["pairings", "walls"],
            &["pairings", "walls"],
        ];
        for (mut corrupted, expected) in spherical_domains().into_iter().zip(expected) {
            let side = &mut corrupted.sides[0];
            let normal = spherical::rotation(1e-7) * side.normal;
            side.normal = normal;
            side.covector = normal;
            side.pairing = spherical::reflection(normal);
            assert_failed(&corrupted, expected, "wall and pairing tilted by 1e-7");
        }
    }

    #[test]
    fn verify_fails_on_a_scaled_spherical_pairing() {
        for mut corrupted in spherical_domains() {
            corrupted.sides[0].pairing *= 1.0 + 1e-6;
            assert_failed(
                &corrupted,
                &["isometries", "pairings", "walls"],
                "scaled pairing",
            );
        }
    }

    #[test]
    fn verify_fails_on_the_spherical_triangle_of_432_labelled_532() {
        let corrupted = spherical(
            crate::spherical::triangle([4, 3, 2]),
            vec![5, 3, 2],
            PI / 30.0,
        );
        assert_failed(
            &corrupted,
            &["relators", "angles", "area"],
            "*432 labelled with the orders of *532",
        );
    }

    #[test]
    fn verify_fails_on_a_lune_of_twice_its_angle() {
        for order in [5, 12] {
            let angle = 2.0 * PI / f64::from(order);
            let corrupted = spherical(
                crate::spherical::lune_of_angle(angle),
                vec![order; 2],
                PI / f64::from(order) * 2.0,
            );
            assert_failed(
                &corrupted,
                &["angles", "area"],
                &format!("lune of the angle 2 pi / {order} labelled *{order}{order}"),
            );
        }
    }

    #[test]
    fn verify_fails_on_lune_vertices_that_are_not_antipodal_unit_vectors() {
        for vertices in [
            vec![DVec3::Y, DVec3::Y],
            vec![0.5 * DVec3::Y, -3.0 * DVec3::Y],
        ] {
            let mut corrupted = built("*55");
            corrupted.vertices = vertices;
            assert_failed(
                &corrupted,
                &["walls"],
                "lune vertices moved along their axis",
            );
        }
    }

    #[test]
    fn verify_fails_on_a_wrong_spherical_partner() {
        for mut corrupted in spherical_domains() {
            corrupted.sides[0].partner = 1;
            assert_failed(&corrupted, &["pairings"], "wrong partner");
        }
    }

    #[test]
    fn verify_fails_on_a_negated_spherical_centre() {
        for mut corrupted in spherical_domains() {
            corrupted.centre = -corrupted.centre;
            assert_failed(&corrupted, &["pairings"], "centre negated");
        }
    }

    #[test]
    fn verify_reports_infinite_residuals_for_a_zero_spherical_vertex() {
        for mut corrupted in spherical_domains() {
            corrupted.vertices[0] = DVec3::ZERO;
            assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "vertex 0 set to zero");
        }
    }

    fn kites() -> [FundamentalDomain; 4] {
        [built("732"), built("442"), built("532"), built("722")]
    }

    fn doubles() -> Vec<FundamentalDomain> {
        let mut doubles = kites().to_vec();
        doubles.extend([built("2222"), built("55")]);
        doubles
    }

    fn labelled_kite(orders: [u32; 3], label: &str) -> FundamentalDomain {
        let Some(Supported::Rotation(shape)) = supported(&signature(label)).ok() else {
            panic!("{label} is not a rotation signature");
        };
        let (plane, polygon) = match shape {
            Shape::HyperbolicTriangle { area, .. } => {
                let triangle = match construct(orders) {
                    Ok(triangle) => triangle,
                    Err(error) => panic!("{orders:?} has no triangle: {error:?}"),
                };
                let polygon = Polygon {
                    sides: triangle.sides.to_vec(),
                    vertices: triangle.vertices.to_vec(),
                    centre: triangle.centre,
                };
                (Plane::Hyperbolic { area }, polygon)
            }
            Shape::SphericalTriangle { area, .. } => match crate::spherical::triangle(orders) {
                Ok(polygon) => (Plane::Spherical { area }, polygon),
                Err(error) => panic!("{orders:?} has no triangle: {error:?}"),
            },
            Shape::HyperbolicPolygon { .. }
            | Shape::EuclideanTriangle { .. }
            | Shape::Rectangle
            | Shape::Lune { .. } => {
                panic!("{label} is not a kite")
            }
        };
        let generating = FundamentalDomain {
            plane,
            corners: shape.orders().into_iter().map(Corner::single).collect(),
            sides: polygon.sides,
            vertices: polygon.vertices,
            centre: polygon.centre,
        };
        match double(&generating, plane) {
            Ok(double) => double,
            Err(error) => panic!("{orders:?} has no double: {error}"),
        }
    }

    #[test]
    fn verify_passes_on_built_doubles() {
        for double in doubles() {
            let verification = double.verify();
            assert!(
                failed(&verification).is_empty(),
                "{:?}: {verification:?}",
                double.corners
            );
        }
    }

    #[test]
    fn walk_finds_the_vertex_cycles_of_each_double() {
        let kite_cycles = |[first, second, third]: [u32; 3]| {
            vec![(vec![0], first), (vec![1, 3], third), (vec![2], second)]
        };
        let cases = [
            ("732", kite_cycles([7, 3, 2])),
            ("542", kite_cycles([5, 4, 2])),
            ("444", kite_cycles([4, 4, 4])),
            ("433", kite_cycles([4, 3, 3])),
            ("(12)32", kite_cycles([12, 3, 2])),
            ("632", kite_cycles([6, 3, 2])),
            ("442", kite_cycles([4, 4, 2])),
            ("333", kite_cycles([3, 3, 3])),
            ("332", kite_cycles([3, 3, 2])),
            ("432", kite_cycles([4, 3, 2])),
            ("532", kite_cycles([5, 3, 2])),
            ("222", kite_cycles([2, 2, 2])),
            ("722", kite_cycles([7, 2, 2])),
            (
                "2222",
                vec![(vec![0], 2), (vec![1, 5], 2), (vec![2, 4], 2), (vec![3], 2)],
            ),
            ("22", vec![(vec![0], 2), (vec![1], 2)]),
            ("55", vec![(vec![0], 5), (vec![1], 5)]),
        ];
        for (text, expected) in cases {
            let double = built(text);
            let mut found = Vec::new();
            let mut visited = vec![false; double.sides.len()];
            for start in 0..double.sides.len() {
                if visited[start] {
                    continue;
                }
                let Some(cycle) = vertex_cycle(&double, start) else {
                    panic!("{text}: the walk from vertex {start} does not return");
                };
                let mut vertices = cycle.vertices;
                for &vertex in &vertices {
                    visited[vertex] = true;
                }
                vertices.sort_unstable();
                found.push((vertices, double.corners[start].order));
            }
            assert_eq!(found, expected, "{text}");
        }
    }

    #[test]
    fn verify_fails_on_a_double_side_that_is_its_own_partner() {
        for mut corrupted in kites() {
            corrupted.sides[0].partner = 0;
            assert_infinite_failures(&corrupted, &["pairings", "relators"], "own partner");
        }
    }

    #[test]
    fn verify_fails_on_a_double_side_labelled_as_a_mirror() {
        for mut corrupted in kites() {
            corrupted.sides[0].kind = PairingKind::Mirror;
            assert_failed(
                &corrupted,
                &["pairings", "walls", "relators"],
                "side 0 labelled as a mirror",
            );
            assert_eq!(corrupted.verify().relators().residual(), f64::INFINITY);
        }
    }

    #[test]
    fn verify_fails_on_an_inverted_double_pairing() {
        for mut corrupted in kites() {
            corrupted.sides[0].pairing = corrupted.sides[0].pairing.inverse();
            assert_failed(&corrupted, &["pairings", "relators"], "inverted pairing");
        }
    }

    #[test]
    fn verify_fails_on_an_image_side_that_was_not_reflected() {
        for text in ["732", "442", "532", "722"] {
            let mut corrupted = built(text);
            let generating = built(&format!("*{text}"));
            corrupted.sides[1].normal = generating.sides[0].normal;
            corrupted.sides[1].covector = generating.sides[0].covector;
            assert_failed(
                &corrupted,
                &["pairings", "walls"],
                "image side not reflected",
            );
            assert_eq!(corrupted.verify().pairings().residual(), f64::INFINITY);
        }
    }

    #[test]
    fn verify_fails_on_a_kite_with_the_wrong_signature() {
        let cases: [([u32; 3], &str); 5] = [
            ([8, 3, 2], "732"),
            ([7, 4, 2], "732"),
            ([7, 3, 3], "732"),
            ([4, 3, 2], "532"),
            ([5, 2, 2], "532"),
        ];
        for (orders, label) in cases {
            assert_failed(
                &labelled_kite(orders, label),
                &["relators", "angles", "area"],
                &format!("kite of {orders:?} labelled {label}"),
            );
        }
    }

    #[test]
    fn verify_fails_on_a_double_with_wrong_copies() {
        for mut corrupted in kites().into_iter().chain([built("55")]) {
            corrupted.corners[0].copies = 1;
            assert_failed(&corrupted, &["relators", "angles"], "copies of W0 set to 1");
            assert_eq!(corrupted.verify().relators().residual(), f64::INFINITY);
        }
    }

    #[test]
    fn verify_fails_on_split_halves_merged_into_a_side_that_is_its_own_partner() {
        for (text, flat) in [("722", 2), ("2222", 3)] {
            let mut corrupted = built(text);
            let merged = corrupted.sides[flat + 1];
            corrupted.sides.remove(flat + 1);
            corrupted.sides[flat] = Side {
                partner: flat,
                ..merged
            };
            corrupted.vertices.remove(flat);
            corrupted.corners.remove(flat);
            let count = corrupted.sides.len();
            for side in &mut corrupted.sides {
                if side.partner > flat + 1 && side.partner < count + 1 {
                    side.partner -= 1;
                }
            }
            assert_failed(&corrupted, &["pairings"], "split halves merged");
        }
    }

    #[test]
    fn verify_fails_on_a_half_turn_in_place_of_a_translation() {
        let mut corrupted = built("2222");
        let midpoint = (corrupted.vertices[1] + corrupted.vertices[2]) / 2.0;
        let half_turn = glam::DMat3::from_cols(
            -DVec3::X,
            -DVec3::Y,
            DVec3::new(2.0 * midpoint.x, 2.0 * midpoint.y, 1.0),
        );
        corrupted.sides[2].pairing = half_turn;
        corrupted.sides[5].pairing = half_turn;
        assert_failed(&corrupted, &["pairings", "relators"], "half-turn of side 2");
    }

    #[test]
    fn verify_fails_on_a_flat_vertex_moved_along_its_wall_outside_another_wall() {
        let mut corrupted = built("2222");
        let moved = corrupted.vertices[4] - DVec3::new(0.5, 0.0, 0.0);
        corrupted.vertices[3] = moved;
        let verification = corrupted.verify();
        let checks = [
            verification.isometries(),
            verification.pairings(),
            verification.walls(),
            verification.relators(),
            verification.angles(),
            verification.area(),
        ];
        for (name, check) in NAMES.into_iter().zip(checks) {
            println!(
                "{name}: residual {:e}, tolerance {:e}",
                check.residual(),
                check.tolerance()
            );
            let clear = if check.passed() {
                8.0 * check.residual() <= check.tolerance()
            } else {
                check.residual() >= 8.0 * check.tolerance()
            };
            assert!(clear, "{name}: {check:?}");
        }
        assert_failed(
            &corrupted,
            &["pairings", "walls", "relators", "angles", "area"],
            "flat vertex 3 moved past vertex 4 along its wall",
        );
    }

    #[test]
    fn verify_fails_on_a_doubled_lune_paired_by_the_half_turn_about_its_vertices() {
        let mut corrupted = built("44");
        let half_turn = glam::DMat3::from_diagonal(DVec3::new(-1.0, 1.0, -1.0));
        for side in &mut corrupted.sides {
            side.pairing = half_turn;
        }
        assert_failed(&corrupted, &["pairings"], "half-turn about the vertices");
        println!("44 with the half-turn: {:?}", corrupted.verify().pairings());
    }

    #[test]
    fn verify_fails_on_a_kite_paired_by_the_half_turn_about_vertex_0() {
        let mut corrupted = built("732");
        let frame = escher_geometry::hyperbolic::translation(corrupted.vertices[0]);
        let half_turn = frame
            * glam::DMat3::from_diagonal(DVec3::new(-1.0, -1.0, 1.0))
            * escher_geometry::hyperbolic::inverse(frame);
        corrupted.sides[0].pairing = half_turn;
        corrupted.sides[1].pairing = half_turn;
        assert_failed(
            &corrupted,
            &["pairings", "relators"],
            "half-turn about vertex 0",
        );
        println!(
            "732 with the half-turn: {:?}",
            corrupted.verify().pairings()
        );
    }

    #[test]
    fn verify_fails_on_a_pairing_labelled_with_the_wrong_kind() {
        let mut rectangle = built("2222");
        for index in [2, 5] {
            rectangle.sides[index].kind = PairingKind::Rotation;
        }
        let mut kite = built("442");
        for index in [0, 1] {
            kite.sides[index].kind = PairingKind::Translation;
        }
        for corrupted in [rectangle, kite] {
            assert_infinite_failures(&corrupted, &["pairings"], "wrong kind");
        }
    }

    #[test]
    fn verify_reports_infinite_residuals_for_a_nan_double_vertex() {
        for mut corrupted in doubles() {
            corrupted.vertices[1].x = f64::NAN;
            assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "NaN vertex");
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

    const REGULAR_CORNERS: [u32; 4] = [5, 6, 7, 8];

    const REGULAR_ORDERS: [u32; 6] = [2, 3, 4, 7, 12, 1000];

    const POLYGON_ANGLE_BOUND: f64 = 1e-14;

    const POLYGON_AREA_BOUND: f64 = 4e-14;

    const DOUBLE_ANGLE_BOUND: f64 = 4e-12;

    const DOUBLE_AREA_BOUND: f64 = 1e-11;

    const FLAT_VERTEX_BOUND: f64 = 4e-14;

    const REGULAR_POLYGON_VERTICES: usize = 156;

    const REGULAR_DOUBLE_VERTICES: usize = 264;

    fn size(corners: u32) -> usize {
        match usize::try_from(corners) {
            Ok(size) => size,
            Err(error) => panic!("{corners} corners do not fit in memory: {error}"),
        }
    }

    fn polygon_from_vertices(vertices: Vec<DVec3>, orders: &[u32], area: f64) -> FundamentalDomain {
        let centre = match normalize_point(vertices.iter().copied().sum()) {
            Ok(centre) => centre,
            Err(error) => panic!("the vertices have no centre: {error}"),
        };
        let count = vertices.len();
        let sides = (0..count)
            .map(|index| {
                let (start, end) = (vertices[(index + count - 1) % count], vertices[index]);
                let wall = match wall_through(start, end) {
                    Ok(wall) => wall,
                    Err(error) => panic!("side {index} has no wall: {error}"),
                };
                let normal = if side(covector(wall), centre) > 0.0 {
                    -wall
                } else {
                    wall
                };
                Side {
                    normal,
                    covector: covector(normal),
                    pairing: reflection(normal),
                    partner: index,
                    kind: PairingKind::Mirror,
                    orientation: Orientation::Reversing,
                }
            })
            .collect();
        FundamentalDomain {
            plane: Plane::Hyperbolic { area },
            corners: orders.iter().copied().map(Corner::single).collect(),
            sides,
            vertices,
            centre,
        }
    }

    fn polygon_area(orders: &[u32]) -> f64 {
        orders
            .iter()
            .fold(-2.0 * PI, |area, order| area + PI - PI / f64::from(*order))
    }

    fn regular(corners: u32, order: u32) -> FundamentalDomain {
        let (half_centre, half_corner) = (PI / f64::from(corners), PI / f64::from(2 * order));
        let radius = ((half_centre + half_corner).cos() / (half_centre - half_corner).cos()).sqrt();
        let vertices = (0..corners)
            .map(|index| {
                let turn = 2.0 * PI * f64::from(index) / f64::from(corners);
                match from_disk(DVec2::from_angle(turn) * radius) {
                    Some(vertex) => vertex,
                    None => panic!("vertex {index} of {corners} corners lies outside the disk"),
                }
            })
            .collect();
        let orders = vec![order; size(corners)];
        polygon_from_vertices(vertices, &orders, polygon_area(&orders))
    }

    struct ExactPolygon {
        domain: FundamentalDomain,
        angle: BigFloat,
        area: BigFloat,
    }

    fn exact_regular(reference: &mut Reference, corners: u32, order: u32) -> ExactPolygon {
        let pi = reference.pi();
        let half_centre = div(&pi, &int(corners));
        let angle = div(&pi, &int(order));
        let half_corner = div(&angle, &int(2));
        let radius = sqrt(&div(
            &reference.cos(&add(&half_centre, &half_corner)),
            &reference.cos(&sub(&half_centre, &half_corner)),
        ));
        let square = mul(&radius, &radius);
        let scale = div(&int(1), &sub(&int(1), &square));
        let twice_radius = mul(&mul(&int(2), &radius), &scale);
        let height = mul(&add(&int(1), &square), &scale);
        let vertices = (0..corners)
            .map(|index| {
                let turn = div(&mul(&mul(&int(2), &pi), &int(index)), &int(corners));
                let x = mul(&twice_radius, &reference.cos(&turn));
                let y = mul(&twice_radius, &reference.sin(&turn));
                DVec3::new(
                    reference.approximate(&x),
                    reference.approximate(&y),
                    reference.approximate(&height),
                )
            })
            .collect();
        let area = sub(&mul(&int(corners - 2), &pi), &mul(&int(corners), &angle));
        let orders = vec![order; size(corners)];
        let domain = polygon_from_vertices(vertices, &orders, reference.approximate(&area));
        ExactPolygon {
            domain,
            angle,
            area,
        }
    }

    fn doubled(polygon: &FundamentalDomain) -> FundamentalDomain {
        let Plane::Hyperbolic { area } = polygon.plane else {
            panic!("the polygon is not hyperbolic");
        };
        match double(polygon, Plane::Hyperbolic { area: 2.0 * area }) {
            Ok(domain) => domain,
            Err(error) => panic!("the polygon has no double: {error}"),
        }
    }

    fn regular_name(corners: u32, order: u32) -> String {
        let corner = if order < 10 {
            order.to_string()
        } else {
            format!("({order})")
        };
        format!("*{}", corner.repeat(size(corners)))
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
    fn regular_polygons_have_the_reference_angles_and_area() {
        let mut reference = Reference::new();
        let count = REGULAR_CORNERS.len() * REGULAR_ORDERS.len();
        let mut angles = Worst::new(
            "regular polygon corner angle, absolute error",
            REGULAR_POLYGON_VERTICES,
        );
        let mut areas = Worst::new("regular polygon area, absolute error", count);
        let mut check_5 = Worst::new("regular polygon residual of check 5", count);
        let mut check_6 = Worst::new("regular polygon residual of check 6", count);
        for corners in REGULAR_CORNERS {
            for order in REGULAR_ORDERS {
                let name = regular_name(corners, order);
                let exact = exact_regular(&mut reference, corners, order);
                let measured = exact.domain.corner_angles();
                assert_eq!(measured.len(), size(corners), "{name}");
                for (vertex, angle) in measured.into_iter().enumerate() {
                    angles.record(&error(angle, &exact.angle), &int(1), || {
                        format!("{name} vertex {vertex}")
                    });
                }
                areas.record(&error(exact.domain.area(), &exact.area), &int(1), || {
                    name.clone()
                });
                let verification = exact.domain.verify();
                assert!(failed(&verification).is_empty(), "{name}: {verification:?}");
                check_5.record(&num(verification.angles().residual()), &int(1), || {
                    name.clone()
                });
                check_6.record(&num(verification.area().residual()), &int(1), || {
                    name.clone()
                });
            }
        }
        angles.assert_at_most(&mut reference, POLYGON_ANGLE_BOUND);
        areas.assert_at_most(&mut reference, POLYGON_AREA_BOUND);
        check_5.assert_at_most(&mut reference, ANGLE_TOLERANCE);
        check_6.assert_at_most(&mut reference, AREA_TOLERANCE);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
    fn doubles_of_regular_polygons_have_the_reference_angles_and_area() {
        let mut reference = Reference::new();
        let count = REGULAR_CORNERS.len() * REGULAR_ORDERS.len();
        let mut angles = Worst::new(
            "doubled polygon corner angle, absolute error",
            REGULAR_DOUBLE_VERTICES,
        );
        let mut areas = Worst::new("doubled polygon area, absolute error", count);
        let mut check_5 = Worst::new("doubled polygon residual of check 5", count);
        let mut check_6 = Worst::new("doubled polygon residual of check 6", count);
        for corners in REGULAR_CORNERS {
            for order in REGULAR_ORDERS {
                let name = regular_name(corners, order);
                let exact = exact_regular(&mut reference, corners, order);
                let domain = doubled(&exact.domain);
                let measured = domain.corner_angles();
                assert_eq!(measured.len(), 2 * size(corners) - 2, "{name}");
                for (vertex, (angle, corner)) in
                    measured.into_iter().zip(&domain.corners).enumerate()
                {
                    let expected = mul(&int(corner.copies), &exact.angle);
                    angles.record(&error(angle, &expected), &int(1), || {
                        format!("double of {name} vertex {vertex}")
                    });
                }
                areas.record(
                    &error(domain.area(), &mul(&int(2), &exact.area)),
                    &int(1),
                    || format!("double of {name}"),
                );
                let verification = domain.verify();
                let failures = failed(&verification);
                if order == 1000 {
                    assert!(
                        failures.iter().all(|check| *check == "relators"),
                        "double of {name}: {verification:?}"
                    );
                    if !failures.is_empty() {
                        println!("double of {name} fails: {verification:?}");
                    }
                } else {
                    assert!(failures.is_empty(), "double of {name}: {verification:?}");
                }
                check_5.record(&num(verification.angles().residual()), &int(1), || {
                    format!("double of {name}")
                });
                check_6.record(&num(verification.area().residual()), &int(1), || {
                    format!("double of {name}")
                });
            }
        }
        angles.assert_at_most(&mut reference, DOUBLE_ANGLE_BOUND);
        areas.assert_at_most(&mut reference, DOUBLE_AREA_BOUND);
        check_5.assert_at_most(&mut reference, ANGLE_TOLERANCE);
        check_6.assert_at_most(&mut reference, AREA_TOLERANCE);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
    fn frame_angle_at_a_flat_vertex_is_pi() {
        let mut reference = Reference::new();
        let pi = reference.pi();
        let mut framed = Worst::new("flat vertex in its frame, absolute error", 8);
        let mut arccosine = (0.0, String::new());
        for corners in REGULAR_CORNERS {
            let name = regular_name(corners, 2);
            let domain = doubled(&exact_regular(&mut reference, corners, 2).domain);
            let count = domain.vertices.len();
            let angles = domain.corner_angles();
            for vertex in [0, size(corners) - 1] {
                let place = || format!("double of {name} vertex {vertex}");
                assert_eq!(domain.corners[vertex].copies, 2, "{}", place());
                framed.record(&error(angles[vertex], &pi), &int(1), place);
                let corner = [
                    domain.vertices[vertex],
                    domain.vertices[(vertex + 1) % count],
                    domain.vertices[(vertex + count - 1) % count],
                ];
                let Some([plain, _, _]) = triangle_angles(domain.plane, corner) else {
                    panic!("{} has no law of cosines", place());
                };
                let plain_error = reference.approximate(&error(plain, &pi));
                if plain_error > arccosine.0 {
                    arccosine = (plain_error, place());
                }
            }
        }
        framed.assert_at_most(&mut reference, FLAT_VERTEX_BOUND);
        println!(
            "flat vertex by the law of cosines, absolute error: largest error {:.3e}, at {}",
            arccosine.0, arccosine.1
        );
    }

    #[test]
    fn verify_reports_an_infinite_angle_residual_for_a_pentagon_with_the_angle_sum_three_pi() {
        let vertices = [0.0, 0.1, 0.2, 0.3, 0.4]
            .map(|x| match from_disk(DVec2::new(x, 0.0)) {
                Some(vertex) => vertex,
                None => panic!("{x} lies outside the disk"),
            })
            .to_vec();
        let degenerate = polygon_from_vertices(vertices, &[3; 5], polygon_area(&[3; 5]));
        let angles = degenerate.corner_angles();
        assert_eq!(angles, [0.0, PI, PI, PI, 0.0]);
        assert_eq!(degenerate.verify().angles().residual(), f64::INFINITY);
    }

    fn pentagons_and_hexagon() -> [FundamentalDomain; 3] {
        [regular(5, 3), regular(5, 2), regular(6, 4)]
    }

    #[test]
    fn verify_passes_on_regular_polygons_and_their_doubles() {
        for polygon in pentagons_and_hexagon() {
            for domain in [doubled(&polygon), polygon] {
                let verification = domain.verify();
                assert!(
                    failed(&verification).is_empty(),
                    "{:?}: {verification:?}",
                    domain.corners
                );
            }
        }
    }

    #[test]
    fn verify_fails_on_a_regular_polygon_with_a_corner_order_off_by_one() {
        for polygon in [regular(5, 3), regular(6, 4)] {
            for corner in 0..polygon.corners.len() {
                let mut corrupted = polygon.clone();
                corrupted.corners[corner].order += 1;
                let orders: Vec<u32> = corrupted
                    .corners
                    .iter()
                    .map(|corner| corner.order)
                    .collect();
                corrupted.plane = Plane::Hyperbolic {
                    area: polygon_area(&orders),
                };
                let defect = format!("order of corner {corner} raised by 1");
                assert_failed(&corrupted, &["relators", "angles", "area"], &defect);
                println!("{defect} on {orders:?}: {:?}", corrupted.verify());
            }
        }
    }

    #[test]
    fn verify_fails_on_a_regular_polygon_with_a_vertex_moved_toward_the_centre() {
        let step = 1e-3;
        for mut corrupted in pentagons_and_hexagon() {
            let (vertex, centre) = (corrupted.vertices[2], corrupted.centre);
            let length = distance(vertex, centre);
            corrupted.vertices[2] =
                ((length - step).sinh() * vertex + step.sinh() * centre) / length.sinh();
            assert_failed(
                &corrupted,
                &["pairings", "walls", "angles", "area"],
                "vertex 2 moved toward the centre",
            );
            println!(
                "vertex 2 moved toward the centre on {:?}: {:?}",
                corrupted.corners,
                corrupted.verify()
            );
        }
    }

    #[test]
    fn verify_fails_on_a_regular_polygon_with_its_last_vertex_removed() {
        for mut corrupted in pentagons_and_hexagon() {
            corrupted.vertices.pop();
            assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "last vertex removed");
        }
    }

    #[test]
    fn verify_reports_infinite_residuals_for_a_nan_polygon_vertex() {
        for mut corrupted in pentagons_and_hexagon() {
            corrupted.vertices[1].x = f64::NAN;
            assert_infinite_failures(&corrupted, &READ_THE_COUNTS, "NaN vertex");
        }
    }
}
