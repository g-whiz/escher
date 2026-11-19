use escher_geometry::euclidean;
use glam::{DMat3, DVec3};

use crate::domain::{Closure, Corner, FundamentalDomain, Orientation, PairingKind, Plane, Side};
use crate::polygon::{Polygon, rectangle};

pub(crate) const WALLPAPER_NAMES: [&str; 5] = ["**", "xx", "*x", "22x", "22*"];

#[derive(Clone, Copy)]
enum Pairing {
    Mirror,
    Translation(f64, f64),
    Glide { vector: (f64, f64), axis: DVec3 },
    HalfTurn(usize),
}

struct Layout {
    walls: Vec<usize>,
    vertices: Vec<DVec3>,
    pairings: Vec<(Pairing, usize)>,
    orders: Vec<u32>,
}

pub(crate) fn glide(vector: (f64, f64), axis: DVec3) -> DMat3 {
    euclidean::translation(DVec3::new(vector.0, vector.1, 1.0)) * euclidean::reflection(axis)
}

pub(crate) fn half_turn(centre: DVec3) -> DMat3 {
    DMat3::from_cols(
        DVec3::new(-1.0, 0.0, 0.0),
        DVec3::new(0.0, -1.0, 0.0),
        DVec3::new(2.0 * centre.x, 2.0 * centre.y, 1.0),
    )
}

pub(crate) fn wallpaper(name: &str, aspect: f64) -> FundamentalDomain {
    let base = match rectangle(aspect) {
        Ok(base) => base,
        Err(error) => panic!("aspect {aspect} has no rectangle: {error:?}"),
    };
    let layout = layout(name, &base);
    let sides = layout
        .walls
        .iter()
        .zip(&layout.pairings)
        .map(|(&wall, &(pairing, partner))| {
            let side = base.sides[wall];
            let (pairing, kind, orientation) = match pairing {
                Pairing::Mirror => (side.pairing, PairingKind::Mirror, Orientation::Reversing),
                Pairing::Translation(x, y) => (
                    euclidean::translation(DVec3::new(x, y, 1.0)),
                    PairingKind::Translation,
                    Orientation::Preserving,
                ),
                Pairing::Glide { vector, axis } => (
                    glide(vector, axis),
                    PairingKind::GlideReflection,
                    Orientation::Reversing,
                ),
                Pairing::HalfTurn(centre) => (
                    half_turn(layout.vertices[centre]),
                    PairingKind::Rotation,
                    Orientation::Preserving,
                ),
            };
            Side {
                pairing,
                partner,
                kind,
                orientation,
                ..side
            }
        })
        .collect();
    FundamentalDomain {
        plane: Plane::Euclidean,
        corners: layout.orders.into_iter().map(Corner::single).collect(),
        sides,
        vertices: layout.vertices,
        centre: base.centre,
        closure: Closure::Wallpaper,
    }
}

fn layout(name: &str, base: &Polygon) -> Layout {
    let [w0, w1, w2, w3] = base.vertices[..] else {
        panic!("the rectangle has {} vertices", base.vertices.len());
    };
    let (width, height) = (w0.x - w3.x, w1.y - w0.y);
    let glide_along = |x: f64, y: f64, axis: DVec3| Pairing::Glide {
        vector: (x, y),
        axis,
    };
    let (up, down) = (
        glide_along(0.0, height, DVec3::X),
        glide_along(0.0, -height, DVec3::X),
    );
    let rectangle = |pairings: Vec<(Pairing, usize)>, order: u32| Layout {
        walls: vec![0, 1, 2, 3],
        vertices: base.vertices.clone(),
        pairings,
        orders: vec![order; 4],
    };
    match name {
        "**" => rectangle(
            vec![
                (Pairing::Translation(0.0, height), 2),
                (Pairing::Mirror, 1),
                (Pairing::Translation(0.0, -height), 0),
                (Pairing::Mirror, 3),
            ],
            1,
        ),
        "xx" => rectangle(
            vec![
                (up, 2),
                (Pairing::Translation(-width, 0.0), 3),
                (down, 0),
                (Pairing::Translation(width, 0.0), 1),
            ],
            1,
        ),
        "*x" => rectangle(
            vec![
                (up, 2),
                (Pairing::Mirror, 1),
                (down, 0),
                (Pairing::Mirror, 3),
            ],
            1,
        ),
        "22x" => rectangle(
            vec![
                (up, 2),
                (glide_along(-width, 0.0, DVec3::Y), 3),
                (down, 0),
                (glide_along(width, 0.0, DVec3::Y), 1),
            ],
            2,
        ),
        "22*" => Layout {
            walls: vec![0, 0, 1, 2, 2, 3],
            vertices: vec![
                DVec3::new(0.0, w0.y, 1.0),
                w0,
                w1,
                DVec3::new(0.0, w1.y, 1.0),
                w2,
                w3,
            ],
            pairings: vec![
                (Pairing::HalfTurn(0), 1),
                (Pairing::HalfTurn(0), 0),
                (Pairing::Mirror, 2),
                (Pairing::HalfTurn(3), 4),
                (Pairing::HalfTurn(3), 3),
                (Pairing::Mirror, 5),
            ],
            orders: vec![2, 1, 1, 2, 1, 1],
        },
        _ => panic!("{name} has no hand-built domain"),
    }
}
