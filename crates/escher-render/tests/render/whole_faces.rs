use core::f64::consts::PI;

use escher_domain::FundamentalDomain;
use escher_group::{Colouring, Letter, Word};
use escher_render::{Colour, ErrorKind, Fill, Paint, RenderOptions, Shading, WholeFace, Wythoff};
use escher_signature::Geometry;
use glam::{DMat2, DMat3, DVec2, DVec3};

use crate::golden_images::{
    DUAL, REGULAR, arrangements, bilinear, colourings_of, domain_with_settings, exact_face,
    face_corner, face_triangle_mask, gradient, image_of, linear_table, marks_text, reference_walls,
    renderer, whole_face, whole_face_fill, word_matrix,
};
use crate::image_fills::{exact_point, pixel_at, screen, screen_point, view_pixel};
use crate::reference::{
    ExactFace, ExactPlacement, FlatMatrix, ReferenceExt, Wall, flat_apply, transpose_apply,
};
use escher_reference::{
    Matrix, Reference, Vector, apply, difference, dot, int, largest_magnitude, num,
};

const SIZE: u32 = 192;

const PICTURE: u32 = 192;

const RADIUS: f64 = 0.9;

const KEPT_RADIUS: f64 = 0.97;

const KEPT_PLANE: f64 = 6.5;

const EDGE_CLEARANCE: f64 = 1e-6;

const SAME_POINT: f64 = 1e-6;

const IN_WEDGE: f64 = 1e-12;

const SAME_CENTRE: f64 = 1e-6;

const BACKGROUND: [u8; 3] = [10, 200, 30];

const PAINT: [u8; 3] = [233, 196, 106];

const SOLID_CLEARANCE: u32 = 3;

const WEDGE_STEP: u8 = 12;

const MARK_RADIUS: f64 = 0.1;

const MARK_CENTRES: [[f64; 2]; 3] = [[-0.25, -0.12], [0.22, -0.18], [0.02, 0.27]];

const MARK_COLOURS: [[u8; 3]; 3] = [[230, 20, 20], [20, 200, 20], [20, 20, 230]];

const MARK_MATCH: u8 = 40;

const LEAST_MARK_PIXELS: u32 = 4;

const ROUNDING_BAND: f64 = 1e-3;

const OFF_BY_ONE_SHARE: usize = 10_000;

const FEATURE_OFFSET: f64 = 40.0;

const FEATURE_RADIUS: f64 = 20.0;

const FEATURE_BACKGROUND: [u8; 3] = [128, 128, 128];

const LEAST_FEATURE_PIXELS: u32 = 64;

const UPRIGHT_ANGLE: f64 = 2.0 * PI / 180.0;

const PLANE_ANGLE: f64 = 0.25 * PI / 180.0;

struct GeometricFace {
    matrix: Matrix,
    inverse: Matrix,
    centre: DVec3,
}

struct PixelGeometry {
    face: usize,
    wedge: usize,
    picture: DVec2,
    radius: f64,
}

struct Tiling {
    name: String,
    domain: FundamentalDomain,
    face: WholeFace,
    geometry: Geometry,
    exact: ExactFace,
    stabilizer: Vec<Word>,
    reflections: Vec<Word>,
    vertices: Vec<Vector>,
    faces: Vec<GeometricFace>,
    pixels: Vec<Option<PixelGeometry>>,
    neighbours: Vec<usize>,
    view_turn: f64,
}

fn letter(side: usize) -> Letter {
    Letter::generator(u8::try_from(side).unwrap_or(u8::MAX))
}

fn stabilizer_words(corner: usize, order: u32) -> (Vec<Word>, Vec<Word>) {
    let centre_mirror = letter(if corner == 0 { 0 } else { 2 });
    let vertex_mirror = letter(1);
    let edge = letter(if corner == 0 { 2 } else { 0 });
    let turn = |turns: u32| -> Word {
        (0..turns)
            .flat_map(|_| [centre_mirror, vertex_mirror])
            .collect()
    };
    let elements = (0..order)
        .flat_map(|turns| {
            [
                turn(turns),
                turn(turns)
                    .letters()
                    .iter()
                    .copied()
                    .chain([centre_mirror])
                    .collect(),
            ]
        })
        .collect();
    let reflections = (0..order)
        .map(|turns| {
            let there = turn(turns);
            there
                .letters()
                .iter()
                .copied()
                .chain([edge])
                .chain(there.inverse().letters().iter().copied())
                .collect()
        })
        .collect();
    (elements, reflections)
}

fn identity() -> Matrix {
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| int(u32::from(row == column))))
}

fn approximate(reference: &mut Reference, point: &Vector) -> DVec3 {
    DVec3::new(
        reference.approximate(&point[0]),
        reference.approximate(&point[1]),
        reference.approximate(&point[2]),
    )
}

fn kept(geometry: Geometry, centre: DVec3) -> bool {
    match geometry {
        Geometry::Hyperbolic => centre.truncate().length() / (1.0 + centre.z) < KEPT_RADIUS,
        Geometry::Euclidean => centre.x.abs() <= KEPT_PLANE && centre.y.abs() <= KEPT_PLANE,
        Geometry::Spherical => true,
    }
}

fn closeness(geometry: Geometry, point: DVec3, centre: DVec3) -> f64 {
    match geometry {
        Geometry::Hyperbolic => point.z * centre.z - point.x * centre.x - point.y * centre.y,
        Geometry::Euclidean => (point - centre).truncate().length_squared(),
        Geometry::Spherical => -point.dot(centre),
    }
}

fn find_face(faces: &[GeometricFace], centre: DVec3) -> Option<usize> {
    faces.iter().position(|face| {
        (face.centre - centre).abs().max_element()
            <= SAME_CENTRE * centre.abs().max_element().max(1.0)
    })
}

fn relative_gap(reference: &mut Reference, first: &Vector, second: &Vector) -> f64 {
    let gap = reference.approximate(&largest_magnitude(&difference(first, second)));
    gap / reference.approximate(&largest_magnitude(first)).max(1.0)
}

fn flat_transpose(matrix: &FlatMatrix) -> FlatMatrix {
    [
        [matrix[0][0].clone(), matrix[1][0].clone()],
        [matrix[0][1].clone(), matrix[1][1].clone()],
    ]
}

impl Tiling {
    fn new(text: &str, marks: [bool; 3]) -> Self {
        let mut reference = Reference::new();
        let domain = domain_with_settings(text, &[]);
        let face = whole_face(&domain, marks);
        let geometry = domain.geometry();
        let corner = face_corner(marks);
        let exact = exact_face(&mut reference, text, &domain, marks);
        let mask = face_triangle_mask(&mut reference, text, &domain, marks);
        let order = face.index() / 2;
        let (stabilizer, reflections) = stabilizer_words(corner, order);
        let walls = reference_walls(&domain);
        let vertices: Vec<Vector> = domain
            .vertices()
            .iter()
            .map(|vertex| escher_reference::vector(vertex.to_array()))
            .collect();
        let centre = vertices[corner].clone();
        let reflection_matrices: Vec<Matrix> = reflections
            .iter()
            .map(|word| word_matrix(&walls, word))
            .collect();
        let faces = walk_faces(&mut reference, geometry, &reflection_matrices, &centre);
        let edge_walls: Vec<DVec3> = edge_walls(&walls, &stabilizer, corner, &centre)
            .iter()
            .map(|wall| approximate(&mut reference, wall))
            .collect();
        let wedges: Vec<Vec<DVec3>> = wedge_walls(&walls, &exact, order)
            .iter()
            .map(|covectors| {
                covectors
                    .iter()
                    .map(|covector| approximate(&mut reference, covector))
                    .collect()
            })
            .collect();
        let oracle = PixelOracle {
            name: format!("{text} {}", marks_text(marks)),
            geometry,
            faces: &faces,
            edge_walls,
            wedges,
            exact: &exact,
            mask,
            order,
        };
        let pixels: Vec<Option<PixelGeometry>> = (0..SIZE * SIZE)
            .map(|index| oracle.pixel(&mut reference, index % SIZE, index / SIZE))
            .collect();
        let name = oracle.name;
        let neighbours = neighbours(&mut reference, &exact, &walls, &centre, &faces, &name);
        let home = pixels
            .iter()
            .position(|pixel| pixel.as_ref().is_some_and(|pixel| pixel.face == 0))
            .unwrap_or_else(|| panic!("{name}: no pixel lies in the reference face"));
        let view_turn = view_turn(
            &mut reference,
            geometry,
            &vertices,
            u32::try_from(home).unwrap_or(u32::MAX),
        );
        Self {
            name,
            domain,
            face,
            geometry,
            exact,
            stabilizer,
            reflections,
            vertices,
            faces,
            pixels,
            neighbours,
            view_turn,
        }
    }

    fn carrying(&self, arrangement: &Colouring, name: &str) -> Vec<Matrix> {
        let table = arrangement.table();
        let walls = reference_walls(&self.domain);
        let element_of = |word: &Word| {
            let coset = table.trace(0, word);
            let holders: Vec<usize> = (0..self.stabilizer.len())
                .filter(|&element| table.trace(0, &self.stabilizer[element]) == coset)
                .collect();
            match holders.as_slice() {
                &[element] => element,
                _ => panic!(
                    "{name}: {} elements of the stabilizer reach one coset",
                    holders.len()
                ),
            }
        };
        self.reflections
            .iter()
            .map(|reflection| {
                let element = element_of(reflection);
                let word: Word = reflection
                    .letters()
                    .iter()
                    .chain(self.stabilizer[element].inverse().letters())
                    .copied()
                    .collect();
                assert_eq!(table.trace(0, &word), Some(0), "{name}");
                word_matrix(&walls, &word)
            })
            .collect()
    }

    fn arrangement_wedges(&self, arrangement: &Colouring) -> Vec<Option<usize>> {
        let mut reference = Reference::new();
        let name = format!("{} {}", self.name, arrangement.id());
        let carrying = self.carrying(arrangement, &name);
        let corner = self.exact.corner;
        let probes = [
            self.vertices[corner].clone(),
            self.vertices[1 - corner].clone(),
            escher_reference::vector(self.domain.centre().to_array()),
        ];
        let mut placed: Vec<Option<Matrix>> = vec![None; self.faces.len()];
        placed[0] = Some(identity());
        let mut queue = vec![0];
        let mut repeats = 0;
        while let Some(face) = queue.pop() {
            let Some(matrix) = placed[face].clone() else {
                continue;
            };
            for step in &carrying {
                let moved = escher_reference::product(&matrix, step);
                let centre = approximate(&mut reference, &apply(&moved, &probes[0]));
                if !kept(self.geometry, centre) {
                    continue;
                }
                let target = find_face(&self.faces, centre).unwrap_or_else(|| {
                    panic!("{name}: a face of the walk is not a face of the tiling")
                });
                if let Some(earlier) = &placed[target] {
                    for probe in &probes {
                        let gap = relative_gap(
                            &mut reference,
                            &apply(earlier, probe),
                            &apply(&moved, probe),
                        );
                        assert!(
                            gap <= SAME_POINT,
                            "{name}: face {target} is reached with two pictures, {gap:e} apart"
                        );
                    }
                    repeats += 1;
                } else {
                    placed[target] = Some(moved);
                    queue.push(target);
                }
            }
        }
        assert!(
            repeats >= self.faces.len(),
            "{name}: {repeats} faces reached twice"
        );
        let elements: Vec<Vector> = self
            .exact
            .elements
            .iter()
            .map(|element| apply(&element.matrix, &probes[2]))
            .collect();
        placed
            .iter()
            .zip(&self.faces)
            .map(|(matrix, geometric)| {
                let matrix = matrix.as_ref()?;
                let relative = escher_reference::product(&geometric.inverse, matrix);
                let probe = apply(&relative, &probes[2]);
                let (element, gap) = elements
                    .iter()
                    .enumerate()
                    .map(|(index, image)| (index, relative_gap(&mut reference, image, &probe)))
                    .min_by(|first, second| first.1.total_cmp(&second.1))?;
                assert!(
                    gap <= SAME_POINT,
                    "{name}: a face is placed by no element of the stabilizer, {gap:e}"
                );
                Some(element)
            })
            .collect()
    }
}

fn render_whole_face(
    domain: &FundamentalDomain,
    face: &WholeFace,
    arrangement: &Colouring,
    paint: Paint,
) -> Vec<u8> {
    let fill = whole_face_fill(face.clone(), arrangement.clone(), paint);
    let [red, green, blue] = BACKGROUND;
    let options = RenderOptions::builder(Fill::WholeFace(fill))
        .size(SIZE, SIZE)
        .samples(1)
        .border_width(0.0)
        .shading(Shading::Flat)
        .background(Colour::from_srgb8(red, green, blue))
        .build()
        .unwrap_or_else(|error| panic!("{}: {error}", arrangement.id()));
    renderer(domain, options).render().0
}

struct PictureFrame {
    origin: DVec2,
    axes: DMat2,
}

impl PictureFrame {
    fn new(reference: &mut Reference, exact: &ExactFace) -> Self {
        let placement = ExactPlacement::new(&exact.polygon, PICTURE, PICTURE);
        let [origin, across, down] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
            .map(|[x, y]| DVec2::from_array(placement.position(reference, &[num(x), num(y)])));
        Self {
            origin,
            axes: DMat2::from_cols(across - origin, down - origin),
        }
    }

    fn image(&self, point: DVec2) -> DVec2 {
        self.origin + self.axes * point
    }

    fn flat(&self, position: DVec2) -> DVec2 {
        self.axes.inverse() * (position - self.origin)
    }

    fn marked_picture(&self) -> Vec<u8> {
        let shade = |coordinate: u32| u8::try_from(96 + coordinate * 128 / PICTURE).unwrap_or(0);
        (0..PICTURE * PICTURE)
            .flat_map(|index| {
                let (column, row) = (index % PICTURE, index / PICTURE);
                let point = self.flat(DVec2::new(f64::from(column) + 0.5, f64::from(row) + 0.5));
                MARK_CENTRES
                    .iter()
                    .zip(MARK_COLOURS)
                    .find(|(centre, _)| point.distance(DVec2::from_array(**centre)) <= MARK_RADIUS)
                    .map_or([shade(column), shade(row), 160], |(_, colour)| colour)
            })
            .collect()
    }
}

fn turn_of(points: [DVec2; 3]) -> f64 {
    let [first, second, third] = points;
    (second - first).perp_dot(third - first).signum()
}

fn mark_of(pixel: &[u8]) -> Option<usize> {
    MARK_COLOURS.iter().position(|colour| {
        pixel
            .iter()
            .zip(colour)
            .all(|(&found, &wanted)| found.abs_diff(wanted) <= MARK_MATCH)
    })
}

fn rendered_turn(tiling: &Tiling, pixels: &[u8], face: usize) -> Option<(f64, [u32; 3])> {
    let mut sums = [(DVec2::ZERO, 0_u32); 3];
    for (index, geometry) in (0..).zip(&tiling.pixels) {
        if geometry
            .as_ref()
            .is_none_or(|geometry| geometry.face != face)
        {
            continue;
        }
        let (column, row) = (index % SIZE, index / SIZE);
        if let Some(mark) = mark_of(pixel_at(pixels, SIZE, column, row)) {
            sums[mark].0 += screen(tiling.geometry, [SIZE, SIZE], column, row);
            sums[mark].1 += 1;
        }
    }
    let counts = sums.map(|(_, count)| count);
    (counts.iter().all(|&count| count >= LEAST_MARK_PIXELS)).then(|| {
        (
            turn_of(sums.map(|(sum, count)| sum / f64::from(count))),
            counts,
        )
    })
}

fn rounding_bounds() -> Vec<f64> {
    let mut reference = Reference::new();
    (0..u32::from(u8::MAX))
        .map(|byte| {
            let encoded = escher_reference::div(&int(2 * byte + 1), &int(510));
            let linear = reference.srgb_decode_unit(&encoded);
            reference.approximate(&linear)
        })
        .collect()
}

fn rounded(value: f64, bounds: &[f64]) -> u8 {
    u8::try_from(bounds.partition_point(|&bound| bound < value)).unwrap_or(u8::MAX)
}

struct Agreement {
    compared: usize,
    off_by_one: usize,
    widest_off: f64,
}

impl Agreement {
    fn check(&mut self, name: &str, found: &[u8], expected: [f64; 3], tables: [&[f64]; 2]) {
        let [table, bounds] = tables;
        self.compared += 1;
        let mut differs = false;
        for (&byte, value) in found.iter().zip(expected) {
            let wanted = rounded(value, bounds);
            if byte == wanted {
                continue;
            }
            assert_eq!(
                byte.abs_diff(wanted),
                1,
                "{name}: {found:?} is more than one step from the oracle {expected:?}"
            );
            let low = usize::from(byte.min(wanted));
            let off = (value - bounds[low]).abs() / (table[low + 1] - table[low]);
            assert!(
                off <= ROUNDING_BAND,
                "{name}: {found:?} differs from the oracle {expected:?} {off:e} of a step from the rounding bound"
            );
            self.widest_off = self.widest_off.max(off);
            differs = true;
        }
        if differs {
            self.off_by_one += 1;
        }
    }
}

fn assert_handedness(
    tiling: &Tiling,
    arrangement: &Colouring,
    name: &str,
    pixels: &[u8],
) -> [usize; 2] {
    let picture_turn = turn_of(MARK_CENTRES.map(DVec2::from_array));
    let (home, counts) = rendered_turn(tiling, pixels, 0)
        .unwrap_or_else(|| panic!("{name}: the reference face does not show the three marks"));
    assert_eq!(
        home,
        picture_turn * tiling.view_turn,
        "{name}: the reference face shows the marks with {counts:?} pixels mirrored"
    );
    let edges = tiling
        .face
        .edges(arrangement)
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    let mut checked = [0; 2];
    for (edge, meeting) in edges.iter().enumerate() {
        let Some((turn, _)) = rendered_turn(tiling, pixels, tiling.neighbours[edge]) else {
            continue;
        };
        let wanted = if meeting.mirrored() { -home } else { home };
        assert_eq!(
            turn,
            wanted,
            "{name}: the face across edge {edge}, mirrored {}, shows the marks the other way",
            meeting.mirrored()
        );
        checked[usize::from(meeting.mirrored())] += 1;
    }
    for (mirrored, checked) in [false, true].into_iter().zip(checked) {
        assert!(
            checked > 0 || edges.iter().all(|meeting| meeting.mirrored() != mirrored),
            "{name}: no face across an edge with mirrored {mirrored} shows the three marks"
        );
    }
    checked
}

fn assert_whole_picture(text: &str, marks: [bool; 3], least: usize) {
    let tiling = Tiling::new(text, marks);
    let table = linear_table();
    let bounds = rounding_bounds();
    let mut reference = Reference::new();
    let frame = PictureFrame::new(&mut reference, &tiling.exact);
    let picture = frame.marked_picture();
    let image = image_of(PICTURE, PICTURE, &picture);
    let turns: Vec<DMat2> = tiling
        .exact
        .elements
        .iter()
        .map(|element| {
            let turn = flat_transpose(&element.flat);
            DMat2::from_cols_array(&[
                reference.approximate(&turn[0][0]),
                reference.approximate(&turn[1][0]),
                reference.approximate(&turn[0][1]),
                reference.approximate(&turn[1][1]),
            ])
        })
        .collect();
    let mut agreement = Agreement {
        compared: 0,
        off_by_one: 0,
        widest_off: 0.0,
    };
    let (mut turned, mut mirrored) = (0, 0);
    let arrangements = arrangements(&tiling.face);
    assert!(!arrangements.is_empty(), "{}", tiling.name);
    for arrangement in &arrangements {
        let name = format!("{} {}", tiling.name, arrangement.id());
        let wedges = tiling.arrangement_wedges(arrangement);
        let pixels = render_whole_face(
            &tiling.domain,
            &tiling.face,
            arrangement,
            Paint::Image(image.clone()),
        );
        let before = agreement.compared;
        for ((index, geometry), column) in (0..).zip(&tiling.pixels).zip((0..SIZE).cycle()) {
            let Some(geometry) = geometry else {
                continue;
            };
            let element = wedges[geometry.face]
                .unwrap_or_else(|| panic!("{name}: face {} is not reached", geometry.face));
            let position = frame.image(turns[element] * geometry.picture);
            let expected = bilinear(&picture, PICTURE, PICTURE, position, &table);
            let row = index / SIZE;
            let found = pixel_at(&pixels, SIZE, column, row);
            agreement.check(
                &format!("{name}: column {column} row {row}"),
                found,
                expected,
                [&table, &bounds],
            );
        }
        let compared = agreement.compared - before;
        assert!(compared >= least, "{name}: {compared} pixels compared");
        let [turned_here, mirrored_here] = assert_handedness(&tiling, arrangement, &name, &pixels);
        turned += turned_here;
        mirrored += mirrored_here;
    }
    println!(
        "{}: {} arrangements, {} pixels compared, {} of them one step off at most {:e} of a step from a rounding bound, view turn {}, {turned} turned and {mirrored} mirrored neighbours checked",
        tiling.name,
        arrangements.len(),
        agreement.compared,
        agreement.off_by_one,
        agreement.widest_off,
        tiling.view_turn
    );
    assert!(
        agreement.off_by_one * OFF_BY_ONE_SHARE <= agreement.compared,
        "{}: {} of {} pixels are one step off",
        tiling.name,
        agreement.off_by_one,
        agreement.compared
    );
}

#[test]
fn every_hexagon_shows_the_picture_once_turned_or_mirrored_as_each_arrangement_says() {
    assert_whole_picture("*632", REGULAR, 30_000);
}

#[test]
fn every_triangle_of_the_hexagon_dual_shows_the_picture_once() {
    assert_whole_picture("*632", DUAL, 30_000);
}

#[test]
fn every_square_shows_the_picture_once_turned_or_mirrored_as_each_arrangement_says() {
    assert_whole_picture("*442", REGULAR, 30_000);
}

#[test]
fn every_heptagon_shows_the_picture_once_turned_or_mirrored_as_each_arrangement_says() {
    assert_whole_picture("*732", REGULAR, 20_000);
}

#[test]
fn every_pentagon_of_the_dodecahedron_shows_the_picture_once() {
    assert_whole_picture("*532", REGULAR, 20_000);
}

fn assert_wedge_continuity(text: &str, radius: f64, least: usize) {
    let bound = WEDGE_STEP;
    let tiling = Tiling::new(text, REGULAR);
    let arrangement = arrangements(&tiling.face)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{} has no arrangement", tiling.name));
    let pixels = render_whole_face(
        &tiling.domain,
        &tiling.face,
        &arrangement,
        Paint::Image(gradient(0, PICTURE)),
    );
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    let (mut across, mut within, mut largest, mut largest_within) = (0, 0, 0_u8, 0_u8);
    for index in 0..tiling.pixels.len() {
        let Some(first) = &tiling.pixels[index] else {
            continue;
        };
        let neighbours = [
            (index % size + 1 < size).then_some(index + 1),
            (index + size < tiling.pixels.len()).then_some(index + size),
        ];
        for other in neighbours.into_iter().flatten() {
            let Some(second) = &tiling.pixels[other] else {
                continue;
            };
            if first.face != second.face
                || (tiling.geometry != Geometry::Euclidean
                    && first.radius.max(second.radius) > radius)
            {
                continue;
            }
            let step = pixels[3 * index..3 * index + 3]
                .iter()
                .zip(&pixels[3 * other..3 * other + 3])
                .map(|(first, second)| first.abs_diff(*second))
                .max()
                .unwrap_or(0);
            if first.wedge == second.wedge {
                within += 1;
                largest_within = largest_within.max(step);
            } else {
                across += 1;
                largest = largest.max(step);
                assert!(
                    step <= bound,
                    "{}: pixels {index} and {other} differ by {step} across a line between wedges",
                    tiling.name
                );
            }
        }
    }
    println!(
        "{}: {across} pixel pairs across the lines between wedges differ by at most {largest}, allowed {bound}, {within} pairs inside a wedge by at most {largest_within}",
        tiling.name
    );
    assert!(across >= least, "{}: {across} pairs", tiling.name);
}

#[test]
fn a_smooth_picture_runs_on_across_the_wedges_of_each_heptagon() {
    assert_wedge_continuity("*732", 0.6, 1000);
}

#[test]
fn a_smooth_picture_runs_on_across_the_wedges_of_each_hexagon() {
    assert_wedge_continuity("*632", 1.0, 3000);
}

#[test]
fn a_smooth_picture_runs_on_across_the_wedges_of_each_pentagon_of_the_dodecahedron() {
    assert_wedge_continuity("*532", 0.7, 500);
}

fn default_view_position(reference: &mut Reference, geometry: Geometry, point: &Vector) -> DVec2 {
    let divisor = match geometry {
        Geometry::Hyperbolic => escher_reference::add(&int(1), &point[2]),
        Geometry::Euclidean => point[2].clone(),
        Geometry::Spherical => {
            assert!(
                point[2].is_positive(),
                "the point faces away from the viewer"
            );
            int(1)
        }
    };
    let [first, second] =
        [0, 1].map(|axis| reference.approximate(&escher_reference::div(&point[axis], &divisor)));
    let pixel = view_pixel(geometry, SIZE, SIZE);
    let half = f64::from(SIZE) / 2.0;
    DVec2::new(half + first / pixel, half - second / pixel)
}

fn upright_picture(centre: DVec2) -> Vec<u8> {
    let features = [
        (centre - DVec2::new(0.0, FEATURE_OFFSET), MARK_COLOURS[0]),
        (centre + DVec2::new(FEATURE_OFFSET, 0.0), MARK_COLOURS[2]),
    ];
    (0..PICTURE * PICTURE)
        .flat_map(|index| {
            let position = DVec2::new(
                f64::from(index % PICTURE) + 0.5,
                f64::from(index / PICTURE) + 0.5,
            );
            features
                .iter()
                .find(|(middle, _)| position.distance(*middle) <= FEATURE_RADIUS)
                .map_or(FEATURE_BACKGROUND, |&(_, colour)| colour)
        })
        .collect()
}

fn signed_angle(from: DVec2, to: DVec2) -> f64 {
    from.perp_dot(to).atan2(from.dot(to))
}

fn upright_angles(text: &str) -> ([f64; 2], [u32; 2]) {
    let mut reference = Reference::new();
    let domain = domain_with_settings(text, &[]);
    let face = whole_face(&domain, REGULAR);
    let geometry = domain.geometry();
    let exact = exact_face(&mut reference, text, &domain, REGULAR);
    let centre = DVec2::from_array(
        ExactPlacement::new(&exact.polygon, PICTURE, PICTURE)
            .position(&mut reference, &[int(0), int(0)]),
    );
    let arrangement = arrangements(&face)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{text} has no arrangement"));
    let picture = upright_picture(centre);
    let pixels = render_whole_face(
        &domain,
        &face,
        &arrangement,
        Paint::Image(image_of(PICTURE, PICTURE, &picture)),
    );
    let walls = reference_walls(&domain);
    let (stabilizer, _) = stabilizer_words(0, face.index() / 2);
    let vertex = escher_reference::vector(domain.vertices()[0].to_array());
    let edges: Vec<DVec3> = edge_walls(&walls, &stabilizer, 0, &vertex)
        .iter()
        .map(|wall| approximate(&mut reference, wall))
        .collect();
    let mut sums = [(DVec2::ZERO, 0_u32); 2];
    for row in 0..SIZE {
        for column in 0..SIZE {
            let inside = screen_point(geometry, screen(geometry, [SIZE, SIZE], column, row))
                .is_some_and(|point| edges.iter().all(|wall| wall.dot(point) > 0.0));
            let feature = match mark_of(pixel_at(&pixels, SIZE, column, row)) {
                Some(0) => 0,
                Some(2) => 1,
                _ => continue,
            };
            if inside {
                sums[feature].0 += DVec2::new(f64::from(column) + 0.5, f64::from(row) + 0.5);
                sums[feature].1 += 1;
            }
        }
    }
    let home = default_view_position(&mut reference, geometry, &vertex);
    let counts = sums.map(|(_, count)| count);
    let angles = [(DVec2::NEG_Y, sums[0]), (DVec2::X, sums[1])].map(|(wanted, (sum, count))| {
        assert!(
            count >= LEAST_FEATURE_PIXELS,
            "{text}: a feature has {count} pixels in the reference face"
        );
        signed_angle(wanted, sum / f64::from(count) - home)
    });
    let foot = &exact.triangle[2];
    let turn = reference
        .approximate(&foot[1])
        .atan2(reference.approximate(&foot[0]));
    println!(
        "{text}: the features lie {:.4} and {:.4} degrees from straight up and straight right, with {counts:?} pixels, allowed {:.1} degrees, with the face mask turned to {:.4} degrees",
        angles[0].to_degrees(),
        angles[1].to_degrees(),
        UPRIGHT_ANGLE.to_degrees(),
        turn.to_degrees()
    );
    for angle in angles {
        assert!(
            angle.abs() <= UPRIGHT_ANGLE,
            "{text}: a feature lies {:.4} degrees from its direction",
            angle.to_degrees()
        );
    }
    (angles, counts)
}

#[test]
fn a_picture_painted_upright_appears_upright_on_the_reference_heptagon() {
    upright_angles("*732");
}

#[test]
fn a_picture_painted_upright_appears_upright_and_unturned_on_the_reference_hexagon() {
    let (angles, _) = upright_angles("*632");
    for angle in angles {
        assert!(
            angle.abs() <= PLANE_ANGLE,
            "*632: a feature lies {:.4} degrees from its direction in the plane",
            angle.to_degrees()
        );
    }
}

#[test]
fn a_picture_painted_upright_appears_upright_on_the_reference_pentagon_of_the_dodecahedron() {
    upright_angles("*532");
}

fn walk_faces(
    reference: &mut Reference,
    geometry: Geometry,
    reflections: &[Matrix],
    centre: &Vector,
) -> Vec<GeometricFace> {
    let mut faces = vec![GeometricFace {
        matrix: identity(),
        inverse: identity(),
        centre: approximate(reference, centre),
    }];
    let mut next = 0;
    while next < faces.len() {
        for reflection in reflections {
            let matrix = escher_reference::product(&faces[next].matrix, reflection);
            let moved = approximate(reference, &apply(&matrix, centre));
            if !kept(geometry, moved) || find_face(&faces, moved).is_some() {
                continue;
            }
            let inverse = escher_reference::product(reflection, &faces[next].inverse);
            faces.push(GeometricFace {
                matrix,
                inverse,
                centre: moved,
            });
        }
        next += 1;
    }
    faces
}

struct PixelOracle<'a> {
    name: String,
    geometry: Geometry,
    faces: &'a [GeometricFace],
    edge_walls: Vec<DVec3>,
    wedges: Vec<Vec<DVec3>>,
    exact: &'a ExactFace,
    mask: crate::reference::ExactMask,
    order: u32,
}

impl PixelOracle<'_> {
    fn pixel(&self, reference: &mut Reference, column: u32, row: u32) -> Option<PixelGeometry> {
        let name = &self.name;
        let geometry = self.geometry;
        let position = screen(geometry, [SIZE, SIZE], column, row);
        if geometry != Geometry::Euclidean && position.length() > RADIUS {
            return None;
        }
        let point = screen_point(geometry, position)?;
        let (face, _) = self
            .faces
            .iter()
            .enumerate()
            .map(|(index, face)| (index, closeness(geometry, point, face.centre)))
            .min_by(|first, second| first.1.total_cmp(&second.1))?;
        let exact_position = exact_point(reference, geometry, position);
        let local = apply(&self.faces[face].inverse, &exact_position);
        let near = approximate(reference, &local);
        let values: Vec<f64> = self.edge_walls.iter().map(|wall| wall.dot(near)).collect();
        if values.iter().any(|value| value.abs() <= EDGE_CLEARANCE) {
            return None;
        }
        assert!(
            values.iter().all(|&value| value > 0.0),
            "{name}: column {column} row {row} lies outside its nearest face"
        );
        let (wedge, margin) = self
            .wedges
            .iter()
            .enumerate()
            .map(|(index, covectors)| {
                let margin = covectors
                    .iter()
                    .map(|covector| -covector.dot(near))
                    .fold(f64::INFINITY, f64::min);
                (index, margin)
            })
            .max_by(|first, second| first.1.total_cmp(&second.1))?;
        assert!(
            margin > -IN_WEDGE,
            "{name}: column {column} row {row} lies in no image of the domain"
        );
        let element = &self.exact.elements[wedge];
        let inverse = &self.exact.elements[inverse_index(wedge, self.order)].matrix;
        let flat = self.mask.map(reference, &apply(inverse, &local));
        let picture = flat_apply(&element.flat, &flat);
        Some(PixelGeometry {
            face,
            wedge,
            picture: DVec2::new(
                reference.approximate(&picture[0]),
                reference.approximate(&picture[1]),
            ),
            radius: position.length(),
        })
    }
}

fn inverse_index(element: usize, order: u32) -> usize {
    let order = usize::try_from(order).unwrap_or(usize::MAX);
    if element % 2 == 1 {
        element
    } else {
        2 * ((order - element / 2) % order)
    }
}

fn neighbours(
    reference: &mut Reference,
    exact: &ExactFace,
    walls: &[Wall],
    centre: &Vector,
    faces: &[GeometricFace],
    name: &str,
) -> Vec<usize> {
    let count = exact.polygon.len();
    let edge_side = if exact.corner == 0 { 2 } else { 0 };
    (0..count)
        .map(|edge| {
            let start = &exact.polygon[(edge + count - 1) % count];
            let end = &exact.polygon[edge];
            let middle = [0, 1].map(|axis| {
                escher_reference::div(&escher_reference::add(&start[axis], &end[axis]), &int(2))
            });
            let element = exact
                .elements
                .iter()
                .find(|element| {
                    let moved = flat_apply(&element.flat, &exact.triangle[2]);
                    (0..2).all(|axis| {
                        let gap = escher_reference::sub(&moved[axis], &middle[axis]);
                        reference.approximate(&gap).abs() <= SAME_POINT
                    })
                })
                .unwrap_or_else(|| panic!("{name}: no element carries the foot to edge {edge}"));
            let across = apply(
                &escher_reference::product(&element.matrix, &walls[edge_side].pairing),
                centre,
            );
            find_face(faces, approximate(reference, &across))
                .unwrap_or_else(|| panic!("{name}: the face across edge {edge} is not walked"))
        })
        .collect()
}

fn view_turn(
    reference: &mut Reference,
    geometry: Geometry,
    vertices: &[Vector],
    pixel: u32,
) -> f64 {
    let corners: Vec<DVec3> = vertices
        .iter()
        .map(|vertex| approximate(reference, vertex))
        .collect();
    let order = DMat3::from_cols(corners[0], corners[1], corners[2]).determinant();
    let (column, row) = (pixel % SIZE, pixel / SIZE);
    let at = |column: u32, row: u32| {
        screen_point(geometry, screen(geometry, [SIZE, SIZE], column, row))
            .unwrap_or_else(|| panic!("pixel {column} {row} has no point"))
    };
    let here = at(column, row);
    let view = DMat3::from_cols(here, at(column + 1, row) - here, at(column, row - 1) - here)
        .determinant();
    order.signum() * view.signum()
}

fn edge_walls(walls: &[Wall], stabilizer: &[Word], corner: usize, centre: &Vector) -> Vec<Vector> {
    let edge = if corner == 0 { 2 } else { 0 };
    stabilizer
        .iter()
        .step_by(2)
        .map(|turn| {
            let covector =
                transpose_apply(&word_matrix(walls, &turn.inverse()), &walls[edge].covector);
            if dot(&covector, centre).is_negative() {
                escher_reference::scale(&int(1).neg(), &covector)
            } else {
                covector
            }
        })
        .collect()
}

fn wedge_walls(walls: &[Wall], exact: &ExactFace, order: u32) -> Vec<Vec<Vector>> {
    (0..exact.elements.len())
        .map(|element| {
            let inverse = &exact.elements[inverse_index(element, order)].matrix;
            walls
                .iter()
                .map(|wall| transpose_apply(inverse, &wall.covector))
                .collect()
        })
        .collect()
}

fn refusal<T>(result: Result<T, escher_render::Error>) -> (ErrorKind, String) {
    match result {
        Ok(_) => panic!("the input is accepted"),
        Err(error) => (error.kind(), error.to_string()),
    }
}

fn invalid(text: &str) -> (ErrorKind, String) {
    (ErrorKind::InvalidOption, text.to_owned())
}

#[test]
fn whole_face_refuses_a_construction_that_is_not_a_regular_tiling_or_of_another_domain() {
    const NOT_REGULAR: &str = "cannot build whole face: the construction is not a regular tiling";
    for (text, marks) in [
        ("*732", [true, true, true]),
        ("*732", [false, false, true]),
        ("*732", [true, true, false]),
        ("*522", REGULAR),
        ("*333", DUAL),
    ] {
        let domain = domain_with_settings(text, &[]);
        let wythoff = Wythoff::new(&domain, marks)
            .unwrap_or_else(|error| panic!("{text} {}: {error}", marks_text(marks)));
        assert_eq!(
            refusal(WholeFace::new(&domain, wythoff)),
            invalid(NOT_REGULAR),
            "{text} {}",
            marks_text(marks)
        );
    }
    let heptagons = domain_with_settings("*732", &[]);
    let other = domain_with_settings("*742", &[]);
    let wythoff = Wythoff::new(&heptagons, REGULAR).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        refusal(WholeFace::new(&other, wythoff.clone())),
        invalid("cannot build whole face: the Wythoff construction was built for another domain")
    );
    let not_regular =
        Wythoff::new(&heptagons, [true, true, true]).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        refusal(WholeFace::new(&other, not_regular)),
        invalid("cannot build whole face: the Wythoff construction was built for another domain")
    );
    let again = domain_with_settings("*732", &[]);
    assert!(WholeFace::new(&again, wythoff).is_ok());
}

#[test]
fn an_arrangement_is_checked_for_its_generators_its_relators_its_colours_and_its_stabilizer_in_order()
 {
    let domain = domain_with_settings("*632", &[]);
    let face = whole_face(&domain, REGULAR);
    let trivial = find_colouring_of(&domain, 1, "1a");
    let twelve = colourings_of(&domain, 12);
    let not_arrangement = twelve
        .iter()
        .find(|colouring| face.edges(colouring).is_err())
        .unwrap_or_else(|| panic!("every colouring with 12 colours is an arrangement"))
        .clone();
    let square = domain_with_settings("*3333", &[]);
    let four_sides = find_colouring_of(&square, 1, "1a");
    let other_relators = arrangements(&whole_face(&domain_with_settings("*642", &[]), REGULAR))
        .into_iter()
        .find(|colouring| {
            face.edges(colouring)
                .is_err_and(|error| error.to_string().contains("relator"))
        })
        .unwrap_or_else(|| panic!("every arrangement of *642 fits the relators of *632"));
    let cases = [
        (
            &four_sides,
            "the colouring has 4 generators, and the domain has 3 sides".to_owned(),
        ),
        (
            &other_relators,
            "relator 4 of the side presentation of the domain moves a coset of the colouring"
                .to_owned(),
        ),
        (
            &trivial,
            "the colouring has 1 coset, and an arrangement of these faces has 12".to_owned(),
        ),
        (
            &not_arrangement,
            "the colouring is not an arrangement of the faces".to_owned(),
        ),
    ];
    for (colouring, text) in &cases {
        assert_eq!(
            refusal(face.edges(colouring)),
            invalid(&format!("cannot read edge pairing: {text}"))
        );
        assert_eq!(
            refusal(face.render_mask(colouring, 0, 0)),
            invalid(&format!("cannot render face mask: {text}"))
        );
        assert_eq!(
            refusal(escher_render::WholeFaceFill::new(
                face.clone(),
                (*colouring).clone(),
                Vec::new()
            )),
            invalid(&format!("cannot build whole-face fill: {text}"))
        );
    }
}

#[test]
fn the_face_mask_refuses_a_size_outside_the_limits_and_the_fill_another_number_of_paints() {
    let face = whole_face(&domain_with_settings("*632", &[]), REGULAR);
    let arrangement = arrangements(&face)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no arrangement"));
    for (width, height, kind, text) in [
        (
            0,
            512,
            ErrorKind::InvalidOption,
            "the image size 0x512 is empty",
        ),
        (
            16385,
            1,
            ErrorKind::LimitExceeded,
            "the image size 16385x1 is larger than 16384 pixels on a side",
        ),
        (
            16384,
            4097,
            ErrorKind::LimitExceeded,
            "the image size 16384x4097 has more than 67108864 pixels",
        ),
    ] {
        assert_eq!(
            refusal(face.render_mask(&arrangement, width, height)),
            (kind, format!("cannot render face mask: {text}"))
        );
    }
    let paint = Paint::Solid(Colour::WHITE);
    for (paints, text) in [
        (
            Vec::new(),
            "the arrangement needs 1 paint, and 0 were given",
        ),
        (
            vec![paint.clone(), paint],
            "the arrangement needs 1 paint, and 2 were given",
        ),
    ] {
        assert_eq!(
            refusal(escher_render::WholeFaceFill::new(
                face.clone(),
                arrangement.clone(),
                paints
            )),
            invalid(&format!("cannot build whole-face fill: {text}"))
        );
    }
}

fn find_colouring_of(domain: &FundamentalDomain, colours: u32, id: &str) -> Colouring {
    crate::golden_images::find_colouring(&colourings_of(domain, colours), id)
}

#[test]
fn the_renderer_refuses_a_whole_face_fill_of_another_domain() {
    let heptagons = domain_with_settings("*732", &[]);
    let face = whole_face(&heptagons, REGULAR);
    let arrangement = arrangements(&face)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no arrangement"));
    let fill = whole_face_fill(face, arrangement, Paint::Solid(Colour::WHITE));
    let options = || {
        RenderOptions::builder(Fill::WholeFace(fill.clone()))
            .size(16, 16)
            .build()
            .unwrap_or_else(|error| panic!("{error}"))
    };
    let other = domain_with_settings("*742", &[]);
    assert_eq!(
        refusal(escher_render::Renderer::new(&other, options())),
        invalid("cannot build renderer: the whole-face fill was built for another domain")
    );
    let again = domain_with_settings("*732", &[]);
    assert!(escher_render::Renderer::new(&again, options()).is_ok());
}

fn far_from_every_edge(tiling: &Tiling, column: u32, row: u32) -> bool {
    let face_at = |column: u32, row: u32| {
        tiling.pixels[usize::try_from(row * SIZE + column).unwrap_or(usize::MAX)]
            .as_ref()
            .map(|pixel| pixel.face)
    };
    let Some(face) = face_at(column, row) else {
        return false;
    };
    let window = |centre: u32| {
        centre.saturating_sub(SOLID_CLEARANCE)..=(centre + SOLID_CLEARANCE).min(SIZE - 1)
    };
    window(row).all(|other_row| {
        window(column).all(|other_column| face_at(other_column, other_row) == Some(face))
    })
}

#[test]
fn a_solid_paint_fills_every_face_and_leaves_the_background_outside_the_disk() {
    let tiling = Tiling::new("*732", REGULAR);
    let arrangement = arrangements(&tiling.face)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no arrangement"));
    let [red, green, blue] = PAINT;
    let fill = whole_face_fill(
        tiling.face.clone(),
        arrangement,
        Paint::Solid(Colour::from_srgb8(red, green, blue)),
    );
    let [red, green, blue] = BACKGROUND;
    let options = RenderOptions::builder(Fill::WholeFace(fill))
        .size(SIZE, SIZE)
        .samples(1)
        .background(Colour::from_srgb8(red, green, blue))
        .build()
        .unwrap_or_else(|error| panic!("{error}"));
    let (pixels, _) = renderer(&tiling.domain, options).render();
    let step = view_pixel(Geometry::Hyperbolic, SIZE, SIZE);
    let (mut painted, mut bordered, mut outside, mut interior) = (0, 0, 0, 0);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let found = pixel_at(&pixels, SIZE, column, row);
            let radius = screen(Geometry::Hyperbolic, [SIZE; 2], column, row).length();
            if radius > 1.0 + step {
                assert_eq!(found, BACKGROUND, "column {column} row {row}");
                outside += 1;
            } else if radius < 1.0 - step {
                if found == [0; 3] {
                    bordered += 1;
                } else {
                    assert_eq!(found, PAINT, "column {column} row {row}");
                    painted += 1;
                }
            }
            if far_from_every_edge(&tiling, column, row) {
                assert_eq!(
                    found, PAINT,
                    "column {column} row {row} lies inside a face, away from its edges"
                );
                interior += 1;
            }
        }
    }
    println!(
        "{painted} painted, {bordered} border, {outside} outside, and {interior} interior pixels"
    );
    assert!(painted >= 20_000 && bordered >= 3000 && outside >= 10_000 && interior >= 9000);
}
