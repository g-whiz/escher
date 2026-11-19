use escher_geometry::euclidean::{ORIGIN, reflection, translation};
use glam::{DMat3, DVec2, DVec3};

use crate::domain::{Closure, Corner, FundamentalDomain, Orientation, PairingKind, Plane, Side};
use crate::double::product;
use crate::error::{Error, Result};
use crate::kite::{MirrorPair, on_kite};
use crate::parameters::Shear;
use crate::polygon::{outward_wall, rectangle};
use crate::triangle::Wallpaper;

enum Construction {
    Torus,
    Rectangle(&'static [Planned]),
    Kite([u32; 3]),
}

#[derive(Clone, Copy)]
enum Step {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Clone, Copy)]
enum Point {
    Corner(usize),
    Middle(usize),
}

impl Point {
    const fn wall(self) -> usize {
        match self {
            Self::Corner(wall) | Self::Middle(wall) => wall,
        }
    }
}

#[derive(Clone, Copy)]
enum Pairing {
    Mirror,
    Translation(Step),
    Glide(Step),
    HalfTurn,
}

#[derive(Clone, Copy)]
struct Planned {
    end: Point,
    pairing: Pairing,
    order: u32,
}

const fn corner(wall: usize, pairing: Pairing, order: u32) -> Planned {
    Planned {
        end: Point::Corner(wall),
        pairing,
        order,
    }
}

const fn middle(wall: usize, pairing: Pairing, order: u32) -> Planned {
    Planned {
        end: Point::Middle(wall),
        pairing,
        order,
    }
}

const ANNULUS: [Planned; 4] = [
    corner(0, Pairing::Translation(Step::Up), 1),
    corner(1, Pairing::Mirror, 1),
    corner(2, Pairing::Translation(Step::Down), 1),
    corner(3, Pairing::Mirror, 1),
];

const KLEIN_BOTTLE: [Planned; 4] = [
    corner(0, Pairing::Glide(Step::Up), 1),
    corner(1, Pairing::Translation(Step::Left), 1),
    corner(2, Pairing::Glide(Step::Down), 1),
    corner(3, Pairing::Translation(Step::Right), 1),
];

const MOBIUS_STRIP: [Planned; 4] = [
    corner(0, Pairing::Glide(Step::Up), 1),
    corner(1, Pairing::Mirror, 1),
    corner(2, Pairing::Glide(Step::Down), 1),
    corner(3, Pairing::Mirror, 1),
];

const TWO_TWO_CROSS_CAP: [Planned; 4] = [
    corner(0, Pairing::Glide(Step::Up), 2),
    corner(1, Pairing::Glide(Step::Left), 2),
    corner(2, Pairing::Glide(Step::Down), 2),
    corner(3, Pairing::Glide(Step::Right), 2),
];

const TWO_TWO_MIRROR: [Planned; 6] = [
    middle(0, Pairing::HalfTurn, 2),
    corner(0, Pairing::HalfTurn, 1),
    corner(1, Pairing::Mirror, 1),
    middle(2, Pairing::HalfTurn, 2),
    corner(2, Pairing::HalfTurn, 1),
    corner(3, Pairing::Mirror, 1),
];

const TWO_MIRROR_TWO_TWO: [Planned; 5] = [
    middle(0, Pairing::HalfTurn, 2),
    corner(0, Pairing::HalfTurn, 1),
    corner(1, Pairing::Mirror, 2),
    corner(2, Pairing::Mirror, 2),
    corner(3, Pairing::Mirror, 1),
];

const fn construction(wallpaper: Wallpaper) -> Construction {
    match wallpaper {
        Wallpaper::Torus => Construction::Torus,
        Wallpaper::Annulus => Construction::Rectangle(&ANNULUS),
        Wallpaper::KleinBottle => Construction::Rectangle(&KLEIN_BOTTLE),
        Wallpaper::MobiusStrip => Construction::Rectangle(&MOBIUS_STRIP),
        Wallpaper::TwoTwoCrossCap => Construction::Rectangle(&TWO_TWO_CROSS_CAP),
        Wallpaper::TwoTwoMirror => Construction::Rectangle(&TWO_TWO_MIRROR),
        Wallpaper::TwoMirrorTwoTwo => Construction::Rectangle(&TWO_MIRROR_TWO_TWO),
        Wallpaper::FourMirrorTwo => Construction::Kite([4, 4, 2]),
        Wallpaper::ThreeMirrorThree => Construction::Kite([6, 3, 2]),
    }
}

pub(crate) const fn kite_triangle(wallpaper: Wallpaper) -> Option<[u32; 3]> {
    match construction(wallpaper) {
        Construction::Kite(orders) => Some(orders),
        Construction::Torus | Construction::Rectangle(_) => None,
    }
}

pub(crate) fn construct(
    wallpaper: Wallpaper,
    values: &[f64],
    kite: Option<&FundamentalDomain>,
) -> Result<FundamentalDomain> {
    let domain = match construction(wallpaper) {
        Construction::Torus => torus(values),
        Construction::Rectangle(layout) => on_rectangle(values, layout),
        Construction::Kite(_) => kite
            .ok_or(None)
            .and_then(|doubled| on_kite(doubled, MirrorPair::Low)),
    }
    .map_err(Error::numerical)?;
    domain.sound()
}

fn torus(
    values: &[f64],
) -> core::result::Result<FundamentalDomain, Option<escher_geometry::Error>> {
    let &[aspect, skew] = values else {
        return Err(None);
    };
    let width = aspect.sqrt();
    let height = 1.0 / width;
    let first = DVec2::new(width, 0.0);
    let second = DVec2::new(skew * width, height);
    let vertices = [
        (first - second) * 0.5,
        (first + second) * 0.5,
        (second - first) * 0.5,
        -(first + second) * 0.5,
    ]
    .map(|vertex| vertex.extend(1.0));
    let shifts = [second, -first, -second, first];
    let mut sides = Vec::with_capacity(vertices.len());
    for (index, shift) in shifts.into_iter().enumerate() {
        let previous = vertices[(index + vertices.len() - 1) % vertices.len()];
        let covector = outward_wall(previous, vertices[index], ORIGIN)?;
        sides.push(Side {
            normal: DVec3::new(covector.x, covector.y, 0.0),
            covector,
            pairing: translation(shift.extend(1.0)),
            partner: (index + 2) % vertices.len(),
            kind: PairingKind::Translation,
            orientation: Orientation::Preserving,
        });
    }
    Ok(wallpaper_domain(sides, vertices.to_vec(), &[1; 4]))
}

fn on_rectangle(
    values: &[f64],
    layout: &[Planned],
) -> core::result::Result<FundamentalDomain, Option<escher_geometry::Error>> {
    let &[aspect] = values else {
        return Err(None);
    };
    let base = rectangle(aspect)?;
    let (width, height) = match base.vertices[..] {
        [right_bottom, right_top, _, left_bottom] => {
            (right_bottom.x - left_bottom.x, right_top.y - right_bottom.y)
        }
        _ => return Err(None),
    };
    let count = base.vertices.len();
    let vertices = layout
        .iter()
        .map(|planned| match planned.end {
            Point::Corner(index) => base.vertices.get(index).copied(),
            Point::Middle(wall) => {
                let start = base.vertices.get((wall + count - 1) % count)?;
                let end = base.vertices.get(wall)?;
                Some(DVec3::new(
                    f64::midpoint(start.x, end.x),
                    f64::midpoint(start.y, end.y),
                    1.0,
                ))
            }
        })
        .collect::<Option<Vec<DVec3>>>()
        .ok_or(None)?;
    let vector = |step| match step {
        Step::Up => DVec3::new(0.0, height, 1.0),
        Step::Right => DVec3::new(width, 0.0, 1.0),
        Step::Down => DVec3::new(0.0, -height, 1.0),
        Step::Left => DVec3::new(-width, 0.0, 1.0),
    };
    let axis = |step| match step {
        Step::Up | Step::Down => DVec3::X,
        Step::Right | Step::Left => DVec3::Y,
    };
    let on_wall = |wall: usize, other: usize| {
        layout
            .iter()
            .enumerate()
            .position(|(index, planned)| index != other && planned.end.wall() == wall)
    };
    let mut sides = Vec::with_capacity(layout.len());
    for (index, planned) in layout.iter().enumerate() {
        let wall = planned.end.wall();
        let source = *base.sides.get(wall).ok_or(None)?;
        let (pairing, partner, kind, orientation) = match planned.pairing {
            Pairing::Mirror => {
                sides.push(Side {
                    partner: index,
                    ..source
                });
                continue;
            }
            Pairing::Translation(step) => (
                translation(vector(step)),
                on_wall((wall + 2) % count, index),
                PairingKind::Translation,
                Orientation::Preserving,
            ),
            Pairing::Glide(step) => (
                translation(vector(step)) * reflection(axis(step)),
                on_wall((wall + 2) % count, index),
                PairingKind::GlideReflection,
                Orientation::Reversing,
            ),
            Pairing::HalfTurn => {
                let centre = layout
                    .iter()
                    .position(|other| matches!(other.end, Point::Middle(split) if split == wall))
                    .and_then(|vertex| vertices.get(vertex))
                    .ok_or(None)?;
                (
                    half_turn(*centre),
                    on_wall(wall, index),
                    PairingKind::Rotation,
                    Orientation::Preserving,
                )
            }
        };
        sides.push(Side {
            pairing,
            partner: partner.ok_or(None)?,
            kind,
            orientation,
            ..source
        });
    }
    let orders: Vec<u32> = layout.iter().map(|planned| planned.order).collect();
    Ok(wallpaper_domain(sides, vertices, &orders))
}

pub(crate) fn half_turn(centre: DVec3) -> DMat3 {
    product(
        reflection(DVec3::new(0.0, 1.0, -centre.y)),
        reflection(DVec3::new(1.0, 0.0, -centre.x)),
    )
}

pub(crate) fn shear(double: &FundamentalDomain, shear: Shear) -> Result<FundamentalDomain> {
    sheared(double, shear)
        .map_err(Error::numerical)
        .and_then(FundamentalDomain::sound)
}

fn sheared(
    double: &FundamentalDomain,
    shear: Shear,
) -> core::result::Result<FundamentalDomain, Option<escher_geometry::Error>> {
    let count = double.vertices.len();
    if double.sides.len() != count {
        return Err(None);
    }
    let factor = shear.skew * shear.aspect;
    let vertices: Vec<DVec3> = double
        .vertices
        .iter()
        .map(|vertex| DVec3::new(factor.mul_add(vertex.y, vertex.x), vertex.y, 1.0))
        .collect();
    let previous = |index: usize| (index + count - 1) % count;
    let pairings = double
        .sides
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let partner = source.partner;
            match source.kind {
                PairingKind::Rotation if partner == (index + 1) % count => {
                    vertices.get(index).copied().map(half_turn)
                }
                PairingKind::Rotation if index == (partner + 1) % count => {
                    vertices.get(partner).copied().map(half_turn)
                }
                PairingKind::Translation => {
                    let start = double.vertices.get(previous(index))?;
                    let end = double.vertices.get(partner)?;
                    let length = 2.0 * shear.aspect.sqrt();
                    Some(translation(DVec3::new(
                        length.copysign(end.x - start.x),
                        0.0,
                        1.0,
                    )))
                }
                PairingKind::Mirror | PairingKind::Rotation | PairingKind::GlideReflection => None,
            }
        })
        .collect::<Option<Vec<DMat3>>>()
        .ok_or(None)?;
    let mut sides = Vec::with_capacity(vertices.len());
    for (index, (source, pairing)) in double.sides.iter().zip(pairings).enumerate() {
        let covector = outward_wall(vertices[previous(index)], vertices[index], ORIGIN)?;
        sides.push(Side {
            normal: DVec3::new(covector.x, covector.y, 0.0),
            covector,
            pairing,
            ..*source
        });
    }
    Ok(FundamentalDomain {
        sides,
        vertices,
        corners: double.corners.clone(),
        centre: ORIGIN,
        plane: double.plane,
        closure: double.closure,
    })
}

fn wallpaper_domain(sides: Vec<Side>, vertices: Vec<DVec3>, orders: &[u32]) -> FundamentalDomain {
    FundamentalDomain {
        plane: Plane::Euclidean,
        corners: orders.iter().copied().map(Corner::single).collect(),
        sides,
        vertices,
        centre: ORIGIN,
        closure: Closure::Cycles,
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;

    use escher_geometry::euclidean;
    use glam::{DMat3, DVec3};

    use super::{Wallpaper, construct};
    use crate::PairingKind;
    use crate::shared::{GRID_ASPECTS, GRID_SKEWS, built_with_aspect};

    fn built(wallpaper: Wallpaper, text: &str, values: &[f64]) -> crate::FundamentalDomain {
        match construct(wallpaper, values, None) {
            Ok(domain) => domain,
            Err(error) => panic!("{text} at {values:?}: {error}"),
        }
    }

    fn shift(x: f64, y: f64) -> DMat3 {
        DMat3::from_cols(DVec3::X, DVec3::Y, DVec3::new(x, y, 1.0))
    }

    fn point(x: f64, y: f64) -> DVec3 {
        DVec3::new(x, y, 1.0)
    }

    fn across_vertical(x: f64, y: f64) -> DMat3 {
        DMat3::from_cols(DVec3::NEG_X, DVec3::Y, DVec3::new(x, y, 1.0))
    }

    fn across_horizontal(x: f64, y: f64) -> DMat3 {
        DMat3::from_cols(DVec3::X, DVec3::NEG_Y, DVec3::new(x, y, 1.0))
    }

    fn half_turn_about(x: f64, y: f64) -> DMat3 {
        DMat3::from_cols(
            DVec3::NEG_X,
            DVec3::NEG_Y,
            DVec3::new(2.0 * x, 2.0 * y, 1.0),
        )
    }

    #[test]
    fn torus_has_exact_vertices_and_translations_at_dyadic_settings() {
        for (aspect, skew) in [(1.0, 0.0), (4.0, 0.0), (1.0, 0.25), (4.0, 0.25)] {
            let domain = built(Wallpaper::Torus, "o", &[aspect, skew]);
            let width = aspect.sqrt();
            let height = 1.0 / width;
            let lean = skew * width;
            assert_eq!(
                domain.vertices,
                [
                    point((width - lean) / 2.0, -height / 2.0),
                    point(f64::midpoint(width, lean), height / 2.0),
                    point((lean - width) / 2.0, height / 2.0),
                    point(-f64::midpoint(width, lean), -height / 2.0),
                ],
                "aspect {aspect}, skew {skew}"
            );
            let pairings: Vec<DMat3> = domain.sides.iter().map(|side| side.pairing).collect();
            assert_eq!(
                pairings,
                [
                    shift(lean, height),
                    shift(-width, 0.0),
                    shift(-lean, -height),
                    shift(width, 0.0),
                ],
                "aspect {aspect}, skew {skew}"
            );
            assert_eq!(domain.centre, euclidean::ORIGIN);
        }
    }

    #[test]
    fn annulus_has_exact_vertices_and_pairings_at_dyadic_settings() {
        for aspect in [1.0, 4.0] {
            let domain = built(Wallpaper::Annulus, "**", &[aspect]);
            let (x, y) = (aspect.sqrt() / 2.0, 0.5 / aspect.sqrt());
            assert_eq!(
                domain.vertices,
                [point(x, -y), point(x, y), point(-x, y), point(-x, -y)],
                "aspect {aspect}"
            );
            let pairings: Vec<DMat3> = domain.sides.iter().map(|side| side.pairing).collect();
            assert_eq!(
                pairings,
                [
                    shift(0.0, 2.0 * y),
                    euclidean::reflection(DVec3::new(1.0, 0.0, -x)),
                    shift(0.0, -2.0 * y),
                    euclidean::reflection(DVec3::new(-1.0, 0.0, -x)),
                ],
                "aspect {aspect}"
            );
            assert_eq!(domain.centre, euclidean::ORIGIN);
        }
    }

    #[test]
    fn glide_groups_have_exact_vertices_and_pairings_at_dyadic_settings() {
        for aspect in [1.0_f64, 4.0] {
            let (x, y) = (aspect.sqrt() / 2.0, 0.5 / aspect.sqrt());
            let (up, down) = (
                across_vertical(0.0, 2.0 * y),
                across_vertical(0.0, -2.0 * y),
            );
            for (wallpaper, text, expected) in [
                (
                    Wallpaper::KleinBottle,
                    "xx",
                    [up, shift(-2.0 * x, 0.0), down, shift(2.0 * x, 0.0)],
                ),
                (
                    Wallpaper::MobiusStrip,
                    "*x",
                    [
                        up,
                        euclidean::reflection(DVec3::new(1.0, 0.0, -x)),
                        down,
                        euclidean::reflection(DVec3::new(-1.0, 0.0, -x)),
                    ],
                ),
                (
                    Wallpaper::TwoTwoCrossCap,
                    "22x",
                    [
                        up,
                        across_horizontal(-2.0 * x, 0.0),
                        down,
                        across_horizontal(2.0 * x, 0.0),
                    ],
                ),
            ] {
                let domain = built(wallpaper, text, &[aspect]);
                assert_eq!(
                    domain.vertices,
                    [point(x, -y), point(x, y), point(-x, y), point(-x, -y)],
                    "{text} at aspect {aspect}"
                );
                let pairings: Vec<DMat3> = domain.sides.iter().map(|side| side.pairing).collect();
                assert_eq!(pairings, expected, "{text} at aspect {aspect}");
                assert_eq!(
                    domain.centre,
                    euclidean::ORIGIN,
                    "{text} at aspect {aspect}"
                );
            }
        }
    }

    #[test]
    fn split_half_turn_groups_have_exact_vertices_and_pairings_at_dyadic_settings() {
        for aspect in [1.0_f64, 4.0] {
            let (x, y) = (aspect.sqrt() / 2.0, 0.5 / aspect.sqrt());
            let (right, top, left) = (
                euclidean::reflection(DVec3::new(1.0, 0.0, -x)),
                euclidean::reflection(DVec3::new(0.0, 1.0, -y)),
                euclidean::reflection(DVec3::new(-1.0, 0.0, -x)),
            );
            let (below, above) = (half_turn_about(0.0, -y), half_turn_about(0.0, y));
            for (wallpaper, text, vertices, pairings) in [
                (
                    Wallpaper::TwoTwoMirror,
                    "22*",
                    vec![
                        point(0.0, -y),
                        point(x, -y),
                        point(x, y),
                        point(0.0, y),
                        point(-x, y),
                        point(-x, -y),
                    ],
                    vec![below, below, right, above, above, left],
                ),
                (
                    Wallpaper::TwoMirrorTwoTwo,
                    "2*22",
                    vec![
                        point(0.0, -y),
                        point(x, -y),
                        point(x, y),
                        point(-x, y),
                        point(-x, -y),
                    ],
                    vec![below, below, right, top, left],
                ),
            ] {
                let domain = built(wallpaper, text, &[aspect]);
                assert_eq!(domain.vertices, vertices, "{text} at aspect {aspect}");
                let found: Vec<DMat3> = domain.sides.iter().map(|side| side.pairing).collect();
                assert_eq!(found, pairings, "{text} at aspect {aspect}");
                assert_eq!(
                    domain.centre,
                    euclidean::ORIGIN,
                    "{text} at aspect {aspect}"
                );
            }
        }
    }

    #[test]
    fn sheared_2222_has_exact_vertices_and_pairings_at_dyadic_settings() {
        for (aspect, expected) in [
            (
                1.0,
                [
                    point(0.375, -0.5),
                    point(1.375, -0.5),
                    point(1.625, 0.5),
                    point(0.625, 0.5),
                    point(-0.375, 0.5),
                    point(-0.625, -0.5),
                ],
            ),
            (
                4.0,
                [
                    point(0.75, -0.25),
                    point(2.75, -0.25),
                    point(3.25, 0.25),
                    point(1.25, 0.25),
                    point(-0.75, 0.25),
                    point(-1.25, -0.25),
                ],
            ),
        ] {
            let domain = built_with_aspect("2222", aspect, Some(0.25));
            assert_eq!(domain.vertices, expected, "aspect {aspect}");
            let width = 2.0 * aspect.sqrt();
            let (low, high) = (expected[0], expected[3]);
            let pairings: Vec<DMat3> = domain.sides.iter().map(|side| side.pairing).collect();
            assert_eq!(
                pairings,
                [
                    half_turn_about(low.x, low.y),
                    half_turn_about(low.x, low.y),
                    shift(-width, 0.0),
                    half_turn_about(high.x, high.y),
                    half_turn_about(high.x, high.y),
                    shift(width, 0.0),
                ],
                "aspect {aspect}"
            );
            assert_eq!(domain.centre, euclidean::ORIGIN, "aspect {aspect}");
        }
    }

    #[test]
    fn every_half_turn_is_exact_about_its_stored_centre_over_the_grid() {
        let rotated = euclidean::rotation(PI);
        assert_ne!(rotated.x_axis.y, 0.0);
        println!("sine of the rotation through pi: {:e}", rotated.x_axis.y);
        let mut settings: Vec<(&str, Option<f64>)> = vec![("22*", None), ("2*22", None)];
        settings.extend(GRID_SKEWS.map(|skew| ("2222", Some(skew))));
        let mut count = 0_usize;
        for aspect in GRID_ASPECTS {
            for &(text, skew) in &settings {
                let domain = built_with_aspect(text, aspect, skew);
                for (index, side) in domain.sides.iter().enumerate() {
                    if side.kind != PairingKind::Rotation {
                        continue;
                    }
                    let centre = domain.vertices[index.min(side.partner)];
                    assert_eq!(
                        side.pairing,
                        half_turn_about(centre.x, centre.y),
                        "{text} at aspect {aspect}, skew {skew:?}, side {index}"
                    );
                    assert_eq!(
                        side.pairing.x_axis.y, 0.0,
                        "{text} at aspect {aspect}, skew {skew:?}, side {index}"
                    );
                    count += 1;
                }
            }
        }
        assert_eq!(count, GRID_ASPECTS.len() * (4 + 2 + 4 * GRID_SKEWS.len()));
    }
}
