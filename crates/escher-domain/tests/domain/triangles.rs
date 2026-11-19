use std::collections::BTreeSet;
use std::time::Instant;

use escher_domain::{FundamentalDomain, Verification};
use escher_geometry::hyperbolic::ORIGIN;

use crate::common::{
    LIMIT_ROTATION_POLYGONS, LargestResiduals, SMALL_ORDER, all_triangles, big, built,
    built_or_refused_at_the_limit, cone_text, domain, is_small, kite, largest_coordinate_error,
    order_text, signature, small_triangles, text,
};
use crate::reference::{Reference, Triangle, Worst, error, int, largest_magnitude, sub};
use crate::rotation::two_vertex_cycle_residual;
use crate::shared::{NAMED_POLYGONS, corner_orders, small_polygons};

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

const SMALL_POLYGONS: usize = 4421;

const POLYGON_SIGNATURES: usize = 4430;

const CLAIMED_MARGIN: f64 = 8.0;

const TIMED_POLYGON: &str = "*(1000)(1000)(1000)(1000)(1000)(1000)(1000)(1000)";

const SWEPT_ORDERS: [u32; 4] = [2, 3, 7, 1000];

const SWEPT_CORNERS: [usize; 3] = [4, 5, 6];

fn smallest_margin(verification: &Verification) -> f64 {
    [
        verification.isometries(),
        verification.pairings(),
        verification.walls(),
        verification.relators(),
        verification.angles(),
        verification.area(),
    ]
    .into_iter()
    .map(|check| check.tolerance() / check.residual())
    .fold(f64::INFINITY, f64::min)
}

struct Margins {
    label: String,
    smallest: (f64, String),
    claimed: (f64, String),
    below: Vec<(String, f64)>,
    samples: usize,
}

impl Margins {
    fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            smallest: (f64::INFINITY, String::new()),
            claimed: (f64::INFINITY, String::new()),
            below: Vec::new(),
            samples: 0,
        }
    }

    fn record(&mut self, verification: &Verification, text: &str, claimed: bool) {
        let margin = smallest_margin(verification);
        self.samples += 1;
        if claimed {
            assert!(margin >= CLAIMED_MARGIN, "{text}: smallest margin {margin}");
            if margin < self.claimed.0 {
                self.claimed = (margin, text.to_owned());
            }
        } else if margin < CLAIMED_MARGIN {
            self.below.push((text.to_owned(), margin));
        }
        if margin < self.smallest.0 {
            self.smallest = (margin, text.to_owned());
        }
    }

    fn print(&self) {
        println!(
            "{}: {} domains, smallest margin {:.3} at {}, inside the claim {:.3} at {}, outside the claim below {CLAIMED_MARGIN}: {:?}",
            self.label,
            self.samples,
            self.smallest.0,
            self.smallest.1,
            self.claimed.0,
            self.claimed.1,
            self.below
        );
    }
}

fn is_claimed_polygon(orders: &[u32]) -> bool {
    let largest = orders.iter().copied().max().unwrap_or(0);
    largest <= SMALL_ORDER || (orders.len() <= 6 && orders.contains(&1000))
}

#[test]
fn checks_pass_at_the_default_shapes_of_the_polygons_and_their_doubles() {
    let small = small_polygons();
    assert_eq!(small.len(), SMALL_POLYGONS);
    let mut texts: BTreeSet<String> = NAMED_POLYGONS
        .iter()
        .map(|text| signature(text).to_string())
        .collect();
    texts.extend(small.iter().map(ToString::to_string));
    assert_eq!(texts.len(), POLYGON_SIGNATURES);
    let mut polygons = LargestResiduals::new();
    let mut doubles = LargestResiduals::new();
    let mut polygon_margins = Margins::new("polygons");
    let mut double_margins = Margins::new("doubles");
    for text in &texts {
        let orders = corner_orders(&signature(text));
        let verification = built(text).verify();
        polygons.record_verification(&verification, text);
        polygon_margins.record(&verification, text, is_claimed_polygon(&orders));
        let rotation = text.trim_start_matches('*');
        let double = if LIMIT_ROTATION_POLYGONS.contains(&rotation) {
            match built_or_refused_at_the_limit(rotation) {
                Some(double) => {
                    let verification = double.verify();
                    println!("{rotation} built: {verification:?}");
                    double_margins.record(&verification, rotation, false);
                }
                None => println!("{rotation} refused by the guard"),
            }
            continue;
        } else {
            built(rotation)
        };
        let verification = double.verify();
        doubles.record_verification(&verification, rotation);
        let claimed = orders.iter().all(|&order| order <= SMALL_ORDER);
        double_margins.record(&verification, rotation, claimed);
    }
    println!("polygons at their default shapes:");
    polygons.print_and_assert_samples(POLYGON_SIGNATURES);
    println!("doubles of the polygons, without the double at the limit:");
    doubles.print_and_assert_samples(POLYGON_SIGNATURES - LIMIT_ROTATION_POLYGONS.len());
    polygon_margins.print();
    double_margins.print();
    for text in [TIMED_POLYGON, TIMED_POLYGON.trim_start_matches('*')] {
        let started = Instant::now();
        let result = FundamentalDomain::build(&signature(text));
        println!(
            "one build of {text} with its guard in this build of the tests: {:.3} ms, {}",
            started.elapsed().as_secs_f64() * 1000.0,
            if result.is_ok() { "built" } else { "refused" }
        );
    }
}

fn polygons_with_a_corner_of_order_1000(corners: usize) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut digits = vec![0; corners - 1];
    loop {
        let orders: String = core::iter::once(1000)
            .chain(digits.iter().map(|&digit| SWEPT_ORDERS[digit]))
            .map(order_text)
            .collect();
        found.insert(signature(&format!("*{orders}")).to_string());
        let Some(position) = digits
            .iter()
            .position(|&digit| digit + 1 < SWEPT_ORDERS.len())
        else {
            break;
        };
        digits[position] += 1;
        for digit in &mut digits[..position] {
            *digit = 0;
        }
    }
    found
}

#[test]
fn polygons_of_four_to_six_corners_with_a_corner_of_order_1000_keep_their_claimed_margin() {
    for corners in SWEPT_CORNERS {
        let mut margins = Margins::new(format!(
            "{corners} corners, at least one of order 1000, the others from {SWEPT_ORDERS:?}"
        ));
        for text in polygons_with_a_corner_of_order_1000(corners) {
            let verification = built(&text).verify();
            assert!(verification.passed(), "{text}: {verification:?}");
            margins.record(&verification, &text, true);
        }
        margins.print();
    }
}
