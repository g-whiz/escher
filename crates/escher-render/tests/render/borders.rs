use astro_float_num::BigFloat;
use escher_geometry::hyperbolic::{distance, from_disk, inverse};
use escher_geometry::{euclidean, spherical};
use escher_reference::{Reference, Vector, add, int, mul, num, sqrt, sub, vector};
use escher_render::{BorderUnit, Colour, Fill, HyperbolicModel, RenderOptions, Shading, fold};
use glam::{DVec2, DVec3};

use crate::common::{
    DEFAULT_CAP, EUCLIDEAN_DOMAINS, SPHERICAL_GROUP_BOUNDS, euclidean_domain, euclidean_name,
    options, pixel_position, plane_pixel, viewing_matrix,
};
use crate::golden_images::{
    GOLDEN_FILL, MODEL_MAPS, ModelMap, PLAN_TRIPLES, WALLPAPER_DOMAINS, disk_pixel,
    domain_with_aspect, domain_with_settings, pixel_centre, reference_walls, renderer,
};
use crate::reference::{self, Measure, ReferenceExt, count_bound};

const SIZE: u32 = 192;

const RADIUS: f64 = 0.9;

const SAME_TILE: f64 = 1e-4;

const EUCLIDEAN_SIZE: u32 = 193;

const GRID_SIZE: u32 = 512;

const CENTRED_SIZE: u32 = 193;

const GRID_ASPECTS: [f64; 6] = [1.0, 4.0, 1.618, 0.125, 8.0, 0.3];

const GRID_HALF_WIDTHS: [f64; 3] = [0.5, 0.75, 1.5];

fn position(column: u32, row: u32) -> DVec2 {
    pixel_position(SIZE, column, row)
}

fn border_mask(domain: &escher_domain::FundamentalDomain, size: u32) -> Vec<bool> {
    rotated_border_mask(domain, size, [0.0; 3])
}

fn rotated_border_mask(
    domain: &escher_domain::FundamentalDomain,
    size: u32,
    degrees: [f64; 3],
) -> Vec<bool> {
    let [yaw, pitch, roll] = degrees.map(f64::to_radians);
    let options = options(
        RenderOptions::builder(Fill::One(Colour::from_srgb8(70, 130, 180)))
            .size(size, size)
            .samples(1)
            .rotation(yaw, pitch, roll)
            .shading(Shading::Flat),
    );
    let (pixels, _) = renderer(domain, options).render();
    pixels
        .as_chunks::<3>()
        .0
        .iter()
        .map(|pixel| *pixel == Colour::BLACK.to_srgb8())
        .collect()
}

fn tile_of(
    domain: &escher_domain::FundamentalDomain,
    point: DVec3,
    inverse: impl Fn(glam::DMat3) -> glam::DMat3,
) -> Option<DVec3> {
    let mut sides = Vec::new();
    let result = fold(domain, point, DEFAULT_CAP, |index| sides.push(index));
    result.converged().then(|| {
        sides.iter().rev().fold(domain.centre(), |centre, &index| {
            inverse(domain.sides()[index].pairing()) * centre
        })
    })
}

fn compare_with_tiles(
    name: &str,
    border: &[bool],
    tiles: &[Option<DVec3>],
    size: usize,
    differ: impl Fn(DVec3, DVec3) -> bool,
) -> (usize, usize) {
    let tile = |column: usize, row: usize| tiles[row * size + column];
    let mut change = vec![false; size * size];
    for row in 0..size {
        for column in 0..size {
            let Some(here) = tile(column, row) else {
                continue;
            };
            let neighbours = [(column + 1, row), (column, row + 1)];
            for (other_column, other_row) in neighbours {
                if other_column >= size || other_row >= size {
                    continue;
                }
                if let Some(there) = tile(other_column, other_row)
                    && differ(here, there)
                {
                    change[row * size + column] = true;
                    change[other_row * size + other_column] = true;
                }
            }
        }
    }
    let near = |marks: &[bool], column: usize, row: usize| {
        (row.saturating_sub(1)..=(row + 1).min(size - 1)).any(|other_row| {
            (column.saturating_sub(1)..=(column + 1).min(size - 1))
                .any(|other_column| marks[other_row * size + other_column])
        })
    };
    let mut changes = 0;
    let mut borders = 0;
    for row in 1..size - 1 {
        for column in 1..size - 1 {
            let inside = (row - 1..=row + 1).all(|other_row| {
                (column - 1..=column + 1)
                    .all(|other_column| tile(other_column, other_row).is_some())
            });
            if !inside {
                continue;
            }
            let index = row * size + column;
            if change[index] {
                changes += 1;
                assert!(
                    near(border, column, row),
                    "{name}: the tile changes at pixel ({column}, {row}) with no border pixel next to it"
                );
            }
            if border[index] {
                borders += 1;
                assert!(
                    near(&change, column, row),
                    "{name}: the border pixel ({column}, {row}) has no change of tile next to it"
                );
            }
        }
    }
    assert!(changes > 0 && borders > 0, "{name}");
    (changes, borders)
}

#[test]
fn border_pixels_lie_where_the_tile_of_a_sample_changes() {
    for text in PLAN_TRIPLES {
        assert_borders_where_the_tile_changes(text, &domain_with_settings(text, &[]));
    }
}

fn assert_borders_where_the_tile_changes(name: &str, domain: &escher_domain::FundamentalDomain) {
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    let border = border_mask(domain, SIZE);
    let mut tiles: Vec<Option<DVec3>> = Vec::with_capacity(size * size);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let position = position(column, row);
            let tile = (position.length() <= RADIUS)
                .then(|| from_disk(position))
                .flatten()
                .and_then(|point| tile_of(domain, point, inverse));
            tiles.push(tile);
        }
    }
    let (changes, borders) = compare_with_tiles(name, &border, &tiles, size, |here, there| {
        distance(here, there) >= SAME_TILE
    });
    println!(
        "{name}: {changes} pixels where the tile changes and {borders} border pixels within disk radius {RADIUS}"
    );
}

#[test]
fn border_pixels_in_each_model_lie_where_the_tile_of_a_sample_changes() {
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    for text in ["*732", "732"] {
        let domain = domain_with_settings(text, &[]);
        for map in &MODEL_MAPS {
            let model = map.model;
            let name = format!("{text} in the {model:?} model");
            let options = options(
                RenderOptions::builder(Fill::One(Colour::from_srgb8(70, 130, 180)))
                    .size(SIZE, SIZE)
                    .samples(1)
                    .model(model),
            );
            let (pixels, _) = renderer(&domain, options).render();
            let border: Vec<bool> = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .map(|pixel| *pixel == Colour::BLACK.to_srgb8())
                .collect();
            let mut tiles: Vec<Option<DVec3>> = Vec::with_capacity(size * size);
            for row in 0..SIZE {
                for column in 0..SIZE {
                    let tile = (map.to_disk)(map.position(SIZE, SIZE, column, row))
                        .map(|(disk, _)| disk)
                        .filter(|disk| disk.length() <= RADIUS)
                        .and_then(from_disk)
                        .and_then(|point| tile_of(&domain, point, inverse));
                    tiles.push(tile);
                }
            }
            let (changes, borders) =
                compare_with_tiles(&name, &border, &tiles, size, |here, there| {
                    distance(here, there) >= SAME_TILE
                });
            println!(
                "{name}: {changes} pixels where the tile changes and {borders} border pixels within disk radius {RADIUS}"
            );
        }
    }
}

#[test]
fn border_pixels_of_a_polygon_lie_where_the_tile_of_a_sample_changes() {
    assert_borders_where_the_tile_changes("*3333", &domain_with_settings("*3333", &[]));
}

fn plane_position(size: u32, column: u32, row: u32) -> DVec2 {
    pixel_centre(size, size, column, row, plane_pixel(size, size))
}

fn assert_plane_borders_where_the_tile_changes(
    name: &str,
    domain: &escher_domain::FundamentalDomain,
) {
    let size = usize::try_from(EUCLIDEAN_SIZE).unwrap_or(usize::MAX);
    let border = border_mask(domain, EUCLIDEAN_SIZE);
    let mut tiles: Vec<Option<DVec3>> = Vec::with_capacity(size * size);
    for row in 0..EUCLIDEAN_SIZE {
        for column in 0..EUCLIDEAN_SIZE {
            let position = plane_position(EUCLIDEAN_SIZE, column, row);
            let point = DVec3::new(position.x, position.y, 1.0);
            tiles.push(tile_of(domain, point, |pairing| pairing.inverse()));
        }
    }
    let (changes, borders) = compare_with_tiles(name, &border, &tiles, size, |here, there| {
        euclidean::distance(here, there) >= SAME_TILE
    });
    println!("{name}: {changes} pixels where the tile changes and {borders} border pixels");
}

#[test]
fn wallpaper_border_pixels_lie_where_the_tile_of_a_sample_changes() {
    for shape in &WALLPAPER_DOMAINS {
        assert_plane_borders_where_the_tile_changes(&shape.name(), &shape.domain());
    }
}

#[test]
fn euclidean_border_pixels_lie_where_the_tile_of_a_sample_changes() {
    for index in 0..EUCLIDEAN_DOMAINS.len() {
        assert_plane_borders_where_the_tile_changes(
            &euclidean_name(index),
            &euclidean_domain(index),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Border,
    Even,
    Odd,
}

fn grid_oracle(width: f64, height: f64, position: DVec2, pixel: f64, half_width: f64) -> Class {
    let across = (position.x + width / 2.0) / width;
    let up = (position.y + height / 2.0) / height;
    let to_line = |ratio: f64, length: f64| (ratio - ratio.round()).abs() * length;
    if to_line(across, width).min(to_line(up, height)) < pixel * half_width {
        return Class::Border;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the tile indices of a position in the image are small whole numbers"
    )]
    let parity = (across.floor() as i64 + up.floor() as i64).rem_euclid(2);
    if parity == 0 { Class::Even } else { Class::Odd }
}

#[test]
fn rectangle_borders_and_parities_match_the_grid_of_the_rectangle() {
    let even = Colour::from_srgb8(245, 222, 179);
    let odd = Colour::from_srgb8(70, 130, 180);
    let pixel = plane_pixel(GRID_SIZE, GRID_SIZE);
    for aspect in GRID_ASPECTS {
        let domain = domain_with_aspect("*2222", Some(aspect));
        let lengths = domain.side_lengths();
        let (width, height) = (lengths[0], lengths[1]);
        for half_width in GRID_HALF_WIDTHS {
            let options = options(
                RenderOptions::builder(Fill::Parity([even, odd]))
                    .size(GRID_SIZE, GRID_SIZE)
                    .samples(1)
                    .border_width(2.0 * half_width),
            );
            let (pixels, _) = renderer(&domain, options).render();
            let mut differing = 0;
            let mut borders = 0;
            for (index, pixel_bytes) in pixels.as_chunks::<3>().0.iter().enumerate() {
                let index = u32::try_from(index).unwrap_or(u32::MAX);
                let (column, row) = (index % GRID_SIZE, index / GRID_SIZE);
                let position = plane_position(GRID_SIZE, column, row);
                let expected = grid_oracle(width, height, position, pixel, half_width);
                let found = if *pixel_bytes == Colour::BLACK.to_srgb8() {
                    Class::Border
                } else if *pixel_bytes == even.to_srgb8() {
                    Class::Even
                } else {
                    Class::Odd
                };
                if expected == Class::Border {
                    borders += 1;
                }
                if found != expected {
                    differing += 1;
                }
            }
            let bound = count_bound(0);
            println!(
                "*2222 aspect {aspect}, border width {}: {differing} pixels differ from the grid oracle, bound {bound}, {borders} border pixels in the oracle",
                2.0 * half_width
            );
            assert!(
                differing <= bound,
                "aspect {aspect}, half width {half_width}"
            );
        }
    }
}

const SPHERE_DOMAINS: [&str; 5] = ["*532", "*432", "*332", "*722", "*55"];

fn assert_ball_borders_where_the_tile_changes(
    text: &str,
    domain: &escher_domain::FundamentalDomain,
    degrees: [f64; 3],
) {
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    let border = rotated_border_mask(domain, SIZE, degrees);
    let turn = viewing_matrix(degrees).transpose();
    let mut tiles: Vec<Option<DVec3>> = Vec::with_capacity(size * size);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let position = position(column, row);
            let tile = (position.length() <= RADIUS)
                .then(|| {
                    let height = (1.0 - position.length_squared()).sqrt();
                    turn * DVec3::new(position.x, position.y, height)
                })
                .and_then(|point| tile_of(domain, point, spherical::inverse));
            tiles.push(tile);
        }
    }
    let name = format!("{text} with rotation {degrees:?}");
    let (changes, borders) = compare_with_tiles(&name, &border, &tiles, size, |here, there| {
        spherical::distance(here, there) >= SAME_TILE
    });
    println!(
        "{name}: {changes} pixels where the tile changes and {borders} border pixels within ball radius {RADIUS}"
    );
}

#[test]
fn spherical_border_pixels_lie_where_the_tile_of_a_sample_changes() {
    for text in SPHERE_DOMAINS {
        assert_ball_borders_where_the_tile_changes(text, &domain_with_aspect(text, None), [0.0; 3]);
    }
}

#[test]
fn spherical_group_border_pixels_lie_where_the_tile_of_a_sample_changes() {
    for row in &SPHERICAL_GROUP_BOUNDS {
        if let Some(degrees) = row.border {
            let text = row.signature;
            assert_ball_borders_where_the_tile_changes(
                text,
                &domain_with_settings(text, &[]),
                degrees,
            );
        }
    }
}

const BORDER_SUBSET_DOMAINS: [(&str, Option<f64>); 28] = [
    ("*732", None),
    ("*542", None),
    ("*444", None),
    ("*433", None),
    ("*(12)32", None),
    ("*(100)32", None),
    ("*(100)(100)(100)", None),
    ("*632", None),
    ("*442", None),
    ("*333", None),
    ("*2222", Some(1.0)),
    ("*2222", Some(1.618)),
    ("*2222", Some(0.125)),
    ("*2222", Some(8.0)),
    ("*2222", Some(4.0)),
    ("*332", None),
    ("*432", None),
    ("*532", None),
    ("*222", None),
    ("*322", None),
    ("*522", None),
    ("*(12)22", None),
    ("*(100)22", None),
    ("*22", None),
    ("*33", None),
    ("*55", None),
    ("*(12)(12)", None),
    ("*(100)(100)", None),
];

fn fill_mask(domain: &escher_domain::FundamentalDomain, size: u32) -> Vec<bool> {
    let fill = Colour::from_srgb8(70, 130, 180);
    let options = options(
        RenderOptions::builder(Fill::One(fill))
            .size(size, size)
            .samples(1)
            .shading(Shading::Flat),
    );
    let (pixels, _) = renderer(domain, options).render();
    pixels
        .as_chunks::<3>()
        .0
        .iter()
        .map(|pixel| *pixel != fill.to_srgb8())
        .collect()
}

#[test]
fn every_border_pixel_of_a_rotation_domain_is_a_border_pixel_of_its_reflection_domain() {
    for (starred, aspect) in BORDER_SUBSET_DOMAINS {
        let reflection = domain_with_aspect(starred, aspect);
        let rotation = domain_with_aspect(&starred[1..], aspect);
        let euclidean = reflection.geometry() == escher_signature::Geometry::Euclidean;
        let size = if euclidean { EUCLIDEAN_SIZE } else { SIZE };
        let counted = |index: usize| {
            let index = u32::try_from(index).unwrap_or(u32::MAX);
            euclidean || position(index % size, index / size).length() <= RADIUS
        };
        let reflection_mask = fill_mask(&reflection, size);
        let rotation_mask = fill_mask(&rotation, size);
        let mut reflection_borders = 0;
        let mut rotation_borders = 0;
        let mut outside_subset = 0;
        for (index, (&reflected, &rotated)) in
            reflection_mask.iter().zip(&rotation_mask).enumerate()
        {
            if !counted(index) {
                continue;
            }
            reflection_borders += usize::from(reflected);
            rotation_borders += usize::from(rotated);
            outside_subset += usize::from(rotated && !reflected);
        }
        let name = format!("{} aspect {aspect:?}", &starred[1..]);
        let bound = count_bound(0);
        println!(
            "{name}: {rotation_borders} border pixels, {reflection_borders} of {starred}, {outside_subset} not border pixels of {starred}, bound {bound}"
        );
        assert!(outside_subset <= bound, "{name}: {outside_subset} pixels");
        assert!(rotation_borders > 0, "{name}");
        assert!(rotation_borders <= reflection_borders, "{name}");
    }
}

const KITE_SUBSETS: [(&str, &str, usize); 2] = [("4*2", "*442", 0), ("3*3", "*632", 0)];

#[test]
fn every_border_pixel_of_a_kite_is_a_border_pixel_of_its_triangle() {
    for (kite, triangle, measured) in KITE_SUBSETS {
        let kite_mask = fill_mask(&domain_with_aspect(kite, None), EUCLIDEAN_SIZE);
        let triangle_mask = fill_mask(&domain_with_aspect(triangle, None), EUCLIDEAN_SIZE);
        let kite_borders = kite_mask.iter().filter(|&&border| border).count();
        let triangle_borders = triangle_mask.iter().filter(|&&border| border).count();
        let outside_subset = kite_mask
            .iter()
            .zip(&triangle_mask)
            .filter(|&(&kite, &triangle)| kite && !triangle)
            .count();
        let bound = count_bound(measured);
        println!(
            "{kite}: {kite_borders} border pixels, {triangle_borders} of {triangle}, {outside_subset} not border pixels of {triangle}, bound {bound}"
        );
        assert!(outside_subset <= bound, "{kite}: {outside_subset} pixels");
        assert!(kite_borders > 0, "{kite}");
        assert!(kite_borders < triangle_borders, "{kite}");
    }
}

#[derive(Clone, Copy)]
enum Picture {
    Disk,
    Model(&'static ModelMap),
    Plane,
    Ball,
}

struct Centre {
    point: Vector,
    rim: Option<f64>,
}

impl Picture {
    fn model(self) -> HyperbolicModel {
        match self {
            Self::Model(map) => map.model,
            Self::Disk | Self::Plane | Self::Ball => HyperbolicModel::Disk,
        }
    }

    fn name(self) -> String {
        match self {
            Self::Disk => "the disk".to_owned(),
            Self::Model(map) => format!("the {:?} model", map.model),
            Self::Plane => "the plane".to_owned(),
            Self::Ball => "the ball".to_owned(),
        }
    }

    fn measure(self) -> Measure {
        match self {
            Self::Disk | Self::Model(_) => Measure::Hyperbolic,
            Self::Plane => Measure::Euclidean,
            Self::Ball => Measure::Spherical,
        }
    }

    fn centre(
        self,
        reference: &mut Reference,
        (width, height): (u32, u32),
        column: u32,
        row: u32,
    ) -> Option<Centre> {
        match self {
            Self::Disk => {
                let position = pixel_centre(width, height, column, row, disk_pixel(width, height));
                from_disk(position)?;
                let [first, second] = position.to_array().map(num);
                Some(Centre {
                    point: reference::from_disk(position.to_array()),
                    rim: Some(rim_distance(reference, &first, &second)),
                })
            }
            Self::Model(map) => {
                let position = map.position(width, height, column, row);
                (map.to_disk)(position).and_then(|(disk, _)| from_disk(disk))?;
                let [first, second] =
                    reference.model_to_disk(map.exact, &position.to_array().map(num));
                Some(Centre {
                    point: reference::from_disk_exact(&first, &second),
                    rim: Some(rim_distance(reference, &first, &second)),
                })
            }
            Self::Plane => {
                let position = pixel_centre(width, height, column, row, plane_pixel(width, height));
                Some(Centre {
                    point: vector([position.x, position.y, 1.0]),
                    rim: None,
                })
            }
            Self::Ball => {
                let position = pixel_centre(width, height, column, row, disk_pixel(width, height));
                if position.length_squared() >= 1.0 {
                    return None;
                }
                let [first, second] = position.to_array().map(num);
                Some(Centre {
                    point: reference.lift(position.to_array(), [0.0; 3]),
                    rim: Some(rim_distance(reference, &first, &second)),
                })
            }
        }
    }
}

fn rim_distance(reference: &mut Reference, first: &BigFloat, second: &BigFloat) -> f64 {
    let radius = sqrt(&add(&mul(first, first), &mul(second, second)));
    reference.approximate(&sub(&int(1), &radius))
}

const INTRINSIC_SIZES: [(u32, u32); 2] = [(48, 32), (96, 64)];

const INTRINSIC_WIDTH: f64 = 0.1;

const INTRINSIC_RANGE: f64 = 1e-3;

const INTRINSIC_CLEARANCE: f64 = 1e-9;

fn assert_intrinsic_border_at_each_pixel(text: &str, picture: Picture) {
    let fill = GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
    let background = Colour::from_srgb8(255, 0, 255);
    let border = Colour::BLACK.to_srgb8();
    let domain = domain_with_settings(text, &[]);
    let walls = reference_walls(&domain);
    let measure = picture.measure();
    let mut reference = Reference::new();
    let half_width = num(INTRINSIC_WIDTH / 2.0);
    let clearance = num(INTRINSIC_CLEARANCE);
    for (width, height) in INTRINSIC_SIZES {
        let name = format!("{text} in {} at {width} by {height}", picture.name());
        let options = options(
            RenderOptions::builder(Fill::Parity(fill))
                .size(width, height)
                .samples(1)
                .border_unit(BorderUnit::Intrinsic)
                .border_width(INTRINSIC_WIDTH)
                .background(background)
                .shading(Shading::Flat)
                .model(picture.model()),
        );
        let (pixels, _) = renderer(&domain, options).render();
        let pixels = pixels.as_chunks::<3>().0;
        let (mut compared, mut marked, mut outside, mut beyond, mut near) = (0, 0, 0, 0, 0);
        for row in 0..height {
            for column in 0..width {
                let index = usize::try_from(row * width + column).unwrap_or(usize::MAX);
                let shown = pixels[index];
                let Some(centre) = picture.centre(&mut reference, (width, height), column, row)
                else {
                    outside += 1;
                    continue;
                };
                if centre.rim.is_some_and(|rim| rim < INTRINSIC_RANGE) {
                    beyond += 1;
                    continue;
                }
                let folded = reference::fold_exact(&walls, centre.point, DEFAULT_CAP);
                assert!(
                    folded.converged,
                    "{name}: the reference fold of pixel ({column}, {row}) does not converge"
                );
                let distances: Vec<_> = walls
                    .iter()
                    .map(|wall| {
                        measure.wall_distance(&mut reference, &wall.covector, &folded.point)
                    })
                    .collect();
                if distances
                    .iter()
                    .any(|distance| sub(distance, &half_width).abs() < clearance)
                {
                    near += 1;
                    continue;
                }
                let expected = distances.iter().any(|distance| *distance < half_width);
                assert_eq!(
                    shown == border,
                    expected,
                    "{name}: pixel ({column}, {row}) shows {shown:?}"
                );
                assert!(
                    expected || fill.iter().any(|colour| colour.to_srgb8() == shown),
                    "{name}: pixel ({column}, {row}) shows {shown:?}, which is no fill colour"
                );
                compared += 1;
                marked += usize::from(expected);
            }
        }
        println!(
            "{name}: {compared} pixels compared, {marked} of them border pixels, {outside} outside, {beyond} within {INTRINSIC_RANGE:e} of the rim, {near} within {INTRINSIC_CLEARANCE:e} of half the width"
        );
        assert!(
            marked > 0 && marked < compared,
            "{name}: {marked} of {compared}"
        );
    }
}

#[test]
fn intrinsic_border_of_each_pixel_of_a_hyperbolic_disk_follows_the_reference_distance() {
    assert_intrinsic_border_at_each_pixel("*732", Picture::Disk);
}

fn model_picture(model: HyperbolicModel) -> Picture {
    match MODEL_MAPS.iter().find(|map| map.model == model) {
        Some(map) => Picture::Model(map),
        None => panic!("the model maps hold no {model:?} model"),
    }
}

#[test]
fn intrinsic_border_of_each_pixel_of_the_half_plane_follows_the_reference_distance() {
    assert_intrinsic_border_at_each_pixel("*732", model_picture(HyperbolicModel::HalfPlane));
}

#[test]
fn intrinsic_border_of_each_pixel_of_the_band_follows_the_reference_distance() {
    assert_intrinsic_border_at_each_pixel("*732", model_picture(HyperbolicModel::Band));
}

#[test]
fn intrinsic_border_of_each_pixel_of_the_klein_disk_follows_the_reference_distance() {
    assert_intrinsic_border_at_each_pixel("*732", model_picture(HyperbolicModel::Klein));
}

#[test]
fn intrinsic_border_of_each_pixel_of_a_euclidean_tiling_follows_the_reference_distance() {
    assert_intrinsic_border_at_each_pixel("*632", Picture::Plane);
}

#[test]
fn intrinsic_border_of_each_pixel_of_a_ball_follows_the_reference_distance() {
    assert_intrinsic_border_at_each_pixel("*532", Picture::Ball);
}

fn centred_border_mask(
    domain: &escher_domain::FundamentalDomain,
    size: u32,
    model: HyperbolicModel,
    corner: u32,
) -> Vec<bool> {
    let options = options(
        RenderOptions::builder(Fill::One(Colour::from_srgb8(70, 130, 180)))
            .size(size, size)
            .samples(1)
            .model(model)
            .shading(Shading::Flat)
            .centre(corner),
    );
    let (pixels, _) = renderer(domain, options).render();
    pixels
        .as_chunks::<3>()
        .0
        .iter()
        .map(|pixel| *pixel == Colour::BLACK.to_srgb8())
        .collect()
}

fn assert_centred_borders_where_the_tile_changes(text: &str, model: HyperbolicModel, corner: u32) {
    let domain = domain_with_settings(text, &[]);
    let vertex =
        domain.vertices()[usize::try_from(corner).expect("a u32 corner position fits in usize")];
    let geometry = domain.geometry();
    let size = CENTRED_SIZE;
    let border = centred_border_mask(&domain, size, model, corner);
    let map = MODEL_MAPS.iter().find(|map| map.model == model);
    let mut tiles: Vec<Option<DVec3>> = Vec::new();
    for row in 0..size {
        for column in 0..size {
            let tile = match geometry {
                escher_signature::Geometry::Hyperbolic => map
                    .map_or(Some(pixel_position(size, column, row)), |map| {
                        (map.to_disk)(map.position(size, size, column, row)).map(|(disk, _)| disk)
                    })
                    .filter(|disk| disk.length() <= RADIUS)
                    .and_then(from_disk)
                    .and_then(|point| {
                        tile_of(
                            &domain,
                            escher_geometry::hyperbolic::translation(vertex) * point,
                            inverse,
                        )
                    }),
                escher_signature::Geometry::Euclidean => {
                    let position = plane_position(size, column, row);
                    let point = DVec3::new(position.x, position.y, 1.0);
                    tile_of(&domain, euclidean::translation(vertex) * point, |pairing| {
                        pairing.inverse()
                    })
                }
                escher_signature::Geometry::Spherical => {
                    let position = pixel_position(size, column, row);
                    (position.length() <= RADIUS)
                        .then(|| {
                            let height = (1.0 - position.length_squared()).sqrt();
                            spherical::translation(vertex)
                                * DVec3::new(position.x, position.y, height)
                        })
                        .and_then(|point| tile_of(&domain, point, spherical::inverse))
                }
            };
            tiles.push(tile);
        }
    }
    let name = format!("{text} in the {model:?} model centred on corner {corner}");
    let pixels = usize::try_from(size).expect("an image side of 193 pixels fits in usize");
    let (changes, borders) = compare_with_tiles(&name, &border, &tiles, pixels, |here, there| {
        let measure = match geometry {
            escher_signature::Geometry::Hyperbolic => distance,
            escher_signature::Geometry::Euclidean => euclidean::distance,
            escher_signature::Geometry::Spherical => spherical::distance,
        };
        measure(here, there) >= SAME_TILE
    });
    println!("{name}: {changes} pixels where the tile changes and {borders} border pixels");
}

#[test]
fn centred_border_pixels_lie_where_the_tile_of_a_sample_changes() {
    let cases = [
        ("*732", HyperbolicModel::Disk, 0),
        ("*732", HyperbolicModel::Disk, 1),
        ("*732", HyperbolicModel::Disk, 2),
        ("*732", HyperbolicModel::Klein, 1),
        ("732", HyperbolicModel::Disk, 1),
        ("*632", HyperbolicModel::Disk, 0),
        ("*532", HyperbolicModel::Disk, 2),
    ];
    for (text, model, corner) in cases {
        assert_centred_borders_where_the_tile_changes(text, model, corner);
    }
}
