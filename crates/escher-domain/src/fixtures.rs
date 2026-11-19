use glam::{DMat3, DVec3};

use crate::domain::{Closure, Corner, FundamentalDomain, Orientation, PairingKind, Plane, Side};
use crate::polygon::rectangle;

pub(crate) fn half_turn(centre: DVec3) -> DMat3 {
    DMat3::from_cols(
        DVec3::new(-1.0, 0.0, 0.0),
        DVec3::new(0.0, -1.0, 0.0),
        DVec3::new(2.0 * centre.x, 2.0 * centre.y, 1.0),
    )
}

pub(crate) fn two_two_mirror(aspect: f64) -> FundamentalDomain {
    let base = match rectangle(aspect) {
        Ok(base) => base,
        Err(error) => panic!("aspect {aspect} has no rectangle: {error:?}"),
    };
    let [w0, w1, w2, w3] = base.vertices[..] else {
        panic!("the rectangle has {} vertices", base.vertices.len());
    };
    let vertices = vec![
        DVec3::new(0.0, w0.y, 1.0),
        w0,
        w1,
        DVec3::new(0.0, w1.y, 1.0),
        w2,
        w3,
    ];
    let walls = [0, 0, 1, 2, 2, 3];
    let half_turns = [
        Some((0, 1)),
        Some((0, 0)),
        None,
        Some((3, 4)),
        Some((3, 3)),
        None,
    ];
    let sides = walls
        .iter()
        .zip(half_turns)
        .enumerate()
        .map(|(index, (&wall, half))| {
            let side = base.sides[wall];
            match half {
                Some((centre, partner)) => Side {
                    pairing: half_turn(vertices[centre]),
                    partner,
                    kind: PairingKind::Rotation,
                    orientation: Orientation::Preserving,
                    ..side
                },
                None => Side {
                    partner: index,
                    ..side
                },
            }
        })
        .collect();
    FundamentalDomain {
        plane: Plane::Euclidean,
        corners: [2, 1, 1, 2, 1, 1].map(Corner::single).to_vec(),
        sides,
        vertices,
        centre: base.centre,
        closure: Closure::Wallpaper,
    }
}
