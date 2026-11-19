use core::f64::consts::PI;

use escher_geometry::euclidean::{ORIGIN, reflection, rotation, side, translation, wall_through};
use glam::DVec3;

use crate::domain::{Orientation, PairingKind, Side};

#[derive(Debug)]
pub(crate) struct Polygon {
    pub(crate) sides: Vec<Side>,
    pub(crate) vertices: Vec<DVec3>,
    pub(crate) centre: DVec3,
}

pub(crate) fn triangle(orders: [u32; 3]) -> Result<Polygon, Option<escher_geometry::Error>> {
    let angles = orders.map(|order| PI / f64::from(order));
    let length = angles[0].sin() / angles[1].sin();
    place(&[
        DVec3::new(1.0, 0.0, 1.0),
        rotation(angles[2]) * DVec3::new(length, 0.0, 1.0),
        ORIGIN,
    ])
}

pub(crate) fn rectangle(aspect: f64) -> Result<Polygon, Option<escher_geometry::Error>> {
    let width = aspect.sqrt();
    let height = 1.0 / width;
    place(&[
        DVec3::new(width / 2.0, -height / 2.0, 1.0),
        DVec3::new(width / 2.0, height / 2.0, 1.0),
        DVec3::new(-width / 2.0, height / 2.0, 1.0),
        DVec3::new(-width / 2.0, -height / 2.0, 1.0),
    ])
}

pub(crate) fn outward_wall(
    start: DVec3,
    end: DVec3,
    centre: DVec3,
) -> Result<DVec3, Option<escher_geometry::Error>> {
    let covector = wall_through(start, end).map_err(Some)?;
    Ok(if side(covector, centre) > 0.0 {
        -covector
    } else {
        covector
    })
}

fn place(placed: &[DVec3]) -> Result<Polygon, Option<escher_geometry::Error>> {
    let sum = placed.iter().fold(DVec3::ZERO, |sum, &vertex| sum + vertex);
    let mean = sum / sum.z;
    let move_to_origin = translation(DVec3::new(-mean.x, -mean.y, 1.0));
    let vertices: Vec<DVec3> = placed
        .iter()
        .map(|&vertex| move_to_origin * vertex)
        .collect();
    let total = vertices
        .iter()
        .fold(DVec3::ZERO, |total, &vertex| total + vertex);
    let centre = total / total.z;

    let count = vertices.len();
    let mut sides = Vec::with_capacity(count);
    for index in 0..count {
        let previous = vertices[(index + count - 1) % count];
        let covector = outward_wall(previous, vertices[index], centre)?;
        let inside = side(covector, centre) < 0.0;
        if !inside {
            return Err(None);
        }
        sides.push(Side::euclidean_mirror(covector, index));
    }

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
    pub(crate) fn euclidean_mirror(covector: DVec3, index: usize) -> Self {
        Self {
            normal: DVec3::new(covector.x, covector.y, 0.0),
            covector,
            pairing: reflection(covector),
            partner: index,
            kind: PairingKind::Mirror,
            orientation: Orientation::Reversing,
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use super::rectangle;

    #[test]
    fn rectangle_at_aspect_1_and_4_has_exact_vertices_and_covectors() {
        for (aspect, width, height) in [(1.0, 1.0, 1.0), (4.0, 2.0, 0.5)] {
            let polygon = match rectangle(aspect) {
                Ok(polygon) => polygon,
                Err(error) => panic!("aspect {aspect} has no rectangle: {error:?}"),
            };
            let (x, y) = (width / 2.0, height / 2.0);
            assert_eq!(
                polygon.vertices,
                [
                    DVec3::new(x, -y, 1.0),
                    DVec3::new(x, y, 1.0),
                    DVec3::new(-x, y, 1.0),
                    DVec3::new(-x, -y, 1.0),
                ],
                "aspect {aspect}"
            );
            let covectors: Vec<DVec3> = polygon.sides.iter().map(|side| side.covector).collect();
            assert_eq!(
                covectors,
                [
                    DVec3::new(0.0, -1.0, -y),
                    DVec3::new(1.0, 0.0, -x),
                    DVec3::new(0.0, 1.0, -y),
                    DVec3::new(-1.0, 0.0, -x),
                ],
                "aspect {aspect}"
            );
        }
    }
}
