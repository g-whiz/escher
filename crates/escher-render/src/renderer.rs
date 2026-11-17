use escher_domain::FundamentalDomain;
use escher_signature::Geometry;
use glam::DVec3;

use crate::colour::{Colour, Fill, encode};
use crate::error::{Error, Result};
use crate::fold::{Wall, fold_carrying};
use crate::options::RenderOptions;
use crate::view::View;

/// Renders the tiling of a fundamental domain into 8-bit sRGB pixels, in the
/// Poincare disk for a hyperbolic domain and in the plane for a Euclidean one.
///
/// Make one per image with [`Renderer::new`], then render the whole image
/// with [`render`](Renderer::render) or bands of rows with
/// [`render_rows`](Renderer::render_rows). For a hyperbolic domain the disk is
/// centred in the image, with the centre of the domain at the centre of the
/// disk. For a Euclidean domain the tiling fills the image, with the centre of
/// the domain at the centre of the image and 8 units of the plane across the
/// shorter side. For each sample of the grid of a pixel, the renderer folds
/// the point of the view into the domain, gives the sample the fill, border,
/// or background colour, and stores the mean colour of the samples, computed in
/// linear light. The colour of a pixel depends only on the domain, the options,
/// and the position of the pixel, so every split into bands gives the same
/// bytes. The renderer starts no thread, so the caller chooses how to run bands
/// in parallel.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_render::{Colour, Fill, RenderOptions, Renderer};
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*732".parse::<Signature>()?)?;
/// let fill = Fill::Parity([Colour::WHITE, Colour::from_srgb8(70, 130, 180)]);
/// let options = RenderOptions::builder(fill).size(64, 48).samples(2).build()?;
/// let renderer = Renderer::new(&domain, options);
/// let (pixels, stats) = renderer.render();
/// assert_eq!(pixels.len(), 3 * 64 * 48);
/// assert_eq!(stats.samples(), 64 * 48 * 4);
/// assert_eq!(stats.unconverged(), 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Renderer {
    walls: Vec<Wall>,
    view: View,
    options: RenderOptions,
    half_width: f64,
    row_bytes: usize,
}

impl Renderer {
    /// Returns a renderer of the tiling of a domain with the given options.
    ///
    /// The renderer chooses the view from the geometry of the domain, which is
    /// the Poincare disk for a hyperbolic domain and the plane for a Euclidean
    /// one. It copies what it needs from the sides of the domain, so it keeps
    /// no reference to the domain. For a geometry without a view, every sample
    /// lies outside the view and takes the background colour.
    #[must_use]
    pub fn new(domain: &FundamentalDomain, options: RenderOptions) -> Self {
        let walls = domain.sides().iter().map(|&side| Wall::new(side)).collect();
        Self::with_walls(walls, domain.geometry(), options)
    }

    fn with_walls(walls: Vec<Wall>, geometry: Geometry, options: RenderOptions) -> Self {
        Self {
            walls,
            view: View::canonical(&options, geometry),
            half_width: options.border_width / 2.0,
            row_bytes: length(3 * u64::from(options.width)),
            options,
        }
    }

    /// Renders a band of whole rows of the image into a buffer that the
    /// caller owns, and returns the statistics of its samples.
    ///
    /// The band starts at row `first_row`, counted from the top row 0, and
    /// has as many rows as the buffer holds. The buffer receives 3 bytes per
    /// pixel, red, green, and blue in 8-bit sRGB, row by row from the top.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidBuffer`](crate::ErrorKind::InvalidBuffer) when the
    /// buffer is empty or its length is not a multiple of 3 times the width,
    /// or when the band extends past the last row of the image. The buffer is
    /// then left unchanged.
    pub fn render_rows(&self, first_row: u32, pixels: &mut [u8]) -> Result<RenderStats> {
        let length = pixels.len();
        if length == 0 || !length.is_multiple_of(self.row_bytes) {
            return Err(Error::partial_rows(length, self.row_bytes));
        }
        let rows = u64::try_from(length / self.row_bytes).unwrap_or(u64::MAX);
        let last = u64::from(first_row).saturating_add(rows - 1);
        if last >= u64::from(self.options.height) {
            return Err(Error::rows_outside(
                u64::from(first_row),
                last,
                self.options.height,
            ));
        }
        Ok(self.fill(first_row, pixels))
    }

    /// Renders the whole image on the calling thread and returns its pixels
    /// with the statistics of its samples.
    ///
    /// The pixels are those that [`render_rows`](Renderer::render_rows)
    /// writes for the whole image, byte for byte.
    #[must_use]
    pub fn render(&self) -> (Vec<u8>, RenderStats) {
        let mut pixels =
            vec![0; length(3 * u64::from(self.options.width) * u64::from(self.options.height))];
        let stats = self.fill(0, &mut pixels);
        (pixels, stats)
    }

    fn fill(&self, first_row: u32, pixels: &mut [u8]) -> RenderStats {
        let grid = self.options.samples;
        let weight = f64::from(grid * grid);
        let mut stats = RenderStats::default();
        for (row, bytes) in (first_row..).zip(pixels.chunks_exact_mut(self.row_bytes)) {
            for (column, pixel) in (0..).zip(bytes.as_chunks_mut::<3>().0) {
                let mut sum = [0.0; 3];
                for down in 0..grid {
                    for across in 0..grid {
                        let colour = self.sample(column, row, across, down, &mut stats);
                        for (total, channel) in sum.iter_mut().zip(colour.linear()) {
                            *total += channel;
                        }
                    }
                }
                for (byte, total) in pixel.iter_mut().zip(sum) {
                    *byte = encode(total / weight);
                }
            }
        }
        stats
    }

    fn sample(
        &self,
        column: u32,
        row: u32,
        across: u32,
        down: u32,
        stats: &mut RenderStats,
    ) -> Colour {
        stats.samples = stats.samples.saturating_add(1);
        let sampled = self
            .view
            .position(column, row, across, down)
            .and_then(|position| self.view.point(position));
        let Some((point, tangents)) = sampled else {
            stats.outside = stats.outside.saturating_add(1);
            return self.options.background;
        };
        let (result, tangents) =
            fold_carrying(&self.walls, point, tangents, self.options.max_folds, |_| {});
        stats.folds = stats.folds.saturating_add(u64::from(result.steps()));
        stats.largest_fold = stats.largest_fold.max(result.steps());
        if !result.converged() {
            stats.unconverged = stats.unconverged.saturating_add(1);
            return self.options.border_colour;
        }
        if on_border(&self.walls, result.point(), tangents, self.half_width) {
            return self.options.border_colour;
        }
        match self.options.fill {
            Fill::One(colour) => colour,
            Fill::Parity([even, odd]) => {
                if result.reversals() % 2 == 0 {
                    even
                } else {
                    odd
                }
            }
        }
    }
}

#[expect(
    clippy::neg_cmp_op_on_partial_ord,
    reason = "the negated comparison is true when a quantity is NaN, which must give a border sample"
)]
fn on_border(walls: &[Wall], point: DVec3, tangents: [DVec3; 2], half_width: f64) -> bool {
    walls.iter().any(|wall| {
        let covector = wall.covector();
        let value = covector.dot(point);
        let [first, second] = tangents.map(|tangent| covector.dot(tangent));
        let gradient = (first * first + second * second).sqrt();
        !(value.abs() >= half_width * gradient)
    })
}

fn length(bytes: u64) -> usize {
    usize::try_from(bytes).unwrap_or(usize::MAX)
}

/// Counts over the samples of a band of rows or of an image.
///
/// Get one from [`Renderer::render_rows`] or [`Renderer::render`], and
/// combine the counts of several bands with
/// [`merged`](RenderStats::merged). Every count is an integer, and a sum stops
/// at the largest value of its type instead of overflowing. The mean number of
/// fold steps per sample inside the view is `folds() / (samples() - outside())`.
///
/// # Examples
///
/// ```
/// use escher_render::RenderStats;
///
/// let empty = RenderStats::default();
/// assert_eq!(empty.merged(empty), empty);
/// assert_eq!(empty.samples(), 0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    samples: u64,
    outside: u64,
    folds: u64,
    largest_fold: u32,
    unconverged: u64,
}

impl RenderStats {
    /// Returns the number of samples, including those outside the view.
    #[must_use]
    pub fn samples(self) -> u64 {
        self.samples
    }

    /// Returns the number of samples outside the view, which take the
    /// background colour.
    ///
    /// For a hyperbolic domain these are the samples outside the disk. It is 0
    /// for a Euclidean domain, whose view covers the whole image, and it equals
    /// [`samples`](RenderStats::samples) for a geometry without a view.
    #[must_use]
    pub fn outside(self) -> u64 {
        self.outside
    }

    /// Returns the total number of steps of the folds of the samples inside
    /// the view.
    #[must_use]
    pub fn folds(self) -> u64 {
        self.folds
    }

    /// Returns the largest number of steps of the fold of one sample.
    #[must_use]
    pub fn largest_fold(self) -> u32 {
        self.largest_fold
    }

    /// Returns the number of samples whose fold did not converge, which
    /// take the border colour.
    #[must_use]
    pub fn unconverged(self) -> u64 {
        self.unconverged
    }

    /// Returns the statistics of two bands together: the sum of each count
    /// and the larger largest fold.
    #[must_use]
    pub fn merged(self, other: Self) -> Self {
        Self {
            samples: self.samples.saturating_add(other.samples),
            outside: self.outside.saturating_add(other.outside),
            folds: self.folds.saturating_add(other.folds),
            largest_fold: self.largest_fold.max(other.largest_fold),
            unconverged: self.unconverged.saturating_add(other.unconverged),
        }
    }
}

#[cfg(test)]
impl Renderer {
    pub(crate) fn with_view(mut self, view: View) -> Self {
        self.view = view;
        self
    }
}

#[cfg(test)]
mod tests {
    use escher_domain::FundamentalDomain;
    use escher_geometry::euclidean;
    use escher_geometry::hyperbolic::{covector, reflection};
    use escher_signature::{Geometry, Signature};
    use glam::{DVec2, DVec3};

    use super::{RenderStats, Renderer, on_border};
    use crate::colour::{Colour, Fill};
    use crate::fold::{Wall, fold_carrying};
    use crate::golden_images::{
        ALLOWED_PIXELS, GOLDEN_GRID, GOLDEN_IMAGES, compare, domain_with_aspect, golden_options,
        linear_table,
    };
    use crate::options::RenderOptions;
    use crate::reference::{Reference, count_bound, plane_border_distance, vector};
    use crate::view::View;

    const EUCLIDEAN_DOMAINS: [(&str, Option<f64>); 7] = [
        ("*632", None),
        ("*442", None),
        ("*333", None),
        ("*2222", Some(1.0)),
        ("*2222", Some(1.618)),
        ("*2222", Some(0.125)),
        ("*2222", Some(8.0)),
    ];

    const FOLD_TRIPLES: [&str; 7] = [
        "*732",
        "*542",
        "*444",
        "*433",
        "*(12)32",
        "*(1000)32",
        "*(1000)(1000)(1000)",
    ];

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

    fn statistics(
        samples: u64,
        outside: u64,
        folds: u64,
        largest_fold: u32,
        unconverged: u64,
    ) -> RenderStats {
        RenderStats {
            samples,
            outside,
            folds,
            largest_fold,
            unconverged,
        }
    }

    #[test]
    fn merged_adds_the_counts_and_takes_the_larger_largest_fold() {
        let first = statistics(10, 2, 30, 7, 1);
        let second = statistics(5, 1, 12, 9, 0);
        assert_eq!(first.merged(second), statistics(15, 3, 42, 9, 1));
        assert_eq!(second.merged(first), statistics(15, 3, 42, 9, 1));
    }

    #[test]
    fn merged_saturates_at_the_largest_count() {
        let full = statistics(u64::MAX, u64::MAX, u64::MAX, u32::MAX, u64::MAX);
        let one = statistics(1, 1, 1, 1, 1);
        assert_eq!(full.merged(one), full);
    }

    #[test]
    fn on_border_marks_a_sample_whose_side_value_is_nan_or_whose_gradient_is_not_finite() {
        let walls = domain("*732")
            .sides()
            .iter()
            .map(|&side| Wall::new(side))
            .collect::<Vec<_>>();
        let tangents = [DVec3::X, DVec3::Y];
        let inside = DVec3::Z;
        assert!(!on_border(&walls, inside, tangents, 0.0));
        assert!(on_border(
            &walls,
            DVec3::new(f64::NAN, 0.0, 1.0),
            tangents,
            0.0
        ));
        assert!(on_border(
            &walls,
            inside,
            [DVec3::splat(f64::NAN), DVec3::Y],
            0.0
        ));
        assert!(on_border(
            &walls,
            inside,
            [DVec3::splat(f64::INFINITY), DVec3::Y],
            0.0
        ));
    }

    #[test]
    fn render_counts_every_sample_and_the_corners_outside_the_disk() {
        let options = match RenderOptions::builder(Fill::One(Colour::WHITE))
            .size(4, 4)
            .samples(1)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        let (_, stats) = Renderer::new(&domain("*732"), options).render();
        assert_eq!(stats.samples(), 16);
        assert_eq!(stats.outside(), 4);
        assert_eq!(stats.unconverged(), 0);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "rendering whole images runs loops too long for the interpreter"
    )]
    fn perturbing_every_normal_by_one_unit_in_the_last_place_changes_the_golden_images_within_their_allowance()
     {
        let linear = linear_table();
        for image in GOLDEN_IMAGES {
            let text = image.signature;
            let domain = image.domain();
            let options = golden_options();
            let (pixels, _) = Renderer::new(&domain, options.clone()).render();
            let geometry = domain.geometry();
            let walls = domain
                .sides()
                .iter()
                .map(|side| match geometry {
                    Geometry::Euclidean => {
                        let moved = next_up(side.covector());
                        Wall::from_parts(moved, euclidean::reflection(moved), side.orientation())
                    }
                    Geometry::Hyperbolic | Geometry::Spherical => {
                        let moved = next_up(side.normal());
                        Wall::from_parts(covector(moved), reflection(moved), side.orientation())
                    }
                })
                .collect();
            let (moved, _) = Renderer::with_walls(walls, geometry, options).render();
            let comparison = compare(&pixels, &moved, GOLDEN_GRID, &linear);
            println!(
                "{text}, golden image {}: {comparison:?}, allowed {ALLOWED_PIXELS} differing pixels",
                image.file
            );
            assert!(comparison.within_allowance(), "{text}: {comparison:?}");
        }
    }

    fn next_up(vector: DVec3) -> DVec3 {
        DVec3::new(vector.x.next_up(), vector.y.next_up(), vector.z.next_up())
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
    fn gradient_form_of_the_border_agrees_with_the_distance_form() {
        const ANGLES: u32 = 512;
        const NEAR: f64 = 4.0;
        const HALF_WIDTHS: [f64; 2] = [0.5, 1.5];
        const MEASURED: [[usize; 2]; 6] = [[0, 2], [7, 0], [0, 0], [0, 0], [0, 0], [0, 0]];
        const MINIMUM: usize = 1000;
        let mut reference = Reference::new();
        let options = match RenderOptions::builder(Fill::One(Colour::WHITE)).build() {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        let view = View::canonical(&options, Geometry::Hyperbolic);
        let Some(pixel) = view.pixel() else {
            panic!("the view has no pixel scale");
        };
        for (band, measured) in (1..=6).zip(MEASURED) {
            let bounds = measured.map(count_bound);
            let radius = 1.0 - 10.0_f64.powi(-band);
            let mut near = 0;
            let mut differing = [0; 2];
            let mut largest = 0.0_f64;
            let mut worst = String::new();
            for text in FOLD_TRIPLES {
                let domain = domain(text);
                let walls: Vec<Wall> = domain.sides().iter().map(|&side| Wall::new(side)).collect();
                let covectors: Vec<_> = walls
                    .iter()
                    .map(|wall| vector(wall.covector().to_array()))
                    .collect();
                for index in 0..ANGLES {
                    let angle = core::f64::consts::TAU * f64::from(index) / f64::from(ANGLES);
                    let position = DVec2::new(radius * angle.cos(), radius * angle.sin());
                    let Some((point, tangents)) = view.point(position) else {
                        panic!("{text}: {position:?} is outside the disk");
                    };
                    let (result, tangents) = fold_carrying(&walls, point, tangents, 1500, |_| {});
                    assert!(result.converged(), "{text}: {position:?} does not converge");
                    let exact = reference.border_distance(
                        &covectors,
                        result.point().to_array(),
                        position.to_array(),
                        pixel,
                    );
                    let exact = reference.approximate(&exact);
                    if exact >= NEAR {
                        continue;
                    }
                    near += 1;
                    let gradient = walls
                        .iter()
                        .map(|wall| {
                            let [first, second] =
                                tangents.map(|tangent| wall.covector().dot(tangent));
                            wall.covector().dot(result.point()).abs()
                                / (first * first + second * second).sqrt()
                        })
                        .fold(f64::INFINITY, f64::min);
                    let relative = (gradient - exact).abs() / exact;
                    if relative > largest {
                        largest = relative;
                        worst = format!(
                            "{text} at angle {index}, gradient form {gradient:.6}, distance form {exact:.6}"
                        );
                    }
                    for (count, half_width) in differing.iter_mut().zip(HALF_WIDTHS) {
                        if on_border(&walls, result.point(), tangents, half_width)
                            != (exact < half_width)
                        {
                            *count += 1;
                        }
                    }
                }
            }
            println!(
                "band {band}: {near} samples within {NEAR} pixels of a wall, {} and {} classified differently at half widths {} and {}, bounds {} and {}, largest relative difference {largest:.3e} at {worst}",
                differing[0], differing[1], HALF_WIDTHS[0], HALF_WIDTHS[1], bounds[0], bounds[1]
            );
            assert!(
                near >= MINIMUM,
                "band {band}: only {near} samples near a wall"
            );
            assert!(
                differing[0] <= bounds[0] && differing[1] <= bounds[1],
                "band {band}: {differing:?}"
            );
        }
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the high-precision arithmetic runs loops too long for the interpreter"
    )]
    fn gradient_form_of_the_border_agrees_with_the_euclidean_distance_form() {
        const ANGLES: u32 = 512;
        const NEAR: f64 = 4.0;
        const HALF_WIDTHS: [f64; 2] = [0.5, 1.5];
        const DISTANCES: [f64; 4] = [1.0, 4.0, 16.0, 65.0];
        const MEASURED: [[usize; 2]; 4] = [[0, 0], [0, 0], [0, 0], [0, 0]];
        const MINIMUM: usize = 1000;
        let mut reference = Reference::new();
        let options = match RenderOptions::builder(Fill::One(Colour::WHITE)).build() {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        let view = View::canonical(&options, Geometry::Euclidean);
        let Some(pixel) = view.pixel() else {
            panic!("the view has no pixel scale");
        };
        for (radius, measured) in DISTANCES.into_iter().zip(MEASURED) {
            let bounds = measured.map(count_bound);
            let mut near = 0;
            let mut differing = [0; 2];
            let mut largest = 0.0_f64;
            let mut worst = String::new();
            for (text, aspect) in EUCLIDEAN_DOMAINS {
                let domain = domain_with_aspect(text, aspect);
                let walls: Vec<Wall> = domain.sides().iter().map(|&side| Wall::new(side)).collect();
                let covectors: Vec<_> = walls
                    .iter()
                    .map(|wall| vector(wall.covector().to_array()))
                    .collect();
                for index in 0..ANGLES {
                    let angle = core::f64::consts::TAU * f64::from(index) / f64::from(ANGLES);
                    let position = DVec2::new(radius * angle.cos(), radius * angle.sin());
                    let Some((point, tangents)) = view.point(position) else {
                        panic!("{text}: the plane view has no point at {position:?}");
                    };
                    let (result, tangents) = fold_carrying(&walls, point, tangents, 1500, |_| {});
                    assert!(result.converged(), "{text}: {position:?} does not converge");
                    let exact = plane_border_distance(&covectors, result.point().to_array(), pixel);
                    let exact = reference.approximate(&exact);
                    if exact >= NEAR {
                        continue;
                    }
                    near += 1;
                    let gradient = walls
                        .iter()
                        .map(|wall| {
                            let [first, second] =
                                tangents.map(|tangent| wall.covector().dot(tangent));
                            wall.covector().dot(result.point()).abs()
                                / (first * first + second * second).sqrt()
                        })
                        .fold(f64::INFINITY, f64::min);
                    let relative = (gradient - exact).abs() / exact;
                    if relative > largest {
                        largest = relative;
                        worst = format!(
                            "{text} aspect {aspect:?} at angle {index}, gradient form {gradient:.6}, distance form {exact:.6}"
                        );
                    }
                    for (count, half_width) in differing.iter_mut().zip(HALF_WIDTHS) {
                        if on_border(&walls, result.point(), tangents, half_width)
                            != (exact < half_width)
                        {
                            *count += 1;
                        }
                    }
                }
            }
            println!(
                "distance {radius}: {near} samples within {NEAR} pixels of a wall, {} and {} classified differently at half widths {} and {}, bounds {} and {}, largest relative difference {largest:.3e} at {worst}",
                differing[0], differing[1], HALF_WIDTHS[0], HALF_WIDTHS[1], bounds[0], bounds[1]
            );
            assert!(
                near >= MINIMUM,
                "distance {radius}: only {near} samples near a wall"
            );
            assert!(
                differing[0] <= bounds[0] && differing[1] <= bounds[1],
                "distance {radius}: {differing:?}"
            );
        }
    }
}
