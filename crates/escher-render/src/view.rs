use escher_geometry::hyperbolic::from_disk;
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
    Missing,
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
        let model = match geometry {
            Geometry::Hyperbolic => Model::Disk {
                pixel: 2.0 / ((1.0 - 2.0 * MARGIN) * shorter),
                placement: None,
            },
            Geometry::Euclidean => Model::Plane {
                pixel: PLANE_SPAN / shorter,
            },
            Geometry::Spherical => Model::Missing,
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

    pub(crate) fn position(&self, column: u32, row: u32, across: u32, down: u32) -> Option<DVec2> {
        let pixel = match self.model {
            Model::Disk { pixel, .. } | Model::Plane { pixel } => pixel,
            Model::Missing => return None,
        };
        let first = f64::from(column) + (f64::from(across) + 0.5) / self.grid;
        let second = f64::from(row) + (f64::from(down) + 0.5) / self.grid;
        Some(DVec2::new(
            (first - self.width / 2.0) * pixel,
            (self.height / 2.0 - second) * pixel,
        ))
    }

    pub(crate) fn point(&self, position: DVec2) -> Option<(DVec3, [DVec3; 2])> {
        match self.model {
            Model::Disk { pixel, placement } => {
                let point = from_disk(position)?;
                let carried = tangents(position, pixel);
                Some(match placement {
                    None => (point, carried),
                    Some(placement) => (
                        placement * point,
                        carried.map(|tangent| placement * tangent),
                    ),
                })
            }
            Model::Plane { pixel } => Some((
                DVec3::new(position.x, position.y, 1.0),
                [DVec3::new(pixel, 0.0, 0.0), DVec3::new(0.0, pixel, 0.0)],
            )),
            Model::Missing => None,
        }
    }
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
    pub(crate) fn pixel(&self) -> Option<f64> {
        match self.model {
            Model::Disk { pixel, .. } | Model::Plane { pixel } => Some(pixel),
            Model::Missing => None,
        }
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

    use super::{Model, View, tangents};
    use crate::colour::{Colour, Fill};
    use crate::golden_images::{
        ALLOWED_PIXELS, GOLDEN_GRID, GOLDEN_SIZE, TRANSLATION_TOLERANCE, compare,
        domain_with_aspect, linear_table, shifted_pairs, translation,
    };
    use crate::options::RenderOptions;
    use crate::reference::{Reference, Worst, difference, largest_magnitude, vector};
    use crate::renderer::Renderer;

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
        assert_eq!(view.pixel(), Some(pixel));
        assert_eq!(
            view.position(0, 0, 0, 0),
            Some(DVec2::new(-31.75, 15.75) * pixel)
        );
        assert_eq!(
            view.position(63, 31, 1, 1),
            Some(DVec2::new(31.75, -15.75) * pixel)
        );
        assert_eq!(
            view.position(32, 15, 0, 1),
            Some(DVec2::new(0.25, 0.25) * pixel)
        );
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
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
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
    #[cfg_attr(
        miri,
        ignore = "rendering whole images runs loops too long for the interpreter"
    )]
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
            let renderer = Renderer::new(&domain, options).with_view(view);
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
        assert_eq!(view.pixel(), Some(0.25));
        let cases = [
            ((0, 0, 0, 0), DVec2::new(-31.75, 15.75)),
            ((63, 31, 1, 1), DVec2::new(31.75, -15.75)),
            ((32, 15, 0, 1), DVec2::new(0.25, 0.25)),
        ];
        for ((column, row, across, down), screen) in cases {
            let Some(position) = view.position(column, row, across, down) else {
                panic!("the plane view has no position for {column}, {row}");
            };
            assert_eq!(position, screen * 0.25);
            let Some((point, tangents)) = view.point(position) else {
                panic!("the plane view has no point at {position:?}");
            };
            assert_eq!(point, DVec3::new(position.x, position.y, 1.0));
            assert_eq!(
                tangents,
                [DVec3::new(0.25, 0.0, 0.0), DVec3::new(0.0, 0.25, 0.0)]
            );
        }
        let far = DVec2::new(1.0e6, -3.0);
        assert_eq!(
            view.point(far).map(|(point, _)| point),
            Some(DVec3::new(1.0e6, -3.0, 1.0))
        );
    }

    #[test]
    fn view_of_each_geometry_is_the_disk_the_plane_or_no_view() {
        let options = options(RenderOptions::builder(Fill::One(Colour::WHITE)).size(512, 256));
        let disk = View::canonical(&options, Geometry::Hyperbolic);
        assert_eq!(
            disk.model,
            Model::Disk {
                pixel: 2.0 / (15.0 / 16.0 * 256.0),
                placement: None
            }
        );
        let plane = View::canonical(&options, Geometry::Euclidean);
        assert_eq!(plane.model, Model::Plane { pixel: 8.0 / 256.0 });
        let missing = View::canonical(&options, Geometry::Spherical);
        assert_eq!(missing.model, Model::Missing);
        assert_eq!(missing.position(0, 0, 0, 0), None);
        for position in [DVec2::ZERO, DVec2::new(0.5, -0.25), DVec2::new(3.0, 0.0)] {
            assert!(missing.point(position).is_none(), "{position:?}");
        }
    }

    #[test]
    fn renderer_draws_only_the_background_for_a_geometry_without_a_view() {
        let background = Colour::from_srgb8(1, 2, 3);
        let options = options(
            RenderOptions::builder(Fill::One(Colour::WHITE))
                .size(4, 4)
                .samples(2)
                .background(background),
        );
        let view = View::canonical(&options, Geometry::Spherical);
        let (pixels, stats) = Renderer::new(&domain("*442"), options)
            .with_view(view)
            .render();
        assert_eq!(stats.samples(), 64);
        assert_eq!(stats.outside(), 64);
        assert_eq!(stats.folds(), 0);
        assert!(
            pixels
                .as_chunks::<3>()
                .0
                .iter()
                .all(|pixel| *pixel == background.to_srgb8())
        );
    }

    #[test]
    fn renderer_chooses_the_plane_view_for_a_euclidean_domain() {
        let options = options(
            RenderOptions::builder(Fill::One(Colour::WHITE))
                .size(4, 4)
                .samples(1),
        );
        let (_, stats) = Renderer::new(&domain("*442"), options).render();
        assert_eq!(stats.samples(), 16);
        assert_eq!(stats.outside(), 0);
        assert_eq!(stats.unconverged(), 0);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "rendering whole images runs loops too long for the interpreter"
    )]
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
        let (pixels, _) = Renderer::new(&domain, options).with_view(view).render();
        let (here, there) = shifted_pairs(&pixels, SIZE, SIZE, columns, 0);
        let comparison = compare(&here, &there, GOLDEN_GRID, &linear_table());
        println!(
            "*2222 aspect 1.618 in the periodic view, shifted by {columns} pixels along the first axis: {comparison:?}, allowed {ALLOWED_PIXELS} differing pixels"
        );
        assert!(comparison.within_allowance(), "{comparison:?}");
    }
}
