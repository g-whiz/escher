use escher_domain::FundamentalDomain;
use escher_signature::Geometry;
use glam::{DVec2, DVec3};

use crate::colour::{Colour, Fill, encode};
use crate::error::{Error, Result};
use crate::fold::{Fold, PointSize, Wall, fold_carrying};
use crate::options::{RenderOptions, Shading};
use crate::view::{Lift, View};

/// Renders the tiling of a fundamental domain into 8-bit sRGB pixels, in the
/// Poincare disk for a hyperbolic domain, in the plane for a Euclidean one,
/// and on a lit ball for a spherical one.
///
/// Make one per image with [`Renderer::new`], then render the whole image
/// with [`render`](Renderer::render) or bands of rows with
/// [`render_rows`](Renderer::render_rows). For a hyperbolic domain the disk is
/// centred in the image, with the centre of the domain at the centre of the
/// disk. For a Euclidean domain the tiling fills the image, with the centre of
/// the domain at the centre of the image and 8 units of the plane across the
/// shorter side. For a spherical domain the ball has the size and the place of
/// the disk and shows the front half of the sphere, seen from far away along
/// its third axis and turned by the viewing rotation of the options. With no
/// rotation, the centre of the domain lies at the centre of the ball.
///
/// A domain whose side pairings are rotations and translations, such as the
/// domain of `732`, has the view of its reflection domain, which is `*732`,
/// and draws borders only on its own sides, which lie on borders of the
/// reflection tiling. The domain of `2222` with a `skew` other than 0 is a
/// shear of the domain of `2222`, so its sides do not lie on the borders of
/// the tiling of `*2222`. Because no pairing of such a domain reverses
/// orientation, [`Fill::Parity`] gives every tile its first colour. The
/// documentation of [`fold`](crate::fold) states the accuracy of its fold.
///
/// For each sample of the grid of a pixel, the renderer folds the point of the
/// view into the domain and gives the sample the fill, border, or background
/// colour. It shades each sample on the ball by the [`Shading`] and the light
/// of the options, and it stores the mean colour of the samples, computed in
/// linear light. The colour of a pixel depends only on the domain, the
/// options, and the position of the pixel, so every split into bands gives the
/// same bytes. The renderer starts no thread, so the caller chooses how to run
/// bands in parallel.
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
/// let renderer = Renderer::new(&domain, options)?;
/// let (pixels, stats) = renderer.render();
/// assert_eq!(pixels.len(), 3 * 64 * 48);
/// assert_eq!(stats.samples(), 64 * 48 * 4);
/// assert_eq!(stats.unconverged(), 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Renderer {
    walls: Vec<Wall>,
    size: PointSize,
    view: View,
    options: RenderOptions,
    half_width: f64,
    row_bytes: usize,
}

impl Renderer {
    /// Returns a renderer of the tiling of a domain with the given options.
    ///
    /// The renderer chooses the view from the geometry of the domain, which is
    /// the Poincare disk for a hyperbolic domain, the plane for a Euclidean
    /// one, and the ball for a spherical one. It copies what it needs from the
    /// sides of the domain, so it keeps no reference to the domain.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidOption`](crate::ErrorKind::InvalidOption) when the
    /// fill is a [`Fill::Cosets`] whose [`CosetFill`](crate::CosetFill) was
    /// built for a domain with another side presentation, or the domain has
    /// no side presentation, because the colouring of such a fill does not
    /// colour the tiles of this domain. A domain with the same side
    /// presentation, such as the same signature with other shape parameters,
    /// is accepted. Every other fill gives no error.
    pub fn new(domain: &FundamentalDomain, options: RenderOptions) -> Result<Self> {
        if let Fill::Cosets(fill) = &options.fill
            && domain.side_presentation().ok().as_ref() != Some(fill.presentation())
        {
            return Err(Error::other_presentation());
        }
        let walls = domain.sides().iter().map(|&side| Wall::new(side)).collect();
        Ok(Self::with_walls(walls, domain.geometry(), options))
    }

    fn with_walls(walls: Vec<Wall>, geometry: Geometry, options: RenderOptions) -> Self {
        Self {
            walls,
            size: PointSize::of(geometry),
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
        let mut sides = Vec::new();
        for (row, bytes) in (first_row..).zip(pixels.chunks_exact_mut(self.row_bytes)) {
            for (column, pixel) in (0..).zip(bytes.as_chunks_mut::<3>().0) {
                let mut sum = [0.0; 3];
                for down in 0..grid {
                    for across in 0..grid {
                        let position = self.view.position(column, row, across, down);
                        let colour = self.sample(position, &mut stats, &mut sides);
                        for (total, channel) in sum.iter_mut().zip(colour) {
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

    fn sample(&self, position: DVec2, stats: &mut RenderStats, sides: &mut Vec<u8>) -> [f64; 3] {
        stats.samples = stats.samples.saturating_add(1);
        let Some(lift) = self.view.point(position) else {
            stats.outside = stats.outside.saturating_add(1);
            return self.options.background.linear();
        };
        let (result, tangents) = match &self.options.fill {
            Fill::Cosets(_) => {
                sides.clear();
                self.fold(&lift, |side| sides.push(side_byte(side)))
            }
            Fill::One(_) | Fill::Parity(_) => self.fold(&lift, |_| {}),
        };
        stats.folds = stats.folds.saturating_add(u64::from(result.steps()));
        stats.largest_fold = stats.largest_fold.max(result.steps());
        let colour = if result.converged() {
            self.converged_colour(result, tangents, sides)
        } else {
            stats.unconverged = stats.unconverged.saturating_add(1);
            self.options.border_colour
        };
        match (lift.normal, self.options.shading) {
            (Some(normal), Shading::Phong) => shade(colour.linear(), normal, self.options.light),
            (Some(_), Shading::Flat) | (None, _) => colour.linear(),
        }
    }

    fn fold(&self, lift: &Lift, visit: impl FnMut(usize)) -> (Fold, [DVec3; 2]) {
        fold_carrying(
            &self.walls,
            self.size,
            lift.point,
            lift.tangents,
            self.options.max_folds,
            visit,
        )
    }

    fn converged_colour(&self, result: Fold, tangents: [DVec3; 2], sides: &[u8]) -> Colour {
        if on_border(&self.walls, result.point(), tangents, self.half_width) {
            return self.options.border_colour;
        }
        match &self.options.fill {
            Fill::One(colour) => *colour,
            Fill::Parity([even, odd]) => {
                if result.reversals().is_multiple_of(2) {
                    *even
                } else {
                    *odd
                }
            }
            Fill::Cosets(fill) => fill.colour(sides),
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "a coset fill is rendered only with a domain that has a side presentation, which has at most 256 sides"
)]
fn side_byte(side: usize) -> u8 {
    u8::try_from(side)
        .expect("internal error: a side of a domain with a side presentation fits a byte")
}

const AMBIENT: f64 = 0.25;
const DIFFUSE: f64 = 0.75;
const SPECULAR: f64 = 0.25;

fn shade(colour: [f64; 3], normal: DVec3, light: DVec3) -> [f64; 3] {
    let facing = normal.dot(light);
    let diffuse = AMBIENT + DIFFUSE * facing.max(0.0);
    let highlight = if facing > 0.0 {
        let reflected = (2.0 * facing * normal.z - light.z).max(0.0);
        let squared = reflected * reflected;
        let fourth = squared * squared;
        let eighth = fourth * fourth;
        let sixteenth = eighth * eighth;
        SPECULAR * (sixteenth * sixteenth)
    } else {
        0.0
    };
    colour.map(|channel| channel * diffuse + highlight * (1.0 - channel))
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
    /// For a hyperbolic domain these are the samples outside the disk, and for
    /// a spherical domain those on or outside the rim of the ball. It is 0 for
    /// a Euclidean domain, whose view covers the whole image.
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

    /// Returns the number of samples whose fold did not converge.
    ///
    /// Each of these samples takes the border colour.
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
    use escher_geometry::hyperbolic::{covector, reflection};
    use escher_geometry::{euclidean, spherical};
    use escher_signature::{Geometry, Signature};
    use glam::{DVec2, DVec3};

    use super::{AMBIENT, DIFFUSE, RenderStats, Renderer, on_border, shade};
    use escher_group::Colouring;

    use crate::ErrorKind;
    use crate::colour::{Colour, CosetFill, Fill};
    use crate::fold::{PointSize, Wall, fold, fold_carrying};
    use crate::golden_images::{
        ALLOWED_PIXELS, COSET_GOLDEN_IMAGES, GOLDEN_COSET_FILL, GOLDEN_GRID, GOLDEN_IMAGES,
        POLYGON_DOMAINS, colouring, colourings_of, compare, coset_fill, domain_with_aspect,
        find_colouring, linear_table, renderer,
    };
    use crate::options::{RenderOptions, Shading};
    use crate::reference::{Reference, Worst, count_bound, error, plane_border_distance, vector};
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
        let (_, stats) = renderer(&domain("*732"), options).render();
        assert_eq!(stats.samples(), 16);
        assert_eq!(stats.outside(), 4);
        assert_eq!(stats.unconverged(), 0);
    }

    #[test]
    fn perturbing_every_normal_by_one_unit_in_the_last_place_changes_the_golden_images_within_their_allowance()
     {
        let linear = linear_table();
        for image in GOLDEN_IMAGES {
            let text = image.signature;
            let domain = image.domain();
            let options = image.options(&domain);
            let (pixels, _) = renderer(&domain, options.clone()).render();
            let geometry = domain.geometry();
            let rotation = !domain.reverses_orientation();
            let walls = domain
                .sides()
                .iter()
                .map(|side| match geometry {
                    _ if rotation => {
                        let pairing = side.pairing();
                        let moved = glam::DMat3::from_cols(
                            next_up(pairing.x_axis),
                            next_up(pairing.y_axis),
                            next_up(pairing.z_axis),
                        );
                        Wall::from_parts(next_up(side.covector()), moved, side.orientation())
                    }
                    Geometry::Euclidean => {
                        let moved = next_up(side.covector());
                        Wall::from_parts(moved, euclidean::reflection(moved), side.orientation())
                    }
                    Geometry::Hyperbolic => {
                        let moved = next_up(side.normal());
                        Wall::from_parts(covector(moved), reflection(moved), side.orientation())
                    }
                    Geometry::Spherical => {
                        let moved = next_up(side.normal());
                        Wall::from_parts(moved, spherical::reflection(moved), side.orientation())
                    }
                })
                .collect();
            let moved_renderer = match geometry {
                Geometry::Spherical => {
                    let viewing = crate::view::viewing_matrix(options.rotation);
                    let moved_viewing = glam::DMat3::from_cols(
                        next_up(viewing.x_axis),
                        next_up(viewing.y_axis),
                        next_up(viewing.z_axis),
                    );
                    let mut moved_options = options;
                    moved_options.light = next_up(moved_options.light);
                    let view = View::rotated(&moved_options, moved_viewing);
                    Renderer::with_walls(walls, geometry, moved_options).with_view(view)
                }
                Geometry::Hyperbolic | Geometry::Euclidean => {
                    Renderer::with_walls(walls, geometry, options)
                }
            };
            let (moved, _) = moved_renderer.render();
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
    fn gradient_form_of_the_border_agrees_with_the_distance_form() {
        const MEASURED: [[usize; 2]; 6] = [[0, 2], [7, 0], [0, 0], [0, 0], [0, 0], [0, 0]];
        let domains = FOLD_TRIPLES.map(|text| (text.to_owned(), domain(text)));
        assert_border_forms(&domains, &BorderBounds::Summed(MEASURED));
    }

    #[test]
    fn gradient_form_of_the_border_agrees_with_the_distance_form_on_the_polygons() {
        const NONE: [[usize; 2]; 6] = [[0; 2]; 6];
        const fn second_band(first: [usize; 2], second: [usize; 2]) -> [[usize; 2]; 6] {
            [first, second, [0; 2], [0; 2], [0; 2], [0; 2]]
        }
        const MEASURED: [[[usize; 2]; 6]; POLYGON_DOMAINS.len()] = [
            second_band([0, 0], [0, 4]),
            second_band([0, 0], [6, 0]),
            second_band([0, 0], [6, 0]),
            NONE,
            second_band([0, 1], [2, 0]),
            second_band([0, 1], [7, 0]),
            second_band([0, 0], [1, 0]),
            second_band([0, 0], [5, 0]),
            second_band([0, 0], [3, 0]),
            second_band([0, 0], [3, 0]),
            second_band([0, 0], [6, 0]),
            second_band([0, 0], [0, 8]),
            second_band([0, 0], [2, 44]),
            second_band([0, 0], [12, 0]),
            second_band([0, 2], [4, 26]),
            second_band([0, 0], [0, 35]),
            second_band([0, 1], [1, 0]),
            second_band([0, 0], [4, 29]),
            second_band([0, 0], [5, 0]),
            second_band([0, 0], [1, 50]),
        ];
        let domains: Vec<(String, FundamentalDomain)> = POLYGON_DOMAINS
            .iter()
            .map(|shape| (shape.name(), shape.domain()))
            .collect();
        assert_border_forms(&domains, &BorderBounds::PerDomain(&MEASURED));
    }

    enum BorderBounds<'a> {
        Summed([[usize; 2]; 6]),
        PerDomain(&'a [[[usize; 2]; 6]]),
    }

    fn gradient_form(walls: &[Wall], point: DVec3, tangents: [DVec3; 2]) -> f64 {
        walls
            .iter()
            .map(|wall| {
                let [first, second] = tangents.map(|tangent| wall.covector().dot(tangent));
                wall.covector().dot(point).abs() / (first * first + second * second).sqrt()
            })
            .fold(f64::INFINITY, f64::min)
    }

    fn assert_border_forms(domains: &[(String, FundamentalDomain)], measured: &BorderBounds<'_>) {
        const ANGLES: u32 = 512;
        const NEAR: f64 = 4.0;
        const HALF_WIDTHS: [f64; 2] = [0.5, 1.5];
        const MINIMUM: usize = 1000;
        let mut reference = Reference::new();
        let options = match RenderOptions::builder(Fill::One(Colour::WHITE)).build() {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        let view = View::canonical(&options, Geometry::Hyperbolic);
        let pixel = view.pixel();
        for (band_index, band) in (1..=6).enumerate() {
            let radius = 1.0 - 10.0_f64.powi(-band);
            let mut near = 0;
            let mut differing = [0; 2];
            let mut largest = 0.0_f64;
            let mut worst = String::new();
            for (domain_index, (text, domain)) in domains.iter().enumerate() {
                let before = differing;
                let walls: Vec<Wall> = domain.sides().iter().map(|&side| Wall::new(side)).collect();
                let covectors: Vec<_> = walls
                    .iter()
                    .map(|wall| vector(wall.covector().to_array()))
                    .collect();
                for index in 0..ANGLES {
                    let angle = core::f64::consts::TAU * f64::from(index) / f64::from(ANGLES);
                    let position = DVec2::new(radius * angle.cos(), radius * angle.sin());
                    let Some(lift) = view.point(position) else {
                        panic!("{text}: {position:?} is outside the disk");
                    };
                    let (result, tangents) = fold_carrying(
                        &walls,
                        PointSize::LastCoordinate,
                        lift.point,
                        lift.tangents,
                        1500,
                        |_| {},
                    );
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
                    let gradient = gradient_form(&walls, result.point(), tangents);
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
                let counts = [0, 1].map(|half| differing[half] - before[half]);
                if counts != [0; 2] {
                    println!(
                        "band {band}, {text}: {} and {} classified differently",
                        counts[0], counts[1]
                    );
                }
                if let BorderBounds::PerDomain(measured) = measured {
                    let bounds = measured[domain_index][band_index].map(count_bound);
                    assert!(
                        counts[0] <= bounds[0] && counts[1] <= bounds[1],
                        "band {band}, {text}: {counts:?}, bounds {bounds:?}"
                    );
                }
            }
            println!(
                "band {band}: {near} samples within {NEAR} pixels of a wall, {} and {} classified differently at half widths {} and {}, largest relative difference {largest:.3e} at {worst}",
                differing[0], differing[1], HALF_WIDTHS[0], HALF_WIDTHS[1]
            );
            assert!(
                near >= MINIMUM,
                "band {band}: only {near} samples near a wall"
            );
            if let BorderBounds::Summed(measured) = measured {
                let bounds = measured[band_index].map(count_bound);
                assert!(
                    differing[0] <= bounds[0] && differing[1] <= bounds[1],
                    "band {band}: {differing:?}, bounds {bounds:?}"
                );
            }
        }
    }

    #[test]
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
        let pixel = view.pixel();
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
                    let Some(lift) = view.point(position) else {
                        panic!("{text}: the plane view has no point at {position:?}");
                    };
                    let (result, tangents) = fold_carrying(
                        &walls,
                        PointSize::LastCoordinate,
                        lift.point,
                        lift.tangents,
                        1500,
                        |_| {},
                    );
                    assert!(result.converged(), "{text}: {position:?} does not converge");
                    let exact = plane_border_distance(&covectors, result.point().to_array(), pixel);
                    let exact = reference.approximate(&exact);
                    if exact >= NEAR {
                        continue;
                    }
                    near += 1;
                    let gradient = gradient_form(&walls, result.point(), tangents);
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

    const SHADING_BOUND: f64 = 2e-14;

    const SPHERE_BORDER_DOMAINS: [&str; 5] = ["*532", "*432", "*332", "*722", "*55"];

    const RIM_DISTANCES: [f64; 7] = [120.0, 24.0, 8.0, 4.0, 2.0, 1.0, 0.5];

    const SPHERE_BORDER_MEASURED: [[usize; 2]; 7] = [
        [0, 3],
        [3, 23],
        [8, 99],
        [39, 336],
        [92, 956],
        [278, 0],
        [1593, 0],
    ];

    fn light_directions() -> [DVec3; 4] {
        [
            DVec3::new(-1.0, 1.0, 2.0),
            DVec3::new(1.0, 1.0, 1.0),
            DVec3::new(0.0, -1.0, 1.0),
            DVec3::new(2.0, 0.0, 1.0),
        ]
    }

    fn normalized_light(direction: DVec3) -> DVec3 {
        match RenderOptions::builder(Fill::One(Colour::WHITE))
            .light(direction)
            .build()
        {
            Ok(options) => options.light,
            Err(error) => panic!("{direction:?}: {error}"),
        }
    }

    fn hemisphere(steps: u32) -> Vec<DVec3> {
        let mut normals = Vec::new();
        for row in 0..=steps {
            for column in 0..=steps {
                let first = -1.0 + 2.0 * f64::from(column) / f64::from(steps);
                let second = -1.0 + 2.0 * f64::from(row) / f64::from(steps);
                let d = 1.0 - (first * first + second * second);
                if d > 0.0 {
                    normals.push(DVec3::new(first, second, d.sqrt()));
                }
            }
        }
        normals
    }

    #[test]
    fn shading_agrees_with_the_reference() {
        let mut reference = Reference::new();
        let normals = hemisphere(32);
        let mut worst = Worst::new("shaded linear-light channel, absolute error", 1000);
        for direction in light_directions() {
            let light = normalized_light(direction);
            for &normal in &normals {
                for step in 0..=16 {
                    let channel = f64::from(step) / 16.0;
                    let [shaded, ..] = shade([channel; 3], normal, light);
                    let exact = reference.shade(normal.to_array(), light.to_array(), channel);
                    worst.record(&error(shaded, &exact), &crate::reference::int(1), || {
                        format!("light {direction:?}, normal {normal:?}, channel {channel}")
                    });
                }
            }
        }
        worst.assert_at_most(&mut reference, SHADING_BOUND);
    }

    #[test]
    fn shading_keeps_every_channel_between_zero_and_one() {
        for direction in light_directions() {
            let light = normalized_light(direction);
            for normal in hemisphere(24) {
                for step in 0..=16 {
                    let channel = f64::from(step) / 16.0;
                    for shaded in shade([channel; 3], normal, light) {
                        assert!(
                            (0.0..=1.0).contains(&shaded),
                            "{shaded} for {normal:?}, {light:?}, {channel}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn shading_gives_exact_values_at_chosen_inputs() {
        let cases = [
            (DVec3::Z, DVec3::Z, 0.0, 0.25),
            (DVec3::Z, DVec3::Z, 0.5, 0.625),
            (DVec3::Z, DVec3::Z, 1.0, 1.0),
            (DVec3::X, DVec3::NEG_X, 0.0, 0.0),
            (DVec3::X, DVec3::NEG_X, 0.5, 0.125),
            (DVec3::X, DVec3::Z, 0.0, 0.0),
            (DVec3::X, DVec3::Z, 0.5, 0.125),
            (DVec3::X, DVec3::new(0.6, 0.0, 0.8), 0.0, 0.0),
        ];
        for (normal, light, channel, expected) in cases {
            assert_eq!(
                shade([channel; 3], normal, light),
                [expected; 3],
                "{normal:?}, {light:?}, {channel}"
            );
        }
    }

    fn small_ball(shading: Shading, colour: Colour) -> Vec<u8> {
        let options = match RenderOptions::builder(Fill::One(colour))
            .size(3, 3)
            .samples(1)
            .border_width(0.0)
            .shading(shading)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        renderer(&domain("*532"), options).render().0
    }

    #[test]
    fn flat_shading_leaves_the_colour_of_the_ball_unchanged() {
        let colour = Colour::from_srgb8(128, 64, 200);
        let flat = small_ball(Shading::Flat, colour);
        let phong = small_ball(Shading::Phong, colour);
        assert_eq!(flat[12..15], colour.to_srgb8());
        assert_ne!(phong[12..15], colour.to_srgb8());
        assert_eq!(flat[0..3], Colour::WHITE.to_srgb8());
        assert_eq!(phong[0..3], Colour::WHITE.to_srgb8());
    }

    #[test]
    fn shading_terms_peak_where_the_normal_meets_the_light_and_the_halfway_vector() {
        let normals = hemisphere(100);
        let spacing = 2.0 / 100.0;
        for direction in light_directions() {
            let light = normalized_light(direction);
            let peak = |channel: f64| {
                normals
                    .iter()
                    .copied()
                    .fold((f64::NEG_INFINITY, DVec3::ZERO), |best, normal| {
                        let [value, ..] = shade([channel; 3], normal, light);
                        if value > best.0 {
                            (value, normal)
                        } else {
                            best
                        }
                    })
                    .1
            };
            let diffuse = peak(1.0);
            assert!(
                diffuse.angle_between(light) <= 2.0 * spacing,
                "{direction:?}: diffuse peak {diffuse:?}"
            );
            let halfway = (light + DVec3::Z).normalize();
            let specular = peak(0.0);
            assert!(
                specular.angle_between(halfway) <= 2.0 * spacing,
                "{direction:?}: specular peak {specular:?}"
            );
        }
        assert!((AMBIENT + DIFFUSE - 1.0).abs() <= 0.0);
    }

    fn grey_ball(direction: DVec3) -> Vec<u8> {
        let options = match RenderOptions::builder(Fill::One(Colour::from_srgb8(128, 128, 128)))
            .size(BRIGHT_SIZE, BRIGHT_SIZE)
            .samples(GOLDEN_GRID)
            .border_width(0.0)
            .background(Colour::BLACK)
            .light(direction)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        renderer(&domain("*532"), options).render().0
    }

    const BRIGHT_SIZE: u32 = 192;

    #[test]
    fn brightest_pixel_follows_the_light_and_a_quarter_turn_of_the_light_turns_the_image() {
        let size = usize::try_from(BRIGHT_SIZE).unwrap_or(usize::MAX);
        let grey = Colour::from_srgb8(128, 128, 128).linear();
        let pixel = 2.0 / (15.0 / 16.0 * f64::from(BRIGHT_SIZE));
        let half = f64::from(BRIGHT_SIZE) / 2.0;
        for direction in light_directions() {
            let light = normalized_light(direction);
            let mut predicted = (f64::NEG_INFINITY, 0, 0);
            for row in 0..size {
                for column in 0..size {
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "the pixel indices are far below 2 to the power 52"
                    )]
                    let position = DVec2::new(
                        (column as f64 + 0.5 - half) * pixel,
                        (half - row as f64 - 0.5) * pixel,
                    );
                    let d = 1.0 - position.length_squared();
                    if d > 0.0 {
                        let normal = DVec3::new(position.x, position.y, d.sqrt());
                        let [value, ..] = shade(grey, normal, light);
                        if value > predicted.0 {
                            predicted = (value, column, row);
                        }
                    }
                }
            }
            let pixels = grey_ball(direction);
            let brightest = pixels.as_chunks::<3>().0.iter().map(|pixel| pixel[0]).max();
            let at: Vec<(usize, usize)> = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .enumerate()
                .filter(|(_, pixel)| Some(pixel[0]) == brightest)
                .map(|(index, _)| (index % size, index / size))
                .collect();
            println!(
                "light {direction:?}: brightest byte {brightest:?} at {at:?}, formula largest at ({}, {})",
                predicted.1, predicted.2
            );
            #[expect(
                clippy::cast_precision_loss,
                reason = "the pixel indices and their count are far below 2 to the power 52"
            )]
            let centroid = {
                let count = at.len() as f64;
                let column = at.iter().map(|&(column, _)| column as f64).sum::<f64>() / count;
                let row = at.iter().map(|&(_, row)| row as f64).sum::<f64>() / count;
                (column, row)
            };
            #[expect(
                clippy::cast_precision_loss,
                reason = "the pixel indices are far below 2 to the power 52"
            )]
            let offset = (
                (centroid.0 - predicted.1 as f64).abs(),
                (centroid.1 - predicted.2 as f64).abs(),
            );
            assert!(
                offset.0 <= 1.0 && offset.1 <= 1.0,
                "{direction:?}: brightest pixels centred at {centroid:?}, predicted ({}, {})",
                predicted.1,
                predicted.2
            );
        }
        let linear = linear_table();
        for direction in light_directions() {
            let turned_light = DVec3::new(-direction.y, direction.x, direction.z);
            let image = grey_ball(direction);
            let turned = grey_ball(turned_light);
            let mut expected = vec![0; image.len()];
            for row in 0..size {
                for column in 0..size {
                    let source = 3 * (column * size + (size - 1 - row));
                    let target = 3 * (row * size + column);
                    expected[target..target + 3].copy_from_slice(&image[source..source + 3]);
                }
            }
            let comparison = compare(&expected, &turned, GOLDEN_GRID, &linear);
            println!(
                "light {direction:?} turned a quarter turn: {comparison:?}, allowed {ALLOWED_PIXELS} differing pixels"
            );
            assert!(
                comparison.within_allowance(),
                "{direction:?}: {comparison:?}"
            );
        }
    }

    fn tile(
        walls: &[Wall],
        pairings: &[glam::DMat3],
        point: DVec3,
        centre: DVec3,
    ) -> Option<DVec3> {
        let mut sides = Vec::new();
        let (result, _) = fold_carrying(
            walls,
            PointSize::LargestCoordinate,
            point,
            [DVec3::ZERO; 2],
            1500,
            |index| sides.push(index),
        );
        result.converged().then(|| {
            sides.iter().rev().fold(centre, |centre, &index| {
                spherical::inverse(pairings[index]) * centre
            })
        })
    }

    #[test]
    fn gradient_form_of_the_border_agrees_with_the_oracle_on_the_ball() {
        const SAMPLES: u32 = 4000;
        const CIRCLE: u32 = 64;
        const HALF_WIDTHS: [f64; 2] = [0.5, 1.5];
        const SAME_TILE: f64 = 1e-4;
        let options = match RenderOptions::builder(Fill::One(Colour::WHITE)).build() {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        let view = View::canonical(&options, Geometry::Spherical);
        let pixel = view.pixel();
        let domains: Vec<_> = SPHERE_BORDER_DOMAINS
            .iter()
            .map(|&text| {
                let domain = domain(text);
                let walls: Vec<Wall> = domain.sides().iter().map(|&side| Wall::new(side)).collect();
                let pairings: Vec<_> = domain.sides().iter().map(|side| side.pairing()).collect();
                (text, walls, pairings, domain.centre())
            })
            .collect();
        for (distance, measured) in RIM_DISTANCES.into_iter().zip(SPHERE_BORDER_MEASURED) {
            let radius = 1.0 - distance * pixel;
            let mut differing = [0_usize; 2];
            let mut tested = [0_usize; 2];
            let mut border = 0_usize;
            for (text, walls, pairings, centre) in &domains {
                for index in 0..SAMPLES {
                    let angle = core::f64::consts::TAU * f64::from(index) / f64::from(SAMPLES);
                    let position = DVec2::new(radius * angle.cos(), radius * angle.sin());
                    let Some(lift) = view.point(position) else {
                        panic!("{text}: {position:?} is outside the ball");
                    };
                    let (result, tangents) = fold_carrying(
                        walls,
                        PointSize::LargestCoordinate,
                        lift.point,
                        lift.tangents,
                        1500,
                        |_| {},
                    );
                    assert!(result.converged(), "{text}: {position:?} does not converge");
                    let Some(here) = tile(walls, pairings, lift.point, *centre) else {
                        panic!("{text}: {position:?} has no tile");
                    };
                    for (slot, half_width) in HALF_WIDTHS.into_iter().enumerate() {
                        if distance < half_width {
                            continue;
                        }
                        tested[slot] += 1;
                        let gradient = on_border(walls, result.point(), tangents, half_width);
                        if slot == 0 && gradient {
                            border += 1;
                        }
                        let oracle = (0..CIRCLE).any(|step| {
                            let turn = core::f64::consts::TAU * f64::from(step) / f64::from(CIRCLE);
                            let offset = DVec2::new(turn.cos(), turn.sin()) * (half_width * pixel);
                            view.point(position + offset).is_some_and(|other| {
                                tile(walls, pairings, other.point, *centre).is_none_or(|there| {
                                    spherical::distance(here, there) >= SAME_TILE
                                })
                            })
                        });
                        if gradient != oracle {
                            differing[slot] += 1;
                        }
                    }
                }
            }
            #[expect(
                clippy::cast_precision_loss,
                reason = "the counts are far below 2 to the power 52"
            )]
            let share = border as f64 / tested[0] as f64;
            let bounds = measured.map(count_bound);
            for (slot, half_width) in HALF_WIDTHS.into_iter().enumerate() {
                if tested[slot] == 0 {
                    println!(
                        "rim distance {distance}: skipped at half width {half_width}, because the band lies closer to the rim than the half width"
                    );
                    continue;
                }
                println!(
                    "rim distance {distance}, half width {half_width}: {} of {} samples classified differently, bound {}",
                    differing[slot], tested[slot], bounds[slot]
                );
                assert!(
                    differing[slot] <= bounds[slot],
                    "rim distance {distance}, half width {half_width}: {differing:?}"
                );
            }
            println!("rim distance {distance}: border share {share:.4} at half width 0.5");
        }
    }

    const COSET_SIZE: u32 = 64;

    fn coset_renderer(domain: &FundamentalDomain, fill: CosetFill) -> Renderer {
        let options = match RenderOptions::builder(Fill::Cosets(fill))
            .size(COSET_SIZE, COSET_SIZE)
            .samples(1)
            .border_width(0.0)
            .shading(Shading::Flat)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        renderer(domain, options)
    }

    fn sample_sides(renderer: &Renderer, domain: &FundamentalDomain) -> Vec<Option<Vec<usize>>> {
        let mut folds = Vec::new();
        for row in 0..COSET_SIZE {
            for column in 0..COSET_SIZE {
                let position = renderer.view.position(column, row, 0, 0);
                let Some(lift) = renderer.view.point(position) else {
                    folds.push(None);
                    continue;
                };
                let mut sides = Vec::new();
                let result = fold(domain, lift.point, renderer.options.max_folds, |side| {
                    sides.push(side);
                });
                folds.push(result.converged().then_some(sides));
            }
        }
        folds
    }

    #[test]
    fn coset_fill_gives_each_converged_sample_the_colour_of_the_sides_of_its_fold() {
        let mut cases: Vec<(FundamentalDomain, Colouring)> = COSET_GOLDEN_IMAGES
            .iter()
            .filter(|image| matches!(image.signature, "*642" | "532"))
            .filter_map(|image| {
                let domain = image.domain();
                let colouring = colouring(&domain, image.colouring?);
                Some((domain, colouring))
            })
            .collect();
        assert_eq!(cases.len(), 2);
        let square = domain("2222");
        let three = colourings_of(&square, 3);
        assert_eq!(three.len(), 12);
        cases.push((square, find_colouring(&three, "3a")));
        for (domain, colouring) in &cases {
            let id = colouring.id();
            let renderer = coset_renderer(domain, coset_fill(domain, colouring.clone()));
            let (pixels, _) = renderer.render();
            let mut converged = 0;
            for (sides, pixel) in sample_sides(&renderer, domain)
                .into_iter()
                .zip(pixels.as_chunks::<3>().0)
            {
                let Some(sides) = sides else { continue };
                converged += 1;
                let sides: Vec<u8> = sides
                    .into_iter()
                    .map(|side| u8::try_from(side).unwrap_or(u8::MAX))
                    .collect();
                let coset = colouring
                    .colour_of(&sides)
                    .unwrap_or_else(|| panic!("{id}: the sides {sides:?} have no colour"));
                let colour = GOLDEN_COSET_FILL[usize::try_from(coset).unwrap_or(usize::MAX)];
                assert_eq!(pixel, &colour, "{id}: the sides {sides:?}");
            }
            assert!(converged > 0, "{id}");
        }
    }

    #[test]
    fn renderer_refuses_a_coset_fill_of_a_domain_with_another_side_presentation() {
        let triangle = domain("*732");
        let fill = coset_fill(&triangle, colouring(&triangle, "2a"));
        let options = match RenderOptions::builder(Fill::Cosets(fill))
            .size(4, 4)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        match Renderer::new(&domain_with_aspect("*2222", None), options.clone()) {
            Ok(_) => panic!("the fill of *732 is accepted for *2222"),
            Err(error) => {
                assert_eq!(error.kind(), ErrorKind::InvalidOption);
                assert_eq!(
                    error.to_string(),
                    "cannot build renderer: the coset fill was built for a domain with another side presentation"
                );
            }
        }
        assert!(Renderer::new(&triangle, options).is_ok());
        let wide = domain_with_aspect("*2222", Some(4.0));
        let fill = coset_fill(&wide, colouring(&wide, "2a"));
        let options = match RenderOptions::builder(Fill::Cosets(fill))
            .size(4, 4)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        assert!(Renderer::new(&domain_with_aspect("*2222", None), options).is_ok());
    }
}
