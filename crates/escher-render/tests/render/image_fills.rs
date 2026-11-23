use escher_domain::FundamentalDomain;
use escher_geometry::hyperbolic;
use escher_group::Colouring;
use escher_render::{Colour, CosetFill, Fill, Image, Paint, RenderOptions, Shading, fold};
use escher_signature::Geometry;
use glam::{DVec2, DVec3};

use crate::common::{DEFAULT_CAP, plane_pixel};
use crate::golden_images::{
    GOLDEN_COSET_FILL, GOLDEN_GRID, bilinear, colouring, compare, coset_fill, disk_pixel,
    domain_with_settings, exact_mask, golden_colouring, image_of, linear_table, masks,
    painted_image, pixel_centre, reference_walls, renderer, texel_index,
};
use crate::reference::{self, ExactMask, Measure, ReferenceExt};
use escher_reference::{Reference, Vector, apply, matrix, num, vector};

const SIZE: u32 = 96;

const SQUARE: [u32; 2] = [SIZE, SIZE];

const WALL_CLEARANCE: f64 = 1e-6;

struct Scene {
    name: String,
    domain: FundamentalDomain,
    colouring: Colouring,
}

fn scene(signature: &str, id: &str, valid: bool) -> Scene {
    let domain = domain_with_settings(signature, &[]);
    let colouring = if valid {
        golden_colouring(&domain, id)
    } else {
        colouring(&domain, id)
    };
    Scene {
        name: format!("{signature} {id}"),
        domain,
        colouring,
    }
}

fn coset_fill_of(scene: &Scene, paints: Vec<Paint>) -> CosetFill {
    CosetFill::new(&scene.domain, scene.colouring.clone(), paints)
        .unwrap_or_else(|error| panic!("{}: {error}", scene.name))
}

struct Painted {
    pixels: Vec<Vec<u8>>,
    fill: CosetFill,
}

fn painted(scene: &Scene, size: u32) -> Painted {
    let masks = masks(&scene.domain, &scene.colouring);
    let (pixels, paints) = (0..masks.count())
        .map(|colour| {
            let (pixels, image) = painted_image(&masks, colour, size, size);
            (pixels, Paint::Image(image))
        })
        .unzip();
    Painted {
        pixels,
        fill: coset_fill_of(scene, paints),
    }
}

fn render(
    scene: &Scene,
    fill: CosetFill,
    background: Colour,
    [width, height]: [u32; 2],
) -> Vec<u8> {
    let options = RenderOptions::builder(Fill::Cosets(fill))
        .size(width, height)
        .samples(1)
        .border_width(0.0)
        .shading(Shading::Flat)
        .background(background)
        .build()
        .unwrap_or_else(|error| panic!("{}: {error}", scene.name));
    renderer(&scene.domain, options).render().0
}

pub(crate) fn view_pixel(geometry: Geometry, width: u32, height: u32) -> f64 {
    match geometry {
        Geometry::Hyperbolic | Geometry::Spherical => disk_pixel(width, height),
        Geometry::Euclidean => plane_pixel(width, height),
    }
}

pub(crate) fn screen(
    geometry: Geometry,
    [width, height]: [u32; 2],
    column: u32,
    row: u32,
) -> DVec2 {
    pixel_centre(
        width,
        height,
        column,
        row,
        view_pixel(geometry, width, height),
    )
}

pub(crate) fn screen_point(geometry: Geometry, position: DVec2) -> Option<DVec3> {
    match geometry {
        Geometry::Hyperbolic => hyperbolic::from_disk(position),
        Geometry::Euclidean => Some(DVec3::new(position.x, position.y, 1.0)),
        Geometry::Spherical => {
            let height = 1.0 - position.length_squared();
            (height > 0.0).then(|| DVec3::new(position.x, position.y, height.sqrt()))
        }
    }
}

pub(crate) fn exact_point(
    reference: &mut Reference,
    geometry: Geometry,
    position: DVec2,
) -> Vector {
    match geometry {
        Geometry::Hyperbolic => reference::from_disk(position.to_array()),
        Geometry::Euclidean => vector([position.x, position.y, 1.0]),
        Geometry::Spherical => reference.lift(position.to_array(), [0.0; 3]),
    }
}

fn expected_value(
    reference: &mut Reference,
    exact: &ExactMask,
    point: &Vector,
    pixels: &[u8],
    background: Colour,
    table: &[f64],
) -> [f64; 3] {
    let flat = exact.map(reference, point);
    let [across, down] = exact.image_position(reference, &flat, SIZE, SIZE);
    let inside =
        (0.0..=f64::from(SIZE)).contains(&across) && (0.0..=f64::from(SIZE)).contains(&down);
    if inside {
        bilinear(pixels, SIZE, SIZE, DVec2::new(across, down), table)
    } else {
        background.to_srgb8().map(|byte| table[usize::from(byte)])
    }
}

pub(crate) fn within_one_step(pixel: &[u8], expected: [f64; 3], table: &[f64]) -> bool {
    pixel.iter().zip(expected).all(|(&byte, value)| {
        let low = table[usize::from(byte.saturating_sub(1))];
        let high = table[usize::from(byte.saturating_add(1))];
        (low..=high).contains(&value)
    })
}

pub(crate) fn pixel_at(pixels: &[u8], width: u32, column: u32, row: u32) -> &[u8] {
    let index = usize::try_from(3 * (row * width + column)).unwrap_or(usize::MAX);
    &pixels[index..index + 3]
}

const MIRROR_SIZE: u32 = 192;

#[test]
fn tile_across_side_0_shows_the_image_mirrored_in_the_wall_of_side_0() {
    let table = linear_table();
    let mut reference = Reference::new();
    let (mut total, mut marked) = (0, 0);
    for scene in [
        scene("*732", "1a", false),
        scene("*632", "1a", false),
        scene("*532", "1a", false),
    ] {
        let geometry = scene.domain.geometry();
        let painted = painted(&scene, MIRROR_SIZE);
        let drawn = &painted.pixels[0];
        let fill_colour = GOLDEN_COSET_FILL[0];
        let rendered = render(&scene, painted.fill, Colour::WHITE, [MIRROR_SIZE; 2]);
        let exact = exact_mask(&mut reference, &scene.domain);
        let mirror = matrix(scene.domain.sides()[0].pairing().to_cols_array());
        let (mut compared, mut mark) = (0, 0);
        for row in 0..MIRROR_SIZE {
            for column in 0..MIRROR_SIZE {
                let position = screen(geometry, [MIRROR_SIZE; 2], column, row);
                let Some(point) = screen_point(geometry, position) else {
                    continue;
                };
                let mut sides = Vec::new();
                let result = fold(&scene.domain, point, DEFAULT_CAP, |side| sides.push(side));
                if !result.converged() || sides != [0] {
                    continue;
                }
                let mirrored = apply(&mirror, &exact_point(&mut reference, geometry, position));
                let flat = exact.map(&mut reference, &mirrored);
                let [across, down] =
                    exact.image_position(&mut reference, &flat, MIRROR_SIZE, MIRROR_SIZE);
                let texel = DVec2::new(across, down);
                let expected = bilinear(drawn, MIRROR_SIZE, MIRROR_SIZE, texel, &table);
                let pixel = pixel_at(&rendered, MIRROR_SIZE, column, row);
                assert!(
                    within_one_step(pixel, expected, &table),
                    "{} at column {column} row {row}: {pixel:?} is not {expected:?}",
                    scene.name
                );
                compared += 1;
                let [first, second] =
                    [texel.x, texel.y].map(|value| texel_index(value, MIRROR_SIZE));
                if pixel_at(drawn, MIRROR_SIZE, first, second) != fill_colour {
                    mark += 1;
                }
            }
        }
        assert!(
            compared >= 100 && mark >= 10,
            "{}: {compared} pixels, {mark} of the mark",
            scene.name
        );
        total += compared;
        marked += mark;
    }
    println!("{total} pixels across side 0 show the mirrored image, {marked} of them the mark");
}

fn assert_texture_contract(scene: &Scene) {
    let table = linear_table();
    let mut reference = Reference::new();
    let domain = &scene.domain;
    let geometry = domain.geometry();
    let measure = Measure::of(geometry);
    let walls = reference_walls(domain);
    let exact = exact_mask(&mut reference, domain);
    let painted = painted(scene, SIZE);
    let background = Colour::from_srgb8(10, 200, 30);
    let rendered = render(scene, painted.fill, background, SQUARE);
    let (mut compared, mut near, mut colours) = (0, 0, vec![0_usize; painted.pixels.len()]);
    for row in 0..SIZE {
        for column in (row % 2..SIZE).step_by(2) {
            let position = screen(geometry, SQUARE, column, row);
            let Some(point) = screen_point(geometry, position) else {
                continue;
            };
            let mut sides = Vec::new();
            let result = fold(domain, point, DEFAULT_CAP, |side| sides.push(side));
            assert!(result.converged(), "{} at {position}", scene.name);
            let exact_screen = exact_point(&mut reference, geometry, position);
            let folded = reference::fold_exact(&walls, exact_screen, DEFAULT_CAP);
            if walls.iter().any(|wall| {
                measure.wall_distance(&mut reference, &wall.covector, &folded.point)
                    < num(WALL_CLEARANCE)
            }) {
                near += 1;
                continue;
            }
            assert_eq!(folded.sides, sides, "{} at {position}", scene.name);
            let bytes: Vec<u8> = sides
                .iter()
                .map(|&side| u8::try_from(side).unwrap_or(u8::MAX))
                .collect();
            let colour = scene
                .colouring
                .colour_of(&bytes)
                .and_then(|colour| usize::try_from(colour).ok())
                .unwrap_or_else(|| panic!("{}: {bytes:?} has no colour", scene.name));
            let expected = expected_value(
                &mut reference,
                &exact,
                &folded.point,
                &painted.pixels[colour],
                background,
                &table,
            );
            let pixel = pixel_at(&rendered, SIZE, column, row);
            assert!(
                within_one_step(pixel, expected, &table),
                "{} colour {colour} at column {column} row {row}: {pixel:?} is not {expected:?}",
                scene.name
            );
            compared += 1;
            colours[colour] += 1;
        }
    }
    println!(
        "{}: {compared} pixels show the texture of the reference arithmetic, {colours:?} by colour, {near} within {WALL_CLEARANCE:e} of a wall",
        scene.name
    );
    assert!(compared >= 3000, "{}: {compared} pixels", scene.name);
    assert!(
        colours.iter().all(|&count| count >= 100),
        "{}: {colours:?}",
        scene.name
    );
}

#[test]
fn texture_contract_holds_on_a_hyperbolic_reflection_domain() {
    assert_texture_contract(&scene("*642", "4a", true));
}

#[test]
fn texture_contract_holds_on_a_hyperbolic_rotation_domain() {
    assert_texture_contract(&scene("3333", "4a", true));
}

#[test]
fn texture_contract_holds_on_a_wallpaper_domain() {
    assert_texture_contract(&scene("xx", "4c", true));
}

#[test]
fn texture_contract_holds_on_a_spherical_group_domain() {
    assert_texture_contract(&scene("3*2", "4a", true));
}

fn flat_image(width: u32, height: u32, colour: [u8; 3]) -> Image {
    image_of(
        width,
        height,
        &colour.repeat(usize::try_from(width * height).unwrap_or(usize::MAX)),
    )
}

#[test]
fn opaque_image_of_one_colour_gives_the_image_of_that_solid_paint() {
    let table = linear_table();
    for scene in [
        scene("*642", "4a", true),
        scene("*632", "4a", true),
        scene("*532", "5a", true),
    ] {
        let paints = GOLDEN_COSET_FILL
            .iter()
            .take(usize::try_from(scene.colouring.colours()).unwrap_or(usize::MAX))
            .map(|&colour| Paint::Image(flat_image(SIZE, SIZE, colour)))
            .collect();
        let options = |fill: CosetFill| {
            RenderOptions::builder(Fill::Cosets(fill))
                .size(SIZE, SIZE)
                .samples(GOLDEN_GRID)
                .build()
                .unwrap_or_else(|error| panic!("{}: {error}", scene.name))
        };
        let images = renderer(&scene.domain, options(coset_fill_of(&scene, paints)))
            .render()
            .0;
        let solid = renderer(
            &scene.domain,
            options(coset_fill(&scene.domain, scene.colouring.clone())),
        )
        .render()
        .0;
        let comparison = compare(&images, &solid, GOLDEN_GRID, &table);
        println!("{}: {comparison:?}", scene.name);
        assert!(
            comparison.within_allowance(),
            "{}: {comparison:?}",
            scene.name
        );
    }
}

fn translucent_image(width: u32, height: u32, texel: [u8; 4]) -> Image {
    let mut file = Vec::new();
    let mut encoder = png::Encoder::new(&mut file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let data = texel.repeat(usize::try_from(width * height).unwrap_or(usize::MAX));
    encoder
        .write_header()
        .and_then(|mut writer| {
            writer
                .write_image_data(&data)
                .and_then(|()| writer.finish())
        })
        .unwrap_or_else(|error| panic!("{error}"));
    Image::decode_png(file.as_slice()).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn translucent_image_gives_the_mean_of_its_colour_and_the_background_in_linear_light() {
    let table = linear_table();
    let triangle = scene("*632", "1a", false);
    let background = [10, 200, 30];
    let texel = [200, 40, 90, 128];
    let rendered = render(
        &triangle,
        coset_fill_of(
            &triangle,
            vec![Paint::Image(translucent_image(8, 8, texel))],
        ),
        Colour::from_srgb8(background[0], background[1], background[2]),
        SQUARE,
    );
    let opacity = 128.0 / 255.0;
    let expected = [0, 1, 2].map(|channel| {
        opacity * table[usize::from(texel[channel])]
            + (1.0 - opacity) * table[usize::from(background[channel])]
    });
    let pixels = rendered.as_chunks::<3>().0;
    for pixel in pixels {
        assert!(
            within_one_step(pixel, expected, &table),
            "{pixel:?} is not {expected:?}"
        );
    }
    assert_eq!(pixels.len(), usize::try_from(SIZE * SIZE).unwrap_or(0));
}
