use core::f64::consts::PI;

use escher_domain::{FundamentalDomain, PairingKind, Side};
use escher_geometry::{euclidean, hyperbolic, spherical};
use escher_signature::Geometry;
use glam::DVec3;

use crate::colour::Colour;
use crate::error::{Error, Result};

/// The Wythoff construction of a reflection triangle for one set of marks,
/// which turns the tiling by triangles into a tiling by faces.
///
/// Build it with [`Wythoff::new`], then give each face type a colour with
/// [`WythoffFill::new`]. Corner `k` of the triangle is vertex `k` of the
/// domain, whose sides are ordered so that side `j` joins vertex `j - 1` and
/// vertex `j`, with indexes modulo 3. Mark `k` turns on the mirror opposite
/// corner `k`, which is side `k + 2`. The generator point is the point of the
/// triangle that lies on every mirror that is off and at equal distance from
/// every mirror that is on. Its images under the group are the vertices of the
/// tiling, and an edge joins it to its image in each mirror that is on.
///
/// Each face of the tiling is centred at an image of a corner of the triangle.
/// A corner of order `n` has no face when neither mirror through it is on,
/// because the generator point is the corner, a face of `n` sides when one is,
/// and a face of `2 n` sides when both are. A face of 2 sides is a mere edge
/// and drops out. The face types are the corners with a face, in the order of
/// the corners, so a construction has 1, 2, or 3 face types. On the triangle
/// of `*pq2` with `p >= q >= 3`, the marks with only mark 1 on give the
/// regular tiling `{p,q}`, whose faces have `p` sides and meet `q` at each
/// vertex, and the marks with only mark 0 on give its dual `{q,p}`.
///
/// For the sphere, the generator points and the faces of each set of marks
/// are those of the uniform polyhedra of H. S. M. Coxeter, M. S.
/// Longuet-Higgins, and J. C. P. Miller, "Uniform polyhedra" (1954). For a
/// Euclidean or hyperbolic triangle, no source that the construction rests on
/// states them, and the construction applies the same rules.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_render::Wythoff;
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*732".parse::<Signature>()?)?;
/// let heptagons = Wythoff::new(&domain, [false, true, false])?;
/// assert_eq!(heptagons.face_types(), 1);
/// let omnitruncated = Wythoff::new(&domain, [true, true, true])?;
/// assert_eq!(omnitruncated.face_types(), 3);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Wythoff {
    faces: Vec<Face>,
    generator: DVec3,
    perpendiculars: [Option<DVec3>; 3],
    feet: [Option<DVec3>; 3],
    covectors: [DVec3; 3],
}

#[derive(Clone, Debug, PartialEq)]
struct Face {
    corner: usize,
    cuts: Vec<Cut>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cut {
    side: usize,
    sign: f64,
}

struct Plane {
    meet: fn(DVec3, DVec3) -> escher_geometry::Result<DVec3>,
    through: fn(DVec3, DVec3) -> escher_geometry::Result<DVec3>,
    sphere_centre: Option<DVec3>,
}

impl Plane {
    fn of(domain: &FundamentalDomain) -> Self {
        match domain.geometry() {
            Geometry::Hyperbolic => Self {
                meet: hyperbolic::wall_meet,
                through: |first, second| {
                    hyperbolic::wall_through(first, second).map(hyperbolic::covector)
                },
                sphere_centre: None,
            },
            Geometry::Euclidean => Self {
                meet: euclidean::wall_meet,
                through: euclidean::wall_through,
                sphere_centre: None,
            },
            Geometry::Spherical => Self {
                meet: spherical::wall_meet,
                through: spherical::wall_through,
                sphere_centre: Some(domain.centre()),
            },
        }
    }

    fn meet(&self, first: DVec3, second: DVec3) -> Result<DVec3> {
        let point = (self.meet)(first, second).map_err(|error| Error::degenerate(Some(error)))?;
        Ok(match self.sphere_centre {
            Some(centre) if point.dot(centre) < 0.0 => -point,
            Some(_) | None => point,
        })
    }

    fn through(&self, first: DVec3, second: DVec3) -> Result<DVec3> {
        (self.through)(first, second).map_err(|error| Error::degenerate(Some(error)))
    }
}

fn corner_orders(angles: &[f64]) -> Option<[f64; 3]> {
    let &[first, second, third] = angles else {
        return None;
    };
    Some([first, second, third].map(|angle| (PI / angle).round()))
}

const fn next(index: usize) -> usize {
    (index + 1) % 3
}

impl Wythoff {
    /// Returns the Wythoff construction of a reflection triangle, with
    /// `marks[k]` on when the mirror opposite corner `k` is on.
    ///
    /// The domain must have three sides, each a mirror. These are the domains
    /// of the hyperbolic signatures `*pqr`, of the Euclidean signatures
    /// `*632`, `*442`, and `*333`, and of the spherical signatures `*332`,
    /// `*432`, `*532`, and `*n22`. The construction reads the order of each
    /// corner as the whole number nearest to pi over its angle, and keeps the
    /// walls of the sides of the domain and no reference to the domain.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidOption`](crate::ErrorKind::InvalidOption) when the
    /// domain does not have three sides that are each a mirror, and then when
    /// the marks give no face type. The marks give none when every mark is
    /// off, on `*n22` when only mark 0 is on, and on `*222` when one mark is
    /// on. Returns an error of kind
    /// [`ErrorKind::Numerical`](crate::ErrorKind::Numerical) when the
    /// generator point, the wall through it perpendicular to a mirror that is
    /// on, or the foot of that wall on the mirror cannot be computed, or when a
    /// corner lies on such a wall through it.
    pub fn new(domain: &FundamentalDomain, marks: [bool; 3]) -> Result<Self> {
        let [first, second, third] = domain.sides() else {
            return Err(Error::not_triangle());
        };
        let sides = [*first, *second, *third];
        if sides
            .iter()
            .any(|side| !matches!(side.kind(), PairingKind::Mirror))
        {
            return Err(Error::not_triangle());
        }
        let (&[v0, v1, v2], Some(orders)) =
            (domain.vertices(), corner_orders(&domain.corner_angles()))
        else {
            return Err(Error::not_triangle());
        };
        let vertices = [v0, v1, v2];
        #[expect(
            clippy::float_cmp,
            reason = "the order is rounded to a whole number, which compares exactly"
        )]
        let is_two = |order: f64| order == 2.0;
        let active = [0, 1, 2].map(|side| marks[next(side)]);
        let corners: Vec<(usize, Vec<usize>)> = (0..3)
            .filter_map(|corner| {
                let through: Vec<usize> = [corner, next(corner)]
                    .into_iter()
                    .filter(|&side| active[side])
                    .collect();
                let digon = through.len() == 1 && is_two(orders[corner]);
                (!through.is_empty() && !digon).then_some((corner, through))
            })
            .collect();
        if corners.is_empty() {
            return Err(Error::no_face());
        }
        let plane = Plane::of(domain);
        let covectors = sides.map(Side::covector);
        let generator = match active {
            [true, false, false] => v1,
            [false, true, false] => v2,
            [false, false, true] => v0,
            [false, true, true] => plane.meet(covectors[0], covectors[1] - covectors[2])?,
            [true, false, true] => plane.meet(covectors[1], covectors[2] - covectors[0])?,
            [true, true, false] => plane.meet(covectors[2], covectors[0] - covectors[1])?,
            _ => plane.meet(covectors[0] - covectors[1], covectors[1] - covectors[2])?,
        };
        let mut perpendiculars = [None; 3];
        let mut feet = [None; 3];
        for (side, &on) in active.iter().enumerate() {
            if on {
                let image = sides[side].pairing() * generator;
                let perpendicular = plane.through(generator, image)?;
                feet[side] = Some(plane.meet(covectors[side], perpendicular)?);
                perpendiculars[side] = Some(perpendicular);
            }
        }
        let mut faces = Vec::with_capacity(corners.len());
        for (corner, through) in corners {
            let mut cuts = Vec::with_capacity(through.len());
            for side in through {
                let value = perpendiculars[side].map_or(f64::NAN, |perpendicular| {
                    perpendicular.dot(vertices[corner])
                });
                if value == 0.0 || !value.is_finite() {
                    return Err(Error::degenerate(None));
                }
                cuts.push(Cut {
                    side,
                    sign: value.signum(),
                });
            }
            faces.push(Face { corner, cuts });
        }
        Ok(Self {
            faces,
            generator,
            perpendiculars,
            feet,
            covectors,
        })
    }

    /// Returns the number of face types, the corners with a face, which is 1,
    /// 2, or 3.
    #[must_use]
    pub fn face_types(&self) -> u32 {
        u32::try_from(self.faces.len()).unwrap_or(u32::MAX)
    }

    pub(crate) fn fits(&self, domain: &FundamentalDomain) -> bool {
        let sides = domain.sides();
        sides.len() == 3
            && sides.iter().zip(self.covectors).all(|(side, covector)| {
                side.covector().to_array().map(f64::to_bits)
                    == covector.to_array().map(f64::to_bits)
            })
    }

    #[expect(
        clippy::expect_used,
        reason = "a construction is built only with at least one face, so its last face exists"
    )]
    pub(crate) fn face_of(&self, point: DVec3) -> usize {
        let matches = |face: &Face| {
            !face
                .cuts
                .iter()
                .any(|cut| cut.sign * self.value(cut.side, point) < 0.0)
        };
        self.faces
            .iter()
            .position(matches)
            .or_else(|| self.faces.len().checked_sub(1))
            .expect("internal error: a Wythoff construction has a face")
    }

    pub(crate) fn perpendiculars_of(&self, face: usize) -> impl Iterator<Item = DVec3> + '_ {
        self.faces
            .get(face)
            .into_iter()
            .flat_map(|face| &face.cuts)
            .filter_map(|cut| self.perpendiculars[cut.side])
    }

    fn value(&self, side: usize, point: DVec3) -> f64 {
        self.perpendiculars[side].map_or(f64::NAN, |perpendicular| perpendicular.dot(point))
    }
}

/// A [`Wythoff`] construction with one colour for each face type.
///
/// Build it with [`WythoffFill::new`] and render it as
/// [`Fill::Wythoff`](crate::Fill::Wythoff) with the domain of the
/// construction. The renderer gives each sample the colour of the face that
/// contains it and draws the borders on the edges of the faces, which join
/// the images of the generator point, and none on the mirrors of the
/// triangle.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_render::{Colour, Fill, RenderOptions, Renderer, Wythoff, WythoffFill};
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*732".parse::<Signature>()?)?;
/// let wythoff = Wythoff::new(&domain, [false, true, false])?;
/// let fill = WythoffFill::new(wythoff, vec![Colour::from_srgb8(233, 196, 106)])?;
/// let options = RenderOptions::builder(Fill::Wythoff(fill)).size(32, 32).build()?;
/// let (_, stats) = Renderer::new(&domain, options)?.render();
/// assert_eq!(stats.unconverged(), 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct WythoffFill {
    wythoff: Box<Wythoff>,
    colours: Vec<Colour>,
}

impl WythoffFill {
    /// Returns the fill that gives face type `t` the colour at index `t` of
    /// `colours`, with the face types in the order of the corners.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidOption`](crate::ErrorKind::InvalidOption) when the
    /// number of colours differs from the number of face types.
    pub fn new(wythoff: Wythoff, colours: Vec<Colour>) -> Result<Self> {
        let needed = wythoff.faces.len();
        if colours.len() != needed {
            return Err(Error::face_colour_count(needed, colours.len()));
        }
        Ok(Self {
            wythoff: Box::new(wythoff),
            colours,
        })
    }

    pub(crate) fn wythoff(&self) -> &Wythoff {
        &self.wythoff
    }

    #[expect(
        clippy::expect_used,
        reason = "the fill holds one colour for each face type of its construction, and a face type is an index below their number"
    )]
    pub(crate) fn colour(&self, face: usize) -> Colour {
        self.colours
            .get(face)
            .copied()
            .expect("internal error: every face type has a colour")
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::TAU;

    use astro_float_num::BigFloat;
    use escher_domain::FundamentalDomain;
    use escher_geometry::{euclidean, hyperbolic, spherical};
    use escher_reference::{Reference, Vector, Worst, apply, num, product, sub, vector};
    use escher_signature::Geometry;
    use glam::DVec3;
    use proptest::prelude::{ProptestConfig, prop_assert, proptest};

    use super::{Wythoff, corner_orders, next};
    use crate::fold::fold;
    use crate::golden_images::{
        MARKINGS, WYTHOFF_DOMAINS, WYTHOFF_GOLDEN_IMAGES, domain_with_settings, exact_wythoff,
        marks_text, reference_walls, signature_orders, table_faces, wythoff,
    };
    use crate::reference::{ExactWythoff, Measure};
    use crate::{ErrorKind, Renderer};

    const DISTINCT_IMAGES: f64 = 1e-9;

    const CORNER_CLEARANCE: f64 = 1e-6;

    const GENERATOR_TOLERANCE: f64 = 1e-14;

    const PERPENDICULAR_CLEARANCE: f64 = 1e-9;

    const CIRCLE_POINTS: u32 = 64;

    fn constructions() -> Vec<(String, FundamentalDomain, [bool; 3], Wythoff)> {
        let mut all = Vec::new();
        for text in WYTHOFF_DOMAINS {
            let domain = domain_with_settings(text, &[]);
            for marks in MARKINGS {
                if let Ok(wythoff) = Wythoff::new(&domain, marks) {
                    let name = format!("{text} with {}", marks_text(marks));
                    all.push((name, domain.clone(), marks, wythoff));
                }
            }
        }
        all
    }

    fn reference_point(point: DVec3) -> Vector {
        vector(point.to_array())
    }

    fn distinct_images(
        reference: &mut Reference,
        domain: &FundamentalDomain,
        corner: usize,
        order: u32,
        point: &Vector,
    ) -> (usize, f64, f64) {
        let walls = reference_walls(domain);
        let (first, second) = (&walls[corner].pairing, &walls[next(corner)].pairing);
        let turn = product(first, second);
        let mut images: Vec<DVec3> = Vec::new();
        let mut current = point.clone();
        for _ in 0..order {
            for image in [current.clone(), apply(first, &current)] {
                let coordinates = image.map(|coordinate| reference.approximate(&coordinate));
                images.push(DVec3::from_array(coordinates));
            }
            current = apply(&turn, &current);
        }
        let mut distinct: Vec<DVec3> = Vec::new();
        let mut closest_distinct = f64::INFINITY;
        let mut farthest_equal: f64 = 0.0;
        for image in images {
            let nearest = distinct
                .iter()
                .map(|kept| kept.distance(image))
                .fold(f64::INFINITY, f64::min);
            if nearest > DISTINCT_IMAGES {
                closest_distinct = closest_distinct.min(nearest);
                distinct.push(image);
            } else {
                farthest_equal = farthest_equal.max(nearest);
            }
        }
        (distinct.len(), closest_distinct, farthest_equal)
    }

    #[test]
    fn face_rule_gives_the_faces_of_the_table_and_every_construction_builds() {
        let mut reference = Reference::new();
        let mut built = 0;
        let mut refused = 0;
        let mut closest_distinct = f64::INFINITY;
        let mut farthest_equal: f64 = 0.0;
        let mut smallest_corner_value = f64::INFINITY;
        for text in WYTHOFF_DOMAINS {
            let domain = domain_with_settings(text, &[]);
            let orders = signature_orders(text);
            assert_eq!(
                corner_orders(&domain.corner_angles()),
                Some(orders.map(f64::from)),
                "{text}"
            );
            for marks in MARKINGS {
                let name = format!("{text} with {}", marks_text(marks));
                let expected = table_faces(orders, marks);
                let wythoff = match Wythoff::new(&domain, marks) {
                    Ok(wythoff) => wythoff,
                    Err(error) => {
                        assert!(expected.is_empty(), "{name}: {error}");
                        assert_eq!(error.kind(), ErrorKind::InvalidOption, "{name}");
                        assert_eq!(
                            error.to_string(),
                            "cannot build Wythoff construction: the marks give no face",
                            "{name}"
                        );
                        refused += 1;
                        continue;
                    }
                };
                built += 1;
                let faces: Vec<(usize, u32)> = wythoff
                    .faces
                    .iter()
                    .map(|face| {
                        let cuts = u32::try_from(face.cuts.len()).unwrap_or(u32::MAX);
                        (face.corner, cuts * orders[face.corner])
                    })
                    .collect();
                assert_eq!(faces, expected, "{name}");
                assert_eq!(
                    wythoff.face_types(),
                    u32::try_from(expected.len()).unwrap_or(0)
                );
                let generator = reference_point(wythoff.generator);
                for &(corner, sides) in &expected {
                    let (count, closest, farthest) = distinct_images(
                        &mut reference,
                        &domain,
                        corner,
                        orders[corner],
                        &generator,
                    );
                    assert_eq!(
                        u32::try_from(count).unwrap_or(u32::MAX),
                        sides,
                        "{name}: the images of the generator point around corner {corner}"
                    );
                    closest_distinct = closest_distinct.min(closest);
                    farthest_equal = farthest_equal.max(farthest);
                }
                for face in &wythoff.faces {
                    let vertex = domain.vertices()[face.corner];
                    for cut in &face.cuts {
                        let Some(perpendicular) = wythoff.perpendiculars[cut.side] else {
                            panic!("{name}: side {} has no perpendicular", cut.side);
                        };
                        let value = perpendicular.dot(vertex).abs()
                            / (perpendicular.length() * vertex.length());
                        smallest_corner_value = smallest_corner_value.min(value);
                        assert!(
                            value > CORNER_CLEARANCE,
                            "{name}: corner {} lies within {value} of the perpendicular of side {}",
                            face.corner,
                            cut.side
                        );
                    }
                }
                for side in (0..3).filter(|&side| marks[next(side)]) {
                    assert!(
                        wythoff
                            .faces
                            .iter()
                            .any(|face| face.cuts.iter().any(|cut| cut.side == side)),
                        "{name}: the active side {side} has no corner with a face"
                    );
                }
            }
        }
        println!(
            "face rule: {built} constructions built and {refused} refused, closest distinct images {closest_distinct:e}, farthest equal images {farthest_equal:e} against {DISTINCT_IMAGES:e}, smallest |q . V| / (|q| |V|) {smallest_corner_value:e} against {CORNER_CLEARANCE:e}"
        );
        assert_eq!(built + refused, WYTHOFF_DOMAINS.len() * MARKINGS.len());
        assert_eq!(refused, WYTHOFF_DOMAINS.len() + 4 + 3);
    }

    #[test]
    fn wythoff_golden_configurations_draw_the_faces_that_their_names_state() {
        let expected: [&[(usize, u32)]; 3] = [
            &[(0, 7)],
            &[(0, 12), (1, 6), (2, 4)],
            &[(0, 5), (1, 3), (2, 4)],
        ];
        for (image, faces) in WYTHOFF_GOLDEN_IMAGES.iter().zip(expected) {
            let domain = image.golden.domain();
            let suffix = format!("-{}.png", marks_text(image.marks));
            assert!(
                image.golden.file.ends_with(&suffix),
                "{}",
                image.golden.file
            );
            let orders = signature_orders(image.golden.signature);
            assert_eq!(
                table_faces(orders, image.marks),
                faces,
                "{}",
                image.golden.file
            );
            let wythoff = wythoff(&domain, image.marks);
            assert_eq!(
                wythoff.face_types(),
                u32::try_from(faces.len()).unwrap_or(0)
            );
            let renderer = Renderer::new(&domain, image.options(&domain));
            assert!(renderer.is_ok(), "{}", image.golden.file);
        }
    }

    #[test]
    fn generator_point_and_feet_lie_where_the_construction_puts_them() {
        let mut reference = Reference::new();
        let mut worst = Worst::new(
            "Wythoff generator point and feet, distances off their walls and beyond the sides in the reference",
            1348,
        );
        let mut record = |distance: &BigFloat, input: &dyn Fn() -> String| {
            worst.record(distance, &num(1.0), input);
        };
        for (name, domain, marks, wythoff) in &constructions() {
            let measure = Measure::of(domain.geometry());
            let walls = reference_walls(domain);
            let beyond = |reference: &mut Reference, side: usize, point: &Vector| {
                if escher_reference::dot(&walls[side].covector, point).is_positive() {
                    measure.wall_distance(reference, &walls[side].covector, point)
                } else {
                    num(0.0)
                }
            };
            let generator = reference_point(wythoff.generator);
            let distances: Vec<BigFloat> = walls
                .iter()
                .map(|wall| measure.wall_distance(&mut reference, &wall.covector, &generator))
                .collect();
            let active: Vec<usize> = (0..3).filter(|&side| marks[next(side)]).collect();
            for (side, distance) in distances.iter().enumerate() {
                let outside = beyond(&mut reference, side, &generator);
                record(&outside, &|| format!("{name}: G beyond side {side}"));
                if !active.contains(&side) {
                    record(distance, &|| {
                        format!("{name}: G off the inactive side {side}")
                    });
                }
            }
            for pair in active.windows(2) {
                let gap = sub(&distances[pair[0]], &distances[pair[1]]).abs();
                record(&gap, &|| {
                    format!("{name}: unequal distances to sides {pair:?}")
                });
            }
            for &side in &active {
                let (Some(foot), Some(perpendicular)) =
                    (wythoff.feet[side], wythoff.perpendiculars[side])
                else {
                    panic!("{name}: the active side {side} has no foot");
                };
                let foot = reference_point(foot);
                let on_side = measure.wall_distance(&mut reference, &walls[side].covector, &foot);
                record(&on_side, &|| format!("{name}: foot {side} off its side"));
                let perpendicular = reference_point(perpendicular);
                let on_perpendicular = measure.wall_distance(&mut reference, &perpendicular, &foot);
                record(&on_perpendicular, &|| {
                    format!("{name}: foot {side} off its perpendicular")
                });
                for other in (0..3).filter(|&other| other != side) {
                    let outside = beyond(&mut reference, other, &foot);
                    record(&outside, &|| {
                        format!("{name}: foot {side} beyond side {other}")
                    });
                }
            }
        }
        worst.assert_at_most(&mut reference, GENERATOR_TOLERANCE);
    }

    fn point_on_segment(geometry: Geometry, from: DVec3, to: DVec3, fraction: f64) -> DVec3 {
        in_plane(geometry, from * (1.0 - fraction) + to * fraction)
    }

    fn in_plane(geometry: Geometry, point: DVec3) -> DVec3 {
        match geometry {
            Geometry::Euclidean => point / point.z,
            Geometry::Hyperbolic | Geometry::Spherical => point,
        }
    }

    #[test]
    fn the_point_a_tenth_of_the_way_from_a_corner_to_the_generator_point_takes_its_face() {
        for (name, domain, _, wythoff) in constructions() {
            for (index, face) in wythoff.faces.iter().enumerate() {
                let vertex = domain.vertices()[face.corner];
                let point = point_on_segment(domain.geometry(), vertex, wythoff.generator, 0.1);
                assert_eq!(
                    wythoff.face_of(point),
                    index,
                    "{name}: corner {}",
                    face.corner
                );
            }
        }
    }

    fn circle(geometry: Geometry, centre: DVec3, radius: f64, angle: f64) -> DVec3 {
        match geometry {
            Geometry::Hyperbolic => {
                let centre = hyperbolic::normalize_point(centre)
                    .unwrap_or_else(|error| panic!("{centre:?}: {error}"));
                hyperbolic::translation(centre)
                    * hyperbolic::rotation(angle)
                    * hyperbolic::boost(radius)
                    * hyperbolic::ORIGIN
            }
            Geometry::Euclidean => {
                euclidean::translation(centre)
                    * euclidean::rotation(angle)
                    * euclidean::translation(DVec3::new(radius, 0.0, 1.0))
                    * euclidean::ORIGIN
            }
            Geometry::Spherical => {
                spherical::translation(centre)
                    * spherical::rotation(angle)
                    * spherical::boost(radius)
                    * spherical::ORIGIN
            }
        }
    }

    #[test]
    fn each_face_covers_its_centre_from_every_side_of_its_mirrors() {
        let mut reference = Reference::new();
        let mut points = 0;
        for (name, domain, marks, wythoff) in constructions() {
            let exact = exact_wythoff(&domain, marks);
            for (index, face) in wythoff.faces.iter().enumerate() {
                let vertex = &exact.vertices[face.corner];
                let mut least = exact
                    .measure
                    .distance(&mut reference, vertex, &exact.generator);
                for side in [face.corner, next(face.corner)] {
                    if let Some(foot) = &exact.feet[side] {
                        let to_foot = exact.measure.distance(&mut reference, vertex, foot);
                        if to_foot < least {
                            least = to_foot;
                        }
                    }
                }
                let radius = reference.approximate(&least) / 2.0;
                for step in 0..CIRCLE_POINTS {
                    let angle = TAU * f64::from(step) / f64::from(CIRCLE_POINTS);
                    let point = circle(
                        domain.geometry(),
                        domain.vertices()[face.corner],
                        radius,
                        angle,
                    );
                    let result = fold(&domain, point, 1500, |_| {});
                    assert!(
                        result.converged(),
                        "{name}: corner {} at {angle}",
                        face.corner
                    );
                    assert_eq!(
                        wythoff.face_of(result.point()),
                        index,
                        "{name}: corner {} at the angle {angle}",
                        face.corner
                    );
                    points += 1;
                }
            }
        }
        println!("faces around a centre: {points} points");
        assert!(points >= 64 * 200, "{points}");
    }

    fn random_point(domain: &FundamentalDomain, weights: [f64; 3]) -> DVec3 {
        let point = domain
            .vertices()
            .iter()
            .zip(weights)
            .fold(DVec3::ZERO, |sum, (&vertex, weight)| sum + vertex * weight);
        in_plane(domain.geometry(), point)
    }

    fn assert_face_region(
        exact: &ExactWythoff,
        wythoff: &Wythoff,
        point: DVec3,
    ) -> Result<bool, String> {
        let mut reference = Reference::new();
        let x = reference_point(point);
        for perpendicular in exact.perpendiculars.iter().flatten() {
            let distance = exact
                .measure
                .wall_distance(&mut reference, perpendicular, &x);
            if distance < num(PERPENDICULAR_CLEARANCE) {
                return Ok(false);
            }
        }
        let face = wythoff.face_of(point);
        let corner = wythoff.faces[face].corner;
        if exact.corner_holds(corner, &x) {
            Ok(true)
        } else {
            Err(format!("{point:?} takes the face of corner {corner}"))
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::default())]
        #[test]
        fn a_point_of_the_triangle_takes_the_face_of_a_corner_whose_sign_tests_hold(
            domain_index in 0..WYTHOFF_DOMAINS.len(),
            marking in 1..MARKINGS.len(),
            weights in [0.001..1.0_f64, 0.001..1.0_f64, 0.001..1.0_f64],
        ) {
            let domain = domain_with_settings(WYTHOFF_DOMAINS[domain_index], &[]);
            let marks = MARKINGS[marking];
            if let Ok(wythoff) = Wythoff::new(&domain, marks) {
                let exact = exact_wythoff(&domain, marks);
                let point = random_point(&domain, weights);
                let checked = assert_face_region(&exact, &wythoff, point);
                prop_assert!(checked.is_ok(), "{} {}: {:?}", WYTHOFF_DOMAINS[domain_index], marks_text(marks), checked);
                }
        }
    }
}
