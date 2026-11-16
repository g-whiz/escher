use escher_domain::{Check, FundamentalDomain};
use escher_geometry::hyperbolic::ORIGIN;

use crate::common::{
    all_triangles, big, domain, is_small, largest_coordinate_error, small_triangles, text,
};
use crate::reference::{Reference, Triangle, Worst, error, int, largest_magnitude, sub};

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

struct LargestResidual {
    name: &'static str,
    residual: f64,
    tolerance: f64,
    input: String,
    samples: usize,
}

impl LargestResidual {
    fn record(&mut self, check: Check, input: &str) {
        assert!(
            check.passed(),
            "{} fails at {input}: residual {:e}, tolerance {:e}",
            self.name,
            check.residual(),
            check.tolerance()
        );
        self.samples += 1;
        self.tolerance = check.tolerance();
        if check.residual() >= self.residual {
            self.residual = check.residual();
            self.input = input.to_owned();
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the checks of the largest orders run loops too long for the interpreter"
)]
fn checks_pass_at_the_worst_cases() {
    let mut largest = [
        "isometries",
        "pairings",
        "walls",
        "relators",
        "angles",
        "area",
    ]
    .map(|name| LargestResidual {
        name,
        residual: 0.0,
        tolerance: 0.0,
        input: String::new(),
        samples: 0,
    });
    for orders in all_triangles() {
        let verification = domain(orders).verify();
        let input = text(orders);
        let checks = [
            verification.isometries(),
            verification.pairings(),
            verification.walls(),
            verification.relators(),
            verification.angles(),
            verification.area(),
        ];
        for (largest, check) in largest.iter_mut().zip(checks) {
            largest.record(check, &input);
        }
        assert!(verification.passed(), "{input}: {verification:?}");
    }
    for largest in &largest {
        println!(
            "{}: largest residual {:.3e}, tolerance {:e}, margin {:.1}, {} samples, at {}",
            largest.name,
            largest.residual,
            largest.tolerance,
            largest.tolerance / largest.residual,
            largest.samples,
            largest.input
        );
        assert!(
            largest.samples >= CHECKED_MINIMUM,
            "{}: checked {} triples, fewer than {CHECKED_MINIMUM}",
            largest.name,
            largest.samples
        );
    }
}
