use escher_geometry::hyperbolic::{distance, from_disk, inverse};
use escher_geometry::{euclidean, spherical};
use escher_render::{Colour, Fill, RenderOptions, Shading, fold};
use glam::{DVec2, DVec3};

use crate::common::{
    DEFAULT_CAP, EUCLIDEAN_DOMAINS, PLAN_TRIPLES, domain, euclidean_domain, euclidean_name,
    options, text,
};
use crate::golden_images::{domain_with_aspect, renderer};
use crate::reference::count_bound;

const SIZE: u32 = 192;

const RADIUS: f64 = 0.9;

const SAME_TILE: f64 = 1e-4;

const EUCLIDEAN_SIZE: u32 = 193;

const GRID_SIZE: u32 = 512;

const GRID_ASPECTS: [f64; 6] = [1.0, 4.0, 1.618, 0.125, 8.0, 0.3];

const GRID_HALF_WIDTHS: [f64; 3] = [0.5, 0.75, 1.5];

const GRID_MEASURED: [[usize; 3]; 6] = [[0; 3]; 6];

fn position(column: u32, row: u32) -> DVec2 {
    let pixel = 2.0 / (15.0 / 16.0 * f64::from(SIZE));
    let half = f64::from(SIZE) / 2.0;
    DVec2::new(
        (f64::from(column) + 0.5 - half) * pixel,
        (half - f64::from(row) - 0.5) * pixel,
    )
}

fn border_mask(domain: &escher_domain::FundamentalDomain, size: u32) -> Vec<bool> {
    let options = options(
        RenderOptions::builder(Fill::One(Colour::from_srgb8(70, 130, 180)))
            .size(size, size)
            .samples(1)
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
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn border_pixels_lie_where_the_tile_of_a_sample_changes() {
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    for orders in PLAN_TRIPLES {
        let domain = domain(orders);
        let border = border_mask(&domain, SIZE);
        let mut tiles: Vec<Option<DVec3>> = Vec::with_capacity(size * size);
        for row in 0..SIZE {
            for column in 0..SIZE {
                let position = position(column, row);
                let tile = (position.length() <= RADIUS)
                    .then(|| from_disk(position))
                    .flatten()
                    .and_then(|point| tile_of(&domain, point, inverse));
                tiles.push(tile);
            }
        }
        let (changes, borders) =
            compare_with_tiles(&text(orders), &border, &tiles, size, |here, there| {
                distance(here, there) >= SAME_TILE
            });
        println!(
            "{}: {changes} pixels where the tile changes and {borders} border pixels within disk radius {RADIUS}",
            text(orders)
        );
    }
}

fn plane_position(size: u32, column: u32, row: u32) -> DVec2 {
    let pixel = 8.0 / f64::from(size);
    let half = f64::from(size) / 2.0;
    DVec2::new(
        (f64::from(column) + 0.5 - half) * pixel,
        (half - f64::from(row) - 0.5) * pixel,
    )
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn euclidean_border_pixels_lie_where_the_tile_of_a_sample_changes() {
    let size = usize::try_from(EUCLIDEAN_SIZE).unwrap_or(usize::MAX);
    for index in 0..EUCLIDEAN_DOMAINS.len() {
        let domain = euclidean_domain(index);
        let border = border_mask(&domain, EUCLIDEAN_SIZE);
        let mut tiles: Vec<Option<DVec3>> = Vec::with_capacity(size * size);
        for row in 0..EUCLIDEAN_SIZE {
            for column in 0..EUCLIDEAN_SIZE {
                let position = plane_position(EUCLIDEAN_SIZE, column, row);
                let point = DVec3::new(position.x, position.y, 1.0);
                tiles.push(tile_of(&domain, point, |pairing| pairing.inverse()));
            }
        }
        let name = euclidean_name(index);
        let (changes, borders) = compare_with_tiles(&name, &border, &tiles, size, |here, there| {
            euclidean::distance(here, there) >= SAME_TILE
        });
        println!("{name}: {changes} pixels where the tile changes and {borders} border pixels");
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
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn rectangle_borders_and_parities_match_the_grid_of_the_rectangle() {
    let even = Colour::from_srgb8(245, 222, 179);
    let odd = Colour::from_srgb8(70, 130, 180);
    let pixel = 8.0 / f64::from(GRID_SIZE);
    for (aspect, measured) in GRID_ASPECTS.into_iter().zip(GRID_MEASURED) {
        let domain = domain_with_aspect("*2222", Some(aspect));
        let lengths = domain.side_lengths();
        let (width, height) = (lengths[0], lengths[1]);
        for (half_width, measured) in GRID_HALF_WIDTHS.into_iter().zip(measured) {
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
            let bound = count_bound(measured);
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

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn spherical_border_pixels_lie_where_the_tile_of_a_sample_changes() {
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    for text in SPHERE_DOMAINS {
        let domain = domain_with_aspect(text, None);
        let border = border_mask(&domain, SIZE);
        let mut tiles: Vec<Option<DVec3>> = Vec::with_capacity(size * size);
        for row in 0..SIZE {
            for column in 0..SIZE {
                let position = position(column, row);
                let tile = (position.length() <= RADIUS)
                    .then(|| {
                        let height = (1.0 - position.length_squared()).sqrt();
                        DVec3::new(position.x, position.y, height)
                    })
                    .and_then(|point| tile_of(&domain, point, spherical::inverse));
                tiles.push(tile);
            }
        }
        let (changes, borders) = compare_with_tiles(text, &border, &tiles, size, |here, there| {
            spherical::distance(here, there) >= SAME_TILE
        });
        println!(
            "{text}: {changes} pixels where the tile changes and {borders} border pixels within ball radius {RADIUS}"
        );
    }
}

const BORDER_SUBSET_DOMAINS: [(&str, Option<f64>); 32] = [
    ("*732", None),
    ("*542", None),
    ("*444", None),
    ("*433", None),
    ("*(12)32", None),
    ("*(1000)32", None),
    ("*(1000)(1000)(1000)", None),
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
    ("*(999)22", None),
    ("*(1000)22", None),
    ("*22", None),
    ("*33", None),
    ("*55", None),
    ("*(12)(12)", None),
    ("*(100)(100)", None),
    ("*(999)(999)", None),
    ("*(1000)(1000)", None),
];

const BORDER_SUBSET_MEASURED: [usize; 32] = [0; 32];

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
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn every_border_pixel_of_a_rotation_domain_is_a_border_pixel_of_its_reflection_domain() {
    for ((starred, aspect), measured) in BORDER_SUBSET_DOMAINS
        .into_iter()
        .zip(BORDER_SUBSET_MEASURED)
    {
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
        let bound = count_bound(measured);
        println!(
            "{name}: {rotation_borders} border pixels, {reflection_borders} of {starred}, {outside_subset} not border pixels of {starred}, bound {bound}"
        );
        assert!(outside_subset <= bound, "{name}: {outside_subset} pixels");
        assert!(rotation_borders > 0, "{name}");
        assert!(rotation_borders <= reflection_borders, "{name}");
    }
}
