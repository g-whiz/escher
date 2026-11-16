use escher_geometry::hyperbolic::{distance, from_disk, inverse};
use escher_render::{Colour, Fill, RenderOptions, Renderer, fold};
use glam::{DVec2, DVec3};

use crate::common::{DEFAULT_CAP, PLAN_TRIPLES, domain, options, text};

const SIZE: u32 = 192;

const RADIUS: f64 = 0.9;

const SAME_TILE: f64 = 1e-4;

fn position(column: u32, row: u32) -> DVec2 {
    let pixel = 2.0 / (15.0 / 16.0 * f64::from(SIZE));
    let half = f64::from(SIZE) / 2.0;
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
fn border_pixels_lie_where_the_tile_of_a_sample_changes() {
    let size = usize::try_from(SIZE).unwrap_or(usize::MAX);
    for orders in PLAN_TRIPLES {
        let domain = domain(orders);
        let fill = Colour::from_srgb8(70, 130, 180);
        let options = options(
            RenderOptions::builder(Fill::One(fill))
                .size(SIZE, SIZE)
                .samples(1),
        );
        let (pixels, _) = Renderer::new(&domain, options).render();
        let border: Vec<bool> = pixels
            .as_chunks::<3>()
            .0
            .iter()
            .map(|pixel| *pixel == Colour::BLACK.to_srgb8())
            .collect();
        let mut tiles: Vec<Option<DVec3>> = Vec::with_capacity(size * size);
        for row in 0..SIZE {
            for column in 0..SIZE {
                let position = position(column, row);
                let tile = (position.length() <= RADIUS)
                    .then(|| from_disk(position))
                    .flatten()
                    .and_then(|point| {
                        let mut sides = Vec::new();
                        let result = fold(&domain, point, DEFAULT_CAP, |index| sides.push(index));
                        result.converged().then(|| {
                            sides.iter().rev().fold(domain.centre(), |centre, &index| {
                                inverse(domain.sides()[index].pairing()) * centre
                            })
                        })
                    });
                tiles.push(tile);
            }
        }
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
                        && distance(here, there) >= SAME_TILE
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
                        near(&border, column, row),
                        "{}: the tile changes at pixel ({column}, {row}) with no border pixel next to it",
                        text(orders)
                    );
                }
                if border[index] {
                    borders += 1;
                    assert!(
                        near(&change, column, row),
                        "{}: the border pixel ({column}, {row}) has no change of tile next to it",
                        text(orders)
                    );
                }
            }
        }
        println!(
            "{}: {changes} pixels where the tile changes and {borders} border pixels within disk radius {RADIUS}",
            text(orders)
        );
        assert!(changes > 0 && borders > 0, "{}", text(orders));
    }
}
