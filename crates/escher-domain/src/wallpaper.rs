use escher_geometry::euclidean::{ORIGIN, reflection, side, translation, wall_through};
use glam::{DVec2, DVec3};

use crate::domain::{Closure, Corner, FundamentalDomain, Orientation, PairingKind, Plane, Side};
use crate::error::{Error, Result};
use crate::polygon::rectangle;
use crate::triangle::Wallpaper;

enum Construction {
    Torus,
    Rectangle(Layout),
}

#[derive(Clone, Copy)]
enum Step {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Clone, Copy)]
enum Pairing {
    Mirror,
    Translation(Step),
    Glide(Step),
}

struct Layout {
    pairings: [Pairing; 4],
    order: u32,
}

fn construction(wallpaper: Wallpaper) -> Option<Construction> {
    let rectangle = |pairings, order| Some(Construction::Rectangle(Layout { pairings, order }));
    match wallpaper {
        Wallpaper::Torus => Some(Construction::Torus),
        Wallpaper::Annulus => rectangle(
            [
                Pairing::Translation(Step::Up),
                Pairing::Mirror,
                Pairing::Translation(Step::Down),
                Pairing::Mirror,
            ],
            1,
        ),
        Wallpaper::KleinBottle => rectangle(
            [
                Pairing::Glide(Step::Up),
                Pairing::Translation(Step::Left),
                Pairing::Glide(Step::Down),
                Pairing::Translation(Step::Right),
            ],
            1,
        ),
        Wallpaper::MobiusStrip => rectangle(
            [
                Pairing::Glide(Step::Up),
                Pairing::Mirror,
                Pairing::Glide(Step::Down),
                Pairing::Mirror,
            ],
            1,
        ),
        Wallpaper::TwoTwoCrossCap => rectangle(
            [
                Pairing::Glide(Step::Up),
                Pairing::Glide(Step::Left),
                Pairing::Glide(Step::Down),
                Pairing::Glide(Step::Right),
            ],
            2,
        ),
        Wallpaper::TwoTwoMirror
        | Wallpaper::TwoMirrorTwoTwo
        | Wallpaper::FourMirrorTwo
        | Wallpaper::ThreeMirrorThree => None,
    }
}

pub(crate) fn is_built(wallpaper: Wallpaper) -> bool {
    construction(wallpaper).is_some()
}

pub(crate) fn construct(wallpaper: Wallpaper, values: &[f64]) -> Result<FundamentalDomain> {
    let domain = match construction(wallpaper) {
        Some(Construction::Torus) => torus(values),
        Some(Construction::Rectangle(layout)) => on_rectangle(values, &layout),
        None => Err(None),
    }
    .map_err(Error::numerical)?;
    if is_sound(&domain) {
        Ok(domain)
    } else {
        Err(Error::numerical(None))
    }
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
        let mut covector = wall_through(previous, vertices[index]).map_err(Some)?;
        if side(covector, ORIGIN) > 0.0 {
            covector = -covector;
        }
        sides.push(Side {
            normal: DVec3::new(covector.x, covector.y, 0.0),
            covector,
            pairing: translation(shift.extend(1.0)),
            partner: (index + 2) % vertices.len(),
            kind: PairingKind::Translation,
            orientation: Orientation::Preserving,
        });
    }
    Ok(wallpaper_domain(sides, vertices.to_vec(), 1))
}

fn on_rectangle(
    values: &[f64],
    layout: &Layout,
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
    let count = base.sides.len();
    let mut sides = base.sides;
    for (index, (paired, pairing)) in sides.iter_mut().zip(layout.pairings).enumerate() {
        let (pairing, kind, orientation) = match pairing {
            Pairing::Mirror => continue,
            Pairing::Translation(step) => (
                translation(vector(step)),
                PairingKind::Translation,
                Orientation::Preserving,
            ),
            Pairing::Glide(step) => (
                translation(vector(step)) * reflection(axis(step)),
                PairingKind::GlideReflection,
                Orientation::Reversing,
            ),
        };
        *paired = Side {
            pairing,
            partner: (index + 2) % count,
            kind,
            orientation,
            ..*paired
        };
    }
    Ok(wallpaper_domain(sides, base.vertices, layout.order))
}

fn wallpaper_domain(sides: Vec<Side>, vertices: Vec<DVec3>, order: u32) -> FundamentalDomain {
    FundamentalDomain {
        plane: Plane::Euclidean,
        corners: vec![Corner::single(order); vertices.len()],
        sides,
        vertices,
        centre: ORIGIN,
        closure: Closure::Wallpaper,
    }
}

fn is_sound(domain: &FundamentalDomain) -> bool {
    domain.vertices.iter().all(|vertex| vertex.is_finite())
        && domain.centre.is_finite()
        && domain.sides.iter().all(|wall| {
            wall.normal.is_finite()
                && wall.covector.is_finite()
                && wall.pairing.is_finite()
                && side(wall.covector, domain.centre) < 0.0
        })
}

#[cfg(test)]
mod tests {
    use escher_geometry::euclidean;
    use glam::{DMat3, DVec3};

    use super::{Wallpaper, construct};

    fn built(wallpaper: Wallpaper, text: &str, values: &[f64]) -> crate::FundamentalDomain {
        match construct(wallpaper, values) {
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
}
