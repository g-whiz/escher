use escher_geometry::hyperbolic::{
    disk_from_band, disk_from_half_plane, disk_from_klein, from_disk,
};
use escher_geometry::{euclidean, hyperbolic, spherical};
use escher_signature::Geometry;
use glam::{DMat2, DMat3, DVec2, DVec3};

use crate::options::{HyperbolicModel, RenderOptions};

pub(crate) const MARGIN: f64 = 1.0 / 32.0;

const PLANE_SPAN: f64 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Model {
    Disk {
        pixel: f64,
        model: HyperbolicModel,
        placement: Option<DMat3>,
    },
    Plane {
        pixel: f64,
        placement: Option<DMat3>,
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
                pixel: match options.model {
                    HyperbolicModel::Disk | HyperbolicModel::Klein => disk,
                    HyperbolicModel::HalfPlane | HyperbolicModel::Band => {
                        2.0 / f64::from(options.height)
                    }
                },
                model: options.model,
                placement: None,
            },
            Geometry::Euclidean => Model::Plane {
                pixel: PLANE_SPAN / shorter,
                placement: None,
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

    pub(crate) fn centred(mut self, corner: DVec3) -> Self {
        match &mut self.model {
            Model::Disk { placement, .. } => *placement = Some(hyperbolic::translation(corner)),
            Model::Plane { placement, .. } => *placement = Some(euclidean::translation(corner)),
            Model::Ball { turn, .. } => *turn = spherical::translation(corner) * *turn,
        }
        self
    }

    pub(crate) fn position(&self, column: u32, row: u32, across: u32, down: u32) -> DVec2 {
        let pixel = match self.model {
            Model::Disk { pixel, .. } | Model::Plane { pixel, .. } | Model::Ball { pixel, .. } => {
                pixel
            }
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
            Model::Disk {
                pixel,
                model,
                placement,
            } => {
                let (point, carried) = match model {
                    HyperbolicModel::Disk => (from_disk(position)?, tangents(position, pixel)),
                    HyperbolicModel::HalfPlane => {
                        through_model(disk_from_half_plane(position + DVec2::Y)?, pixel)?
                    }
                    HyperbolicModel::Band => through_model(disk_from_band(position)?, pixel)?,
                    HyperbolicModel::Klein => through_model(disk_from_klein(position)?, pixel)?,
                };
                let (point, tangents) = placed(placement, point, carried);
                Some(Lift {
                    point,
                    tangents,
                    normal: None,
                })
            }
            Model::Plane { pixel, placement } => {
                let point = DVec3::new(position.x, position.y, 1.0);
                let carried = [DVec3::new(pixel, 0.0, 0.0), DVec3::new(0.0, pixel, 0.0)];
                let (point, tangents) = placed(placement, point, carried);
                Some(Lift {
                    point,
                    tangents,
                    normal: None,
                })
            }
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

fn placed(placement: Option<DMat3>, point: DVec3, tangents: [DVec3; 2]) -> (DVec3, [DVec3; 2]) {
    match placement {
        None => (point, tangents),
        Some(placement) => (
            placement * point,
            tangents.map(|tangent| placement * tangent),
        ),
    }
}

fn through_model((disk, jacobian): (DVec2, DMat2), pixel: f64) -> Option<(DVec3, [DVec3; 2])> {
    let point = from_disk(disk)?;
    let [first, second] = tangents(disk, 1.0);
    let carried = [jacobian.x_axis, jacobian.y_axis]
        .map(|column| (first * column.x + second * column.y) * pixel);
    Some((point, carried))
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
            Model::Disk { pixel, .. } | Model::Plane { pixel, .. } | Model::Ball { pixel, .. } => {
                pixel
            }
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
        Self::with_model(
            options,
            Model::Plane {
                pixel,
                placement: None,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use escher_domain::FundamentalDomain;
    use escher_signature::{Geometry, Signature};
    use glam::{DVec2, DVec3};

    use glam::DMat3;

    use super::{Model, View, from_disk, lift, tangents, through_model, viewing_matrix};
    use crate::colour::{Colour, Fill};
    use crate::golden_images::{
        ALLOWED_PIXELS, GOLDEN_COSET_FILL, GOLDEN_GRID, GOLDEN_SIZE, MODEL_MAPS, PLAN_TRIPLES,
        TRANSLATION_TOLERANCE, compare, domain_with_aspect, linear_table, renderer, shifted_pairs,
        translation, wythoff, wythoff_fill,
    };
    use crate::options::HyperbolicModel;
    use crate::options::RenderOptions;
    use crate::options::Shading;
    use crate::reference::{Measure, ReferenceExt};
    use escher_reference::{Reference, Worst, apply, difference, int, largest_magnitude, vector};

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
                model: HyperbolicModel::Disk,
                placement: None
            }
        );
        let plane = View::canonical(&options, Geometry::Euclidean);
        assert_eq!(
            plane.model,
            Model::Plane {
                pixel: 8.0 / 256.0,
                placement: None
            }
        );
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

    fn model_view(model: HyperbolicModel, width: u32, height: u32, grid: u32) -> View {
        View::canonical(
            &options(
                RenderOptions::builder(Fill::One(Colour::WHITE))
                    .size(width, height)
                    .samples(grid)
                    .model(model),
            ),
            Geometry::Hyperbolic,
        )
    }

    const VIEW_SIZES: [(u32, u32); 5] = [(64, 64), (128, 32), (32, 128), (192, 192), (256, 128)];

    #[test]
    fn view_of_a_hyperbolic_domain_holds_the_model_of_the_options_and_its_scale() {
        for (width, height) in VIEW_SIZES {
            let disk = model_view(HyperbolicModel::Disk, width, height, 2);
            assert_eq!(
                disk.model,
                Model::Disk {
                    pixel: 2.0 / (15.0 / 16.0 * f64::from(width.min(height))),
                    model: HyperbolicModel::Disk,
                    placement: None
                }
            );
            for map in &MODEL_MAPS {
                let view = model_view(map.model, width, height, 1);
                assert_eq!(
                    view.model,
                    Model::Disk {
                        pixel: map.pixel(width, height),
                        model: map.model,
                        placement: None
                    }
                );
                for (column, row) in [(0, 0), (width / 2, height / 2), (width - 1, height - 1)] {
                    let model_position = map.position(width, height, column, row);
                    let expected = (map.to_disk)(model_position)
                        .and_then(|(disk, _)| from_disk(disk))
                        .map(|point| point.to_array());
                    let found = view
                        .point(view.position(column, row, 0, 0))
                        .map(|lift| lift.point.to_array());
                    assert_eq!(
                        found, expected,
                        "{:?} at {width} by {height}: pixel ({column}, {row}) at the model position {model_position:?}",
                        map.model
                    );
                }
            }
        }
    }

    #[test]
    fn band_and_half_plane_views_span_the_image_height_with_two_model_units() {
        for (width, height) in VIEW_SIZES {
            for model in MODEL_MAPS
                .iter()
                .filter(|map| map.fills_height)
                .map(|map| map.model)
            {
                let view = model_view(model, width, height, 2);
                let half_height = view.pixel() * f64::from(height) / 2.0;
                assert!(
                    (half_height - 1.0).abs() <= f64::EPSILON,
                    "{model:?} at {width} by {height}: half the image height is {half_height} model units"
                );
                let top = view.position(width / 2, 0, 0, 0);
                let bottom = view.position(width / 2, height - 1, 1, 1);
                assert_eq!(top.y, -bottom.y, "{model:?} at {width} by {height}");
                assert!(
                    1.0 - top.y <= view.pixel(),
                    "{model:?} at {width} by {height}"
                );
            }
        }
    }

    #[test]
    fn centre_of_each_model_view_shows_the_centre_of_the_disk_the_same_way_up() {
        for map in &MODEL_MAPS {
            let view = model_view(map.model, 64, 64, 1);
            let model = map.model;
            let Some(centre) = view.point(DVec2::ZERO) else {
                panic!("{model:?}: the centre of the image has no point");
            };
            assert_eq!(centre.point, DVec3::Z, "{model:?}");
            let [first, second] = centre.tangents;
            assert!(
                first.x > 0.0 && first.y == 0.0 && first.z == 0.0,
                "{model:?}"
            );
            assert!(
                second.y > 0.0 && second.x == 0.0 && second.z == 0.0,
                "{model:?}"
            );
            for (offset, expected) in [
                (DVec2::new(0.0, 0.25), DVec2::new(0.0, 1.0)),
                (DVec2::new(0.25, 0.0), DVec2::new(1.0, 0.0)),
            ] {
                let Some(moved) = view.point(offset) else {
                    panic!("{model:?}: {offset:?} has no point");
                };
                let direction = DVec2::new(moved.point.x, moved.point.y);
                assert!(direction.dot(expected) > 0.0, "{model:?}: {offset:?}");
            }
        }
        let view = model_view(HyperbolicModel::HalfPlane, 64, 64, 1);
        let pixel = view.pixel();
        let Some(centre) = view.point(DVec2::ZERO) else {
            panic!("the centre of the half-plane view has no point");
        };
        assert_eq!(
            centre.tangents,
            [DVec3::new(pixel, 0.0, 0.0), DVec3::new(0.0, pixel, 0.0)]
        );
    }

    fn count_region(
        model: HyperbolicModel,
        width: u32,
        height: u32,
        inside: impl Fn(DVec2) -> Option<bool>,
    ) -> (usize, usize) {
        const GRID: u32 = 2;
        let view = model_view(model, width, height, GRID);
        let (mut shown, mut outside) = (0, 0);
        for row in 0..height {
            for column in 0..width {
                for down in 0..GRID {
                    for across in 0..GRID {
                        let position = view.position(column, row, across, down);
                        let Some(expected) = inside(position) else {
                            continue;
                        };
                        let found = view.point(position).is_some();
                        assert_eq!(
                            found, expected,
                            "{model:?} at {width} by {height}: the sample {across}, {down} of pixel ({column}, {row}) at {position:?}"
                        );
                        if found {
                            shown += 1;
                        } else {
                            outside += 1;
                        }
                    }
                }
            }
        }
        (shown, outside)
    }

    #[test]
    fn samples_on_and_beyond_the_edge_of_each_model_are_outside_samples() {
        for (width, height) in VIEW_SIZES {
            for model in MODEL_MAPS
                .iter()
                .filter(|map| map.fills_height)
                .map(|map| map.model)
            {
                let (shown, outside) = count_region(model, width, height, |_| Some(true));
                assert_eq!(
                    (shown, outside),
                    (4 * usize::try_from(width * height).unwrap_or(usize::MAX), 0),
                    "{model:?} at {width} by {height}"
                );
            }
            let (shown, outside) =
                count_region(HyperbolicModel::Klein, width, height, |position| {
                    let radius = position.length();
                    if radius >= 1.0 {
                        Some(false)
                    } else {
                        (radius <= 0.99).then_some(true)
                    }
                });
            assert!(shown > 0 && outside > 0, "Klein at {width} by {height}");
        }
        let half_plane = model_view(HyperbolicModel::HalfPlane, 64, 64, 1);
        let quarter = half_plane.pixel() / 4.0;
        for (position, shown) in [
            (DVec2::new(0.0, -1.0 + quarter), true),
            (DVec2::new(0.0, -1.0), false),
            (DVec2::new(0.0, -1.0 - quarter), false),
            (DVec2::new(1.0e9, -1.0 + 1.0e-3), false),
        ] {
            assert_eq!(
                half_plane.point(position).is_some(),
                shown,
                "half-plane at {position:?}"
            );
        }
        let band = model_view(HyperbolicModel::Band, 128, 32, 1);
        let quarter = band.pixel() / 4.0;
        for (position, shown) in [
            (DVec2::new(0.0, 1.0 - quarter), true),
            (DVec2::new(0.0, quarter - 1.0), true),
            (DVec2::new(0.0, 1.0), false),
            (DVec2::new(0.0, -1.0), false),
            (DVec2::new(0.0, 1.0 + quarter), false),
            (DVec2::new(40.0, 0.0), false),
        ] {
            assert_eq!(
                band.point(position).is_some(),
                shown,
                "band at {position:?}"
            );
        }
        for map in &MODEL_MAPS {
            let view = model_view(map.model, 64, 64, 1);
            for position in [
                DVec2::new(f64::NAN, 0.0),
                DVec2::new(0.0, f64::INFINITY),
                DVec2::new(f64::NEG_INFINITY, 0.0),
            ] {
                assert!(
                    view.point(position).is_none(),
                    "{:?} at {position:?}",
                    map.model
                );
            }
        }
    }

    #[test]
    fn tangents_of_each_model_agree_with_central_differences_of_the_reference_map() {
        const POSITIONS: u32 = 2048;
        const BOUNDS: [(HyperbolicModel, f64); 3] = [
            (HyperbolicModel::HalfPlane, 4.0e-9),
            (HyperbolicModel::Band, 4.0e-9),
            (HyperbolicModel::Klein, 2.0e-3),
        ];
        let mut reference = Reference::new();
        let golden_angle = core::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
        for map in &MODEL_MAPS {
            let model = map.model;
            let Some(&(_, bound)) = BOUNDS.iter().find(|(bounded, _)| *bounded == model) else {
                panic!("{model:?} has no bound");
            };
            let pixel = map.pixel(512, 512);
            let mut worst = Worst::new(
                format!(
                    "{model:?} screen tangent, largest coordinate error over the largest coordinate"
                ),
                2 * usize::try_from(POSITIONS + 1).unwrap_or(usize::MAX),
            );
            for index in 0..=POSITIONS {
                let disk = if index == POSITIONS {
                    DVec2::ZERO
                } else {
                    let exponent = -6.0 * f64::from(index) / f64::from(POSITIONS - 1);
                    let radius = 1.0 - 10.0_f64.powf(exponent);
                    let angle = golden_angle * f64::from(index);
                    DVec2::new(radius * angle.cos(), radius * angle.sin())
                };
                let position =
                    DVec2::from_array(reference.disk_to_model(map.exact, disk.to_array()));
                let Some((_, computed)) =
                    (map.to_disk)(position).and_then(|mapped| through_model(mapped, pixel))
                else {
                    panic!("{model:?}: the model position {position:?} of {disk:?} has no point");
                };
                let exact = reference.model_tangents(map.exact, position.to_array(), pixel);
                for (computed, exact) in computed.iter().zip(&exact) {
                    let error = largest_magnitude(&difference(&vector(computed.to_array()), exact));
                    worst.record(&error, &largest_magnitude(exact), || {
                        format!("model position {position:?} of the disk position {disk:?}")
                    });
                }
            }
            worst.assert_at_most(&mut reference, bound);
        }
    }

    #[test]
    fn walls_of_the_domain_in_the_klein_view_lie_on_straight_chords() {
        const SIZE: u32 = 192;
        const GRID: u32 = 4;
        const BOUND: f64 = 64.0 * f64::EPSILON;
        for text in ["*732", "*3333"] {
            let domain = domain(text);
            let view = model_view(HyperbolicModel::Klein, SIZE, SIZE, GRID);
            let covectors: Vec<DVec3> = domain.sides().iter().map(|side| side.covector()).collect();
            let mut nearest: Option<f64> = None;
            let mut sides = [0_usize; 2];
            let mut samples = 0_usize;
            for row in 0..SIZE {
                for column in 0..SIZE {
                    for down in 0..GRID {
                        for across in 0..GRID {
                            let position = view.position(column, row, across, down);
                            let Some(lift) = view.point(position) else {
                                continue;
                            };
                            samples += 1;
                            let affine = DVec3::new(position.x, position.y, 1.0);
                            for covector in &covectors {
                                let wall = covector.dot(lift.point);
                                let chord = covector.dot(affine);
                                let scale = covector.abs().element_sum();
                                let agree =
                                    (wall > 0.0) == (chord > 0.0) && (wall < 0.0) == (chord < 0.0);
                                if !agree {
                                    let ratio = chord.abs() / scale;
                                    nearest = Some(nearest.map_or(ratio, |found| found.max(ratio)));
                                    assert!(
                                        chord.abs() <= BOUND * scale,
                                        "{text}: the wall value {wall:e} and the chord value {chord:e} differ in sign at {position:?}"
                                    );
                                }
                                sides[usize::from(chord > 0.0)] += 1;
                            }
                        }
                    }
                }
            }
            println!(
                "{text} in the Klein view at {SIZE} by {SIZE} with {GRID} by {GRID} samples: {samples} samples, {} side values outside and {} inside or on a wall, largest chord value over the covector scale where the signs differ {nearest:?}, bound {BOUND:e}",
                sides[1], sides[0]
            );
            assert!(sides[0] > 0 && sides[1] > 0, "{text}");
        }
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
                    worst.record(&error, &int(1), || {
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

    const CENTRE_SIZE: u32 = 65;

    const CENTRE_ROTATIONS: [[f64; 3]; 4] = [
        [20.0, 35.0, 0.0],
        [-50.0, 10.0, 30.0],
        [70.0, -40.0, 200.0],
        [0.0, 0.0, 90.0],
    ];

    const CORNER_BOUND: f64 = 1e-14;

    const BETWEEN_BOUND: f64 = 1e-14;

    const BETWEEN_FRACTIONS: [f64; 3] = [0.25, 0.5, 0.75];

    fn centred_renderer(
        domain: &FundamentalDomain,
        corner: Option<usize>,
        builder: crate::options::RenderOptionsBuilder,
    ) -> crate::Renderer {
        let builder = builder.size(CENTRE_SIZE, CENTRE_SIZE).samples(1);
        renderer(
            domain,
            options(match corner {
                None => builder,
                Some(corner) => builder.centre(
                    u32::try_from(corner).expect("a domain has fewer corners than u32 can count"),
                ),
            }),
        )
    }

    fn corners(domain: &FundamentalDomain) -> std::ops::Range<usize> {
        0..domain.vertices().len()
    }

    fn vertex(domain: &FundamentalDomain, corner: usize) -> DVec3 {
        domain.vertices()[corner]
    }

    fn placed_point(view: &View, position: DVec2) -> DVec3 {
        match view.point(position) {
            Some(lift) => lift.point,
            None => panic!("the position {position} lies outside the view"),
        }
    }

    fn reference_distance(
        reference: &mut Reference,
        geometry: Geometry,
        first: DVec3,
        second: DVec3,
    ) -> f64 {
        let distance = Measure::of(geometry).distance(
            reference,
            &vector(first.to_array()),
            &vector(second.to_array()),
        );
        reference.approximate(&distance)
    }

    fn centre_cases() -> Vec<(&'static str, HyperbolicModel)> {
        let mut cases = Vec::new();
        for text in ["*732", "732"] {
            for model in [
                HyperbolicModel::Disk,
                HyperbolicModel::HalfPlane,
                HyperbolicModel::Band,
                HyperbolicModel::Klein,
            ] {
                cases.push((text, model));
            }
        }
        cases.push(("*632", HyperbolicModel::Disk));
        cases.push(("*532", HyperbolicModel::Disk));
        cases
    }

    #[test]
    fn centred_view_maps_the_centre_of_the_image_to_the_corner_of_each_position() {
        let mut reference = Reference::new();
        let mut largest: f64 = 0.0;
        let mut checked = 0;
        for (text, model) in centre_cases() {
            let domain = domain(text);
            for corner in corners(&domain) {
                let renderer = centred_renderer(
                    &domain,
                    Some(corner),
                    RenderOptions::builder(Fill::One(Colour::WHITE)).model(model),
                );
                let view = renderer.view();
                let centre = CENTRE_SIZE / 2;
                let position = view.position(centre, centre, 0, 0);
                let found = placed_point(view, position);
                let point = match MODEL_MAPS.iter().find(|map| map.model == model) {
                    None => {
                        assert_eq!(position, DVec2::ZERO, "{text} {model:?} {corner}");
                        found
                    }
                    Some(map) => {
                        let model_position = map.position(CENTRE_SIZE, CENTRE_SIZE, centre, centre);
                        let Some(canonical) =
                            (map.to_disk)(model_position).and_then(|(disk, _)| from_disk(disk))
                        else {
                            panic!("{text} {model:?}: {model_position:?} has no point");
                        };
                        let Model::Disk {
                            placement: Some(placement),
                            ..
                        } = view.model
                        else {
                            panic!("{text} {model:?} {corner}: the view has no placement");
                        };
                        let expected = placement * canonical;
                        assert_eq!(
                            found, expected,
                            "{text} {model:?} {corner}: the centre pixel at the model position {model_position:?}"
                        );
                        expected
                    }
                };
                let distance = reference_distance(
                    &mut reference,
                    domain.geometry(),
                    point,
                    vertex(&domain, corner),
                );
                println!("{text} in the {model:?} model, corner {corner}: distance {distance:e}");
                assert!(distance < CORNER_BOUND, "{text} {model:?} {corner}");
                largest = largest.max(distance);
                checked += 1;
            }
        }
        println!("largest distance {largest:e} of {checked} corners, bound {CORNER_BOUND:e}");
        assert_eq!(checked, (3 + 4) * 4 + 3 + 3);
    }

    #[test]
    fn rotation_after_centring_turns_the_corner_to_where_the_canonical_view_shows_the_centre() {
        let domain = domain("*532");
        let mut reference = Reference::new();
        let mut largest: f64 = 0.0;
        for degrees in CENTRE_ROTATIONS {
            let angles = radians(degrees);
            let viewing = reference.viewing(angles);
            let screen = apply(&viewing, &vector(domain.centre().to_array()));
            assert!(screen[2].is_positive(), "{degrees:?}");
            let position = DVec2::new(
                reference.approximate(&screen[0]),
                reference.approximate(&screen[1]),
            );
            for corner in corners(&domain) {
                let [yaw, pitch, roll] = angles;
                let renderer = centred_renderer(
                    &domain,
                    Some(corner),
                    RenderOptions::builder(Fill::One(Colour::WHITE)).rotation(yaw, pitch, roll),
                );
                let point = placed_point(renderer.view(), position);
                let distance = reference_distance(
                    &mut reference,
                    Geometry::Spherical,
                    point,
                    vertex(&domain, corner),
                );
                println!("*532 with rotation {degrees:?}, corner {corner}: angle {distance:e}");
                assert!(distance < CORNER_BOUND, "{degrees:?} {corner}");
                largest = largest.max(distance);
            }
        }
        println!("largest angle {largest:e}, bound {CORNER_BOUND:e}");
    }

    #[test]
    fn centred_view_shows_the_centre_of_the_domain_opposite_the_direction_of_the_corner() {
        let mut reference = Reference::new();
        let mut largest: f64 = 0.0;
        for text in ["*732", "*632", "*532"] {
            let domain = domain(text);
            let geometry = domain.geometry();
            let centre = domain.centre();
            for corner in corners(&domain) {
                let target = vertex(&domain, corner);
                let offset = match geometry {
                    Geometry::Hyperbolic => escher_geometry::hyperbolic::to_disk(target),
                    Geometry::Euclidean => target.truncate(),
                    Geometry::Spherical => target.truncate() / target.length(),
                };
                let renderer = centred_renderer(
                    &domain,
                    Some(corner),
                    RenderOptions::builder(Fill::One(Colour::WHITE)),
                );
                for fraction in BETWEEN_FRACTIONS {
                    let position = -fraction * offset;
                    let point = placed_point(renderer.view(), position);
                    let mut distance =
                        |first, second| reference_distance(&mut reference, geometry, first, second);
                    let to_corner = distance(target, point);
                    let excess = to_corner + distance(point, centre) - distance(target, centre);
                    println!(
                        "{text}, corner {corner}, fraction {fraction}: excess {excess:e}, distance to the corner {to_corner}"
                    );
                    assert!(to_corner > 0.0, "{text} {corner} {fraction}");
                    assert!(excess.abs() < BETWEEN_BOUND, "{text} {corner} {fraction}");
                    largest = largest.max(excess.abs());
                }
            }
        }
        println!("largest excess {largest:e}, bound {BETWEEN_BOUND:e}");
    }

    #[test]
    fn centre_pixel_of_a_centred_image_shows_the_border_of_the_corner() {
        for text in ["*732", "732", "*632", "*532"] {
            let domain = domain(text);
            for corner in corners(&domain) {
                let renderer = centred_renderer(
                    &domain,
                    Some(corner),
                    RenderOptions::builder(Fill::One(Colour::WHITE)).shading(Shading::Flat),
                );
                let (pixels, _) = renderer.render();
                let centre = usize::try_from(CENTRE_SIZE * (CENTRE_SIZE / 2) + CENTRE_SIZE / 2)
                    .expect("the index of the centre pixel of a 65 by 65 image fits in usize");
                assert_eq!(
                    pixels.as_chunks::<3>().0[centre],
                    Colour::BLACK.to_srgb8(),
                    "{text} {corner}"
                );
            }
        }
    }

    #[test]
    fn images_centred_on_each_corner_and_on_the_domain_differ_pairwise() {
        let domain = domain("*732");
        let images: Vec<Vec<u8>> = [None, Some(0), Some(1), Some(2)]
            .into_iter()
            .map(|corner| {
                centred_renderer(
                    &domain,
                    corner,
                    RenderOptions::builder(Fill::Parity([Colour::WHITE, Colour::BLACK])),
                )
                .render()
                .0
            })
            .collect();
        for (index, first) in images.iter().enumerate() {
            for (other, second) in images.iter().enumerate().skip(index + 1) {
                assert_ne!(first, second, "{index} {other}");
            }
        }
    }

    fn refusal(text: &str, corner: u32, fill: Fill) -> String {
        let options = options(RenderOptions::builder(fill).centre(corner));
        match crate::Renderer::new(&domain(text), options) {
            Ok(_) => panic!("{text} accepts the corner {corner}"),
            Err(error) => {
                assert_eq!(error.kind(), crate::ErrorKind::InvalidOption);
                error.to_string()
            }
        }
    }

    #[test]
    fn renderer_refuses_a_corner_that_the_domain_does_not_have() {
        let one = || Fill::One(Colour::WHITE);
        assert_eq!(
            refusal("*732", 3, one()),
            "cannot build renderer: the corner 3 is not below the 3 corners of the domain"
        );
        assert_eq!(
            refusal("*732", u32::MAX, one()),
            "cannot build renderer: the corner 4294967295 is not below the 3 corners of the domain"
        );
        assert_eq!(
            refusal("732", 4, one()),
            "cannot build renderer: the corner 4 is not below the 4 corners of the domain"
        );
        for corner in [0, u32::MAX] {
            assert_eq!(
                refusal("1", corner, one()),
                "cannot build renderer: the domain has no corner"
            );
        }
        let other = Fill::Wythoff(wythoff_fill(
            wythoff(&domain("*632"), [false, true, false]),
            &GOLDEN_COSET_FILL,
        ));
        assert_eq!(
            refusal("*732", 3, other),
            "cannot build renderer: the corner 3 is not below the 3 corners of the domain"
        );
        let last = options(RenderOptions::builder(one()).centre(3));
        assert!(crate::Renderer::new(&domain("732"), last).is_ok());
    }
}
