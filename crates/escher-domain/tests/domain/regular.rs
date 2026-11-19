use escher_domain::{FundamentalDomain, ShapeParameters, parameters};
use escher_geometry::hyperbolic::{distance, inverse, to_disk, translation};

use crate::common::{big, domain, signature};
use crate::polygons::exact_defaults;
use crate::reference::{Reference, Worst, add, div, error, int, mul, sqrt, sub};
use crate::shared::corner_orders;

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

const REGULAR_SIDE_BOUND: f64 = 1e-14;

const REGULAR_RADIUS_BOUND: f64 = 2e-15;

const RIGHT_ANGLED: [&str; 2] = ["*22222", "*222222"];

const INSCRIBED: [&str; 5] = ["*3333", "*4444", "*33333", "*2222222", "*22222222"];

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn regular_polygons_have_the_edges_and_circumradii_of_their_tilings() {
    let mut reference = Reference::new();
    let pi = reference.pi();
    let mut sides = Worst::new(
        "regular polygon side against the edge of its tiling, absolute error",
        1,
    );
    let mut radii = Worst::new(
        "regular polygon vertex disk radius against the circumradius of its tiling, absolute error",
        1,
    );
    for text in RIGHT_ANGLED.into_iter().chain(INSCRIBED) {
        let polygon = signature(text);
        let orders = corner_orders(&polygon);
        let (Ok(corners), order) = (u32::try_from(orders.len()), orders[0]) else {
            panic!("{text} has too many corners");
        };
        let (half_centre, half_corner) = (div(&pi, &int(corners)), div(&pi, &int(2 * order)));
        let ratio = div(&reference.cos(&half_centre), &reference.sin(&half_corner));
        let edge = mul(&int(2), &reference.acosh(&ratio));
        let radius = sqrt(&div(
            &reference.cos(&add(&half_centre, &half_corner)),
            &reference.cos(&sub(&half_centre, &half_corner)),
        ));
        let exact = if RIGHT_ANGLED.contains(&text) {
            vec![edge.clone(); parameters(&polygon).len()]
        } else {
            let inscribed = reference.inscribed(&orders);
            exact_defaults(&mut reference, &orders, &inscribed.vertices)
        };
        let mut settings = ShapeParameters::new();
        let mut values = Vec::new();
        for (spec, exact) in parameters(&polygon).iter().zip(&exact) {
            let value = reference.approximate(exact);
            settings.set(spec.name(), value);
            values.push(value);
        }
        let built = match FundamentalDomain::build_with(&polygon, &settings) {
            Ok(built) => built,
            Err(error) => panic!("{text} {values:?} has no domain: {error}"),
        };
        for (side, length) in built.side_lengths().into_iter().enumerate() {
            sides.record(&error(length, &edge), &int(1), || {
                format!("{text} side {side}")
            });
        }
        for (index, vertex) in built.vertices().iter().enumerate() {
            let [x, y, t] = big(*vertex);
            let disk = div(&sqrt(&add(&mul(&x, &x), &mul(&y, &y))), &add(&int(1), &t));
            radii.record(&sub(&disk, &radius).abs(), &int(1), || {
                format!("{text} vertex {index}")
            });
        }
    }
    sides.assert_at_most(&mut reference, REGULAR_SIDE_BOUND);
    radii.assert_at_most(&mut reference, REGULAR_RADIUS_BOUND);
}
