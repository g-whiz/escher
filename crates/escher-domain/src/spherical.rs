use core::f64::consts::PI;

use escher_geometry::spherical::{
    ORIGIN, boost, inverse, normalize_point, reflection, rotation, side, translation, wall_through,
};
use glam::{DMat3, DVec3};

use crate::domain::{Closure, FundamentalDomain, Orientation, PairingKind, Plane, Side};
use crate::error::{Error, Result as DomainResult};
use crate::kite::{MirrorPair, on_kite};
use crate::polygon::Polygon;
use crate::triangle::SphericalGroup;

pub(crate) fn triangle(orders: [u32; 3]) -> Result<Polygon, Option<escher_geometry::Error>> {
    let angles = orders.map(|order| PI / f64::from(order));
    let [(sin_0, cos_0), (sin_1, cos_1), (sin_2, cos_2)] = angles.map(f64::sin_cos);
    let first_cos = (cos_1 + cos_2 * cos_0) / (sin_2 * sin_0);
    let second_cos = (cos_0 + cos_2 * cos_1) / (sin_2 * sin_1);
    let first_length = first_cos.clamp(-1.0, 1.0).acos();
    let second_length = second_cos.clamp(-1.0, 1.0).acos();

    let placed = [
        boost(first_length) * ORIGIN,
        rotation(angles[2]) * (boost(second_length) * ORIGIN),
        ORIGIN,
    ];
    let target = normalize_point(placed[0] + placed[1] + placed[2]).map_err(Some)?;
    let move_to_origin = inverse(translation(target));
    let vertices = placed.map(|vertex| move_to_origin * vertex);

    let mut sides = Vec::with_capacity(3);
    for index in 0..3 {
        let opposite = vertices[(index + 1) % 3];
        let mut normal = wall_through(vertices[(index + 2) % 3], vertices[index]).map_err(Some)?;
        if side(normal, opposite) > 0.0 {
            normal = -normal;
        }
        let inside = side(normal, opposite) < 0.0;
        if !inside {
            return Err(None);
        }
        sides.push(Side::spherical_mirror(normal, index));
    }
    let centre = normalize_point(vertices[0] + vertices[1] + vertices[2]).map_err(Some)?;
    finished(sides, vertices.to_vec(), centre)
}

pub(crate) fn lune(order: u32) -> Result<Polygon, Option<escher_geometry::Error>> {
    if order == 1 {
        return lune_of_normals([DVec3::NEG_Z, DVec3::NEG_Z]);
    }
    lune_of_angle(PI / f64::from(order))
}

pub(crate) fn lune_of_angle(angle: f64) -> Result<Polygon, Option<escher_geometry::Error>> {
    let (sin, cos) = (angle / 2.0).sin_cos();
    lune_of_normals([DVec3::new(cos, 0.0, -sin), DVec3::new(-cos, 0.0, -sin)])
}

fn lune_of_normals(normals: [DVec3; 2]) -> Result<Polygon, Option<escher_geometry::Error>> {
    let [first, second] = normals;
    let sides = vec![
        Side::spherical_mirror(first, 0),
        Side::spherical_mirror(second, 1),
    ];
    let inside = sides.iter().all(|wall| side(wall.normal, ORIGIN) < 0.0);
    if !inside {
        return Err(None);
    }
    finished(sides, vec![DVec3::Y, DVec3::NEG_Y], ORIGIN)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Base {
    Kite { orders: [u32; 3], pair: MirrorPair },
    MirrorLune,
    RotaryLune(u32),
    WholeSphere,
}

pub(crate) const fn base(group: SphericalGroup) -> Base {
    match group {
        SphericalGroup::ThreeMirrorTwo => Base::Kite {
            orders: [4, 3, 2],
            pair: MirrorPair::Low,
        },
        SphericalGroup::TwoMirrorN(order) => Base::Kite {
            orders: [order.saturating_mul(2), 2, 2],
            pair: MirrorPair::Low,
        },
        SphericalGroup::ConeMirror(order) => Base::Kite {
            orders: [order, 2, 2],
            pair: MirrorPair::High,
        },
        SphericalGroup::ConeCrossCap(order) => Base::RotaryLune(order),
        SphericalGroup::CrossCap => Base::RotaryLune(1),
        SphericalGroup::Mirror => Base::MirrorLune,
        SphericalGroup::Trivial => Base::WholeSphere,
    }
}

pub(crate) fn construct(
    doubled: &FundamentalDomain,
    pair: MirrorPair,
) -> DomainResult<FundamentalDomain> {
    on_kite(doubled, pair).map_err(Error::numerical)?.sound()
}

pub(crate) fn rotary_lune(lune: FundamentalDomain) -> DomainResult<FundamentalDomain> {
    let sides = lune
        .sides
        .iter()
        .enumerate()
        .map(|(index, side)| Side {
            pairing: rotated_half_way(side.pairing),
            partner: index ^ 1,
            kind: PairingKind::RotaryReflection,
            orientation: Orientation::Reversing,
            ..*side
        })
        .collect();
    FundamentalDomain {
        sides,
        closure: Closure::Cycles,
        ..lune
    }
    .sound()
}

fn rotated_half_way(reflection: DMat3) -> DMat3 {
    let half_turn = |column: DVec3| DVec3::new(-column.x, -column.y, column.z);
    DMat3::from_cols(
        half_turn(reflection.x_axis),
        half_turn(reflection.y_axis),
        half_turn(reflection.z_axis),
    )
}

pub(crate) fn whole_sphere(area: f64) -> DomainResult<FundamentalDomain> {
    FundamentalDomain {
        plane: Plane::Spherical { area },
        corners: Vec::new(),
        sides: Vec::new(),
        vertices: Vec::new(),
        centre: ORIGIN,
        closure: Closure::PerVertex,
    }
    .sound()
}

fn finished(
    sides: Vec<Side>,
    vertices: Vec<DVec3>,
    centre: DVec3,
) -> Result<Polygon, Option<escher_geometry::Error>> {
    let finite = vertices.iter().all(|vertex| vertex.is_finite())
        && centre.is_finite()
        && sides.iter().all(|side| {
            side.normal.is_finite() && side.covector.is_finite() && side.pairing.is_finite()
        });
    if finite {
        Ok(Polygon {
            sides,
            vertices,
            centre,
        })
    } else {
        Err(None)
    }
}

impl Side {
    pub(crate) fn spherical_mirror(normal: DVec3, index: usize) -> Self {
        Self {
            normal,
            covector: normal,
            pairing: reflection(normal),
            partner: index,
            kind: PairingKind::Mirror,
            orientation: Orientation::Reversing,
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::{DMat3, DVec3};

    use crate::shared::{built, order_text};

    const CLOSING_BOUND: f64 = 4e-15;

    const HALF_TURN: DMat3 = DMat3::from_cols(DVec3::NEG_X, DVec3::NEG_Y, DVec3::Z);

    fn bits(vector: DVec3) -> [u64; 3] {
        vector.to_array().map(f64::to_bits)
    }

    fn largest_entry(matrix: DMat3) -> f64 {
        matrix
            .to_cols_array()
            .into_iter()
            .fold(0.0_f64, |largest, entry| largest.max(entry.abs()))
    }

    #[test]
    fn lune_of_order_one_has_exact_normals_and_pairings() {
        let mirror = built("*");
        let cross_cap = built("x");
        for domain in [&mirror, &cross_cap] {
            for side in &domain.sides {
                assert_eq!(bits(side.normal), bits(DVec3::NEG_Z));
            }
            assert_eq!(domain.vertices, [DVec3::Y, DVec3::NEG_Y]);
        }
        for side in &mirror.sides {
            assert_eq!(
                side.pairing,
                DMat3::from_cols(DVec3::X, DVec3::Y, DVec3::NEG_Z)
            );
        }
        for side in &cross_cap.sides {
            assert_eq!(side.pairing, -DMat3::IDENTITY);
        }
    }

    #[test]
    fn rotary_lune_keeps_the_lune_and_pairs_by_the_half_turn_of_each_reflection() {
        let mut largest = (0.0_f64, String::new());
        for order in (2..=12).chain([100]) {
            let text = format!("{}x", order_text(order));
            let rotary = built(&text);
            let lune = built(&format!("*{0}{0}", order_text(order)));
            assert_eq!(rotary.vertices, lune.vertices, "{text}");
            assert_eq!(bits(rotary.centre), bits(lune.centre), "{text}");
            for (index, (side, mirror)) in rotary.sides.iter().zip(&lune.sides).enumerate() {
                assert_eq!(bits(side.normal), bits(mirror.normal), "{text}");
                assert_eq!(bits(side.covector), bits(mirror.covector), "{text}");
                assert_eq!(
                    side.pairing,
                    HALF_TURN * mirror.pairing,
                    "{text}, side {index}"
                );
            }
            let closing =
                largest_entry(rotary.sides[0].pairing * rotary.sides[1].pairing - DMat3::IDENTITY);
            if closing >= largest.0 {
                largest = (closing, text);
            }
        }
        println!(
            "largest entry of P0 P1 - I of nx: {:e}, at {}, bound {CLOSING_BOUND:e}, margin {:.1}",
            largest.0,
            largest.1,
            CLOSING_BOUND / largest.0
        );
        assert!(largest.0 <= CLOSING_BOUND, "{}: {:e}", largest.1, largest.0);
    }

    #[test]
    fn rotary_lune_of_order_two_has_the_written_out_pairings() {
        let rotary = built("2x");
        let expected = [
            DMat3::from_cols(DVec3::Z, DVec3::NEG_Y, DVec3::NEG_X),
            DMat3::from_cols(DVec3::NEG_Z, DVec3::NEG_Y, DVec3::X),
        ];
        for (side, expected) in rotary.sides.iter().zip(expected) {
            let difference = largest_entry(side.pairing - expected);
            assert!(difference <= 4.0 * f64::EPSILON, "{difference:e}");
        }
    }
}
