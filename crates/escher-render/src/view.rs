use escher_geometry::hyperbolic::from_disk;
use glam::{DMat3, DVec2, DVec3};

use crate::options::RenderOptions;

const MARGIN: f64 = 1.0 / 32.0;

#[derive(Clone, Debug)]
pub(crate) struct View {
    width: f64,
    height: f64,
    grid: f64,
    pixel: f64,
    placement: Option<DMat3>,
}

impl View {
    pub(crate) fn canonical(options: &RenderOptions) -> Self {
        let shorter = options.width.min(options.height);
        Self {
            width: f64::from(options.width),
            height: f64::from(options.height),
            grid: f64::from(options.samples),
            pixel: 2.0 / ((1.0 - 2.0 * MARGIN) * f64::from(shorter)),
            placement: None,
        }
    }

    pub(crate) fn position(&self, column: u32, row: u32, across: u32, down: u32) -> DVec2 {
        let first = f64::from(column) + (f64::from(across) + 0.5) / self.grid;
        let second = f64::from(row) + (f64::from(down) + 0.5) / self.grid;
        DVec2::new(
            (first - self.width / 2.0) * self.pixel,
            (self.height / 2.0 - second) * self.pixel,
        )
    }

    pub(crate) fn point(&self, position: DVec2) -> Option<(DVec3, [DVec3; 2])> {
        let point = from_disk(position)?;
        let carried = tangents(position, self.pixel);
        Some(match self.placement {
            None => (point, carried),
            Some(placement) => (
                placement * point,
                carried.map(|tangent| placement * tangent),
            ),
        })
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
    pub(crate) fn pixel(&self) -> f64 {
        self.pixel
    }

    pub(crate) fn symmetric(
        options: &RenderOptions,
        domain: &escher_domain::FundamentalDomain,
    ) -> Self {
        use escher_geometry::hyperbolic::{inverse, rotation, translation};

        let vertices = domain.vertices();
        let to_vertex = translation(vertices[2]);
        let toward = inverse(to_vertex) * vertices[0];
        let placement = to_vertex * rotation(toward.y.atan2(toward.x));
        Self {
            placement: Some(placement),
            ..Self::canonical(options)
        }
    }
}

#[cfg(test)]
mod tests {
    use escher_domain::FundamentalDomain;
    use escher_signature::Signature;
    use glam::DVec2;

    use super::{View, tangents};
    use crate::colour::{Colour, Fill};
    use crate::golden_images::{ALLOWED_PIXELS, GOLDEN_GRID, GOLDEN_SIZE, compare, linear_table};
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
        let view = View::canonical(&options(
            RenderOptions::builder(Fill::One(Colour::WHITE))
                .size(64, 32)
                .samples(2),
        ));
        assert_eq!(view.pixel(), 2.0 / 30.0);
        assert_eq!(
            view.position(0, 0, 0, 0),
            DVec2::new(-31.75, 15.75) * view.pixel()
        );
        assert_eq!(
            view.position(63, 31, 1, 1),
            DVec2::new(31.75, -15.75) * view.pixel()
        );
        assert_eq!(
            view.position(32, 15, 0, 1),
            DVec2::new(0.25, 0.25) * view.pixel()
        );
    }

    #[test]
    fn point_is_none_outside_the_disk() {
        let view = View::canonical(&options(RenderOptions::builder(Fill::One(Colour::WHITE))));
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
}
