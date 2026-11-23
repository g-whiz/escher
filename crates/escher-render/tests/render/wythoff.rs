use escher_domain::FundamentalDomain;
use escher_geometry::{euclidean, hyperbolic, spherical};
use escher_reference::Reference;
use escher_render::{
    BorderUnit, Colour, ErrorKind, Fill, RenderOptions, Renderer, Shading, Wythoff, WythoffFill,
    fold,
};
use escher_signature::Geometry;
use glam::{DMat3, DVec2, DVec3};

use crate::common::{DEFAULT_CAP, options, plane_pixel};
use crate::golden_images::{
    GOLDEN_COSET_FILL, MARKINGS, WYTHOFF_DOMAINS, disk_pixel, domain_with_settings, exact_wythoff,
    marks_text, pixel_centre, renderer, signature_orders, table_faces, wythoff, wythoff_fill,
};

const SIZE: u32 = 193;

const RADIUS: f64 = 0.9;

const SAME_FACE: f64 = 1e-4;

const CLEARANCE: f64 = 1.0;

const PIXEL_HALF_WIDTH: f64 = 0.5;

const INTRINSIC_WIDTHS: [(Geometry, f64); 3] = [
    (Geometry::Hyperbolic, 0.06),
    (Geometry::Euclidean, 0.1),
    (Geometry::Spherical, 0.03),
];

const REACH: f64 = 4.0;

const COLLINEAR: f64 = 1e-12;

const LEAST_COMPARED: usize = 10_000;

const LEAST_EDGE_PIXELS: usize = 100;

const EDGE_TOLERANCE: f64 = 1e-13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Space {
    Hyperbolic,
    Euclidean,
    Spherical,
}

impl Space {
    const fn of(geometry: Geometry) -> Self {
        match geometry {
            Geometry::Hyperbolic => Self::Hyperbolic,
            Geometry::Euclidean => Self::Euclidean,
            Geometry::Spherical => Self::Spherical,
        }
    }

    fn normalize(self, point: DVec3) -> DVec3 {
        match self {
            Self::Hyperbolic => point / (-hyperbolic::form(point, point)).sqrt(),
            Self::Euclidean => point / point.z,
            Self::Spherical => point / point.length(),
        }
    }

    fn distance(self, first: DVec3, second: DVec3) -> f64 {
        match self {
            Self::Hyperbolic => hyperbolic::distance(first, second),
            Self::Euclidean => euclidean::distance(first, second),
            Self::Spherical => spherical::distance(first, second),
        }
    }

    fn wall_distance(self, covector: DVec3, point: DVec3) -> f64 {
        match self {
            Self::Hyperbolic => hyperbolic::wall_distance(covector, point),
            Self::Euclidean => euclidean::wall_distance(covector, point),
            Self::Spherical => spherical::wall_distance(covector, point),
        }
    }

    fn wall(self, first: DVec3, second: DVec3) -> Option<DVec3> {
        match self {
            Self::Hyperbolic => hyperbolic::wall_through(first, second)
                .ok()
                .map(hyperbolic::covector),
            Self::Euclidean => euclidean::wall_through(first, second).ok(),
            Self::Spherical => spherical::wall_through(first, second).ok(),
        }
    }

    fn towards(self, from: DVec3, to: DVec3) -> DVec3 {
        match self {
            Self::Hyperbolic => to + from * hyperbolic::form(to, from),
            Self::Euclidean => (to - from).with_z(0.0),
            Self::Spherical => to - from * to.dot(from),
        }
    }

    fn ahead(self, from: DVec3, direction: DVec3, point: DVec3) -> bool {
        match self {
            Self::Hyperbolic => hyperbolic::form(point, direction) > 0.0,
            Self::Euclidean => (point - from).with_z(0.0).dot(direction) > 0.0,
            Self::Spherical => point.dot(direction) > 0.0,
        }
    }

    fn segment_distance(
        self,
        point: DVec3,
        (start, end): (DVec3, DVec3),
        wall: Option<DVec3>,
    ) -> f64 {
        let between = self.ahead(start, self.towards(start, end), point)
            && self.ahead(end, self.towards(end, start), point);
        let to_ends = self.distance(point, start).min(self.distance(point, end));
        match wall {
            Some(wall) if between => self.wall_distance(wall, point).min(to_ends),
            Some(_) | None => to_ends,
        }
    }
}

fn orientation(first: DVec3, second: DVec3, third: DVec3) -> f64 {
    let value = first.dot(second.cross(third));
    let scale = first.length() * second.length() * third.length();
    if value.abs() <= COLLINEAR * scale {
        0.0
    } else {
        value.signum()
    }
}

fn crosses((start, end): (DVec3, DVec3), (other_start, other_end): (DVec3, DVec3)) -> bool {
    orientation(start, end, other_start) * orientation(start, end, other_end) < 0.0
        && orientation(other_start, other_end, start) * orientation(other_start, other_end, end)
            < 0.0
}

struct Scene {
    name: String,
    domain: FundamentalDomain,
    space: Space,
    marks: [bool; 3],
    corners: Vec<usize>,
    vertices: [DVec3; 3],
    generator: DVec3,
    segments: Vec<(DVec3, DVec3)>,
    star_segments: Vec<((DVec3, DVec3), Option<DVec3>)>,
    altitudes: [f64; 3],
    covectors: [DVec3; 3],
}

fn approximate(reference: &mut Reference, value: &escher_reference::Vector) -> DVec3 {
    DVec3::from_array([0, 1, 2].map(|index| reference.approximate(&value[index])))
}

impl Scene {
    fn new(text: &str, marks: [bool; 3]) -> Option<Self> {
        let orders = signature_orders(text);
        let corners: Vec<usize> = table_faces(orders, marks)
            .into_iter()
            .map(|(corner, _)| corner)
            .collect();
        if corners.is_empty() {
            return None;
        }
        let domain = domain_with_settings(text, &[]);
        let space = Space::of(domain.geometry());
        let mut reference = Reference::new();
        let exact = exact_wythoff(&domain, marks);
        let generator = space.normalize(approximate(&mut reference, &exact.generator));
        let segments: Vec<(DVec3, DVec3)> = exact
            .feet
            .iter()
            .flatten()
            .map(|foot| {
                (
                    generator,
                    space.normalize(approximate(&mut reference, foot)),
                )
            })
            .collect();
        let sides = domain.sides();
        let vertices = [0, 1, 2].map(|index| space.normalize(domain.vertices()[index]));
        let covectors = [0, 1, 2].map(|index| sides[index].covector());
        let altitudes = [0, 1, 2]
            .map(|corner| space.wall_distance(covectors[(corner + 2) % 3], vertices[corner]));
        let mut star_segments = Vec::new();
        for corner in 0..3 {
            let first = sides[corner].pairing();
            let turn = first * sides[(corner + 1) % 3].pairing();
            let mut element = DMat3::IDENTITY;
            for _ in 0..orders[corner] {
                for image in [element, element * first] {
                    star_segments.extend(segments.iter().map(|&(start, end)| {
                        let (start, end) =
                            (space.normalize(image * start), space.normalize(image * end));
                        ((start, end), space.wall(start, end))
                    }));
                }
                element *= turn;
            }
        }
        Some(Self {
            name: format!("{text} with {}", marks_text(marks)),
            domain,
            space,
            marks,
            corners,
            vertices,
            generator,
            segments,
            star_segments,
            altitudes,
            covectors,
        })
    }

    fn all(geometry: Geometry) -> Vec<Self> {
        WYTHOFF_DOMAINS
            .iter()
            .filter(|text| domain_with_settings(text, &[]).geometry() == geometry)
            .flat_map(|text| MARKINGS.iter().filter_map(|&marks| Self::new(text, marks)))
            .collect()
    }

    fn fill(&self) -> Fill {
        Fill::Wythoff(wythoff_fill(
            wythoff(&self.domain, self.marks),
            &GOLDEN_COSET_FILL,
        ))
    }

    fn render(&self, border: BorderUnit, width: f64) -> Vec<[u8; 3]> {
        let options = options(
            RenderOptions::builder(self.fill())
                .size(SIZE, SIZE)
                .samples(1)
                .border_unit(border)
                .border_width(width)
                .shading(Shading::Flat),
        );
        let (pixels, _) = renderer(&self.domain, options).render();
        pixels.as_chunks::<3>().0.to_vec()
    }

    fn pixel_size(&self) -> f64 {
        match self.space {
            Space::Euclidean => plane_pixel(SIZE, SIZE),
            Space::Hyperbolic | Space::Spherical => disk_pixel(SIZE, SIZE),
        }
    }

    fn screen_position(&self, column: u32, row: u32) -> DVec2 {
        pixel_centre(SIZE, SIZE, column, row, self.pixel_size())
    }

    fn point(&self, position: DVec2) -> Option<DVec3> {
        match self.space {
            Space::Euclidean => Some(position.extend(1.0)),
            Space::Hyperbolic => (position.length() <= RADIUS)
                .then(|| hyperbolic::from_disk(position))
                .flatten(),
            Space::Spherical => (position.length() <= RADIUS)
                .then(|| position.extend((1.0 - position.length_squared()).sqrt())),
        }
    }

    fn screen_scale(&self, position: DVec2) -> f64 {
        let pixel = self.pixel_size();
        let reach = position.length() + REACH * pixel;
        match self.space {
            Space::Euclidean => 1.0 / pixel,
            Space::Hyperbolic => (1.0 - reach * reach).max(0.0) / (2.0 * pixel),
            Space::Spherical => (1.0 - reach * reach).max(0.0).sqrt() / pixel,
        }
    }

    fn face(&self, folded: DVec3) -> Option<usize> {
        let point = self.space.normalize(folded);
        let mut found = self.corners.iter().enumerate().filter(|&(_, &corner)| {
            let path = (point, self.vertices[corner]);
            !self.segments.iter().any(|&segment| crosses(path, segment))
        });
        match (found.next(), found.next()) {
            (Some((index, _)), None) => Some(index),
            _ => None,
        }
    }

    fn sample(&self, column: u32, row: u32) -> Option<Sample> {
        let position = self.screen_position(column, row);
        let point = self.point(position)?;
        let mut sides = Vec::new();
        let result = fold(&self.domain, point, DEFAULT_CAP, |side| sides.push(side));
        assert!(result.converged(), "{}: pixel ({column}, {row})", self.name);
        let folded = self.space.normalize(result.point());
        let face = self.face(folded);
        let instance = face.map(|index| {
            sides
                .iter()
                .rev()
                .fold(self.vertices[self.corners[index]], |centre, &side| {
                    self.domain.sides()[side].pairing() * centre
                })
        });
        let own = self
            .segments
            .iter()
            .map(|&segment| {
                self.space
                    .segment_distance(folded, segment, self.space.wall(segment.0, segment.1))
            })
            .fold(f64::INFINITY, f64::min);
        let to_sides = self
            .covectors
            .iter()
            .map(|&covector| self.space.wall_distance(covector, folded))
            .fold(f64::INFINITY, f64::min);
        Some(Sample {
            folded,
            face,
            instance,
            near: own.min(to_sides),
            scale: self.screen_scale(position),
        })
    }

    fn beyond(&self, sample: &Sample) -> f64 {
        (0..3)
            .map(|corner| {
                self.altitudes[corner] - self.space.distance(sample.folded, self.vertices[corner])
            })
            .fold(f64::NEG_INFINITY, f64::max)
    }

    fn star<'a>(&'a self, sample: &'a Sample) -> impl Iterator<Item = f64> + 'a {
        self.star_segments
            .iter()
            .map(|&(segment, wall)| self.space.segment_distance(sample.folded, segment, wall))
    }

    fn clearance(&self, sample: &Sample) -> f64 {
        let star = self.star(sample).fold(f64::INFINITY, f64::min);
        sample.near.max(star.min(self.beyond(sample)))
    }

    fn far(&self, sample: &Sample) -> bool {
        let limit = CLEARANCE / sample.scale;
        sample.near > limit
            || (self.beyond(sample) > limit && self.star(sample).all(|distance| distance > limit))
    }

    fn samples(&self) -> Vec<Option<Sample>> {
        (0..SIZE)
            .flat_map(|row| (0..SIZE).map(move |column| (column, row)))
            .map(|(column, row)| self.sample(column, row))
            .collect()
    }
}

struct Sample {
    folded: DVec3,
    face: Option<usize>,
    instance: Option<DVec3>,
    near: f64,
    scale: f64,
}

fn index(column: usize, row: usize) -> usize {
    row * usize::try_from(SIZE).unwrap_or(usize::MAX) + column
}

fn neighbours() -> Vec<(usize, usize)> {
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    let mut pairs = Vec::new();
    for row in 0..size {
        for column in 0..size {
            if column + 1 < size {
                pairs.push((index(column, row), index(column + 1, row)));
            }
            if row + 1 < size {
                pairs.push((index(column, row), index(column, row + 1)));
            }
        }
    }
    pairs
}

fn colour_of(face: usize) -> [u8; 3] {
    GOLDEN_COSET_FILL[face]
}

fn assert_face_oracle(geometry: Geometry) {
    let mut total = 0;
    for scene in Scene::all(geometry) {
        let samples = scene.samples();
        let pixels = scene.render(BorderUnit::Pixels, 0.0);
        let colours: Vec<[u8; 3]> = (0..scene.corners.len()).map(colour_of).collect();
        let mut compared = 0;
        for (sample, pixel) in samples.iter().zip(&pixels) {
            let Some(sample) = sample else {
                continue;
            };
            assert!(
                colours.contains(pixel),
                "{}: the colour {pixel:?}",
                scene.name
            );
            if scene.far(sample) {
                let Some(face) = sample.face else {
                    panic!("{}: no face at {:?}", scene.name, sample.folded);
                };
                assert_eq!(
                    *pixel,
                    colour_of(face),
                    "{}: at {:?}",
                    scene.name,
                    sample.folded
                );
                compared += 1;
            }
        }
        let mut changes = 0;
        for (first, second) in neighbours() {
            let (Some(Some(here)), Some(Some(there))) = (samples.get(first), samples.get(second))
            else {
                continue;
            };
            if pixels[first] != pixels[second] {
                changes += 1;
                assert!(
                    !(scene.far(here) && scene.far(there)),
                    "{}: the colour changes between pixels {first} and {second}, farther than a pixel from every edge",
                    scene.name
                );
            }
        }
        println!(
            "{}: {compared} pixels farther than a pixel from every edge show the colour of their face, {changes} changes of colour lie within a pixel of an edge",
            scene.name
        );
        assert!(compared >= LEAST_COMPARED, "{}: {compared}", scene.name);
        assert_eq!(changes > 0, scene.corners.len() > 1, "{}", scene.name);
        total += compared;
    }
    println!("{total} pixels compared with the face of the reference");
}

#[test]
fn face_oracle_on_hyperbolic_triangles() {
    assert_face_oracle(Geometry::Hyperbolic);
}

#[test]
fn face_oracle_on_euclidean_triangles() {
    assert_face_oracle(Geometry::Euclidean);
}

#[test]
fn face_oracle_on_spherical_triangles() {
    assert_face_oracle(Geometry::Spherical);
}

fn assert_borders(geometry: Geometry, unit: BorderUnit) {
    let width = match unit {
        BorderUnit::Intrinsic => INTRINSIC_WIDTHS
            .iter()
            .find(|&&(of, _)| of == geometry)
            .map_or(0.0, |&(_, width)| width),
        _ => 2.0 * PIXEL_HALF_WIDTH,
    };
    let black = Colour::BLACK.to_srgb8();
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    for scene in Scene::all(geometry) {
        let samples = scene.samples();
        let pixels = scene.render(unit, width);
        let border: Vec<bool> = pixels.iter().map(|pixel| *pixel == black).collect();
        let mut change = vec![false; border.len()];
        for (first, second) in neighbours() {
            let (Some(Some(here)), Some(Some(there))) = (samples.get(first), samples.get(second))
            else {
                continue;
            };
            let differ = match (here.instance, there.instance) {
                (Some(here), Some(there)) => scene.space.distance(here, there) >= SAME_FACE,
                (None, _) | (_, None) => true,
            };
            if differ {
                change[first] = true;
                change[second] = true;
            }
        }
        let near = |marks: &[bool], column: usize, row: usize| {
            (row.saturating_sub(1)..=(row + 1).min(size - 1)).any(|other_row| {
                (column.saturating_sub(1)..=(column + 1).min(size - 1))
                    .any(|other_column| marks[index(other_column, other_row)])
            })
        };
        let (mut changes, mut borders) = (0, 0);
        for row in 1..size - 1 {
            for column in 1..size - 1 {
                let here = index(column, row);
                let Some(Some(sample)) = samples.get(here) else {
                    continue;
                };
                let inside = (row - 1..=row + 1).all(|other_row| {
                    (column - 1..=column + 1)
                        .all(|other_column| samples[index(other_column, other_row)].is_some())
                });
                if !inside {
                    continue;
                }
                let shown = match unit {
                    BorderUnit::Intrinsic => width * sample.scale > CLEARANCE,
                    _ => true,
                };
                if change[here] && shown {
                    changes += 1;
                    assert!(
                        near(&border, column, row),
                        "{}: the face changes at pixel ({column}, {row}), where the border is at least a pixel wide, with no border pixel next to it",
                        scene.name
                    );
                }
                if border[here] {
                    borders += 1;
                    let clearance = scene.clearance(sample) - EDGE_TOLERANCE;
                    match unit {
                        BorderUnit::Intrinsic => assert!(
                            clearance < width / 2.0,
                            "{}: the border pixel ({column}, {row}) lies at least {clearance} from every edge, not within half the border width, {}, the distance from the wall of an edge below which a sample is a border sample",
                            scene.name,
                            width / 2.0
                        ),
                        _ => assert!(
                            clearance * sample.scale < PIXEL_HALF_WIDTH,
                            "{}: the border pixel ({column}, {row}) lies at least {} pixels from every edge, not within half the border width, the estimated distance in pixels from the wall of an edge below which a sample is a border sample",
                            scene.name,
                            clearance * sample.scale
                        ),
                    }
                    assert!(
                        near(&change, column, row),
                        "{}: the border pixel ({column}, {row}) has no change of face next to it",
                        scene.name
                    );
                }
            }
        }
        println!(
            "{} with the {unit:?} unit: {changes} pixels where the face changes and {borders} border pixels",
            scene.name
        );
        assert!(
            changes >= LEAST_EDGE_PIXELS && borders >= LEAST_EDGE_PIXELS,
            "{}: {changes} changes of face and {borders} border pixels",
            scene.name
        );
    }
}

#[test]
fn wythoff_borders_in_pixels_on_hyperbolic_triangles() {
    assert_borders(Geometry::Hyperbolic, BorderUnit::Pixels);
}

#[test]
fn wythoff_borders_in_pixels_on_euclidean_triangles() {
    assert_borders(Geometry::Euclidean, BorderUnit::Pixels);
}

#[test]
fn wythoff_borders_in_pixels_on_spherical_triangles() {
    assert_borders(Geometry::Spherical, BorderUnit::Pixels);
}

#[test]
fn wythoff_borders_in_the_intrinsic_unit_on_hyperbolic_triangles() {
    assert_borders(Geometry::Hyperbolic, BorderUnit::Intrinsic);
}

#[test]
fn wythoff_borders_in_the_intrinsic_unit_on_euclidean_triangles() {
    assert_borders(Geometry::Euclidean, BorderUnit::Intrinsic);
}

#[test]
fn wythoff_borders_in_the_intrinsic_unit_on_spherical_triangles() {
    assert_borders(Geometry::Spherical, BorderUnit::Intrinsic);
}

fn screen_pixel(scene: &Scene, point: DVec3) -> Option<usize> {
    let position = match scene.space {
        Space::Hyperbolic => hyperbolic::to_disk(point),
        Space::Euclidean => point.truncate(),
        Space::Spherical => (point.z > 0.0).then(|| point.truncate())?,
    } / scene.pixel_size();
    let half = f64::from(SIZE) / 2.0;
    let column = (position.x + half).floor();
    let row = (half - position.y).floor();
    let inside = (0.0..f64::from(SIZE)).contains(&column) && (0.0..f64::from(SIZE)).contains(&row);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the column and the row are whole numbers inside the image"
    )]
    inside.then(|| index(column as usize, row as usize))
}

#[test]
fn the_pixel_a_tenth_of_the_way_from_a_corner_to_the_generator_point_shows_its_face() {
    let mut checked = 0;
    for geometry in [
        Geometry::Hyperbolic,
        Geometry::Euclidean,
        Geometry::Spherical,
    ] {
        for scene in Scene::all(geometry) {
            let pixels = scene.render(BorderUnit::Pixels, 0.0);
            for (face, &corner) in scene.corners.iter().enumerate() {
                let point = scene
                    .space
                    .normalize(scene.vertices[corner] * 0.9 + scene.generator * 0.1);
                let Some(pixel) = screen_pixel(&scene, point) else {
                    panic!("{}: corner {corner} is outside the image", scene.name);
                };
                assert_eq!(
                    pixels[pixel],
                    colour_of(face),
                    "{}: corner {corner}",
                    scene.name
                );
                checked += 1;
            }
        }
    }
    println!("{checked} faces show their colour near their corner");
    assert_eq!(checked, 226);
}

fn refusal(domain: &FundamentalDomain, marks: [bool; 3]) -> (ErrorKind, String) {
    match Wythoff::new(domain, marks) {
        Ok(wythoff) => panic!("{} is accepted: {wythoff:?}", marks_text(marks)),
        Err(error) => (error.kind(), error.to_string()),
    }
}

#[test]
fn wythoff_new_refuses_a_domain_that_is_not_a_reflection_triangle_before_marks_without_a_face() {
    let text = "cannot build Wythoff construction: the domain is not a reflection triangle";
    for signature in ["732", "*2222", "*22", "*3333"] {
        let domain = domain_with_settings(signature, &[]);
        for marks in [[true, true, true], [false, false, false]] {
            assert_eq!(
                refusal(&domain, marks),
                (ErrorKind::InvalidOption, text.to_owned()),
                "{signature}"
            );
        }
    }
}

#[test]
fn wythoff_new_refuses_marks_that_give_no_face() {
    let text = "cannot build Wythoff construction: the marks give no face".to_owned();
    let cases = [
        ("*732", [false, false, false]),
        ("*522", [true, false, false]),
        ("*222", [true, false, false]),
        ("*222", [false, true, false]),
        ("*222", [false, false, true]),
    ];
    for (signature, marks) in cases {
        let domain = domain_with_settings(signature, &[]);
        assert_eq!(
            refusal(&domain, marks),
            (ErrorKind::InvalidOption, text.clone()),
            "{signature} with {}",
            marks_text(marks)
        );
    }
}

#[test]
fn wythoff_fill_refuses_another_number_of_colours_and_the_renderer_another_domain() {
    let domain = domain_with_settings("*732", &[]);
    let construction = wythoff(&domain, [true, true, true]);
    assert_eq!(construction.face_types(), 3);
    let colours = vec![Colour::WHITE, Colour::BLACK];
    match WythoffFill::new(construction.clone(), colours) {
        Ok(fill) => panic!("2 colours are accepted: {fill:?}"),
        Err(error) => {
            assert_eq!(error.kind(), ErrorKind::InvalidOption);
            assert_eq!(
                error.to_string(),
                "cannot build Wythoff fill: the construction needs 3 colours, and 2 were given"
            );
        }
    }
    let one = wythoff(&domain, [false, true, false]);
    match WythoffFill::new(one, Vec::new()) {
        Ok(fill) => panic!("no colour is accepted: {fill:?}"),
        Err(error) => assert_eq!(
            error.to_string(),
            "cannot build Wythoff fill: the construction needs 1 colour, and 0 were given"
        ),
    }
    let fill = wythoff_fill(construction, &GOLDEN_COSET_FILL);
    let options = options(RenderOptions::builder(Fill::Wythoff(fill)).size(8, 8));
    let other = domain_with_settings("*742", &[]);
    match Renderer::new(&other, options.clone()) {
        Ok(renderer) => panic!("another domain is accepted: {renderer:?}"),
        Err(error) => {
            assert_eq!(error.kind(), ErrorKind::InvalidOption);
            assert_eq!(
                error.to_string(),
                "cannot build renderer: the Wythoff fill was built for another domain"
            );
        }
    }
    assert!(Renderer::new(&domain, options).is_ok());
}
