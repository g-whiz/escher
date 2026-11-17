use core::f64::consts::PI;

use escher_geometry::spherical::{
    ORIGIN, boost, inverse, normalize_point, reflection, rotation, side, translation, wall_through,
};
use glam::DVec3;

use crate::domain::{Orientation, PairingKind, Side};
use crate::polygon::Polygon;

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
    lune_of_angle(PI / f64::from(order))
}

pub(crate) fn lune_of_angle(angle: f64) -> Result<Polygon, Option<escher_geometry::Error>> {
    let (sin, cos) = (angle / 2.0).sin_cos();
    let sides = vec![
        Side::spherical_mirror(DVec3::new(cos, 0.0, -sin), 0),
        Side::spherical_mirror(DVec3::new(-cos, 0.0, -sin), 1),
    ];
    let inside = sides.iter().all(|wall| side(wall.normal, ORIGIN) < 0.0);
    if !inside {
        return Err(None);
    }
    finished(sides, vec![DVec3::Y, DVec3::NEG_Y], ORIGIN)
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
    fn spherical_mirror(normal: DVec3, index: usize) -> Self {
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
