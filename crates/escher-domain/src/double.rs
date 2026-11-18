use escher_geometry::hyperbolic;
use glam::{DMat3, DVec3};

use crate::domain::{Corner, FundamentalDomain, Orientation, PairingKind, Plane, Side};
use crate::error::{Error, Result};
use crate::verify::is_translation;

pub(crate) fn double(generating: &FundamentalDomain, plane: Plane) -> Result<FundamentalDomain> {
    let count = generating.vertices.len();
    let degenerate =
        count < 2 || generating.sides.len() != count || generating.corners.len() != count;
    if degenerate {
        return Err(Error::numerical(None));
    }
    let (vertices, corners) = doubled_vertices(generating);
    let wall = generating.sides[1].pairing;
    let total = vertices.len();
    let sides: Vec<Side> = (0..total)
        .map(|index| doubled_side(generating, plane, wall, index))
        .collect();
    let centre = generating.centre;
    let finite = vertices.iter().all(|vertex| vertex.is_finite())
        && centre.is_finite()
        && sides.iter().all(|side| {
            side.normal.is_finite() && side.covector.is_finite() && side.pairing.is_finite()
        });
    let inside = sides
        .iter()
        .all(|side| plane.side(side.covector, centre) < 0.0);
    if !(finite && inside) {
        return Err(Error::numerical(None));
    }
    Ok(FundamentalDomain {
        plane,
        corners,
        sides,
        vertices,
        centre,
    })
}

fn doubled_vertices(generating: &FundamentalDomain) -> (Vec<DVec3>, Vec<Corner>) {
    let count = generating.vertices.len();
    let (vertices, orders) = (&generating.vertices, &generating.corners);
    let wall = generating.sides[1].pairing;
    let mut doubled = Vec::with_capacity(2 * count - 2);
    let mut corners = Vec::with_capacity(2 * count - 2);
    doubled.push(vertices[0]);
    corners.push(Corner {
        order: orders[0].order,
        copies: 2,
    });
    for index in 1..count - 1 {
        doubled.push(wall * vertices[count - index]);
        corners.push(Corner::single(orders[count - index].order));
    }
    doubled.push(vertices[1]);
    corners.push(Corner {
        order: orders[1].order,
        copies: 2,
    });
    for index in 1..count - 1 {
        doubled.push(vertices[1 + index]);
        corners.push(Corner::single(orders[1 + index].order));
    }
    (doubled, corners)
}

fn doubled_side(generating: &FundamentalDomain, plane: Plane, wall: DMat3, index: usize) -> Side {
    let count = generating.vertices.len();
    let total = 2 * count - 2;
    let (original, image) = if index == 0 {
        (0, false)
    } else if index < count {
        ((count + 1 - index) % count, true)
    } else {
        (index + 2 - count, false)
    };
    let source = generating.sides[original];
    let (normal, covector, pairing) = if image {
        let (normal, covector) = match plane {
            Plane::Hyperbolic { .. } => {
                let normal = wall * source.normal;
                (normal, hyperbolic::covector(normal))
            }
            Plane::Euclidean => {
                let covector = wall.transpose() * source.covector;
                (DVec3::new(covector.x, covector.y, 0.0), covector)
            }
            Plane::Spherical { .. } => {
                let normal = wall * source.normal;
                (normal, normal)
            }
        };
        (normal, covector, product(source.pairing, wall))
    } else {
        (
            source.normal,
            source.covector,
            product(wall, source.pairing),
        )
    };
    let translation = match plane {
        Plane::Hyperbolic { .. } => original != 0 && original != 2,
        Plane::Euclidean | Plane::Spherical { .. } => is_translation(plane, pairing),
    };
    Side {
        normal,
        covector,
        pairing,
        partner: (2 * count - 1 - index) % total,
        kind: if translation {
            PairingKind::Translation
        } else {
            PairingKind::Rotation
        },
        orientation: Orientation::Preserving,
    }
}

fn product(left: DMat3, right: DMat3) -> DMat3 {
    let entry = |row: usize, column: usize| {
        accurate_dot(left.row(row).to_array(), right.col(column).to_array())
    };
    DMat3::from_cols(
        DVec3::new(entry(0, 0), entry(1, 0), entry(2, 0)),
        DVec3::new(entry(0, 1), entry(1, 1), entry(2, 1)),
        DVec3::new(entry(0, 2), entry(1, 2), entry(2, 2)),
    )
}

fn accurate_dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    let (mut sum, mut error) = two_product(left[0], right[0]);
    for (first, second) in left.into_iter().zip(right).skip(1) {
        let (product, product_error) = two_product(first, second);
        let (next, sum_error) = two_sum(sum, product);
        sum = next;
        error += sum_error + product_error;
    }
    sum + error
}

pub(crate) fn two_product(left: f64, right: f64) -> (f64, f64) {
    let product = left * right;
    (product, left.mul_add(right, -product))
}

pub(crate) fn two_sum(left: f64, right: f64) -> (f64, f64) {
    let sum = left + right;
    let part = sum - left;
    (sum, (left - (sum - part)) + (right - part))
}

#[cfg(test)]
mod tests {
    use glam::{DMat3, DVec3};

    use super::{accurate_dot, product, two_product};

    const TWO_TO_THE_MINUS_30: f64 = 1.0 / 1_073_741_824.0;

    const TWO_TO_THE_MINUS_60: f64 = TWO_TO_THE_MINUS_30 * TWO_TO_THE_MINUS_30;

    #[test]
    fn two_product_gives_the_exact_rounding_error() {
        let near_one = 1.0 + TWO_TO_THE_MINUS_30;
        let (product, error) = two_product(near_one, near_one);
        assert_eq!(product, 1.0 + 2.0 * TWO_TO_THE_MINUS_30);
        assert_eq!(error, TWO_TO_THE_MINUS_60);
    }

    #[test]
    fn accurate_dot_keeps_a_sum_that_plain_arithmetic_cancels() {
        let left = [1.0, TWO_TO_THE_MINUS_60, -1.0];
        let right = [1.0, 1.0, 1.0];
        assert_eq!(accurate_dot(left, right), TWO_TO_THE_MINUS_60);
        let plain = left[0] * right[0] + left[1] * right[1] + left[2] * right[2];
        assert_eq!(plain, 0.0);
    }

    #[test]
    fn product_of_exact_matrices_is_the_exact_product() {
        let left = DMat3::from_cols(
            DVec3::new(1.0, 2.0, 3.0),
            DVec3::new(4.0, 5.0, 6.0),
            DVec3::new(7.0, 8.0, 9.0),
        );
        let right = DMat3::from_cols(
            DVec3::new(-1.0, 0.5, 2.0),
            DVec3::new(0.25, 3.0, -2.0),
            DVec3::new(1.0, 1.0, 1.0),
        );
        assert_eq!(product(left, right), left * right);
    }
}
