use astro_float_num::BigFloat;
use escher_geometry::hyperbolic::{
    ORIGIN, boost, covector, distance, form, from_disk, inverse, isometry_defect, normalize_point,
    reflection, rotation, side, to_disk, translation, wall_through,
};
use escher_geometry::{euclidean, spherical};
use glam::{DVec2, DVec3};

use crate::common::{
    CASES, DOMAIN_TRIPLES, EUCLIDEAN_RADIUS, Model, REFERENCE_CASES, Rng, SPHERE_SEPARATION_FLOOR,
    SphereDomain, SphereFactor, Triangle, pair, point_off_wall, polar_point, table_points,
    table_vectors, tangent_basis, wall_normal,
};
use crate::reference::{central_jacobian, complex, epsilon, squared_modulus};
use escher_reference::{
    Reference, Vector, Worst, add, difference, div, dot, error, form as exact_form, int, max, mul,
    num, scale, sqrt, sub, vector,
};

const DISTANCE_SEPARATED_MINIMUM: usize = 8_000;
const TRANSLATION_COMPOSITION_TOLERANCE: f64 = 16.0;
const TRANSLATION_INVERSE_TOLERANCE: f64 = 16.0;
const CLOSE_POINTS_MEASURED_MINIMUM: usize = 9_500;
const CLOSE_POINTS_MOST_DEGENERATE: usize = 136;
const EUCLIDEAN_REFLECTION_TOLERANCE: f64 = 16.0;
const EUCLIDEAN_REFLECTION_DEFECT_TOLERANCE: f64 = 32.0;
const EUCLIDEAN_ORTHOGONALITY_TOLERANCE: f64 = 8.0;
const EUCLIDEAN_NORM_TOLERANCE: f64 = 32.0;
const EUCLIDEAN_DISTANCE_TOLERANCE: f64 = 16.0;
const SPHERE_SIDE_TOLERANCE: f64 = 16.0;
const SPHERE_NORMALIZE_TOLERANCE: f64 = 16.0;
const SPHERE_DISTANCE_TOLERANCE: f64 = 16.0;
const SPHERE_SAME_POINT_TOLERANCE: f64 = 32.0;
const SPHERE_ORTHOGONALITY_TOLERANCE: f64 = 8.0;
const SPHERE_NORM_TOLERANCE: f64 = 32.0;
const SPHERE_BOOST_TOLERANCE: f64 = 1e-14;
const SPHERE_TRANSLATION_TOLERANCE: f64 = 16.0;
const SPHERE_TRANSLATION_COMPOSITION_TOLERANCE: f64 = 32.0;
const SPHERE_TRANSLATION_DEFECT_TOLERANCE: f64 = 32.0;
const SPHERE_REFLECTION_TOLERANCE: f64 = 16.0;
const SPHERE_REFLECTION_DEFECT_TOLERANCE: f64 = 64.0;
const SPHERE_PRODUCT_DEFECT_TOLERANCE: f64 = 128.0;
const HALF_PLANE_POSITION_TOLERANCE: f64 = 16.0;
const HALF_PLANE_JACOBIAN_TOLERANCE: f64 = 32.0;
const BAND_POSITION_TOLERANCE: f64 = 16.0;
const BAND_JACOBIAN_TOLERANCE: f64 = 32.0;
const KLEIN_POSITION_TOLERANCE: f64 = 8.0;
const KLEIN_JACOBIAN_TOLERANCE: f64 = 16.0;
const MODEL_ROUND_TRIP_TOLERANCE: f64 = 16.0;

fn big(value: DVec3) -> Vector {
    vector(value.to_array())
}

fn terms_bound(x: &Vector, y: &Vector) -> BigFloat {
    add(
        &add(&mul(&x[0], &y[0]).abs(), &mul(&x[1], &y[1]).abs()),
        &mul(&x[2], &y[2]).abs(),
    )
}

fn check_form(worst: &mut Worst, x: DVec3, y: DVec3) {
    let (rx, ry) = (big(x), big(y));
    let scale = mul(&epsilon(), &terms_bound(&rx, &ry));
    worst.record(&error(form(x, y), &exact_form(&rx, &ry)), &scale, || {
        format!("{x:?} {y:?}")
    });
}

fn check_side(worst: &mut Worst, normal: DVec3, x: DVec3) {
    let c = covector(normal);
    let (rc, rx) = (big(c), big(x));
    let exact = add(
        &add(&mul(&rc[0], &rx[0]), &mul(&rc[1], &rx[1])),
        &mul(&rc[2], &rx[2]),
    );
    let scale = mul(&epsilon(), &terms_bound(&rc, &rx));
    worst.record(&error(side(c, x), &exact), &scale, || {
        format!("{c:?} {x:?}")
    });
}

#[test]
fn form_is_within_its_bound_of_the_reference() {
    let mut reference = Reference::new();
    let table = table_vectors();
    let mut worst = Worst::of_the_scale(
        "form, scaled by eps * sum |x_i y_i|",
        table.len() * table.len() + CASES,
    );
    for &x in &table {
        for &y in &table {
            check_form(&mut worst, x, y);
        }
    }
    let mut rng = Rng::new(1);
    for _ in 0..CASES {
        let x = rng.point_within(20.0) * rng.uniform(0.5, 2.0);
        let y = rng.point_within(20.0) * rng.uniform(0.5, 2.0);
        check_form(&mut worst, x, y);
    }
    worst.assert_at_most(&mut reference, 16.0);
}

#[test]
fn side_is_within_its_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut triangles = Worst::of_the_scale(
        "side on the domain triangles, scaled by eps * sum |c_i x_i|",
        DOMAIN_TRIPLES.len() * 12,
    );
    for orders in DOMAIN_TRIPLES {
        let triangle = Triangle::of(orders);
        for normal in triangle.normals {
            for x in triangle.points() {
                check_side(&mut triangles, normal, x);
            }
        }
    }
    triangles.assert_at_most(&mut reference, 8.0);
    let mut random = Worst::of_the_scale(
        "side on random walls and points, scaled by eps * sum |c_i x_i|",
        CASES,
    );
    let mut rng = Rng::new(2);
    for _ in 0..CASES {
        let normal = wall_normal(rng.uniform(0.0, 10.0), rng.angle());
        check_side(&mut random, normal, rng.point_within(10.0));
    }
    random.assert_at_most(&mut reference, 16.0);
}

fn check_normalize(worst: &mut Worst, x: DVec3) {
    let rx = big(x);
    let q = exact_form(&rx, &rx).neg();
    assert!(
        q.is_positive() && !q.is_zero(),
        "the exact form of {x:?} is not negative"
    );
    let Ok(computed) = normalize_point(x) else {
        panic!("normalize_point rejects {x:?}");
    };
    let y = scale(&div(&int(1), &sqrt(&q)), &rx);
    let kappa = div(&mul(&rx[2], &rx[2]), &q);
    let bound = mul(&mul(&epsilon(), &kappa), &y[2]);
    for (value, exact) in computed.to_array().into_iter().zip(&y) {
        worst.record(&error(value, exact), &bound, || format!("{x:?}"));
    }
}

#[test]
fn normalize_point_is_within_its_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale(
        "normalize_point, scaled by eps * kappa * y3",
        6 * REFERENCE_CASES,
    );
    for point in table_points() {
        if point.z <= 15.0_f64.cosh() {
            for factor in [0.5, 1.0, 2.0] {
                check_normalize(&mut worst, point * factor);
            }
        }
    }
    let mut rng = Rng::new(3);
    for _ in 0..REFERENCE_CASES {
        let point = rng.point_within(15.0);
        check_normalize(&mut worst, point);
        check_normalize(&mut worst, point * rng.uniform(0.5, 2.0));
    }
    worst.assert_at_most(&mut reference, 16.0);
}

struct DistanceWorst {
    square_root: Worst,
    with_result: Worst,
}

impl DistanceWorst {
    fn new(pairs: usize, separated: usize) -> Self {
        Self {
            square_root: Worst::of_the_scale("distance, scaled by sqrt(eps * x3 y3)", pairs),
            with_result: Worst::of_the_scale(
                "distance, scaled by eps * (x3 y3 / sinh d + d)",
                separated,
            ),
        }
    }

    fn check(&mut self, reference: &mut Reference, x: DVec3, y: DVec3) {
        let computed = distance(x, y);
        let (rx, ry) = (big(x), big(y));
        let d = reference.distance(&rx, &ry);
        let err = error(computed, &d);
        let product = mul(&num(x.z), &num(y.z));
        let input = || format!("{x:?} {y:?}");
        if !d.is_zero() {
            let conditioning = div(&product, &reference.sinh(&d));
            self.with_result
                .record(&err, &mul(&epsilon(), &add(&conditioning, &d)), input);
        }
        self.square_root
            .record(&err, &sqrt(&mul(&epsilon(), &product)), input);
    }

    fn assert_within_bounds(&self, reference: &mut Reference) {
        self.square_root.assert_at_most(reference, 16.0);
        self.with_result.assert_at_most(reference, 64.0);
    }
}

#[test]
fn distance_of_two_points_is_within_its_bound_of_the_reference() {
    let mut reference = Reference::new();
    let table: Vec<DVec3> = table_points()
        .into_iter()
        .filter(|point| point.z <= 10.0_f64.cosh())
        .collect();
    let table_pairs = table.len() * (table.len() - 1) / 2;
    let mut worst = DistanceWorst::new(table_pairs + REFERENCE_CASES, DISTANCE_SEPARATED_MINIMUM);
    for (index, &x) in table.iter().enumerate() {
        for &y in table.iter().skip(index + 1) {
            worst.check(&mut reference, x, y);
        }
    }
    let mut rng = Rng::new(4);
    for _ in 0..REFERENCE_CASES {
        let radius = rng.uniform(0.0, 10.0);
        let angle = rng.angle();
        let separation = rng.log_uniform(1e-8, 20.0);
        let (x, y) = pair(radius, angle, separation, rng.angle());
        worst.check(&mut reference, x, y);
    }
    worst.assert_within_bounds(&mut reference);
}

#[test]
fn distance_of_a_point_to_itself_is_at_least_zero_and_within_its_bound() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale("distance(x, x), scaled by sqrt(eps) * x3", CASES);
    let mut check = |x: DVec3| {
        let computed = distance(x, x);
        assert!(computed >= 0.0, "distance({x:?}, {x:?}) is {computed}");
        let scale = mul(&sqrt(&epsilon()), &num(x.z));
        worst.record(&num(computed), &scale, || format!("{x:?}"));
    };
    table_points().into_iter().for_each(&mut check);
    let mut rng = Rng::new(5);
    for _ in 0..CASES {
        check(rng.point_within(20.0));
    }
    worst.assert_at_most(&mut reference, 32.0);
}

struct WallWorst {
    orthogonality: Worst,
    norm: Worst,
    degenerate: usize,
    most_degenerate: usize,
}

impl WallWorst {
    fn new(
        name: &'static str,
        norm: &'static str,
        measured: usize,
        most_degenerate: usize,
    ) -> Self {
        Self {
            orthogonality: Worst::of_the_scale(name, 2 * measured),
            norm: Worst::of_the_scale(norm, measured),
            degenerate: 0,
            most_degenerate,
        }
    }

    fn check(&mut self, reference: &mut Reference, first: DVec3, second: DVec3) {
        let Ok(normal) = wall_through(first, second) else {
            self.degenerate += 1;
            return;
        };
        let (ra, rb, ru) = (big(first), big(second), big(normal));
        let d = reference.distance(&ra, &rb);
        if d.is_zero() {
            return;
        }
        let k = div(&mul(&ra[2], &rb[2]), &reference.sinh(&d));
        let input = || format!("{first:?} {second:?}");
        let orthogonality_scale = mul(&mul(&epsilon(), &k), &max(&ra[2], &rb[2]));
        for point in [&ra, &rb] {
            let value = exact_form(&ru, point).abs();
            self.orthogonality
                .record(&value, &orthogonality_scale, input);
        }
        let norm = sub(&exact_form(&ru, &ru), &int(1)).abs();
        self.norm.record(&norm, &mul(&epsilon(), &k), input);
    }

    fn assert_within_bounds(&self, reference: &mut Reference, orthogonality: f64) {
        println!(
            "wall_through: {} results were DegenerateWall",
            self.degenerate
        );
        assert!(
            self.degenerate <= self.most_degenerate,
            "{} results were DegenerateWall, more than {}",
            self.degenerate,
            self.most_degenerate
        );
        self.orthogonality.assert_at_most(reference, orthogonality);
        self.norm.assert_at_most(reference, 64.0);
    }
}

#[test]
fn wall_through_close_points_is_orthogonal_and_unit_within_its_bounds() {
    let mut reference = Reference::new();
    let mut worst = WallWorst::new(
        "wall_through close points, orthogonality scaled by eps * K * max(a3, b3)",
        "wall_through close points, norm scaled by eps * K",
        CLOSE_POINTS_MEASURED_MINIMUM,
        CLOSE_POINTS_MOST_DEGENERATE,
    );
    let mut rng = Rng::new(6);
    for _ in 0..REFERENCE_CASES {
        let radius = rng.uniform(0.0, 10.0);
        let angle = rng.angle();
        let separation = rng.log_uniform(1e-5, 5.0);
        let (a, b) = pair(radius, angle, separation, rng.angle());
        worst.check(&mut reference, a, b);
    }
    worst.assert_within_bounds(&mut reference, 256.0);
}

#[test]
fn wall_through_points_near_the_origin_is_orthogonal_and_unit_within_its_bounds() {
    let mut reference = Reference::new();
    let mut worst = WallWorst::new(
        "wall_through points near the origin, orthogonality scaled by eps * K * max(a3, b3)",
        "wall_through points near the origin, norm scaled by eps * K",
        REFERENCE_CASES,
        0,
    );
    let mut rng = Rng::new(7);
    for _ in 0..REFERENCE_CASES {
        let (first, second) = rng.separated_pair();
        worst.check(&mut reference, first, second);
    }
    worst.assert_within_bounds(&mut reference, 32.0);
}

fn relative_error(value: f64, exact: &BigFloat) -> (BigFloat, BigFloat) {
    (error(value, exact), mul(&epsilon(), &exact.abs()))
}

#[test]
fn boost_entries_are_within_their_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale(
        "boost entries, relative to eps * |cosh l| or |sinh l|",
        4 * REFERENCE_CASES,
    );
    let mut rng = Rng::new(8);
    let edges = [0.0, 700.0, -700.0, 1e-300, -1e-300, 5e-324, 1.0, -1.0];
    let random = (0..REFERENCE_CASES).map(|index| {
        if index % 2 == 0 {
            rng.uniform(-700.0, 700.0)
        } else {
            rng.sign() * rng.log_uniform(1e-300, 700.0)
        }
    });
    for length in edges.into_iter().chain(random.collect::<Vec<_>>()) {
        let matrix = boost(length);
        let rl = num(length);
        let (cosh, sinh) = (reference.cosh(&rl), reference.sinh(&rl));
        for (value, exact) in [
            (matrix.x_axis.x, &cosh),
            (matrix.z_axis.z, &cosh),
            (matrix.x_axis.z, &sinh),
            (matrix.z_axis.x, &sinh),
        ] {
            let (err, scale) = relative_error(value, exact);
            worst.record(&err, &scale, || format!("{length:e}"));
        }
        assert_eq!(
            [
                matrix.x_axis.y,
                matrix.y_axis.x,
                matrix.y_axis.y,
                matrix.y_axis.z,
                matrix.z_axis.y
            ],
            [0.0, 0.0, 1.0, 0.0, 0.0],
        );
    }
    worst.assert_at_most(&mut reference, 16.0);
}

#[test]
fn rotation_entries_are_within_their_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale("rotation entries, absolute", 4 * REFERENCE_CASES);
    let mut rng = Rng::new(9);
    let edges = [0.0, 1000.0, -1000.0, core::f64::consts::PI, 1e-300];
    let random: Vec<f64> = (0..REFERENCE_CASES)
        .map(|_| rng.uniform(-1000.0, 1000.0))
        .collect();
    for angle in edges.into_iter().chain(random) {
        let matrix = rotation(angle);
        let ra = num(angle);
        let (cos, sin) = (reference.cos(&ra), reference.sin(&ra));
        let minus_sin = sin.neg();
        for (value, exact) in [
            (matrix.x_axis.x, &cos),
            (matrix.y_axis.y, &cos),
            (matrix.x_axis.y, &sin),
            (matrix.y_axis.x, &minus_sin),
        ] {
            worst.record(&error(value, exact), &int(1), || format!("{angle:e}"));
        }
        assert_eq!(
            [
                matrix.x_axis.z,
                matrix.y_axis.z,
                matrix.z_axis.x,
                matrix.z_axis.y,
                matrix.z_axis.z
            ],
            [0.0, 0.0, 0.0, 0.0, 1.0],
        );
    }
    worst.assert_at_most(&mut reference, 1e-14);
}

fn record_terms(worst: &mut Worst, computed: DVec3, terms: &[[BigFloat; 2]; 3], input: &str) {
    for (value, [first, second]) in computed.to_array().into_iter().zip(terms) {
        let exact = add(first, second);
        let scale = mul(&epsilon(), &max(&first.abs(), &second.abs()));
        worst.record(&error(value, &exact), &scale, || input.to_owned());
    }
}

#[test]
fn boost_and_rotation_of_a_point_match_the_componentwise_maps() {
    let mut reference = Reference::new();
    let mut boosts = Worst::of_the_scale(
        "boost(l) * x against the componentwise map, scaled by eps * s",
        3 * REFERENCE_CASES,
    );
    let mut rotations = Worst::of_the_scale(
        "rotation(angle) * x against the componentwise map, scaled by eps * s",
        3 * REFERENCE_CASES,
    );
    let mut rng = Rng::new(10);
    let zero = int(0);
    for index in 0..REFERENCE_CASES {
        let x = if index == 0 {
            ORIGIN
        } else {
            rng.point_within(3.0)
        };
        let rx = big(x);
        let length = rng.uniform(-20.0, 20.0);
        let rl = num(length);
        let (cosh, sinh) = (reference.cosh(&rl), reference.sinh(&rl));
        let terms = [
            [mul(&rx[0], &cosh), mul(&rx[2], &sinh)],
            [rx[1].clone(), zero.clone()],
            [mul(&rx[2], &cosh), mul(&rx[0], &sinh)],
        ];
        record_terms(
            &mut boosts,
            boost(length) * x,
            &terms,
            &format!("{length:e} {x:?}"),
        );
        let angle = rng.uniform(-1000.0, 1000.0);
        let ra = num(angle);
        let (cos, sin) = (reference.cos(&ra), reference.sin(&ra));
        let terms = [
            [mul(&rx[0], &cos), mul(&rx[1], &sin).neg()],
            [mul(&rx[1], &cos), mul(&rx[0], &sin)],
            [rx[2].clone(), zero.clone()],
        ];
        record_terms(
            &mut rotations,
            rotation(angle) * x,
            &terms,
            &format!("{angle:e} {x:?}"),
        );
    }
    boosts.assert_at_most(&mut reference, 32.0);
    rotations.assert_at_most(&mut reference, 32.0);
}

fn check_translation(reference: &mut Reference, worst: &mut Worst, target: DVec3) {
    let image = translation(target) * ORIGIN;
    let (t1, t2) = (num(target.x), num(target.y));
    let spatial = add(&mul(&t1, &t1), &mul(&t2, &t2));
    let p3 = sqrt(&add(&int(1), &spatial));
    let r = reference.asinh(&sqrt(&spatial));
    let scale = mul(&mul(&epsilon(), &add(&int(1), &r)), &p3);
    for (value, exact) in image.to_array().into_iter().zip([&t1, &t2, &p3]) {
        worst.record(&error(value, exact), &scale, || format!("{target:?}"));
    }
}

#[test]
fn translation_maps_the_origin_to_its_target_within_its_bound() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale(
        "translation(t) * ORIGIN, scaled by eps * (1 + r) * p3",
        3 * REFERENCE_CASES,
    );
    for target in SMALL_TARGETS {
        check_translation(&mut reference, &mut worst, target);
    }
    let mut rng = Rng::new(11);
    for index in 0..REFERENCE_CASES {
        let radius = if index % 2 == 0 {
            rng.uniform(0.1, 709.0)
        } else {
            rng.log_uniform(0.1, 709.0)
        };
        let target = polar_point(radius, rng.angle());
        check_translation(&mut reference, &mut worst, target);
    }
    worst.assert_at_most(&mut reference, 8.0);
}

const SMALL_TARGETS: [DVec3; 3] = [
    DVec3::new(0.0, 0.0, 1.0),
    DVec3::new(1e-300, 0.0, 1.0),
    DVec3::new(5e-324, 0.0, 1.0),
];

#[test]
fn translation_of_the_three_small_targets_maps_the_origin_to_the_rounded_point() {
    for target in SMALL_TARGETS {
        let image = translation(target) * ORIGIN;
        let t1 = num(target.x);
        let p3 = sqrt(&add(&int(1), &mul(&t1, &t1)));
        for (value, exact) in image.to_array().into_iter().zip([&t1, &int(0), &p3]) {
            let unit = mul(&epsilon(), &exact.abs());
            assert!(
                error(value, exact) <= unit,
                "{image:?} for {target:?} is more than one rounding unit from the point"
            );
        }
    }
}

fn translation_composition(t1: f64, t2: f64, p: &Vector) -> (Vector, BigFloat) {
    let (t1, t2) = (num(t1), num(t2));
    let spatial = sqrt(&add(&mul(&t1, &t1), &mul(&t2, &t2)));
    let (c, s) = if spatial.is_zero() {
        (int(1), int(0))
    } else {
        (div(&t1, &spatial), div(&t2, &spatial))
    };
    let cosh = sqrt(&add(&int(1), &mul(&spatial, &spatial)));
    let sinh = spatial;
    let turned = [
        add(&mul(&p[0], &c), &mul(&p[1], &s)),
        sub(&mul(&p[1], &c), &mul(&p[0], &s)),
        p[2].clone(),
    ];
    let boosted = [
        add(&mul(&turned[0], &cosh), &mul(&turned[2], &sinh)),
        turned[1].clone(),
        add(&mul(&turned[2], &cosh), &mul(&turned[0], &sinh)),
    ];
    let image = [
        sub(&mul(&boosted[0], &c), &mul(&boosted[1], &s)),
        add(&mul(&boosted[1], &c), &mul(&boosted[0], &s)),
        boosted[2].clone(),
    ];
    (image, cosh)
}

fn translation_radius(rng: &mut Rng, index: usize, largest: f64) -> f64 {
    if index.is_multiple_of(2) {
        rng.uniform(0.1, largest)
    } else {
        rng.log_uniform(0.1, largest)
    }
}

#[test]
fn translation_maps_points_off_the_axis_as_its_composition_does() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale(
        "translation(t) * p against its composition, scaled by eps * (1 + r) * cosh(rho) * p3",
        3 * REFERENCE_CASES,
    );
    let mut rng = Rng::new(16);
    for index in 0..REFERENCE_CASES {
        let target = polar_point(translation_radius(&mut rng, index, 700.0), rng.angle());
        let point = if index % 4 == 0 {
            polar_point(rng.uniform(0.5, 3.0), core::f64::consts::FRAC_PI_2)
        } else {
            rng.point_within(3.0)
        };
        let computed = translation(target) * point;
        let rp = big(point);
        let (image, cosh) = translation_composition(target.x, target.y, &rp);
        let r = reference.asinh(&sqrt(&sub(&mul(&cosh, &cosh), &int(1))));
        let scale = mul(&mul(&mul(&epsilon(), &add(&int(1), &r)), &cosh), &rp[2]);
        for (value, exact) in computed.to_array().into_iter().zip(&image) {
            worst.record(&error(value, exact), &scale, || {
                format!("{target:?} {point:?}")
            });
        }
    }
    worst.assert_at_most(&mut reference, TRANSLATION_COMPOSITION_TOLERANCE);
}

#[test]
fn translation_is_an_isometry_within_the_bound_of_longer_boosts() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale(
        "isometry_defect of translation, scaled by eps * cosh(rho)^2",
        CASES,
    );
    let mut rng = Rng::new(17);
    for index in 0..CASES {
        let target = polar_point(translation_radius(&mut rng, index, 350.0), rng.angle());
        let g = (1.0 + target.x * target.x + target.y * target.y).sqrt();
        worst.record(
            &num(isometry_defect(translation(target))),
            &num(f64::EPSILON * g * g),
            || format!("{target:?}"),
        );
    }
    worst.assert_at_most(&mut reference, 512.0);
}

#[test]
fn inverse_of_translation_maps_its_target_to_the_origin() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale(
        "inverse(translation(t)) * t against ORIGIN, scaled by eps * (1 + r) * t3^2",
        3 * CASES,
    );
    let mut rng = Rng::new(18);
    for index in 0..CASES {
        let radius = translation_radius(&mut rng, index, 350.0);
        let target = polar_point(radius, rng.angle());
        let back = inverse(translation(target)) * target;
        let scale = f64::EPSILON * (1.0 + radius) * target.z * target.z;
        for (value, exact) in back.to_array().into_iter().zip(ORIGIN.to_array()) {
            worst.record(&error(value, &num(exact)), &num(scale), || {
                format!("{target:?}")
            });
        }
    }
    worst.assert_at_most(&mut reference, TRANSLATION_INVERSE_TOLERANCE);
}

fn reflection_reference(u: &Vector, x: &Vector) -> (Vector, Vector) {
    let c = [u[0].clone(), u[1].clone(), u[2].neg()];
    let twice = mul(&int(2), &exact_form(u, x));
    let image = difference(x, &scale(&twice, u));
    let sum = add(
        &add(&mul(&c[0], &x[0]).abs(), &mul(&c[1], &x[1]).abs()),
        &mul(&c[2], &x[2]).abs(),
    );
    let sigma =
        [0, 1, 2].map(|index| add(&x[index].abs(), &mul(&mul(&int(2), &u[index].abs()), &sum)));
    (image, sigma)
}

fn check_reflection(worst: &mut Worst, normal: DVec3, x: DVec3) {
    let computed = reflection(normal) * x;
    let (image, sigma) = reflection_reference(&big(normal), &big(x));
    for ((value, exact), sigma) in computed.to_array().into_iter().zip(&image).zip(&sigma) {
        worst.record(&error(value, exact), &mul(&epsilon(), sigma), || {
            format!("{normal:?} {x:?}")
        });
    }
}

fn check_reflected_normal(worst: &mut Worst, normal: DVec3) {
    let computed = reflection(normal) * normal;
    let ru = big(normal);
    let squares = add(
        &add(&mul(&ru[0], &ru[0]), &mul(&ru[1], &ru[1])),
        &mul(&ru[2], &ru[2]),
    );
    let factor = add(&int(1), &mul(&int(2), &squares));
    for (value, exact) in computed.to_array().into_iter().zip(&ru) {
        let scale = mul(&mul(&epsilon(), &exact.abs()), &factor);
        worst.record(&error(value, &exact.neg()), &scale, || {
            format!("{normal:?}")
        });
    }
}

#[test]
fn reflection_is_within_its_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut points = Worst::of_the_scale("reflection(u) * x, scaled by eps * sigma_i", 3 * CASES);
    let mut normals = Worst::of_the_scale(
        "reflection(u) * u against -u, scaled by eps |u_i| (1 + 2 sum u_j^2)",
        3 * CASES,
    );
    let mut rng = Rng::new(12);
    for index in 0..CASES {
        let distance = rng.uniform(0.0, 10.0);
        let direction = rng.angle();
        let normal = wall_normal(distance, direction);
        let x = if index % 2 == 0 {
            rng.point_within(3.0)
        } else {
            let offset = rng.sign() * rng.log_uniform(1e-12, 1.0);
            point_off_wall(distance, direction, rng.uniform(-3.0, 3.0), offset)
        };
        check_reflection(&mut points, normal, x);
        check_reflected_normal(&mut normals, normal);
    }
    points.assert_at_most(&mut reference, 16.0);
    normals.assert_at_most(&mut reference, 32.0);
}

fn check_to_disk(worst: &mut Worst, x: DVec3) {
    let computed = to_disk(x);
    let rx = big(x);
    let denominator = add(&rx[2], &int(1));
    for (value, numerator) in computed.to_array().into_iter().zip(&rx) {
        let (err, scale) = relative_error(value, &div(numerator, &denominator));
        worst.record(&err, &scale, || format!("{x:?}"));
    }
}

#[test]
fn to_disk_is_within_its_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale("to_disk, relative to eps * |reference|", 2 * CASES);
    check_to_disk(&mut worst, ORIGIN);
    check_to_disk(&mut worst, polar_point(38.0, 0.3));
    let mut rng = Rng::new(13);
    for _ in 0..CASES {
        check_to_disk(&mut worst, rng.point_within(38.0));
    }
    worst.assert_at_most(&mut reference, 8.0);
}

fn disk_position(rng: &mut Rng) -> DVec2 {
    let gap = rng.log_uniform(1e-12, 1.0);
    let angle = rng.angle();
    DVec2::new(angle.cos(), angle.sin()) * (1.0 - gap)
}

fn check_from_disk(worst: &mut Worst, z: DVec2) {
    let Some(computed) = from_disk(z) else {
        panic!("from_disk rejects {z:?}");
    };
    let (z1, z2) = (num(z.x), num(z.y));
    let s = add(&mul(&z1, &z1), &mul(&z2, &z2));
    let d = sub(&int(1), &s);
    let exact = [
        div(&mul(&int(2), &z1), &d),
        div(&mul(&int(2), &z2), &d),
        div(&add(&int(1), &s), &d),
    ];
    for (value, exact) in computed.to_array().into_iter().zip(&exact) {
        let scale = div(&mul(&epsilon(), &exact.abs()), &d);
        worst.record(&error(value, exact), &scale, || format!("{z:?}"));
    }
}

#[test]
fn from_disk_is_within_its_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale("from_disk, relative to eps / (1 - |z|^2)", 3 * CASES);
    check_from_disk(&mut worst, DVec2::ZERO);
    check_from_disk(&mut worst, DVec2::new(1.0 - 1e-12, 0.0));
    check_from_disk(&mut worst, DVec2::new(1.0 - 1e-9, 0.0));
    let mut rng = Rng::new(14);
    for _ in 0..CASES {
        check_from_disk(&mut worst, disk_position(&mut rng));
    }
    worst.assert_at_most(&mut reference, 8.0);
}

#[test]
fn disk_round_trips_are_within_their_bounds() {
    let mut reference = Reference::new();
    let mut from_then_to =
        Worst::of_the_scale("from_disk then to_disk, absolute in eps", 2 * (CASES + 2));
    let mut to_then_from =
        Worst::of_the_scale("to_disk then from_disk, scaled by eps * x3^2", 3 * CASES);
    let mut rng = Rng::new(15);
    let mut check_position = |z: DVec2| {
        let Some(back) = from_disk(z).map(to_disk) else {
            panic!("from_disk rejects {z:?}");
        };
        for (value, exact) in back.to_array().into_iter().zip(z.to_array()) {
            from_then_to.record(&error(value, &num(exact)), &epsilon(), || format!("{z:?}"));
        }
    };
    check_position(DVec2::new(1.0 - 1e-12, 0.0));
    check_position(DVec2::new(1.0 - 1e-9, 0.0));
    for index in 0..CASES {
        let z = if index == 0 {
            DVec2::ZERO
        } else {
            disk_position(&mut rng)
        };
        check_position(z);
        let x = if index == 0 {
            ORIGIN
        } else {
            rng.point_within(12.0)
        };
        let Some(back) = from_disk(to_disk(x)) else {
            panic!("from_disk rejects to_disk of {x:?}");
        };
        let scale = mul(&epsilon(), &mul(&num(x.z), &num(x.z)));
        for (value, exact) in back.to_array().into_iter().zip(x.to_array()) {
            to_then_from.record(&error(value, &num(exact)), &scale, || format!("{x:?}"));
        }
    }
    from_then_to.assert_at_most(&mut reference, 8.0);
    to_then_from.assert_at_most(&mut reference, 32.0);
}

struct ModelBounds {
    seed: u64,
    position: f64,
    jacobian: f64,
}

fn position_scale(model: Model, squared_klein_radius: &BigFloat) -> BigFloat {
    match model {
        Model::HalfPlane | Model::Band => epsilon(),
        Model::Klein => div(&epsilon(), &sqrt(&sub(&int(1), squared_klein_radius))),
    }
}

fn jacobian_scale(model: Model, largest: &BigFloat, squared_klein_radius: &BigFloat) -> BigFloat {
    match model {
        Model::HalfPlane => mul(&epsilon(), largest),
        Model::Band => epsilon(),
        Model::Klein => div(
            &mul(&epsilon(), largest),
            &sub(&int(1), squared_klein_radius),
        ),
    }
}

fn scale_names(model: Model) -> (&'static str, &'static str) {
    match model {
        Model::HalfPlane => ("absolute in eps", "scaled by eps * m"),
        Model::Band => ("absolute in eps", "absolute in eps"),
        Model::Klein => (
            "scaled by eps / sqrt(1 - |s|^2)",
            "scaled by eps * m / (1 - |s|^2)",
        ),
    }
}

fn check_model_map(model: Model, bounds: &ModelBounds) {
    let mut reference = Reference::new();
    let name = model.name();
    let (position_unit, jacobian_unit) = scale_names(model);
    let cases = model.reference_cases();
    let samples = cases + 1;
    let mut positions =
        Worst::of_the_scale(format!("{name}, position, {position_unit}"), 2 * samples);
    let mut jacobians =
        Worst::of_the_scale(format!("{name}, Jacobian, {jacobian_unit}"), 4 * samples);
    let mut trips = Worst::of_the_scale(
        format!("{name}, round trip, scaled by eps / (1 - |z|)"),
        2 * samples,
    );
    let mut rng = Rng::new(bounds.seed);
    for sample in model.model_set(&mut reference, &mut rng, cases) {
        let (w, disk) = (sample.position, sample.disk);
        let Some((computed, jacobian)) = model.to_disk(w) else {
            panic!("{name} rejects {w:?}");
        };
        let input = || format!("{w:?} from {disk:?}");
        let exact_w = complex(w.to_array());
        let exact = model.exact_to_disk(&mut reference, &exact_w);
        let exact_jacobian = central_jacobian(
            &mut |position| model.exact_to_disk(&mut reference, position),
            &exact_w,
        );
        let largest = exact_jacobian
            .iter()
            .flatten()
            .fold(int(0), |largest, entry| max(&largest, &entry.abs()));
        let squared_radius = squared_modulus(&exact_w);
        let scale = position_scale(model, &squared_radius);
        for (value, exact) in computed.to_array().into_iter().zip(&exact) {
            positions.record(&error(value, exact), &scale, input);
        }
        let scale = jacobian_scale(model, &largest, &squared_radius);
        for (column, exact_column) in [jacobian.x_axis, jacobian.y_axis]
            .into_iter()
            .zip(&exact_jacobian)
        {
            for (value, exact) in column.to_array().into_iter().zip(exact_column) {
                jacobians.record(&error(value, exact), &scale, input);
            }
        }
        let gap = sub(&int(1), &sqrt(&squared_modulus(&complex(disk.to_array()))));
        let scale = div(&epsilon(), &gap);
        for (value, exact) in computed.to_array().into_iter().zip(disk.to_array()) {
            trips.record(&error(value, &num(exact)), &scale, input);
        }
    }
    positions.assert_at_most(&mut reference, bounds.position);
    jacobians.assert_at_most(&mut reference, bounds.jacobian);
    trips.assert_at_most(&mut reference, MODEL_ROUND_TRIP_TOLERANCE);
}

#[test]
fn disk_from_half_plane_is_within_its_bounds_of_the_reference() {
    check_model_map(
        Model::HalfPlane,
        &ModelBounds {
            seed: 50,
            position: HALF_PLANE_POSITION_TOLERANCE,
            jacobian: HALF_PLANE_JACOBIAN_TOLERANCE,
        },
    );
}

#[test]
fn disk_from_band_is_within_its_bounds_of_the_reference() {
    check_model_map(
        Model::Band,
        &ModelBounds {
            seed: 51,
            position: BAND_POSITION_TOLERANCE,
            jacobian: BAND_JACOBIAN_TOLERANCE,
        },
    );
}

#[test]
fn disk_from_klein_is_within_its_bounds_of_the_reference() {
    check_model_map(
        Model::Klein,
        &ModelBounds {
            seed: 52,
            position: KLEIN_POSITION_TOLERANCE,
            jacobian: KLEIN_JACOBIAN_TOLERANCE,
        },
    );
}

#[test]
fn domain_triangle_inputs_are_within_the_bounds_of_each_row() {
    let mut reference = Reference::new();
    let sides = 3 * DOMAIN_TRIPLES.len();
    let mut distances = DistanceWorst::new(sides, sides);
    let mut walls = WallWorst::new(
        "wall_through on the domain triangles, orthogonality",
        "wall_through on the domain triangles, norm",
        sides,
        0,
    );
    let mut translations = Worst::of_the_scale("translation on the domain triangles", sides);
    let mut reflections = Worst::of_the_scale("reflection on the domain triangles", 3 * 4 * sides);
    for orders in DOMAIN_TRIPLES {
        let triangle = Triangle::of(orders);
        check_translation(&mut reference, &mut translations, triangle.target);
        for (index, &first) in triangle.vertices.iter().enumerate() {
            let second = triangle.vertices[(index + 1) % 3];
            distances.check(&mut reference, first, second);
            walls.check(&mut reference, first, second);
        }
        for normal in triangle.normals {
            for x in triangle.points() {
                check_reflection(&mut reflections, normal, x);
            }
        }
    }
    distances.assert_within_bounds(&mut reference);
    walls.assert_within_bounds(&mut reference, 32.0);
    translations.assert_at_most(&mut reference, 8.0);
    reflections.assert_at_most(&mut reference, 16.0);
}

#[test]
fn to_disk_radius_at_distances_37_and_38_point_2_is_near_its_reference() {
    let mut reference = Reference::new();
    let mut worst =
        Worst::of_the_scale("to_disk radius at distance 37 and 38.2, absolute in eps", 2);
    for distance in [37.0_f64, 38.2] {
        let point = DVec3::new(distance.sinh(), 0.0, distance.cosh());
        let radius = to_disk(point).length();
        let exact = div(&num(point.x), &add(&num(point.z), &int(1)));
        worst.record(&error(radius, &exact), &epsilon(), || format!("{distance}"));
    }
    worst.assert_at_most(&mut reference, 4.0);
}

fn euclidean_reflection_reference(c: &Vector, x: &Vector) -> (Vector, Vector) {
    let offset = c[2].neg();
    let along = add(&mul(&c[0], &x[0]), &mul(&c[1], &x[1]));
    let twice = mul(&int(2), &sub(&offset, &along));
    let image = [
        add(&x[0], &mul(&twice, &c[0])),
        add(&x[1], &mul(&twice, &c[1])),
        x[2].clone(),
    ];
    let sum = add(
        &add(&mul(&c[0], &x[0]).abs(), &mul(&c[1], &x[1]).abs()),
        &mul(&c[2], &x[2]).abs(),
    );
    let u = [c[0].clone(), c[1].clone(), int(0)];
    let sigma =
        [0, 1, 2].map(|index| add(&x[index].abs(), &mul(&mul(&int(2), &u[index].abs()), &sum)));
    (image, sigma)
}

fn plane_length(x: &Vector) -> BigFloat {
    sqrt(&add(&mul(&x[0], &x[0]), &mul(&x[1], &x[1])))
}

#[test]
fn euclidean_reflection_is_within_its_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut points = Worst::of_the_scale(
        "euclidean reflection(c) * x, scaled by eps * sigma_i",
        3 * CASES,
    );
    let mut defects = Worst::of_the_scale(
        "euclidean isometry_defect(reflection(c)), scaled by eps",
        CASES,
    );
    let mut rng = Rng::new(21);
    for index in 0..CASES {
        let line = rng.line_covector(u32::try_from(index).unwrap_or(0));
        let x = rng.euclidean_point_within(EUCLIDEAN_RADIUS);
        let computed = euclidean::reflection(line) * x;
        let (image, sigma) = euclidean_reflection_reference(&big(line), &big(x));
        for ((value, exact), sigma) in computed.to_array().into_iter().zip(&image).zip(&sigma) {
            points.record(&error(value, exact), &mul(&epsilon(), sigma), || {
                format!("{line:?} {x:?}")
            });
        }
        let defect = euclidean::isometry_defect(euclidean::reflection(line));
        defects.record(&num(defect), &epsilon(), || format!("{line:?}"));
    }
    points.assert_at_most(&mut reference, EUCLIDEAN_REFLECTION_TOLERANCE);
    defects.assert_at_most(&mut reference, EUCLIDEAN_REFLECTION_DEFECT_TOLERANCE);
}

#[test]
fn euclidean_wall_through_and_distance_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut orthogonality = Worst::of_the_scale(
        "euclidean wall_through, |c . p| and |c . q|, scaled by eps * (1 + |p| + |q| + |p| |q| / |p - q|)",
        2 * REFERENCE_CASES,
    );
    let mut norm = Worst::of_the_scale(
        "euclidean wall_through, |c_1^2 + c_2^2 - 1|, scaled by eps",
        REFERENCE_CASES,
    );
    let mut distances = Worst::of_the_scale(
        "euclidean distance, relative error, scaled by eps",
        REFERENCE_CASES,
    );
    let mut rng = Rng::new(22);
    for _ in 0..REFERENCE_CASES {
        let (first, second) = rng.euclidean_pair();
        let (rp, rq) = (big(first), big(second));
        let apart = plane_length(&difference(&rp, &rq));
        let (p_length, q_length) = (plane_length(&rp), plane_length(&rq));
        let scale = mul(
            &epsilon(),
            &add(
                &add(&add(&int(1), &p_length), &q_length),
                &div(&mul(&p_length, &q_length), &apart),
            ),
        );
        let input = || format!("{first:?} {second:?}");
        let Ok(line) = euclidean::wall_through(first, second) else {
            panic!("no line through {first:?} {second:?}");
        };
        let rc = big(line);
        orthogonality.record(&dot(&rc, &rp).abs(), &scale, input);
        orthogonality.record(&dot(&rc, &rq).abs(), &scale, input);
        let squares = add(&mul(&rc[0], &rc[0]), &mul(&rc[1], &rc[1]));
        norm.record(&sub(&squares, &int(1)).abs(), &epsilon(), input);
        distances.record(
            &error(euclidean::distance(first, second), &apart),
            &mul(&epsilon(), &apart),
            input,
        );
    }
    orthogonality.assert_at_most(&mut reference, EUCLIDEAN_ORTHOGONALITY_TOLERANCE);
    norm.assert_at_most(&mut reference, EUCLIDEAN_NORM_TOLERANCE);
    distances.assert_at_most(&mut reference, EUCLIDEAN_DISTANCE_TOLERANCE);
}

fn unit(x: &Vector) -> Vector {
    let length = sqrt(&dot(x, x));
    [
        div(&x[0], &length),
        div(&x[1], &length),
        div(&x[2], &length),
    ]
}

fn record_vector(worst: &mut Worst, computed: DVec3, exact: &Vector, input: &dyn Fn() -> String) {
    for (value, exact) in computed.to_array().into_iter().zip(exact) {
        worst.record(&error(value, exact), &epsilon(), input);
    }
}

#[test]
fn spherical_side_is_within_its_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale("spherical side, scaled by eps * sum |u_i x_i|", CASES);
    let mut check = |normal: DVec3, point: DVec3, name: &str| {
        let (ru, rx) = (big(normal), big(point));
        let scale = mul(&epsilon(), &terms_bound(&ru, &rx));
        worst.record(
            &error(spherical::side(normal, point), &dot(&ru, &rx)),
            &scale,
            || format!("{name} {normal:?} {point:?}"),
        );
    };
    let mut rng = Rng::new(30);
    for _ in 0..CASES {
        check(rng.sphere_point(), rng.sphere_point(), "random");
    }
    for domain in SphereDomain::all() {
        for &normal in &domain.normals {
            for &point in &domain.points {
                check(normal, point, &domain.name);
            }
        }
    }
    worst.assert_at_most(&mut reference, SPHERE_SIDE_TOLERANCE);
}

#[test]
fn spherical_normalize_point_is_within_its_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale(
        "spherical normalize_point, each coordinate, scaled by eps",
        3 * CASES,
    );
    let mut rng = Rng::new(31);
    let mut measured = 0;
    while measured < CASES {
        let vector = match measured % 3 {
            0 => rng.sphere_point() * rng.uniform(0.5, 2.0),
            1 => rng.sphere_point() + rng.sphere_point(),
            _ => rng.sphere_point() + rng.sphere_point() + rng.sphere_point(),
        };
        if vector.length() < 1e-3 {
            continue;
        }
        measured += 1;
        let Ok(point) = spherical::normalize_point(vector) else {
            panic!("normalize_point rejects {vector:?}");
        };
        record_vector(&mut worst, point, &unit(&big(vector)), &|| {
            format!("{vector:?}")
        });
    }
    worst.assert_at_most(&mut reference, SPHERE_NORMALIZE_TOLERANCE);
}

#[test]
fn spherical_distance_and_wall_through_are_within_their_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut distances =
        Worst::of_the_scale("spherical distance, scaled by eps / sin d", REFERENCE_CASES);
    let mut orthogonality = Worst::of_the_scale(
        "spherical wall_through, |u . a| and |u . b|, scaled by eps / sin d",
        2 * REFERENCE_CASES,
    );
    let mut norm = Worst::of_the_scale(
        "spherical wall_through, |u . u - 1|, scaled by eps",
        REFERENCE_CASES,
    );
    let mut rng = Rng::new(32);
    for _ in 0..REFERENCE_CASES {
        let (first, second) = rng.sphere_pair(SPHERE_SEPARATION_FLOOR);
        let (ra, rb) = (big(first), big(second));
        let cosine = dot(&ra, &rb);
        let d = reference.acos(&cosine);
        let scale = div(&epsilon(), &reference.sin(&d));
        let input = || format!("{first:?} {second:?}");
        distances.record(
            &error(spherical::distance(first, second), &d),
            &scale,
            input,
        );
        let Ok(normal) = spherical::wall_through(first, second) else {
            panic!("no great circle through {first:?} {second:?}");
        };
        let ru = big(normal);
        orthogonality.record(&dot(&ru, &ra).abs(), &scale, input);
        orthogonality.record(&dot(&ru, &rb).abs(), &scale, input);
        norm.record(&sub(&dot(&ru, &ru), &int(1)).abs(), &epsilon(), input);
    }
    distances.assert_at_most(&mut reference, SPHERE_DISTANCE_TOLERANCE);
    orthogonality.assert_at_most(&mut reference, SPHERE_ORTHOGONALITY_TOLERANCE);
    norm.assert_at_most(&mut reference, SPHERE_NORM_TOLERANCE);
}

#[test]
fn spherical_distance_of_a_point_to_itself_is_at_least_zero_and_within_its_bound() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale("spherical distance(x, x), scaled by sqrt(eps)", CASES);
    let mut rng = Rng::new(33);
    let points: Vec<DVec3> = (0..CASES).map(|_| rng.sphere_point()).collect();
    for point in core::iter::once(spherical::ORIGIN).chain(points) {
        let d = spherical::distance(point, point);
        assert!(d >= 0.0, "{d} for {point:?}");
        worst.record(&num(d), &num(f64::EPSILON.sqrt()), || format!("{point:?}"));
    }
    worst.assert_at_most(&mut reference, SPHERE_SAME_POINT_TOLERANCE);
}

#[test]
fn spherical_boost_entries_are_within_their_bound_of_the_reference() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale("spherical boost entries, absolute", 4 * REFERENCE_CASES);
    let mut rng = Rng::new(34);
    let edges = [0.0, 1000.0, -1000.0, core::f64::consts::PI, 1e-300];
    let random: Vec<f64> = (0..REFERENCE_CASES)
        .map(|_| rng.uniform(-1000.0, 1000.0))
        .collect();
    for angle in edges.into_iter().chain(random) {
        let matrix = spherical::boost(angle);
        let ra = num(angle);
        let (cos, sin) = (reference.cos(&ra), reference.sin(&ra));
        let minus_sin = sin.neg();
        for (value, exact) in [
            (matrix.x_axis.x, &cos),
            (matrix.z_axis.z, &cos),
            (matrix.z_axis.x, &sin),
            (matrix.x_axis.z, &minus_sin),
        ] {
            worst.record(&error(value, exact), &int(1), || format!("{angle:e}"));
        }
        assert_eq!(
            [
                matrix.x_axis.y,
                matrix.y_axis.x,
                matrix.y_axis.y,
                matrix.y_axis.z,
                matrix.z_axis.y
            ],
            [0.0, 0.0, 1.0, 0.0, 0.0],
        );
    }
    worst.assert_at_most(&mut reference, SPHERE_BOOST_TOLERANCE);
}

fn sphere_translation_composition(target: &Vector, x: &Vector) -> Vector {
    let planar = sqrt(&add(
        &mul(&target[0], &target[0]),
        &mul(&target[1], &target[1]),
    ));
    let length = sqrt(&dot(target, target));
    let (cos_rho, sin_rho) = (div(&target[2], &length), div(&planar, &length));
    let (cos_phi, sin_phi) = if planar.is_zero() {
        (int(1), int(0))
    } else {
        (div(&target[0], &planar), div(&target[1], &planar))
    };
    let turn = |cos: &BigFloat, sin: &BigFloat, p: &Vector| -> Vector {
        [
            sub(&mul(&p[0], cos), &mul(&p[1], sin)),
            add(&mul(&p[1], cos), &mul(&p[0], sin)),
            p[2].clone(),
        ]
    };
    let back = turn(&cos_phi, &sin_phi.neg(), x);
    let moved = [
        add(&mul(&back[0], &cos_rho), &mul(&back[2], &sin_rho)),
        back[1].clone(),
        sub(&mul(&back[2], &cos_rho), &mul(&back[0], &sin_rho)),
    ];
    turn(&cos_phi, &sin_phi, &moved)
}

#[test]
fn spherical_translation_is_within_its_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut images = Worst::of_the_scale(
        "spherical translation(t) * ORIGIN, each coordinate, scaled by eps",
        3 * REFERENCE_CASES,
    );
    let mut composed = Worst::of_the_scale(
        "spherical translation(t) * x, each coordinate, scaled by eps",
        3 * REFERENCE_CASES,
    );
    let mut defects = Worst::of_the_scale(
        "spherical isometry_defect(translation(t)), scaled by eps",
        REFERENCE_CASES,
    );
    let mut rng = Rng::new(35);
    let tiny = 1e-300;
    let mut targets = vec![
        spherical::ORIGIN,
        -spherical::ORIGIN,
        DVec3::new(tiny, 0.0, 1.0),
        DVec3::new(0.0, -tiny, 1.0),
        DVec3::new(-tiny, tiny, 1.0),
        DVec3::new(tiny, 0.0, -1.0),
        DVec3::new(0.0, tiny, -1.0),
        DVec3::new(-tiny, -tiny, -1.0),
    ];
    let kept = targets.len() + REFERENCE_CASES;
    targets.extend((0..CASES).map(|_| rng.sphere_point()));
    targets.truncate(kept);
    for target in targets {
        let matrix = spherical::translation(target);
        let rt = big(target);
        let input = || format!("{target:?}");
        record_vector(&mut images, matrix * spherical::ORIGIN, &unit(&rt), &input);
        let x = rng.sphere_point();
        let exact = sphere_translation_composition(&rt, &big(x));
        record_vector(&mut composed, matrix * x, &exact, &|| {
            format!("{target:?} {x:?}")
        });
        defects.record(&num(spherical::isometry_defect(matrix)), &epsilon(), input);
    }
    images.assert_at_most(&mut reference, SPHERE_TRANSLATION_TOLERANCE);
    composed.assert_at_most(&mut reference, SPHERE_TRANSLATION_COMPOSITION_TOLERANCE);
    defects.assert_at_most(&mut reference, SPHERE_TRANSLATION_DEFECT_TOLERANCE);
}

#[test]
fn spherical_reflection_is_within_its_bounds_of_the_reference() {
    let mut reference = Reference::new();
    let mut points = Worst::of_the_scale(
        "spherical reflection(u) * x, scaled by eps * sigma_i",
        3 * CASES,
    );
    let mut defects = Worst::of_the_scale(
        "spherical isometry_defect(reflection(u)), scaled by eps",
        CASES,
    );
    let mut rng = Rng::new(36);
    for index in 0..CASES {
        let normal = rng.sphere_point();
        let x = if index % 2 == 0 {
            rng.sphere_point()
        } else {
            let (first, second) = tangent_basis(normal);
            let along = rng.angle();
            let foot = first * along.cos() + second * along.sin();
            let offset = rng.sign() * rng.log_uniform(1e-12, 1.0);
            foot * offset.cos() + normal * offset.sin()
        };
        let (ru, rx) = (big(normal), big(x));
        let along_normal = dot(&ru, &rx);
        let twice = mul(&int(2), &along_normal);
        let sum = terms_bound(&ru, &rx);
        let computed = spherical::reflection(normal) * x;
        for (axis, value) in computed.to_array().into_iter().enumerate() {
            let exact = sub(&rx[axis], &mul(&twice, &ru[axis]));
            let sigma = add(&rx[axis].abs(), &mul(&mul(&int(2), &ru[axis].abs()), &sum));
            points.record(&error(value, &exact), &mul(&epsilon(), &sigma), || {
                format!("{normal:?} {x:?}")
            });
        }
        let defect = spherical::isometry_defect(spherical::reflection(normal));
        defects.record(&num(defect), &epsilon(), || format!("{normal:?}"));
    }
    points.assert_at_most(&mut reference, SPHERE_REFLECTION_TOLERANCE);
    defects.assert_at_most(&mut reference, SPHERE_REFLECTION_DEFECT_TOLERANCE);
}

#[test]
fn spherical_isometry_defect_of_products_is_within_its_bound() {
    let mut reference = Reference::new();
    let mut worst = Worst::of_the_scale(
        "spherical isometry_defect of a product, scaled by eps",
        CASES,
    );
    let mut rng = Rng::new(37);
    for _ in 0..CASES {
        let count = 1 + rng.choice(3);
        let factors: Vec<SphereFactor> = (0..count).map(|_| rng.sphere_factor()).collect();
        let mut matrix = glam::DMat3::IDENTITY;
        for factor in &factors {
            let Some(next) = factor.matrix() else {
                panic!("no reflection for {factor:?}");
            };
            matrix *= next;
        }
        worst.record(&num(spherical::isometry_defect(matrix)), &epsilon(), || {
            format!("{factors:?}")
        });
    }
    worst.assert_at_most(&mut reference, SPHERE_PRODUCT_DEFECT_TOLERANCE);
}
