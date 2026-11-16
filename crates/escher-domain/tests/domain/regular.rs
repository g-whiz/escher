use escher_geometry::hyperbolic::{distance, inverse, to_disk, translation};

use crate::common::domain;
use crate::reference::{Reference, Worst, add, div, error, int, mul, sqrt, sub};

const EDGE_TOLERANCE: f64 = 2e-13;
const CIRCUMRADIUS_TOLERANCE: f64 = 1e-13;

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn edge_of_the_heptagonal_tiling_is_twice_the_side_from_vertex_1_to_vertex_2() {
    let mut reference = Reference::new();
    let built = domain([7, 3, 2]);
    let vertices = built.vertices();
    let edge = 2.0 * distance(vertices[1], vertices[2]);
    let pi = reference.pi();
    let ratio = div(
        &reference.cos(&div(&pi, &int(7))),
        &reference.sin(&div(&pi, &int(3))),
    );
    let exact = mul(&int(2), &reference.acosh(&ratio));
    let mut worst = Worst::new("edge of {7, 3}, absolute error", 1);
    worst.record(&error(edge, &exact), &int(1), || "*732".to_owned());
    worst.assert_at_most(&mut reference, EDGE_TOLERANCE);
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn circumradius_of_the_heptagonal_tiling_is_the_disk_radius_of_vertex_1_seen_from_vertex_0() {
    let mut reference = Reference::new();
    let built = domain([7, 3, 2]);
    let vertices = built.vertices();
    let radius = to_disk(inverse(translation(vertices[0])) * vertices[1]).length();
    let pi = reference.pi();
    let (first, second) = (div(&pi, &int(7)), div(&pi, &int(3)));
    let exact = sqrt(&div(
        &reference.cos(&add(&first, &second)),
        &reference.cos(&sub(&first, &second)),
    ));
    let mut worst = Worst::new("circumradius of {7, 3} in the disk, absolute error", 1);
    worst.record(&error(radius, &exact), &int(1), || "*732".to_owned());
    worst.assert_at_most(&mut reference, CIRCUMRADIUS_TOLERANCE);
}
