use escher_geometry::hyperbolic::from_disk;
use escher_geometry::spherical;
use escher_signature::Geometry;
use glam::{DMat3, DVec2, DVec3};

use crate::options::RenderOptions;

const MARGIN: f64 = 1.0 / 32.0;

const PLANE_SPAN: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Model {
    Disk {
        pixel: f64,
        placement: Option<DMat3>,
    },
    Plane {
        pixel: f64,
    },
    Ball {
        pixel: f64,
        turn: DMat3,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Lift {
    pub(crate) point: DVec3,
    pub(crate) tangents: [DVec3; 2],
    pub(crate) normal: Option<DVec3>,
}

#[derive(Clone, Debug)]
pub(crate) struct View {
    width: f64,
    height: f64,
    grid: f64,
    model: Model,
}

impl View {
    pub(crate) fn canonical(options: &RenderOptions, geometry: Geometry) -> Self {
        let shorter = f64::from(options.width.min(options.height));
        let disk = 2.0 / ((1.0 - 2.0 * MARGIN) * shorter);
        let model = match geometry {
            Geometry::Hyperbolic => Model::Disk {
                pixel: disk,
                placement: None,
            },
            Geometry::Euclidean => Model::Plane {
                pixel: PLANE_SPAN / shorter,
            },
            Geometry::Spherical => Model::Ball {
                pixel: disk,
                turn: viewing_matrix(options.rotation).transpose(),
            },
        };
        Self::with_model(options, model)
    }

    fn with_model(options: &RenderOptions, model: Model) -> Self {
        Self {
            width: f64::from(options.width),
            height: f64::from(options.height),
            grid: f64::from(options.samples),
            model,
        }
    }

    pub(crate) fn position(&self, column: u32, row: u32, across: u32, down: u32) -> DVec2 {
        let pixel = match self.model {
            Model::Disk { pixel, .. } | Model::Plane { pixel } | Model::Ball { pixel, .. } => pixel,
        };
        let first = f64::from(column) + (f64::from(across) + 0.5) / self.grid;
        let second = f64::from(row) + (f64::from(down) + 0.5) / self.grid;
        DVec2::new(
            (first - self.width / 2.0) * pixel,
            (self.height / 2.0 - second) * pixel,
        )
    }

    pub(crate) fn point(&self, position: DVec2) -> Option<Lift> {
        match self.model {
            Model::Disk { pixel, placement } => {
                let point = from_disk(position)?;
                let carried = tangents(position, pixel);
                let (point, tangents) = match placement {
                    None => (point, carried),
                    Some(placement) => (
                        placement * point,
                        carried.map(|tangent| placement * tangent),
                    ),
                };
                Some(Lift {
                    point,
                    tangents,
                    normal: None,
                })
            }
            Model::Plane { pixel } => Some(Lift {
                point: DVec3::new(position.x, position.y, 1.0),
                tangents: [DVec3::new(pixel, 0.0, 0.0), DVec3::new(0.0, pixel, 0.0)],
                normal: None,
            }),
            Model::Ball { pixel, turn } => {
                let (screen, carried) = lift(position, pixel)?;
                Some(Lift {
                    point: turn * screen,
                    tangents: carried.map(|tangent| turn * tangent),
                    normal: Some(screen),
                })
            }
        }
    }
}

pub(crate) fn viewing_matrix([yaw, pitch, roll]: [f64; 3]) -> DMat3 {
    let (sin, cos) = pitch.sin_cos();
    let pitched = DMat3::from_cols(
        DVec3::X,
        DVec3::new(0.0, cos, -sin),
        DVec3::new(0.0, sin, cos),
    );
    spherical::rotation(roll) * pitched * spherical::boost(yaw)
}

pub(crate) fn lift(position: DVec2, pixel: f64) -> Option<(DVec3, [DVec3; 2])> {
    let d = 1.0 - position.length_squared();
    (d > 0.0).then(|| {
        let height = d.sqrt();
        (
            DVec3::new(position.x, position.y, height),
            [
                DVec3::new(1.0, 0.0, -position.x / height) * pixel,
                DVec3::new(0.0, 1.0, -position.y / height) * pixel,
            ],
        )
    })
}

pub(crate) fn tangents(position: DVec2, pixel: f64) -> [DVec3; 2] {
    let (first, second) = (position.x, position.y);
    let d = 1.0 - position.length_squared();
    let squared = d * d;
    let along = 2.0 / d;
    let cross = 4.0 * first * second / squared;
    [
        DVec3::new(
            along + 4.0 * first * first / squared,
            cross,
            4.0 * first / squared,
        ) * pixel,
        DVec3::new(
            cross,
            along + 4.0 * second * second / squared,
            4.0 * second / squared,
        ) * pixel,
    ]
}

#[cfg(test)]
impl View {
    pub(crate) fn pixel(&self) -> f64 {
        match self.model {
            Model::Disk { pixel, .. } | Model::Plane { pixel } | Model::Ball { pixel, .. } => pixel,
        }
    }

    pub(crate) fn rotated(options: &RenderOptions, viewing: DMat3) -> Self {
        let mut view = Self::canonical(options, Geometry::Spherical);
        if let Model::Ball { turn, .. } = &mut view.model {
            *turn = viewing.transpose();
        }
        view
    }

    pub(crate) fn symmetric(
        options: &RenderOptions,
        domain: &escher_domain::FundamentalDomain,
    ) -> Self {
        use escher_geometry::hyperbolic::{inverse, rotation, translation};

        let vertices = domain.vertices();
        let to_vertex = translation(vertices[2]);
        let toward = inverse(to_vertex) * vertices[0];
        let mut view = Self::canonical(options, Geometry::Hyperbolic);
        if let Model::Disk { placement, .. } = &mut view.model {
            *placement = Some(to_vertex * rotation(toward.y.atan2(toward.x)));
        }
        view
    }

    pub(crate) fn periodic(options: &RenderOptions, pixel: f64) -> Self {
        Self::with_model(options, Model::Plane { pixel })
    }
}

#[cfg(test)]
mod tests {
    use escher_domain::FundamentalDomain;
    use escher_signature::{Geometry, Signature};
    use glam::{DVec2, DVec3};

    use glam::DMat3;

    use super::{Model, View, lift, tangents, viewing_matrix};
    use crate::colour::{Colour, Fill};
    use crate::golden_images::{
        ALLOWED_PIXELS, GOLDEN_GRID, GOLDEN_SIZE, TRANSLATION_TOLERANCE, compare,
        domain_with_aspect, linear_table, renderer, shifted_pairs, translation,
    };
    use crate::options::RenderOptions;
    use crate::options::Shading;
    use crate::reference::{Reference, Worst, difference, largest_magnitude, vector};

    const PLAN_TRIPLES: [&str; 6] = ["*732", "*542", "*444", "*433", "*(12)32", "*(1000)32"];

    fn domain(text: &str) -> FundamentalDomain {
        let signature: Signature = match text.parse() {
            Ok(signature) => signature,
            Err(error) => panic!("{text:?} is rejected: {error}"),
        };
        match FundamentalDomain::build(&signature) {
            Ok(domain) => domain,
            Err(error) => panic!("{text} has no domain: {error}"),
        }
    }

    fn options(builder: crate::options::RenderOptionsBuilder) -> RenderOptions {
        match builder.build() {
            Ok(options) => options,
            Err(error) => panic!("the options are rejected: {error}"),
        }
    }

    #[test]
    fn position_of_each_sample_follows_the_grid_and_the_margin() {
        let view = View::canonical(
            &options(
                RenderOptions::builder(Fill::One(Colour::WHITE))
                    .size(64, 32)
                    .samples(2),
            ),
            Geometry::Hyperbolic,
        );
        let pixel = 2.0 / 30.0;
        assert_eq!(view.pixel(), pixel);
        assert_eq!(view.position(0, 0, 0, 0), DVec2::new(-31.75, 15.75) * pixel);
        assert_eq!(
            view.position(63, 31, 1, 1),
            DVec2::new(31.75, -15.75) * pixel
        );
        assert_eq!(view.position(32, 15, 0, 1), DVec2::new(0.25, 0.25) * pixel);
    }

    #[test]
    fn point_is_none_outside_the_disk() {
        let view = View::canonical(
            &options(RenderOptions::builder(Fill::One(Colour::WHITE))),
            Geometry::Hyperbolic,
        );
        assert!(view.point(DVec2::new(1.0, 0.0)).is_none());
        assert!(view.point(DVec2::new(f64::NAN, 0.0)).is_none());
        assert!(view.point(DVec2::ZERO).is_some());
    }

    #[test]
    fn tangents_agree_with_central_differences_of_the_disk_map() {
        const POSITIONS: u32 = 2048;
        const BOUND: f64 = 1.0e-9;
        let mut reference = Reference::new();
        let pixel = 2.0 / (15.0 / 16.0 * 512.0);
        let mut worst = Worst::new(
            "screen tangent, largest coordinate error over the largest coordinate",
            2 * usize::try_from(POSITIONS).unwrap_or(usize::MAX),
        );
        let golden_angle = core::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
        for index in 0..POSITIONS {
            let exponent = -6.0 * f64::from(index) / f64::from(POSITIONS - 1);
            let radius = 1.0 - 10.0_f64.powf(exponent);
            let angle = golden_angle * f64::from(index);
            let position = DVec2::new(radius * angle.cos(), radius * angle.sin());
            let computed = tangents(position, pixel);
            let exact = reference.tangents(position.to_array(), pixel);
            for (computed, exact) in computed.iter().zip(&exact) {
                let error = largest_magnitude(&difference(&vector(computed.to_array()), exact));
                worst.record(&error, &largest_magnitude(exact), || {
                    format!("position {position:?}, 1 - |z| = 1e{exponent:.3}")
                });
            }
        }
        worst.assert_at_most(&mut reference, BOUND);
    }

    #[test]
    fn symmetric_view_differs_from_its_mirror_image_in_the_first_axis_within_the_golden_allowance()
    {
        let linear = linear_table();
        for text in PLAN_TRIPLES {
            let domain = domain(text);
            let options = options(
                RenderOptions::builder(Fill::One(Colour::from_srgb8(70, 130, 180)))
                    .size(GOLDEN_SIZE, GOLDEN_SIZE)
                    .samples(GOLDEN_GRID),
            );
            let view = View::symmetric(&options, &domain);
            let renderer = renderer(&domain, options).with_view(view);
            let (pixels, _) = renderer.render();
            let (top, bottom) = pixels.split_at(pixels.len() / 2);
            let mirrored: Vec<u8> = bottom
                .chunks_exact(3 * usize::try_from(GOLDEN_SIZE).unwrap_or(usize::MAX))
                .rev()
                .flatten()
                .copied()
                .collect();
            let comparison = compare(top, &mirrored, GOLDEN_GRID, &linear);
            println!("{text}: {comparison:?}, allowed {ALLOWED_PIXELS} differing pixels");
            assert!(comparison.within_allowance(), "{text}: {comparison:?}");
        }
    }

    #[test]
    fn plane_view_places_each_sample_at_its_plane_position_with_constant_tangents() {
        let view = View::canonical(
            &options(
                RenderOptions::builder(Fill::One(Colour::WHITE))
                    .size(64, 32)
                    .samples(2),
            ),
            Geometry::Euclidean,
        );
        assert_eq!(view.pixel(), 0.25);
        let cases = [
            ((0, 0, 0, 0), DVec2::new(-31.75, 15.75)),
            ((63, 31, 1, 1), DVec2::new(31.75, -15.75)),
            ((32, 15, 0, 1), DVec2::new(0.25, 0.25)),
        ];
        for ((column, row, across, down), screen) in cases {
            let position = view.position(column, row, across, down);
            assert_eq!(position, screen * 0.25);
            let Some(lift) = view.point(position) else {
                panic!("the plane view has no point at {position:?}");
            };
            assert_eq!(lift.point, DVec3::new(position.x, position.y, 1.0));
            assert_eq!(
                lift.tangents,
                [DVec3::new(0.25, 0.0, 0.0), DVec3::new(0.0, 0.25, 0.0)]
            );
            assert_eq!(lift.normal, None);
        }
        let far = DVec2::new(1.0e6, -3.0);
        assert_eq!(
            view.point(far).map(|lift| lift.point),
            Some(DVec3::new(1.0e6, -3.0, 1.0))
        );
    }

    #[test]
    fn view_of_each_geometry_is_the_disk_the_plane_or_the_ball() {
        let options = options(RenderOptions::builder(Fill::One(Colour::WHITE)).size(512, 256));
        let disk = View::canonical(&options, Geometry::Hyperbolic);
        let pixel = 2.0 / (15.0 / 16.0 * 256.0);
        assert_eq!(
            disk.model,
            Model::Disk {
                pixel,
                placement: None
            }
        );
        let plane = View::canonical(&options, Geometry::Euclidean);
        assert_eq!(plane.model, Model::Plane { pixel: 8.0 / 256.0 });
        let ball = View::canonical(&options, Geometry::Spherical);
        assert_eq!(
            ball.model,
            Model::Ball {
                pixel,
                turn: viewing_matrix([0.0; 3]).transpose()
            }
        );
    }

    #[test]
    fn renderer_chooses_the_plane_view_for_a_euclidean_domain() {
        let options = options(
            RenderOptions::builder(Fill::One(Colour::WHITE))
                .size(4, 4)
                .samples(1),
        );
        let (_, stats) = renderer(&domain("*442"), options).render();
        assert_eq!(stats.samples(), 16);
        assert_eq!(stats.outside(), 0);
        assert_eq!(stats.unconverged(), 0);
    }

    #[test]
    fn periodic_view_of_a_rectangle_equals_itself_shifted_by_a_translation_within_the_golden_allowance()
     {
        const SIZE: u32 = 512;
        const SHIFT: i64 = 128;
        let domain = domain_with_aspect("*2222", Some(1.618));
        let width = domain.side_lengths()[0];
        let pixel = 2.0 * width / 128.0;
        let options = options(
            RenderOptions::builder(Fill::Parity([
                Colour::from_srgb8(245, 222, 179),
                Colour::from_srgb8(70, 130, 180),
            ]))
            .size(SIZE, SIZE)
            .samples(GOLDEN_GRID),
        );
        let word = [1, 3];
        let found = translation(&domain, &word);
        println!(
            "*2222 aspect 1.618, word {word:?}: offset {:?}, linear defect {:e}, last row defect {:e}",
            found.offset, found.linear_defect, found.last_row_defect
        );
        assert!(found.linear_defect <= TRANSLATION_TOLERANCE);
        assert!(found.last_row_defect <= TRANSLATION_TOLERANCE);
        assert!(found.offset.y.abs() <= TRANSLATION_TOLERANCE);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the shift is a whole number of pixels within the image"
        )]
        let columns = (found.offset.x / pixel).round() as i64;
        assert_eq!(columns.abs(), SHIFT, "{:?}", found.offset);
        let view = View::periodic(&options, pixel);
        let (pixels, _) = renderer(&domain, options).with_view(view).render();
        let (here, there) = shifted_pairs(&pixels, SIZE, SIZE, columns, 0);
        let comparison = compare(&here, &there, GOLDEN_GRID, &linear_table());
        println!(
            "*2222 aspect 1.618 in the periodic view, shifted by {columns} pixels along the first axis: {comparison:?}, allowed {ALLOWED_PIXELS} differing pixels"
        );
        assert!(comparison.within_allowance(), "{comparison:?}");
    }

    const RIM_DISTANCES: [f64; 7] = [120.0, 24.0, 8.0, 4.0, 2.0, 1.0, 0.5];

    const SPHERE_ROTATIONS: [[f64; 3]; 2] = [[0.0; 3], [20.0, 35.0, 0.0]];

    const BALL_TANGENT_BOUNDS: [f64; 8] = [1e-15, 3e-15, 7e-15, 2e-14, 3e-14, 6e-14, 9e-14, 6e-12];

    const LIFT_BOUNDS: [f64; 7] = [2e-15, 2e-15, 3e-15, 4e-15, 4e-15, 6e-15, 9e-15];

    const ORTHOGONALITY_BOUND: f64 = 4e-15;

    const DETERMINANT_BOUND: f64 = 4e-15;

    fn ball_pixel(size: u32) -> f64 {
        2.0 / (15.0 / 16.0 * f64::from(size))
    }

    fn radians(degrees: [f64; 3]) -> [f64; 3] {
        degrees.map(f64::to_radians)
    }

    #[test]
    fn ball_view_lifts_the_centre_sample_to_the_pole_and_rejects_the_rim() {
        let view = View::canonical(
            &options(
                RenderOptions::builder(Fill::One(Colour::WHITE))
                    .size(5, 5)
                    .samples(1),
            ),
            Geometry::Spherical,
        );
        let pixel = ball_pixel(5);
        assert_eq!(view.pixel(), pixel);
        assert_eq!(view.position(0, 0, 0, 0), DVec2::new(-2.0, 2.0) * pixel);
        assert_eq!(view.position(4, 1, 0, 0), DVec2::new(2.0, 1.0) * pixel);
        let centre = view.position(2, 2, 0, 0);
        assert_eq!(centre, DVec2::ZERO);
        let Some(lift) = view.point(centre) else {
            panic!("the centre of the ball has no point");
        };
        assert_eq!(lift.point, DVec3::Z);
        assert_eq!(
            lift.tangents,
            [DVec3::new(pixel, 0.0, 0.0), DVec3::new(0.0, pixel, 0.0)]
        );
        assert_eq!(lift.normal, Some(DVec3::Z));
        for position in [
            DVec2::X,
            DVec2::NEG_Y,
            DVec2::new(1.5, 0.0),
            DVec2::new(f64::NAN, 0.0),
            DVec2::new(0.0, f64::INFINITY),
        ] {
            assert!(view.point(position).is_none(), "{position:?}");
        }
        let inside = DVec2::new(1.0_f64.next_down(), 0.0);
        assert!(view.point(inside).is_some());
    }

    #[test]
    fn viewing_matrix_is_a_rotation_with_the_signs_of_the_screen() {
        let triples = [
            [0.0, 0.0, 0.0],
            radians([20.0, 35.0, 0.0]),
            [1.0, 2.0, 3.0],
            [-2.5, 0.7, 5.0],
            [1.0e3, -1.0e3, 0.1],
            [core::f64::consts::FRAC_PI_2; 3],
            [-7.0, 100.0, -0.25],
        ];
        let mut orthogonality = (0.0_f64, [0.0; 3]);
        let mut determinant = (0.0_f64, [0.0; 3]);
        for angles in triples {
            let viewing = viewing_matrix(angles);
            let defect = (viewing.transpose() * viewing - DMat3::IDENTITY)
                .to_cols_array()
                .iter()
                .fold(0.0_f64, |largest, entry| largest.max(entry.abs()));
            if defect >= orthogonality.0 {
                orthogonality = (defect, angles);
            }
            let away = (viewing.determinant() - 1.0).abs();
            if away >= determinant.0 {
                determinant = (away, angles);
            }
        }
        println!(
            "viewing matrix: largest orthogonality defect {:e} at {:?}, bound {ORTHOGONALITY_BOUND:e}; largest determinant defect {:e} at {:?}, bound {DETERMINANT_BOUND:e}",
            orthogonality.0, orthogonality.1, determinant.0, determinant.1
        );
        assert!(orthogonality.0 <= ORTHOGONALITY_BOUND);
        assert!(determinant.0 <= DETERMINANT_BOUND);
        let angle = 0.5;
        assert!((viewing_matrix([angle, 0.0, 0.0]) * DVec3::Z).x > 0.0);
        assert!((viewing_matrix([0.0, angle, 0.0]) * DVec3::Z).y > 0.0);
        assert!((viewing_matrix([0.0, 0.0, angle]) * DVec3::X).y > 0.0);
        let still = viewing_matrix([0.0; 3]);
        for screen in [
            DVec3::Z,
            DVec3::new(0.3, -0.2, 0.9),
            DVec3::new(-0.75, 0.5, 0.125),
            DVec3::new(1.0e-300, -3.0, 7.5),
        ] {
            assert_eq!(still * screen, screen);
            assert_eq!(still.transpose() * screen, screen);
        }
    }

    fn rim_bin(distance: f64) -> usize {
        RIM_DISTANCES
            .iter()
            .position(|&band| distance >= band)
            .unwrap_or(RIM_DISTANCES.len())
    }

    #[test]
    fn ball_tangents_and_lift_agree_with_the_reference_lift() {
        const POSITIONS: u32 = 2048;
        const ANGLES: u32 = 512;
        let mut reference = Reference::new();
        let pixel = ball_pixel(512);
        let name = |index: usize| {
            RIM_DISTANCES.get(index).map_or_else(
                || "under 0.5 pixels from the rim".to_owned(),
                |band| format!("at least {band} pixels from the rim"),
            )
        };
        let mut tangent_worst: Vec<Worst> = (0..=RIM_DISTANCES.len())
            .map(|index| {
                Worst::new(
                    format!(
                        "ball tangent {}, largest coordinate error over the largest coordinate",
                        name(index)
                    ),
                    1,
                )
            })
            .collect();
        let golden_angle = core::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
        for index in 0..POSITIONS {
            let exponent =
                -2.0 + (2.0 + 240.0_f64.log10()) * f64::from(index) / f64::from(POSITIONS - 1);
            let distance = 10.0_f64.powf(exponent);
            let radius = 1.0 - distance * pixel;
            let angle = golden_angle * f64::from(index);
            let position = DVec2::new(radius * angle.cos(), radius * angle.sin());
            let Some((_, computed)) = lift(position, pixel) else {
                panic!("{position:?} is outside the ball");
            };
            let exact = reference.ball_tangents(position.to_array(), pixel);
            for (computed, exact) in computed.iter().zip(&exact) {
                let error = largest_magnitude(&difference(&vector(computed.to_array()), exact));
                tangent_worst[rim_bin(distance)].record(&error, &largest_magnitude(exact), || {
                    format!("position {position:?}, {distance:.4} pixels from the rim")
                });
            }
        }
        for (worst, bound) in tangent_worst.iter().zip(BALL_TANGENT_BOUNDS) {
            worst.assert_at_most(&mut reference, bound);
        }
        let mut lift_worst: Vec<Worst> = RIM_DISTANCES
            .iter()
            .map(|band| {
                Worst::new(
                    format!(
                        "lifted point, angle from the reference lift, {band} pixels from the rim"
                    ),
                    2 * usize::try_from(ANGLES).unwrap_or(usize::MAX),
                )
            })
            .collect();
        for degrees in SPHERE_ROTATIONS {
            let angles = radians(degrees);
            let [yaw, pitch, roll] = angles;
            let view = View::canonical(
                &options(
                    RenderOptions::builder(Fill::One(Colour::WHITE)).rotation(yaw, pitch, roll),
                ),
                Geometry::Spherical,
            );
            for (worst, distance) in lift_worst.iter_mut().zip(RIM_DISTANCES) {
                for index in 0..ANGLES {
                    let angle = core::f64::consts::TAU * f64::from(index) / f64::from(ANGLES);
                    let radius = 1.0 - distance * pixel;
                    let position = DVec2::new(radius * angle.cos(), radius * angle.sin());
                    let Some(lifted) = view.point(position) else {
                        panic!("{position:?} is outside the ball");
                    };
                    let exact = reference.lift(position.to_array(), angles);
                    let error = reference.angle(&vector(lifted.point.to_array()), &exact);
                    worst.record(&error, &crate::reference::int(1), || {
                        format!("rotation {degrees:?} at angle {index} of {ANGLES}")
                    });
                }
            }
        }
        for (worst, bound) in lift_worst.iter().zip(LIFT_BOUNDS) {
            worst.assert_at_most(&mut reference, bound);
        }
    }

    const ROTATION_CASES: [(&str, [f64; 3]); 4] = [
        ("*532", [0.0; 3]),
        ("*532", [20.0, 35.0, 0.0]),
        ("*55", [0.0; 3]),
        ("*722", [0.0; 3]),
    ];

    fn word_of(domain: &FundamentalDomain, word: &[usize]) -> Vec<usize> {
        let last = domain.sides().len() - 1;
        word.iter().map(|&index| index.min(last)).collect()
    }

    fn word_product(domain: &FundamentalDomain, word: &[usize]) -> DMat3 {
        word.iter().fold(DMat3::IDENTITY, |product, &index| {
            product * domain.sides()[index].pairing()
        })
    }

    #[test]
    fn turning_the_ball_by_an_element_of_the_group_leaves_the_image_within_the_golden_allowance() {
        let linear = linear_table();
        let [first, second] = [
            Colour::from_srgb8(245, 222, 179),
            Colour::from_srgb8(70, 130, 180),
        ];
        for (text, degrees) in ROTATION_CASES {
            let domain = domain(text);
            let [yaw, pitch, roll] = radians(degrees);
            let viewing = viewing_matrix([yaw, pitch, roll]);
            for shading in [Shading::Phong, Shading::Flat] {
                let render = |fill: Fill, viewing: DMat3| {
                    let options = options(
                        RenderOptions::builder(fill)
                            .size(GOLDEN_SIZE, GOLDEN_SIZE)
                            .samples(GOLDEN_GRID)
                            .rotation(yaw, pitch, roll)
                            .shading(shading),
                    );
                    let view = View::rotated(&options, viewing);
                    renderer(&domain, options).with_view(view).render().0
                };
                let plain = render(Fill::Parity([first, second]), viewing);
                let swapped = render(Fill::Parity([second, first]), viewing);
                for (word, expected) in [
                    (&[0_usize][..], &swapped),
                    (&[0, 1][..], &plain),
                    (&[0, 1, 0, 2][..], &plain),
                ] {
                    let word = word_of(&domain, word);
                    let element = word_product(&domain, &word);
                    let turned = render(Fill::Parity([first, second]), viewing * element);
                    let comparison = compare(expected, &turned, GOLDEN_GRID, &linear);
                    println!(
                        "{text} rotation {degrees:?} {shading:?}, word {word:?}: {comparison:?}, allowed {ALLOWED_PIXELS} differing pixels"
                    );
                    assert!(
                        comparison.within_allowance(),
                        "{text} {degrees:?} {shading:?} {word:?}: {comparison:?}"
                    );
                }
            }
        }
    }
    const GROUP_ROTATION_SIZE: u32 = 96;

    const GROUP_ROTATION_GRID: u32 = 2;

    const GROUP_ROTATION_CASES: [(&str, [f64; 3]); 8] = [
        ("3*2", [0.0; 3]),
        ("3*2", [30.0, 50.0, 10.0]),
        ("5*", [0.0; 3]),
        ("5*", [30.0, 50.0, 10.0]),
        ("5x", [0.0; 3]),
        ("5x", [30.0, 50.0, 10.0]),
        ("2*5", [0.0; 3]),
        ("2*5", [30.0, 50.0, 10.0]),
    ];

    #[test]
    fn turning_the_ball_by_an_element_of_a_spherical_group_gives_the_same_image_or_the_image_with_the_colours_exchanged()
     {
        let [first, second] = [
            Colour::from_srgb8(245, 222, 179),
            Colour::from_srgb8(70, 130, 180),
        ];
        for (text, degrees) in GROUP_ROTATION_CASES {
            let domain = domain(text);
            let [yaw, pitch, roll] = radians(degrees);
            let viewing = viewing_matrix([yaw, pitch, roll]);
            let render = |fill: Fill, viewing: DMat3| {
                let options = options(
                    RenderOptions::builder(fill)
                        .size(GROUP_ROTATION_SIZE, GROUP_ROTATION_SIZE)
                        .samples(GROUP_ROTATION_GRID)
                        .rotation(yaw, pitch, roll),
                );
                let view = View::rotated(&options, viewing);
                renderer(&domain, options).with_view(view).render().0
            };
            let plain = render(Fill::Parity([first, second]), viewing);
            let swapped = render(Fill::Parity([second, first]), viewing);
            let count = domain.sides().len();
            let words = (0..count)
                .map(|side| vec![side])
                .chain((0..count).map(|side| vec![side, (side + 1) % count]));
            for word in words {
                let reversing = word
                    .iter()
                    .filter(|&&side| {
                        domain.sides()[side].orientation() == escher_domain::Orientation::Reversing
                    })
                    .count();
                let expected = if reversing % 2 == 1 { &swapped } else { &plain };
                let element = word_product(&domain, &word);
                let turned = render(Fill::Parity([first, second]), viewing * element);
                let differing = expected
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .zip(turned.as_chunks::<3>().0)
                    .filter(|(left, right)| left != right)
                    .count();
                println!(
                    "{text} rotation {degrees:?}, word {word:?} with {reversing} reversing sides: {differing} differing pixels"
                );
                assert!(
                    *expected == turned,
                    "{text} {degrees:?} {word:?}: {differing} differing pixels"
                );
            }
        }
    }
}
