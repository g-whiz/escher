use escher_domain::FundamentalDomain;
use escher_geometry::hyperbolic::ORIGIN;

use crate::common::{
    LargestResiduals, all_triangles, big, cone_text, domain, is_small, kite,
    largest_coordinate_error, small_triangles, text,
};
use crate::reference::{Reference, Triangle, Worst, error, int, largest_magnitude, sub};
use crate::rotation::two_vertex_cycle_residual;

const QUANTITIES: [&str; 6] = [
    "vertex, largest coordinate error over the largest coordinate",
    "normal, largest coordinate error",
    "side length, absolute error",
    "centre, largest coordinate distance from the origin",
    "corner angle, absolute error",
    "area, absolute error",
];

const SMALL_ORDERS_BOUNDS: [f64; 6] = [1e-13, 1e-13, 1e-13, 1e-13, 1e-9, 1e-9];
const LIMIT_TRIPLE_BOUNDS: [f64; 6] = [1e-11, 1e-11, 1e-11, 1e-11, 1e-9, 1e-9];
const SUPPORTED_BOUNDS: [f64; 6] = [1e-8, 1e-8, 1e-8, 1e-9, 1e-9, 1e-9];

const SAMPLES_PER_TRIPLE: [usize; 6] = [3, 3, 3, 1, 3, 1];

const LIMIT_TRIPLE: [u32; 3] = [1000, 3, 2];

const CHECKED_MINIMUM: usize = 294;

struct Column {
    bounds: [f64; 6],
    worst: Vec<Worst>,
}

impl Column {
    fn new(label: &str, bounds: [f64; 6], triples: usize) -> Self {
        let worst = QUANTITIES
            .iter()
            .zip(SAMPLES_PER_TRIPLE)
            .map(|(quantity, samples)| {
                Worst::new(format!("{quantity}, {label}"), samples * triples)
            })
            .collect();
        Self { bounds, worst }
    }

    fn assert_within_bounds(&self, reference: &mut Reference) {
        for (worst, bound) in self.worst.iter().zip(self.bounds) {
            worst.assert_at_most(reference, bound);
        }
    }
}

fn record(column: &mut Column, orders: [u32; 3], built: &FundamentalDomain, exact: &Triangle) {
    let input = || text(orders);
    let [vertices, normals, lengths, centre, angles, area] = &mut column.worst[..] else {
        panic!("a column has six quantities");
    };
    for (vertex, exact_vertex) in built.vertices().iter().zip(&exact.vertices) {
        vertices.record(
            &largest_coordinate_error(*vertex, exact_vertex),
            &largest_magnitude(exact_vertex),
            input,
        );
    }
    for (side, exact_normal) in built.sides().iter().zip(&exact.normals) {
        normals.record(
            &largest_coordinate_error(side.normal(), exact_normal),
            &int(1),
            input,
        );
    }
    for (length, exact_length) in built.side_lengths().into_iter().zip(&exact.lengths) {
        lengths.record(&error(length, exact_length), &int(1), input);
    }
    centre.record(
        &largest_coordinate_error(built.centre(), &big(ORIGIN)),
        &int(1),
        input,
    );
    for (angle, exact_angle) in built.corner_angles().into_iter().zip(&exact.angles) {
        angles.record(&error(angle, exact_angle), &int(1), input);
    }
    area.record(&error(built.area(), &exact.area), &int(1), input);
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn triangles_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let (small, all) = (small_triangles(), all_triangles());
    let mut columns = [
        Column::new("orders up to 12", SMALL_ORDERS_BOUNDS, small.len()),
        Column::new("(1000, 3, 2)", LIMIT_TRIPLE_BOUNDS, 1),
        Column::new("every supported triple", SUPPORTED_BOUNDS, all.len()),
    ];
    for orders in all {
        let built = domain(orders);
        let exact = reference.triangle(orders);
        if is_small(orders) {
            record(&mut columns[0], orders, &built, &exact);
        }
        if orders == LIMIT_TRIPLE {
            record(&mut columns[1], orders, &built, &exact);
        }
        record(&mut columns[2], orders, &built, &exact);
    }
    for column in &columns {
        column.assert_within_bounds(&mut reference);
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn third_side_matches_the_law_of_cosines() {
    let mut reference = Reference::new();
    let placement_tolerance = reference.parse("1e-60");
    let (small, all) = (small_triangles(), all_triangles());
    let mut small_worst = Worst::new("third side, absolute error, orders up to 12", small.len());
    let mut supported = Worst::new(
        "third side, absolute error, every supported triple",
        all.len(),
    );
    for orders in all {
        let exact = reference.triangle(orders);
        let placement = sub(&exact.standard_third_side, &exact.lengths[1]).abs();
        assert!(
            placement <= placement_tolerance,
            "{}: the placed third side differs from the law of cosines by {placement}",
            text(orders)
        );
        let built = domain(orders);
        let measured =
            escher_geometry::hyperbolic::distance(built.vertices()[0], built.vertices()[1]);
        let difference = error(measured, &exact.lengths[1]);
        if is_small(orders) {
            small_worst.record(&difference, &int(1), || text(orders));
        }
        supported.record(&difference, &int(1), || text(orders));
    }
    small_worst.assert_at_most(&mut reference, SMALL_ORDERS_BOUNDS[2]);
    supported.assert_at_most(&mut reference, SUPPORTED_BOUNDS[2]);
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the checks of the largest orders run loops too long for the interpreter"
)]
fn checks_pass_at_the_worst_cases() {
    let mut largest = LargestResiduals::new();
    let mut kites = LargestResiduals::new();
    let mut two_vertex_cycles = (0.0, String::new(), 0);
    for orders in all_triangles() {
        largest.record(&domain(orders), &text(orders));
        let double = kite(orders);
        kites.record(&double, &cone_text(orders));
        let residual = two_vertex_cycle_residual(&double, orders[2]);
        two_vertex_cycles.2 += 1;
        if residual >= two_vertex_cycles.0 {
            two_vertex_cycles = (residual, cone_text(orders), two_vertex_cycles.2);
        }
    }
    println!("reflection triangles:");
    largest.print_and_assert_samples(CHECKED_MINIMUM);
    println!("kites:");
    kites.print_and_assert_samples(CHECKED_MINIMUM);
    let (residual, input, samples) = two_vertex_cycles;
    let tolerance = kite([7, 3, 2]).verify().relators().tolerance();
    println!(
        "relators of the kites, cycles of two vertices: largest residual {residual:.3e}, tolerance {tolerance:e}, margin {:.1}, {samples} samples, at {input}",
        tolerance / residual
    );
    assert!(samples >= CHECKED_MINIMUM, "{samples}");
    assert!(residual <= tolerance, "{input}: {residual:e}");
}
