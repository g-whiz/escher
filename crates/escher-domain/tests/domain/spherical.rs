use core::f64::consts::PI;

use escher_domain::{FundamentalDomain, Orientation, PairingKind, parameters};
use escher_geometry::spherical;
use escher_signature::Geometry;
use glam::{DMat3, DVec3};

use crate::common::{
    LargestResiduals, SMALL_ORDER, assert_same_domain, built, largest_coordinate_error, order_text,
    signature,
};
use crate::reference::{Reference, SphericalPolygon, Worst, error, int};

const VERTEX_BOUND: f64 = 4e-13;
const NORMAL_BOUND: f64 = 1e-13;
const LENGTH_BOUND: f64 = 4e-13;
const CENTRE_BOUND: f64 = 2e-15;
const ANGLE_BOUND: f64 = 2e-13;
const AREA_BOUND: f64 = 4e-13;

const MEASURED_ORDERS: [u32; 7] = [2, 3, 5, 12, 100, 999, 1000];

pub(crate) const MERGE_TOLERANCE: f64 = 1e-9;

pub(crate) const FOLD_BACK_TOLERANCE: f64 = 1e-9;

const DEFAULT_FOLD_CAP: usize = 1500;

const FOLD_TOLERANCE: f64 = 4e-15;

const IMAGE_FLOOR: f64 = 2e-2;

pub(crate) const LIMIT_IMAGE_FLOOR: f64 = 2e-4;

const DOUBLE_IMAGE_FLOOR: f64 = 4e-2;

const LIMIT_DOUBLE_IMAGE_FLOOR: f64 = 4e-4;

const SPELLINGS: [(&str, &str); 9] = [
    ("*323", "*332"),
    ("*233", "*332"),
    ("*234", "*432"),
    ("*342", "*432"),
    ("*235", "*532"),
    ("*253", "*532"),
    ("*227", "*722"),
    ("*272", "*722"),
    ("*221", "*22"),
];

fn triangle_text(orders: [u32; 3]) -> String {
    let corners: String = orders.into_iter().map(order_text).collect();
    format!("*{corners}")
}

fn lune_text(order: u32) -> String {
    format!("*{}{}", order_text(order), order_text(order))
}

enum Shape {
    Triangle([u32; 3]),
    Lune(u32),
    Kite([u32; 3]),
    LuneDouble(u32),
}

impl Shape {
    fn text(&self) -> String {
        match *self {
            Self::Triangle(orders) => triangle_text(orders),
            Self::Lune(order) => lune_text(order),
            Self::Kite(orders) => triangle_text(orders).replacen('*', "", 1),
            Self::LuneDouble(order) => lune_text(order).replacen('*', "", 1),
        }
    }

    const fn is_double(&self) -> bool {
        matches!(self, Self::Kite(_) | Self::LuneDouble(_))
    }

    fn reference(&self, reference: &mut Reference) -> Option<SphericalPolygon> {
        match *self {
            Self::Triangle(orders) => Some(reference.spherical_triangle(orders)),
            Self::Lune(order) => Some(reference.lune(order)),
            Self::Kite(_) | Self::LuneDouble(_) => None,
        }
    }
}

fn measured_shapes() -> Vec<Shape> {
    let mut shapes: Vec<Shape> = [[3, 3, 2], [4, 3, 2], [5, 3, 2], [2, 2, 2]]
        .into_iter()
        .map(Shape::Triangle)
        .collect();
    for order in MEASURED_ORDERS {
        shapes.push(Shape::Triangle([order, 2, 2]));
        shapes.push(Shape::Lune(order));
    }
    shapes
}

fn certificate_shapes(orders: impl Iterator<Item = u32> + Clone) -> Vec<Shape> {
    let triples = [[3, 3, 2], [4, 3, 2], [5, 3, 2]];
    let mut shapes: Vec<Shape> = triples.into_iter().map(Shape::Triangle).collect();
    shapes.extend(orders.clone().map(|order| Shape::Triangle([order, 2, 2])));
    shapes.extend(orders.clone().map(Shape::Lune));
    shapes.extend(triples.map(Shape::Kite));
    shapes.extend(orders.clone().map(|order| Shape::Kite([order, 2, 2])));
    shapes.extend(orders.map(Shape::LuneDouble));
    shapes
}

#[test]
fn spherical_signatures_build_and_pass_their_checks() {
    let cases: [(&str, &[u32]); 7] = [
        ("*332", &[3, 3, 2]),
        ("*432", &[4, 3, 2]),
        ("*532", &[5, 3, 2]),
        ("*222", &[2, 2, 2]),
        ("*722", &[7, 2, 2]),
        ("*22", &[2, 2]),
        ("*55", &[5, 5]),
    ];
    for (text, orders) in cases {
        let domain = built(text);
        assert_eq!(domain.geometry(), Geometry::Spherical, "{text}");
        assert!(domain.verify().passed(), "{text}: {:?}", domain.verify());
        assert!(domain.reverses_orientation(), "{text}");
        assert!(parameters(&signature(text)).is_empty(), "{text}");
        assert_eq!(domain.sides().len(), orders.len(), "{text}");
        assert_eq!(domain.vertices().len(), orders.len(), "{text}");
        for (index, side) in domain.sides().iter().enumerate() {
            assert_eq!(side.partner(), index, "{text}");
            assert_eq!(side.kind(), PairingKind::Mirror, "{text}");
            assert_eq!(side.orientation(), Orientation::Reversing, "{text}");
        }
        for (angle, &order) in domain.corner_angles().into_iter().zip(orders) {
            let difference = (angle - PI / f64::from(order)).abs();
            assert!(difference <= ANGLE_BOUND, "{text}: {difference:e}");
        }
        if let [first, second, third] = domain.vertices() {
            let orientation = first.cross(*second).dot(*third);
            assert!(orientation > 0.0, "{text}: {orientation}");
        }
    }
    for (spelling, canonical) in SPELLINGS {
        assert_same_domain(
            &built(spelling),
            &built(canonical),
            2.0 * VERTEX_BOUND,
            spelling,
        );
    }
}

struct Quantities {
    worst: [Worst; 6],
}

impl Quantities {
    fn new(sides: usize, domains: usize) -> Self {
        let names = [
            "vertex, largest coordinate error",
            "normal, largest coordinate error",
            "side length, absolute error",
            "centre, largest coordinate error against the reference centre",
            "corner angle, absolute error against pi / n",
            "area, absolute error against 2 pi chi",
        ];
        let samples = [sides, sides, sides, domains, sides, domains];
        let mut worst = names
            .into_iter()
            .zip(samples)
            .map(|(name, samples)| Worst::new(format!("spherical {name}"), samples));
        Self {
            worst: [(); 6].map(|()| worst.next().expect("six quantities have names")),
        }
    }

    fn record(&mut self, domain: &FundamentalDomain, exact: &SphericalPolygon, input: &str) {
        let input = || input.to_owned();
        let [vertices, normals, lengths, centre, angles, area] = &mut self.worst;
        for (vertex, exact) in domain.vertices().iter().zip(&exact.vertices) {
            vertices.record(&largest_coordinate_error(*vertex, exact), &int(1), input);
        }
        for (side, exact) in domain.sides().iter().zip(&exact.normals) {
            normals.record(
                &largest_coordinate_error(side.normal(), exact),
                &int(1),
                input,
            );
        }
        for (length, exact) in domain.side_lengths().into_iter().zip(&exact.lengths) {
            lengths.record(&error(length, exact), &int(1), input);
        }
        centre.record(
            &largest_coordinate_error(domain.centre(), &exact.centre),
            &int(1),
            input,
        );
        for (angle, exact) in domain.corner_angles().into_iter().zip(&exact.angles) {
            angles.record(&error(angle, exact), &int(1), input);
        }
        area.record(&error(domain.area(), &exact.area), &int(1), input);
    }

    fn assert_within_bounds(&self, reference: &mut Reference) {
        let bounds = [
            VERTEX_BOUND,
            NORMAL_BOUND,
            LENGTH_BOUND,
            CENTRE_BOUND,
            ANGLE_BOUND,
            AREA_BOUND,
        ];
        for (worst, bound) in self.worst.iter().zip(bounds) {
            worst.assert_at_most(reference, bound);
        }
    }
}

#[test]
fn spherical_domains_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let shapes = measured_shapes();
    let sides = shapes
        .iter()
        .map(|shape| match shape {
            Shape::Triangle(_) | Shape::Kite(_) => 3,
            Shape::Lune(_) | Shape::LuneDouble(_) => 2,
        })
        .sum();
    let mut quantities = Quantities::new(sides, shapes.len());
    let mut residuals = LargestResiduals::new();
    for shape in &shapes {
        let text = shape.text();
        let domain = built(&text);
        let Some(exact) = shape.reference(&mut reference) else {
            panic!("{text} is not a reflection domain");
        };
        quantities.record(&domain, &exact, &text);
        residuals.record(&domain, &text);
        if let Shape::Lune(_) = shape {
            assert_eq!(domain.vertices(), [DVec3::Y, DVec3::NEG_Y], "{text}");
            assert_eq!(
                domain.vertices()[0].dot(domain.vertices()[1]),
                -1.0,
                "{text}"
            );
            assert_eq!(domain.centre(), spherical::ORIGIN, "{text}");
        }
    }
    quantities.assert_within_bounds(&mut reference);
    residuals.print_and_assert_samples(shapes.len());
}

pub(crate) struct Group {
    pub(crate) elements: Vec<(DMat3, usize)>,
}

impl Group {
    pub(crate) fn enumerate(domain: &FundamentalDomain, largest: usize) -> Self {
        let generators: Vec<DMat3> = domain.sides().iter().map(|side| side.pairing()).collect();
        let mut elements = vec![(DMat3::IDENTITY, 0)];
        let mut next = 0;
        while next < elements.len() {
            let (element, depth) = elements[next];
            next += 1;
            for generator in &generators {
                let product = element * *generator;
                let known = elements.iter().any(|(other, _)| {
                    (product - *other)
                        .to_cols_array()
                        .iter()
                        .all(|entry| entry.abs() <= MERGE_TOLERANCE)
                });
                if !known {
                    elements.push((product, depth + 1));
                    assert!(
                        elements.len() <= largest,
                        "the enumeration exceeds {largest} elements"
                    );
                }
            }
        }
        Self { elements }
    }
}

pub(crate) fn fold(domain: &FundamentalDomain, start: DVec3) -> Option<(DVec3, usize)> {
    let mut point = start;
    for steps in 0..=DEFAULT_FOLD_CAP {
        let size = point.abs().max_element();
        let outside = domain.sides().iter().find(|side| {
            let covector = side.covector();
            let scale = covector.abs().element_sum();
            spherical::side(covector, point) > FOLD_TOLERANCE * scale * size
        });
        match outside {
            None => return Some((point, steps)),
            Some(wall) if steps < DEFAULT_FOLD_CAP => point = wall.pairing() * point,
            Some(_) => return None,
        }
    }
    None
}

pub(crate) fn sphere_sample(count: u32) -> Vec<DVec3> {
    let golden = PI * (3.0 - 5.0_f64.sqrt());
    (0..count)
        .map(|index| {
            let height = 1.0 - 2.0 * (f64::from(index) + 0.5) / f64::from(count);
            let radius = (1.0 - height * height).sqrt();
            let angle = golden * f64::from(index);
            DVec3::new(radius * angle.cos(), radius * angle.sin(), height)
        })
        .collect()
}

fn lune_sample(domain: &FundamentalDomain) -> Vec<DVec3> {
    let mut points = sphere_sample(2000);
    let centre = domain.centre();
    points.extend([centre, -centre]);
    for &vertex in domain.vertices() {
        points.push(vertex);
        for exponent in [-3, -10, -30, -100, -200, -300] {
            let offset = 10.0_f64.powi(exponent);
            for direction in [
                DVec3::X,
                DVec3::Z,
                -DVec3::X,
                -DVec3::Z,
                DVec3::new(0.6, 0.0, 0.8),
            ] {
                points.push(vertex + direction * offset);
            }
        }
    }
    for side in domain.sides() {
        let normal = side.normal();
        let across = normal.cross(DVec3::Y);
        for step in 0..=32 {
            let angle = PI * f64::from(step) / 32.0;
            let on_side = DVec3::Y * angle.cos() + across * angle.sin();
            for offset in [0.0, 1e-12, -1e-12, 1e-6, -1e-6] {
                points.push(on_side + normal * offset);
            }
        }
    }
    points
}

struct Floor {
    smallest: f64,
    input: String,
    excess: usize,
    excess_input: String,
}

impl Floor {
    const fn new() -> Self {
        Self {
            smallest: f64::INFINITY,
            input: String::new(),
            excess: 0,
            excess_input: String::new(),
        }
    }
}

fn certify(shapes: &[Shape], reflections: &mut Floor, doubles: &mut Floor) {
    for shape in shapes {
        let floor = if shape.is_double() {
            &mut *doubles
        } else {
            &mut *reflections
        };
        let text = shape.text();
        let domain = built(&text);
        let order = signature(&text).group_order().map_or(0, |order| {
            usize::try_from(order.get()).unwrap_or(usize::MAX)
        });
        let group = Group::enumerate(&domain, order);
        assert_eq!(group.elements.len(), order, "{text}");
        let centre = domain.centre();
        let images: Vec<DVec3> = group
            .elements
            .iter()
            .map(|(element, _)| *element * centre)
            .collect();
        for (index, first) in images.iter().enumerate() {
            for second in &images[index + 1..] {
                let apart = spherical::distance(*first, *second);
                if apart < floor.smallest {
                    floor.smallest = apart;
                    floor.input = text.clone();
                }
            }
        }
        for ((_, depth), image) in group.elements.iter().zip(&images) {
            let Some((folded, steps)) = fold(&domain, *image) else {
                panic!("{text}: the image {image:?} reaches the fold cap");
            };
            if shape.is_double() {
                assert!(steps >= *depth, "{text}: {image:?}: {steps} steps");
                if steps - depth > floor.excess {
                    floor.excess = steps - depth;
                    floor.excess_input = text.clone();
                }
            } else {
                assert_eq!(steps, *depth, "{text}: {image:?}");
            }
            let offset = (folded - centre).abs().max_element();
            assert!(offset <= FOLD_BACK_TOLERANCE, "{text}: {offset:e}");
        }
        if let Shape::Lune(_) | Shape::LuneDouble(_) = shape {
            for point in lune_sample(&domain) {
                assert!(
                    fold(&domain, point).is_some(),
                    "{text}: {point:?} reaches the fold cap"
                );
            }
        }
    }
}

fn print_excess(floor: &Floor, name: &str) {
    println!(
        "{name}: largest number of fold steps above the breadth-first depth {}, at {}",
        floor.excess, floor.excess_input
    );
}

fn assert_floor(floor: &Floor, bound: f64, name: &str) {
    println!(
        "{name}: smallest distance between two images of the centre {:.3e}, floor {bound:e}, margin {:.1}, at {}",
        floor.smallest,
        floor.smallest / bound,
        floor.input
    );
    assert!(
        floor.smallest > bound,
        "{name}: {} at {}",
        floor.smallest,
        floor.input
    );
}

#[test]
fn spherical_groups_have_their_order_and_fold_back_their_images() {
    let (mut reflections, mut doubles) = (Floor::new(), Floor::new());
    certify(
        &certificate_shapes(2..=SMALL_ORDER),
        &mut reflections,
        &mut doubles,
    );
    assert_floor(&reflections, IMAGE_FLOOR, "certificate");
    assert_floor(&doubles, DOUBLE_IMAGE_FLOOR, "certificate of the doubles");
    print_excess(&doubles, "certificate of the doubles");
}

#[test]
fn spherical_groups_at_the_limit_have_their_order_and_fold_back_their_images() {
    let (mut reflections, mut doubles) = (Floor::new(), Floor::new());
    let shapes = [
        Shape::Triangle([1000, 2, 2]),
        Shape::Lune(1000),
        Shape::Kite([1000, 2, 2]),
        Shape::LuneDouble(1000),
    ];
    certify(&shapes, &mut reflections, &mut doubles);
    assert_floor(&reflections, LIMIT_IMAGE_FLOOR, "certificate at the limit");
    assert_floor(
        &doubles,
        LIMIT_DOUBLE_IMAGE_FLOOR,
        "certificate of the doubles at the limit",
    );
    print_excess(&doubles, "certificate of the doubles at the limit");
}
