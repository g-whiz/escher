use core::f64::consts::{FRAC_PI_2, PI};
use core::hint::black_box;

use escher_geometry::hyperbolic::{
    ORIGIN, boost, covector, distance, form, from_disk, inverse, isometry_defect, normalize_point,
    reflection, rotation, side, to_disk, translation, wall_through,
};
use escher_geometry::{ErrorKind, euclidean, spherical};
use glam::{DMat3, DVec2, DVec3};
use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};

use crate::common::{
    CASES, EUCLIDEAN_RADIUS, Factor, Product, Rng, SPHERE_SEPARATION_FLOOR, Wall, euclidean_point,
    largest_entry, line_covector, pair, polar_point, proptest_config, sphere_point, sphere_step,
    tangent_basis,
};
use crate::reference::{Reference, Worst, error, num};

const INVERSE_TOLERANCE: f64 = 1e-11;
const LONG_BOOST_DEFECT_TOLERANCE: f64 = 512.0;
const PRESERVED_TOLERANCE: f64 = 16_384.0;
const TWICE_REFLECTED_TOLERANCE: f64 = 128.0;
const FIXED_WALL_POINT_TOLERANCE: f64 = 16.0;
const BOOST_DISTANCE_TOLERANCE: f64 = 16.0;
const DISK_DISTANCE_TOLERANCE: f64 = 512.0;
const EUCLIDEAN_TWICE_REFLECTED_TOLERANCE: f64 = 32.0;
const EUCLIDEAN_FIXED_POINT_TOLERANCE: f64 = 16.0;
const EUCLIDEAN_RELATOR_TOLERANCE: f64 = 256.0;
const EUCLIDEAN_REFLECTION_DEFECT_TOLERANCE: f64 = 32.0;
const EUCLIDEAN_ORTHOGONALITY_TOLERANCE: f64 = 8.0;
const EUCLIDEAN_NORM_TOLERANCE: f64 = 32.0;
const LARGEST_CORNER_ORDER: u32 = 6;
const SPHERE_TWICE_REFLECTED_TOLERANCE: f64 = 64.0;
const SPHERE_FIXED_POINT_TOLERANCE: f64 = 64.0;
const SPHERE_RELATOR_TOLERANCE: f64 = 1024.0;
const SPHERE_REFLECTION_DEFECT_TOLERANCE: f64 = 64.0;
const SPHERE_ORTHOGONALITY_TOLERANCE: f64 = 8.0;
const SPHERE_NORM_TOLERANCE: f64 = 32.0;
const SPHERE_LARGEST_CORNER_ORDER: u32 = 12;

fn point_within(radius: f64) -> impl Strategy<Value = DVec3> {
    (0.0..radius, -PI..PI).prop_map(|(distance, angle)| polar_point(distance, angle))
}

fn factor() -> impl Strategy<Value = Factor> {
    prop_oneof![
        (-2.0..2.0).prop_map(Factor::Boost),
        (-PI..PI).prop_map(Factor::Rotation),
        (point_within(3.0), point_within(3.0))
            .prop_map(|(first, second)| Factor::Reflection(first, second)),
    ]
}

fn product() -> impl Strategy<Value = Product> {
    vec(factor(), 1..=3).prop_filter_map("the reflections span walls", |factors| {
        Product::of(&factors)
    })
}

fn wall() -> impl Strategy<Value = Wall> {
    (point_within(3.0), point_within(3.0))
        .prop_filter_map("the points span a wall", |(first, second)| {
            Wall::through(first, second)
        })
}

fn separated_points() -> impl Strategy<Value = (DVec3, DVec3)> {
    (point_within(3.0), point_within(3.0))
        .prop_filter("the points are at least 0.1 apart", |&(first, second)| {
            distance(first, second) >= 0.1
        })
}

fn any_f64() -> impl Strategy<Value = f64> {
    prop_oneof![
        any::<u64>().prop_map(f64::from_bits),
        any::<f64>(),
        Just(f64::NAN),
        Just(f64::INFINITY),
        Just(f64::NEG_INFINITY),
        Just(0.0),
        Just(-0.0),
        Just(f64::MIN_POSITIVE / 2.0),
        -2.0..2.0,
    ]
}

fn any_vector() -> impl Strategy<Value = DVec3> {
    (any_f64(), any_f64(), any_f64()).prop_map(|(x, y, z)| DVec3::new(x, y, z))
}

fn any_matrix() -> impl Strategy<Value = DMat3> {
    (any_vector(), any_vector(), any_vector())
        .prop_map(|(first, second, third)| DMat3::from_cols(first, second, third))
}

fn euclidean_point_within(radius: f64) -> impl Strategy<Value = DVec3> {
    (0.0..radius, -PI..PI).prop_map(|(distance, angle)| euclidean_point(distance, angle))
}

fn euclidean_line() -> impl Strategy<Value = DVec3> {
    (-PI..PI, -EUCLIDEAN_RADIUS..EUCLIDEAN_RADIUS)
        .prop_map(|(angle, offset)| line_covector(angle, offset))
}

fn separated_euclidean_points() -> impl Strategy<Value = (DVec3, DVec3)> {
    (
        euclidean_point_within(EUCLIDEAN_RADIUS),
        euclidean_point_within(EUCLIDEAN_RADIUS),
    )
        .prop_filter("the points are at least 0.1 apart", |&(first, second)| {
            euclidean::distance(first, second) >= 0.1
        })
}

fn euclidean_line_point(line: DVec3, along: f64) -> DVec3 {
    DVec3::new(
        -line.z * line.x - along * line.y,
        -line.z * line.y + along * line.x,
        1.0,
    )
}

fn euclidean_fixed_point_scale(line: DVec3, point: DVec3) -> DVec3 {
    let terms = (line * point).abs().element_sum();
    let u = DVec3::new(line.x, line.y, 0.0);
    f64::EPSILON * (point.abs() + 2.0 * u.abs() * terms)
}

fn euclidean_twice_reflected_error(line: DVec3) -> f64 {
    let matrix = euclidean::reflection(line);
    let mu = largest_entry(matrix);
    identity_error(matrix * matrix) / (f64::EPSILON * mu * mu)
}

fn euclidean_fixed_point_error(line: DVec3, along: f64) -> f64 {
    let point = euclidean_line_point(line, along);
    let error = (euclidean::reflection(line) * point - point).abs();
    (error / euclidean_fixed_point_scale(line, point))
        .to_array()
        .into_iter()
        .filter(|ratio| !ratio.is_nan())
        .fold(0.0, f64::max)
}

fn euclidean_relator_error(centre: DVec3, angle: f64, order: u32) -> f64 {
    let lines = [angle, angle + PI / f64::from(order)].map(|direction| {
        let normal = DVec3::new(direction.cos(), direction.sin(), 0.0);
        line_covector(direction, normal.dot(centre - DVec3::Z))
    });
    let rotation = euclidean::reflection(lines[0]) * euclidean::reflection(lines[1]);
    let mut power = rotation;
    for _ in 1..order {
        power *= rotation;
    }
    identity_error(power) / (f64::EPSILON * (1.0 + centre.x.hypot(centre.y)))
}

fn euclidean_orthogonality_scale(first: DVec3, second: DVec3) -> f64 {
    let (p, q) = (first.x.hypot(first.y), second.x.hypot(second.y));
    f64::EPSILON * (1.0 + p + q + p * q / euclidean::distance(first, second))
}

fn any_sphere_point() -> impl Strategy<Value = DVec3> {
    (-1.0..1.0, -PI..PI).prop_map(|(height, angle)| sphere_point(height, angle))
}

fn separated_sphere_points() -> impl Strategy<Value = (DVec3, DVec3)> {
    (
        any_sphere_point(),
        -PI..PI,
        SPHERE_SEPARATION_FLOOR.ln()..FRAC_PI_2.ln(),
        any::<bool>(),
    )
        .prop_map(|(first, direction, exponent, towards_antipode)| {
            let near = exponent.exp();
            let separation = if towards_antipode { PI - near } else { near };
            (first, sphere_step(first, direction, separation))
        })
}

fn great_circle_point(normal: DVec3, along: f64) -> DVec3 {
    let (first, second) = tangent_basis(normal);
    first * along.cos() + second * along.sin()
}

fn sphere_twice_reflected_error(normal: DVec3) -> f64 {
    let matrix = spherical::reflection(normal);
    identity_error(matrix * matrix) / f64::EPSILON
}

fn sphere_fixed_point_error(normal: DVec3, along: f64) -> f64 {
    let point = great_circle_point(normal, along);
    (spherical::reflection(normal) * point - point)
        .abs()
        .max_element()
        / f64::EPSILON
}

fn sphere_relator_error(centre: DVec3, angle: f64, order: u32) -> f64 {
    let normals = [angle, angle + PI / f64::from(order)]
        .map(|direction| great_circle_point(centre, direction));
    let rotation = spherical::reflection(normals[0]) * spherical::reflection(normals[1]);
    let mut power = rotation;
    for _ in 1..order {
        power *= rotation;
    }
    identity_error(power) / f64::EPSILON
}

fn sphere_orthogonality_error(first: DVec3, second: DVec3, normal: DVec3) -> f64 {
    let scale = f64::EPSILON / spherical::distance(first, second).sin();
    spherical::side(normal, first)
        .abs()
        .max(spherical::side(normal, second).abs())
        / scale
}

fn identity_error(matrix: DMat3) -> f64 {
    (matrix - DMat3::IDENTITY)
        .abs()
        .to_cols_array()
        .into_iter()
        .fold(0.0, f64::max)
}

fn fixed_point_scale(wall: &Wall, point: DVec3) -> DVec3 {
    let normal = wall.normal;
    let terms = (covector(normal) * point).abs().element_sum();
    let (first, second) = (wall.first, wall.second);
    let k = first.z * second.z / distance(first, second).sinh();
    let off_wall = k * first.z.max(second.z);
    f64::EPSILON * (point.abs() + 2.0 * normal.abs() * (terms + off_wall))
}

fn disk_distance(first: DVec3, second: DVec3) -> f64 {
    let (u, v) = (to_disk(first), to_disk(second));
    let ratio =
        2.0 * (u - v).length_squared() / ((1.0 - u.length_squared()) * (1.0 - v.length_squared()));
    (1.0 + ratio).acosh()
}

fn disk_distance_scale(first: DVec3, second: DVec3, d: f64) -> f64 {
    f64::EPSILON * first.z * second.z / d.sinh()
}

fn boost_distance_scale(length: f64) -> f64 {
    let size = length.abs();
    let conditioned = f64::EPSILON * (length.cosh() / size.sinh() + size);
    let square_root = (f64::EPSILON * length.cosh()).sqrt();
    conditioned.min(square_root)
}

fn preserved_form_scale(product: &Product, x: DVec3, y: DVec3) -> f64 {
    let mu = largest_entry(product.matrix);
    f64::EPSILON * mu * mu * x.z * y.z
}

fn assert_finite_normal_or_degenerate_wall(first: DVec3, second: DVec3) -> bool {
    match wall_through(first, second) {
        Ok(normal) => {
            assert!(normal.is_finite(), "{normal:?} for {first:?} {second:?}");
            false
        }
        Err(error) => {
            assert_eq!(error.kind(), ErrorKind::DegenerateWall);
            true
        }
    }
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn wall_through_of_points_closer_than_1e_minus_5_is_a_finite_normal_or_degenerate_wall(
        radius in 0.0..10.0,
        angle in -PI..PI,
        exponent in -9.0..-5.0,
        direction in -PI..PI,
    ) {
        let (first, second) = pair(radius, angle, 10.0_f64.powf(exponent), direction);
        assert_finite_normal_or_degenerate_wall(first, second);
    }

    #[test]
    fn inverse_of_a_product_times_the_product_is_the_identity(product in product()) {
        let mu = largest_entry(product.matrix);
        let tolerance = INVERSE_TOLERANCE * mu * mu;
        let error = identity_error(inverse(product.matrix) * product.matrix);
        prop_assert!(error <= tolerance, "{error} > {tolerance} for {product:?}");
        let defect = isometry_defect(product.matrix);
        prop_assert!(defect <= tolerance, "{defect} > {tolerance} for {product:?}");
    }

    #[test]
    fn from_disk_then_to_disk_returns_the_position(
        radius in 0.0..(1.0 - 1e-9),
        angle in -PI..PI,
    ) {
        let position = DVec2::new(angle.cos(), angle.sin()) * radius;
        let back = from_disk(position).map(to_disk);
        prop_assert!(back.is_some(), "{position:?}");
        if let Some(back) = back {
            let error = (back - position).abs().max_element();
            prop_assert!(error <= 8.0 * f64::EPSILON, "{error} for {position:?}");
        }
    }

    #[test]
    fn to_disk_then_from_disk_returns_the_point(point in point_within(12.0)) {
        let back = from_disk(to_disk(point));
        prop_assert!(back.is_some(), "{point:?}");
        if let Some(back) = back {
            let error = (back - point).abs().max_element();
            let tolerance = 32.0 * f64::EPSILON * point.z * point.z;
            prop_assert!(error <= tolerance, "{error} > {tolerance} for {point:?}");
        }
    }

    #[test]
    fn wall_through_is_orthogonal_unit_and_negated_by_a_swap(points in separated_points()) {
        let (first, second) = points;
        let normal = wall_through(first, second);
        let swapped = wall_through(second, first);
        prop_assert!(normal.is_ok() && swapped.is_ok(), "{first:?} {second:?}");
        if let (Ok(normal), Ok(swapped)) = (normal, swapped) {
            let k = first.z * second.z / distance(first, second).sinh();
            let orthogonality = 32.0 * f64::EPSILON * k * first.z.max(second.z);
            prop_assert!(form(normal, first).abs() <= orthogonality);
            prop_assert!(form(normal, second).abs() <= orthogonality);
            prop_assert!((form(normal, normal) - 1.0).abs() <= 64.0 * f64::EPSILON * k);
            prop_assert_eq!(swapped, -normal);
        }
    }

    #[test]
    fn reflection_is_an_involution(wall in wall()) {
        let mu = largest_entry(wall.matrix);
        let error = identity_error(wall.matrix * wall.matrix);
        let tolerance = TWICE_REFLECTED_TOLERANCE * f64::EPSILON * mu * mu;
        prop_assert!(error <= tolerance, "{error} > {tolerance} for {wall:?}");
    }

    #[test]
    fn reflection_fixes_the_points_of_its_wall(wall in wall()) {
        for point in wall.points() {
            let error = (wall.matrix * point - point).abs();
            let tolerance = FIXED_WALL_POINT_TOLERANCE * fixed_point_scale(&wall, point);
            prop_assert!(error.cmple(tolerance).all(), "{error} > {tolerance} for {point:?}");
        }
    }

    #[test]
    fn isometry_preserves_form_and_distance(
        product in product(),
        points in separated_points(),
    ) {
        let (x, y) = points;
        let (mx, my) = (product.matrix * x, product.matrix * y);
        let scale = preserved_form_scale(&product, x, y);
        let form_error = (form(mx, my) - form(x, y)).abs();
        prop_assert!(form_error <= PRESERVED_TOLERANCE * scale, "{form_error} for {product:?}");
        let d = distance(x, y);
        let distance_error = (distance(mx, my) - d).abs();
        prop_assert!(
            distance_error <= PRESERVED_TOLERANCE * scale / d.sinh(),
            "{distance_error} for {product:?}"
        );
    }

    #[test]
    fn distance_from_the_origin_of_a_boosted_origin_is_the_length(length in -20.0..20.0) {
        let error = (distance(ORIGIN, boost(length) * ORIGIN) - length.abs()).abs();
        let tolerance = BOOST_DISTANCE_TOLERANCE * boost_distance_scale(length);
        prop_assert!(error <= tolerance, "{error} > {tolerance} for {length}");
    }

    #[test]
    fn distance_agrees_with_the_disk_formula(
        first in point_within(3.0),
        second in point_within(3.0),
    ) {
        let expected = disk_distance(first, second);
        let error = (distance(first, second) - expected).abs();
        let tolerance = DISK_DISTANCE_TOLERANCE * disk_distance_scale(first, second, expected);
        prop_assert!(error <= tolerance, "{error} > {tolerance} for {first:?} {second:?}");
    }

    #[test]
    fn euclidean_reflection_is_an_involution_that_fixes_its_line(
        line in euclidean_line(),
        along in -EUCLIDEAN_RADIUS..EUCLIDEAN_RADIUS,
    ) {
        let twice = euclidean_twice_reflected_error(line);
        prop_assert!(twice <= EUCLIDEAN_TWICE_REFLECTED_TOLERANCE, "{twice} for {line:?}");
        let fixed = euclidean_fixed_point_error(line, along);
        prop_assert!(fixed <= EUCLIDEAN_FIXED_POINT_TOLERANCE, "{fixed} for {line:?} {along}");
        let defect = euclidean::isometry_defect(euclidean::reflection(line));
        prop_assert!(
            defect <= EUCLIDEAN_REFLECTION_DEFECT_TOLERANCE * f64::EPSILON,
            "{defect} for {line:?}"
        );
    }

    #[test]
    fn euclidean_reflections_in_lines_at_pi_over_n_close_up_after_n_turns(
        centre in euclidean_point_within(EUCLIDEAN_RADIUS),
        angle in -PI..PI,
        order in 2..=LARGEST_CORNER_ORDER,
    ) {
        let error = euclidean_relator_error(centre, angle, order);
        prop_assert!(
            error <= EUCLIDEAN_RELATOR_TOLERANCE,
            "{error} for {centre:?} {angle} {order}"
        );
    }

    #[test]
    fn euclidean_wall_through_is_orthogonal_unit_and_negated_by_a_swap(
        points in separated_euclidean_points(),
    ) {
        let (first, second) = points;
        let line = euclidean::wall_through(first, second);
        let swapped = euclidean::wall_through(second, first);
        prop_assert!(line.is_ok() && swapped.is_ok(), "{first:?} {second:?}");
        if let (Ok(line), Ok(swapped)) = (line, swapped) {
            let orthogonality =
                EUCLIDEAN_ORTHOGONALITY_TOLERANCE * euclidean_orthogonality_scale(first, second);
            prop_assert!(euclidean::side(line, first).abs() <= orthogonality);
            prop_assert!(euclidean::side(line, second).abs() <= orthogonality);
            let norm = (line.x * line.x + line.y * line.y - 1.0).abs();
            prop_assert!(norm <= EUCLIDEAN_NORM_TOLERANCE * f64::EPSILON);
            prop_assert_eq!(swapped, -line);
        }
    }

    #[test]
    fn euclidean_wall_through_is_finite_or_degenerate_for_any_pair(
        first in any_vector(),
        second in any_vector(),
        scale in prop_oneof![Just(1.0), Just(1e-170), Just(1e145), Just(1e160), Just(1e200)],
    ) {
        let near = DVec3::new(first.x, first.x, 1.0);
        let pairs = [
            (first, second),
            (first * scale, second * scale),
            (DVec3::new(first.x, first.y, 1.0) * scale, DVec3::new(second.x, second.y, 1.0)),
            (near, near + DVec3::new(0.0, second.x, 0.0)),
        ];
        for (one, other) in pairs {
            match euclidean::wall_through(one, other) {
                Ok(line) => prop_assert!(line.is_finite(), "{line:?} for {one:?} {other:?}"),
                Err(error) => prop_assert_eq!(error.kind(), ErrorKind::DegenerateWall),
            }
        }
    }

    #[test]
    fn spherical_reflection_is_an_involution_that_fixes_its_great_circle(
        normal in any_sphere_point(),
        along in -PI..PI,
    ) {
        let twice = sphere_twice_reflected_error(normal);
        prop_assert!(twice <= SPHERE_TWICE_REFLECTED_TOLERANCE, "{twice} for {normal:?}");
        let fixed = sphere_fixed_point_error(normal, along);
        prop_assert!(fixed <= SPHERE_FIXED_POINT_TOLERANCE, "{fixed} for {normal:?} {along}");
        let defect = spherical::isometry_defect(spherical::reflection(normal));
        prop_assert!(
            defect <= SPHERE_REFLECTION_DEFECT_TOLERANCE * f64::EPSILON,
            "{defect} for {normal:?}"
        );
    }

    #[test]
    fn spherical_reflections_at_pi_over_n_close_up_after_n_turns(
        centre in any_sphere_point(),
        angle in -PI..PI,
        order in 2..=SPHERE_LARGEST_CORNER_ORDER,
    ) {
        let error = sphere_relator_error(centre, angle, order);
        prop_assert!(
            error <= SPHERE_RELATOR_TOLERANCE,
            "{error} for {centre:?} {angle} {order}"
        );
    }

    #[test]
    fn spherical_wall_through_is_orthogonal_unit_and_negated_by_a_swap(
        points in separated_sphere_points(),
    ) {
        let (first, second) = points;
        let normal = spherical::wall_through(first, second);
        let swapped = spherical::wall_through(second, first);
        prop_assert!(normal.is_ok() && swapped.is_ok(), "{first:?} {second:?}");
        if let (Ok(normal), Ok(swapped)) = (normal, swapped) {
            let orthogonality = sphere_orthogonality_error(first, second, normal);
            prop_assert!(orthogonality <= SPHERE_ORTHOGONALITY_TOLERANCE, "{orthogonality}");
            let norm = (normal.dot(normal) - 1.0).abs();
            prop_assert!(norm <= SPHERE_NORM_TOLERANCE * f64::EPSILON, "{norm}");
            prop_assert_eq!(swapped, -normal);
        }
    }

    #[test]
    fn every_function_returns_for_any_input(
        x in any_vector(),
        y in any_vector(),
        scalar in any_f64(),
        matrix in any_matrix(),
    ) {
        let position = DVec2::new(x.x, x.y);
        black_box(form(x, y));
        black_box(side(x, y));
        black_box(boost(scalar));
        black_box(rotation(scalar));
        black_box(translation(x));
        black_box(reflection(x));
        black_box(to_disk(x));
        let flipped = covector(x);
        prop_assert_eq!(flipped.z.to_bits(), (-x.z).to_bits());
        let d = distance(x, y);
        prop_assert!(d.is_nan() || d >= 0.0, "{d}");
        let defect = isometry_defect(matrix);
        prop_assert!(defect >= 0.0, "{defect}");
        let twice = inverse(inverse(matrix)).to_cols_array().map(f64::to_bits);
        prop_assert_eq!(twice, matrix.to_cols_array().map(f64::to_bits));
        if let Ok(point) = normalize_point(x) {
            prop_assert!(point.is_finite() && point.z > 0.0, "{point:?}");
        }
        if let Ok(normal) = wall_through(x, y) {
            prop_assert!(normal.is_finite(), "{normal:?}");
        }
        if let Some(point) = from_disk(position) {
            prop_assert!(point.is_finite() && point.z >= 1.0, "{point:?}");
        }
        black_box(euclidean::side(x, y));
        black_box(euclidean::rotation(scalar));
        black_box(euclidean::translation(x));
        let plane_distance = euclidean::distance(x, y);
        prop_assert!(plane_distance.is_nan() || plane_distance >= 0.0, "{plane_distance}");
        let plane_defect = euclidean::isometry_defect(matrix);
        prop_assert!(plane_defect >= 0.0, "{plane_defect}");
        let mirror = euclidean::reflection(x);
        if x.is_finite() {
            let last_row = [mirror.x_axis.z, mirror.y_axis.z, mirror.z_axis.z];
            prop_assert_eq!(last_row.map(f64::to_bits), [0.0, 0.0, 1.0].map(f64::to_bits));
        }
        if let Ok(line) = euclidean::wall_through(x, y) {
            prop_assert!(line.is_finite(), "{line:?}");
        }
        black_box(spherical::side(x, y));
        black_box(spherical::boost(scalar));
        black_box(spherical::rotation(scalar));
        black_box(spherical::translation(x));
        black_box(spherical::reflection(x));
        let arc = spherical::distance(x, y);
        prop_assert!(arc.is_nan() || (0.0..=PI).contains(&arc), "{arc}");
        let sphere_defect = spherical::isometry_defect(matrix);
        prop_assert!(sphere_defect >= 0.0, "{sphere_defect}");
        let transposed_twice = spherical::inverse(spherical::inverse(matrix));
        prop_assert_eq!(
            transposed_twice.to_cols_array().map(f64::to_bits),
            matrix.to_cols_array().map(f64::to_bits)
        );
        if let Ok(point) = spherical::normalize_point(x) {
            prop_assert!(point.is_finite(), "{point:?}");
        }
        match spherical::wall_through(x, y) {
            Ok(normal) => prop_assert!(normal.is_finite(), "{normal:?}"),
            Err(error) => prop_assert_eq!(error.kind(), ErrorKind::DegenerateWall),
        }
    }
}

fn boost_product(rng: &mut Rng) -> Product {
    let with_rotations = rng.choice(2) == 0;
    let mut factors = Vec::new();
    for index in 0..3 {
        if with_rotations && index > 0 {
            factors.push(Factor::Rotation(rng.angle()));
        }
        factors.push(Factor::Boost(rng.uniform(-2.0, 2.0)));
    }
    Product::of(&factors).unwrap_or_else(|| panic!("boosts and rotations form a product"))
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn inverse_and_isometry_defect_of_products_are_within_their_bounds() {
    let mut reference = Reference::new();
    let mut inverses = Worst::new(
        "inverse(m) * m against the identity, scaled by mu^2",
        9 * CASES,
    );
    let mut defects = Worst::new("isometry_defect of products, scaled by mu^2", CASES);
    let mut long = Worst::new(
        "isometry_defect with boosts up to 5, scaled by eps * g^2",
        CASES,
    );
    let mut rng = Rng::new(20);
    for _ in 0..CASES {
        let product = rng.product(2.0);
        let mu = largest_entry(product.matrix);
        let scale = num(mu * mu);
        let input = || format!("{:?}", product.factors);
        let identity = inverse(product.matrix) * product.matrix;
        for (value, exact) in identity
            .to_cols_array()
            .into_iter()
            .zip(DMat3::IDENTITY.to_cols_array())
        {
            inverses.record(&error(value, &num(exact)), &scale, input);
        }
        defects.record(&num(isometry_defect(product.matrix)), &scale, input);
        let product = rng.product(5.0);
        let g = product.factor_bound;
        let scale = num(f64::EPSILON * g * g);
        long.record(&num(isometry_defect(product.matrix)), &scale, || {
            format!("{:?}", product.factors)
        });
    }
    inverses.assert_at_most(&mut reference, INVERSE_TOLERANCE);
    defects.assert_at_most(&mut reference, INVERSE_TOLERANCE);
    long.assert_at_most(&mut reference, LONG_BOOST_DEFECT_TOLERANCE);
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn reflections_are_involutions_that_fix_their_walls_within_their_bounds() {
    let mut reference = Reference::new();
    let mut twice = Worst::new(
        "reflection applied twice against the identity, scaled by eps * mu^2",
        9 * CASES,
    );
    let mut fixed = Worst::new(
        "reflection of a point of its wall, scaled by eps * (|x_i| + 2 |u_i| (sum |c_j x_j| + K max(a3, b3)))",
        6 * CASES,
    );
    let mut rng = Rng::new(21);
    for _ in 0..CASES {
        let wall = rng.wall();
        let mu = largest_entry(wall.matrix);
        let input = || format!("{wall:?}");
        let square = wall.matrix * wall.matrix;
        let scale = num(f64::EPSILON * mu * mu);
        for (value, exact) in square
            .to_cols_array()
            .into_iter()
            .zip(DMat3::IDENTITY.to_cols_array())
        {
            twice.record(&error(value, &num(exact)), &scale, input);
        }
        for point in wall.points() {
            let image = wall.matrix * point;
            let scale = fixed_point_scale(&wall, point);
            for index in 0..3 {
                fixed.record(
                    &error(image[index], &num(point[index])),
                    &num(scale[index]),
                    input,
                );
            }
        }
    }
    twice.assert_at_most(&mut reference, TWICE_REFLECTED_TOLERANCE);
    fixed.assert_at_most(&mut reference, FIXED_WALL_POINT_TOLERANCE);
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn isometries_preserve_form_and_distance_within_their_bounds() {
    let mut reference = Reference::new();
    let mut forms = Worst::new("preserved form, scaled by eps * mu^2 * x3 * y3", 2 * CASES);
    let mut distances = Worst::new(
        "preserved distance, scaled by eps * mu^2 * x3 * y3 / sinh d",
        2 * CASES,
    );
    let mut rng = Rng::new(22);
    for index in 0..2 * CASES {
        let product = if index % 2 == 0 {
            rng.product(2.0)
        } else {
            boost_product(&mut rng)
        };
        let (x, y) = rng.separated_pair();
        let (mx, my) = (product.matrix * x, product.matrix * y);
        let scale = preserved_form_scale(&product, x, y);
        let input = || format!("{:?} {x:?} {y:?}", product.factors);
        forms.record(&error(form(mx, my), &num(form(x, y))), &num(scale), input);
        let d = distance(x, y);
        distances.record(
            &error(distance(mx, my), &num(d)),
            &num(scale / d.sinh()),
            input,
        );
    }
    forms.assert_at_most(&mut reference, PRESERVED_TOLERANCE);
    distances.assert_at_most(&mut reference, PRESERVED_TOLERANCE);
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn distances_of_boosts_and_disk_positions_are_within_their_bounds() {
    let mut reference = Reference::new();
    let mut boosts = Worst::new("distance(ORIGIN, boost(l) * ORIGIN) against |l|", CASES);
    let mut conditioned = Worst::new(
        "distance against the disk formula, scaled by eps * x3 y3 / sinh d",
        CASES,
    );
    let mut rng = Rng::new(23);
    let edges = [0.0, 1e-300, 1e-9, 20.0, -20.0];
    let lengths: Vec<f64> = (0..CASES)
        .map(|index| {
            if index % 2 == 0 {
                rng.uniform(-20.0, 20.0)
            } else {
                rng.sign() * rng.log_uniform(1e-12, 20.0)
            }
        })
        .collect();
    for length in edges.into_iter().chain(lengths) {
        let computed = distance(ORIGIN, boost(length) * ORIGIN);
        boosts.record(
            &error(computed, &num(length.abs())),
            &num(boost_distance_scale(length)),
            || format!("{length:e}"),
        );
    }
    for _ in 0..CASES {
        let (first, second) = (rng.point_within(3.0), rng.point_within(3.0));
        let expected = disk_distance(first, second);
        let err = error(distance(first, second), &num(expected));
        let scale = disk_distance_scale(first, second, expected);
        conditioned.record(&err, &num(scale), || format!("{first:?} {second:?}"));
    }
    boosts.assert_at_most(&mut reference, BOOST_DISTANCE_TOLERANCE);
    conditioned.assert_at_most(&mut reference, DISK_DISTANCE_TOLERANCE);
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn wall_through_swapped_points_is_the_exact_negation() {
    let mut rng = Rng::new(24);
    for _ in 0..CASES {
        let (first, second) = rng.separated_pair();
        let forward = wall_through(first, second);
        let backward = wall_through(second, first);
        assert_eq!(
            backward,
            forward.map(|normal| -normal),
            "{first:?} {second:?}"
        );
    }
}

#[test]
fn normalize_point_beyond_distance_15_rejects_only_vectors_whose_computed_form_is_not_negative() {
    let mut rng = Rng::new(25);
    let mut rejected = 0;
    for _ in 0..256 {
        let distance = rng.uniform(15.0, 20.0);
        let vector = polar_point(distance, rng.angle()) * rng.uniform(0.5, 2.0);
        if let Ok(point) = normalize_point(vector) {
            assert!(point.is_finite() && point.z > 0.0, "{vector:?}");
        } else {
            let q = -form(vector, vector);
            let accepted = q > 0.0 && q.is_finite() && vector.z > 0.0;
            assert!(!accepted, "normalize_point rejects {vector:?}");
            rejected += 1;
        }
    }
    println!("normalize_point beyond distance 15: {rejected} of 256 rejected");
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn wall_through_sweep_of_points_closer_than_1e_minus_5_gives_finite_normals_or_degenerate_walls() {
    let mut rng = Rng::new(26);
    let mut degenerate = 0;
    for _ in 0..CASES {
        let radius = rng.uniform(0.0, 10.0);
        let angle = rng.angle();
        let separation = rng.log_uniform(1e-9, 1e-5);
        let (first, second) = pair(radius, angle, separation, rng.angle());
        if assert_finite_normal_or_degenerate_wall(first, second) {
            degenerate += 1;
        }
    }
    println!(
        "wall_through below a separation of 1e-5: {degenerate} of {CASES} were DegenerateWall"
    );
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn euclidean_reflections_and_relators_are_within_their_bounds() {
    let mut largest: [(f64, String); 4] = Default::default();
    let mut record = |slot: usize, value: f64, input: &dyn Fn() -> String| {
        assert!(!value.is_nan(), "NaN for {}", input());
        if value > largest[slot].0 {
            largest[slot] = (value, input());
        }
    };
    let mut rng = Rng::new(23);
    for index in 0..CASES {
        let line = rng.line_covector(u32::try_from(index).unwrap_or(0));
        let along = rng.uniform(-EUCLIDEAN_RADIUS, EUCLIDEAN_RADIUS);
        record(0, euclidean_twice_reflected_error(line), &|| {
            format!("{line:?}")
        });
        record(1, euclidean_fixed_point_error(line, along), &|| {
            format!("{line:?} {along}")
        });
        let centre = rng.euclidean_point_within(EUCLIDEAN_RADIUS);
        let angle = rng.angle();
        let order = 2 + u32::try_from(rng.choice(5)).unwrap_or(0);
        record(2, euclidean_relator_error(centre, angle, order), &|| {
            format!("{centre:?} {angle} {order}")
        });
        let (first, second) = rng.euclidean_pair();
        if let Ok(line) = euclidean::wall_through(first, second) {
            let scale = euclidean_orthogonality_scale(first, second);
            let value = euclidean::side(line, first)
                .abs()
                .max(euclidean::side(line, second).abs());
            record(3, value / scale, &|| format!("{first:?} {second:?}"));
        }
    }
    let names = [
        "euclidean reflection applied twice, scaled by eps * mu^2",
        "euclidean reflection of a point of its line, scaled by eps * sigma_i",
        "euclidean relator of two lines at pi / n, scaled by eps * (1 + |X|)",
        "euclidean wall_through orthogonality in f64, scaled by eps * (1 + |p| + |q| + |p| |q| / |p - q|)",
    ];
    let bounds = [
        EUCLIDEAN_TWICE_REFLECTED_TOLERANCE,
        EUCLIDEAN_FIXED_POINT_TOLERANCE,
        EUCLIDEAN_RELATOR_TOLERANCE,
        EUCLIDEAN_ORTHOGONALITY_TOLERANCE,
    ];
    for ((name, bound), (value, input)) in names.into_iter().zip(bounds).zip(&largest) {
        println!(
            "{name}: largest error {value:.3e} of the scale, bound {bound:e}, margin {:.1}, {CASES} samples, at {input}",
            bound / value
        );
        assert!(
            *value <= bound,
            "{name}: {value} exceeds {bound} at {input}"
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn spherical_reflections_relators_and_walls_are_within_their_bounds() {
    let mut largest: [(f64, String); 4] = Default::default();
    let mut record = |slot: usize, value: f64, input: &dyn Fn() -> String| {
        assert!(!value.is_nan(), "NaN for {}", input());
        if value > largest[slot].0 {
            largest[slot] = (value, input());
        }
    };
    let mut rng = Rng::new(38);
    for _ in 0..CASES {
        let normal = rng.sphere_point();
        let along = rng.angle();
        record(0, sphere_twice_reflected_error(normal), &|| {
            format!("{normal:?}")
        });
        record(1, sphere_fixed_point_error(normal, along), &|| {
            format!("{normal:?} {along}")
        });
        let centre = rng.sphere_point();
        let angle = rng.angle();
        let order = 2 + u32::try_from(rng.choice(11)).unwrap_or(0);
        record(2, sphere_relator_error(centre, angle, order), &|| {
            format!("{centre:?} {angle} {order}")
        });
        let (first, second) = rng.sphere_pair(SPHERE_SEPARATION_FLOOR);
        let Ok(normal) = spherical::wall_through(first, second) else {
            panic!("no great circle through {first:?} {second:?}");
        };
        let swapped = spherical::wall_through(second, first);
        assert_eq!(swapped, Ok(-normal), "{first:?} {second:?}");
        record(
            3,
            sphere_orthogonality_error(first, second, normal),
            &|| format!("{first:?} {second:?}"),
        );
    }
    let names = [
        "spherical reflection applied twice, scaled by eps",
        "spherical reflection of a point of its great circle, scaled by eps",
        "spherical relator of two great circles at pi / n, scaled by eps",
        "spherical wall_through orthogonality in f64, scaled by eps / sin d",
    ];
    let bounds = [
        SPHERE_TWICE_REFLECTED_TOLERANCE,
        SPHERE_FIXED_POINT_TOLERANCE,
        SPHERE_RELATOR_TOLERANCE,
        SPHERE_ORTHOGONALITY_TOLERANCE,
    ];
    for ((name, bound), (value, input)) in names.into_iter().zip(bounds).zip(&largest) {
        println!(
            "{name}: largest error {value:.3e} of the scale, bound {bound:e}, margin {:.1}, {CASES} samples, at {input}",
            bound / value
        );
        assert!(
            *value <= bound,
            "{name}: {value} exceeds {bound} at {input}"
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "the sweep runs a loop too long for the interpreter")]
fn spherical_wall_through_below_the_separation_floor_gives_finite_normals_or_degenerate_walls() {
    let mut rng = Rng::new(39);
    let mut degenerate = 0;
    for _ in 0..CASES {
        let first = rng.sphere_point();
        let near = rng.log_uniform(1e-12, SPHERE_SEPARATION_FLOOR);
        let separation = if rng.choice(2) == 0 { near } else { PI - near };
        let second = sphere_step(first, rng.angle(), separation);
        match spherical::wall_through(first, second) {
            Ok(normal) => assert!(normal.is_finite(), "{normal:?} for {first:?} {second:?}"),
            Err(error) => {
                assert_eq!(error.kind(), ErrorKind::DegenerateWall);
                degenerate += 1;
            }
        }
    }
    println!(
        "spherical wall_through below the separation floor: {degenerate} of {CASES} were DegenerateWall"
    );
}
