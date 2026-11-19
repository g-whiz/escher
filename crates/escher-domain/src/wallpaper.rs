use escher_geometry::euclidean::{ORIGIN, side, translation, wall_through};
use glam::{DVec2, DVec3};

use crate::domain::{Closure, Corner, FundamentalDomain, Orientation, PairingKind, Plane, Side};
use crate::error::{Error, Result};
use crate::polygon::rectangle;
use crate::triangle::Wallpaper;

type Construction =
    fn(&[f64]) -> core::result::Result<FundamentalDomain, Option<escher_geometry::Error>>;

fn construction(wallpaper: Wallpaper) -> Option<Construction> {
    match wallpaper {
        Wallpaper::Torus => Some(torus),
        Wallpaper::Annulus => Some(annulus),
        Wallpaper::KleinBottle
        | Wallpaper::MobiusStrip
        | Wallpaper::TwoTwoMirror
        | Wallpaper::TwoTwoCrossCap
        | Wallpaper::TwoMirrorTwoTwo
        | Wallpaper::FourMirrorTwo
        | Wallpaper::ThreeMirrorThree => None,
    }
}

pub(crate) fn is_built(wallpaper: Wallpaper) -> bool {
    construction(wallpaper).is_some()
}

pub(crate) fn construct(wallpaper: Wallpaper, values: &[f64]) -> Result<FundamentalDomain> {
    let domain = construction(wallpaper)
        .ok_or(None)
        .and_then(|build| build(values))
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
    Ok(wallpaper_domain(sides, vertices.to_vec()))
}

fn annulus(
    values: &[f64],
) -> core::result::Result<FundamentalDomain, Option<escher_geometry::Error>> {
    let &[aspect] = values else {
        return Err(None);
    };
    let base = rectangle(aspect)?;
    let height = match base.vertices[..] {
        [bottom, top, ..] => top.y - bottom.y,
        _ => return Err(None),
    };
    let mut sides = base.sides;
    for (index, partner, shift) in [(0, 2, height), (2, 0, -height)] {
        let Some(paired) = sides.get_mut(index) else {
            return Err(None);
        };
        *paired = Side {
            pairing: translation(DVec3::new(0.0, shift, 1.0)),
            partner,
            kind: PairingKind::Translation,
            orientation: Orientation::Preserving,
            ..*paired
        };
    }
    Ok(wallpaper_domain(sides, base.vertices))
}

fn wallpaper_domain(sides: Vec<Side>, vertices: Vec<DVec3>) -> FundamentalDomain {
    FundamentalDomain {
        plane: Plane::Euclidean,
        corners: vec![Corner::single(1); vertices.len()],
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
}
