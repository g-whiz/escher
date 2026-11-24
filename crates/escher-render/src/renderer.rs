use escher_domain::FundamentalDomain;
use escher_geometry::{euclidean, hyperbolic, spherical};
use escher_signature::Geometry;
use glam::{DVec2, DVec3};

use crate::colour::{Colour, CosetFill, Fill, Paint, decode_table, encode};
use crate::error::{Error, Result};
use crate::fold::{Fold, PointSize, Wall, fold_carrying};
use crate::image::{Image, Placement, sample};
use crate::mask::MaskMap;
use crate::options::{BorderUnit, RenderOptions, Shading};
use crate::view::{Lift, View};
use crate::wythoff::Wythoff;

/// Renders the tiling of a fundamental domain into 8-bit sRGB pixels, in the
/// Poincare disk or another hyperbolic model for a hyperbolic domain, in the
/// plane for a Euclidean one, and on a lit ball for a spherical one.
///
/// Make one per image with [`Renderer::new`], then render the whole image
/// with [`render`](Renderer::render) or bands of rows with
/// [`render_rows`](Renderer::render_rows). For a hyperbolic domain the disk is
/// centred in the image, with the centre of the domain at the centre of the
/// disk, or the domain is drawn in the
/// [`HyperbolicModel`](crate::HyperbolicModel) of the options with its centre
/// at the centre of the image. For a Euclidean domain the tiling fills the
/// image, with the centre of the domain at the centre of the image and 8 units
/// of the plane across the shorter side. For a spherical domain the ball has
/// the size and the place of the disk and shows the front half of the sphere,
/// seen from far away along its third axis and turned by the viewing rotation
/// of the options. With no rotation, the centre of the domain lies at the
/// centre of the ball.
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
    border: Border,
    row_bytes: usize,
    textures: Option<Textures>,
}

#[derive(Clone, Debug)]
struct Textures {
    map: MaskMap,
    placements: Vec<Option<Placement>>,
    linear: Box<[f64; 256]>,
}

impl Textures {
    fn of(fill: &CosetFill, domain: &FundamentalDomain) -> Result<Option<Self>> {
        let images = fill
            .paints()
            .iter()
            .any(|paint| matches!(paint, Paint::Image(_)));
        if !images {
            return Ok(None);
        }
        let map = MaskMap::new(domain).ok_or_else(Error::no_mask_polygon)?;
        let placements = fill
            .paints()
            .iter()
            .map(|paint| match paint {
                Paint::Solid(_) => None,
                Paint::Image(image) => {
                    Some(Placement::new(map.polygon(), image.width(), image.height()))
                }
            })
            .collect();
        Ok(Some(Self {
            map,
            placements,
            linear: Box::new(decode_table()),
        }))
    }
}

#[derive(Clone, Copy, Debug)]
enum Border {
    Pixels {
        half_width: f64,
    },
    Intrinsic {
        half_width: f64,
        distance: fn(DVec3, DVec3) -> f64,
    },
}

impl Border {
    fn of(options: &RenderOptions, geometry: Geometry) -> Self {
        let half_width = options.border_width / 2.0;
        match options.border_unit {
            BorderUnit::Pixels => Self::Pixels { half_width },
            BorderUnit::Intrinsic => Self::Intrinsic {
                half_width,
                distance: wall_distance(geometry),
            },
        }
    }
}

fn centre_corner(domain: &FundamentalDomain, position: u32) -> Result<DVec3> {
    let vertices = domain.vertices();
    if vertices.is_empty() {
        return Err(Error::no_corner());
    }
    usize::try_from(position)
        .ok()
        .and_then(|index| vertices.get(index))
        .copied()
        .ok_or_else(|| Error::corner_outside(position, vertices.len()))
}

const fn wall_distance(geometry: Geometry) -> fn(DVec3, DVec3) -> f64 {
    match geometry {
        Geometry::Hyperbolic => hyperbolic::wall_distance,
        Geometry::Euclidean => euclidean::wall_distance,
        Geometry::Spherical => spherical::wall_distance,
    }
}

impl Renderer {
    /// Returns a renderer of the tiling of a domain with the given options.
    ///
    /// The renderer chooses the view from the geometry of the domain, which is
    /// the hyperbolic model of the options for a hyperbolic domain, the plane
    /// for a Euclidean one, and the ball for a spherical one. It copies what it
    /// needs from the sides of the domain, so it keeps no reference to the
    /// domain. For a coset fill with a [`Paint::Image`](crate::Paint::Image),
    /// it also builds the flat polygon of the domain that
    /// [`Masks`](crate::Masks) draws, and the map that carries each point of
    /// the domain onto that polygon.
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
    /// is accepted. Returns the same kind for a coset fill with an image paint
    /// when the domain has no flat polygon, as
    /// [`Masks::new`](crate::Masks::new) states, and for a
    /// [`Fill::Wythoff`] or a [`Fill::WholeFace`] whose construction was
    /// built for a domain with other walls than this one. Every other fill
    /// gives no error.
    ///
    /// Before it checks the fill, it returns the same kind when the options
    /// name a centre corner at or above the number of vertices of the domain,
    /// and for every centre corner on a domain without a vertex.
    pub fn new(domain: &FundamentalDomain, options: RenderOptions) -> Result<Self> {
        let corner = options
            .centre
            .map(|position| centre_corner(domain, position))
            .transpose()?;
        if let Fill::Cosets(fill) = &options.fill
            && domain.side_presentation().ok().as_ref() != Some(fill.presentation())
        {
            return Err(Error::other_presentation());
        }
        if let Fill::Wythoff(fill) = &options.fill
            && !fill.wythoff().fits(domain)
        {
            return Err(Error::other_domain());
        }
        if let Fill::WholeFace(fill) = &options.fill
            && !fill.wythoff().fits(domain)
        {
            return Err(Error::other_whole_face_domain());
        }
        let textures = match &options.fill {
            Fill::Cosets(fill) => Textures::of(fill, domain)?,
            Fill::One(_) | Fill::Parity(_) | Fill::Wythoff(_) | Fill::WholeFace(_) => None,
        };
        let walls = domain.sides().iter().map(|&side| Wall::new(side)).collect();
        let renderer = Self::with_walls(walls, domain.geometry(), options);
        Ok(Self {
            textures,
            view: match corner {
                None => renderer.view,
                Some(corner) => renderer.view.centred(corner),
            },
            ..renderer
        })
    }

    fn with_walls(walls: Vec<Wall>, geometry: Geometry, options: RenderOptions) -> Self {
        Self {
            walls,
            size: PointSize::of(geometry),
            view: View::canonical(&options, geometry),
            border: Border::of(&options, geometry),
            row_bytes: length(3 * u64::from(options.width)),
            options,
            textures: None,
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
            Fill::Cosets(_) | Fill::WholeFace(_) => {
                sides.clear();
                self.fold(&lift, |side| sides.push(side_byte(side)))
            }
            Fill::One(_) | Fill::Parity(_) | Fill::Wythoff(_) => self.fold(&lift, |_| {}),
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
        match &self.options.fill {
            Fill::Wythoff(fill) => match self.face(fill.wythoff(), result.point(), tangents) {
                Some(face) => fill.colour(face),
                None => self.options.border_colour,
            },
            Fill::WholeFace(fill) => match self.face(fill.wythoff(), result.point(), tangents) {
                Some(face) => fill.value(face, sides, result.point(), self.options.background),
                None => self.options.border_colour,
            },
            Fill::One(_) | Fill::Parity(_) | Fill::Cosets(_)
                if self.side_border(result.point(), tangents) =>
            {
                self.options.border_colour
            }
            Fill::One(colour) => *colour,
            Fill::Parity([even, odd]) => {
                if result.reversals().is_multiple_of(2) {
                    *even
                } else {
                    *odd
                }
            }
            Fill::Cosets(fill) => match fill.paint(sides) {
                (_, Paint::Solid(colour)) => *colour,
                (coset, Paint::Image(image)) => self.texture(coset, image, result.point()),
            },
        }
    }

    fn side_border(&self, point: DVec3, tangents: [DVec3; 2]) -> bool {
        self.border(self.walls.iter().map(Wall::covector), point, tangents)
    }

    fn face(&self, wythoff: &Wythoff, point: DVec3, tangents: [DVec3; 2]) -> Option<usize> {
        let face = wythoff.face_of(point);
        (!self.border(wythoff.perpendiculars_of(face), point, tangents)).then_some(face)
    }

    #[expect(
        clippy::expect_used,
        reason = "the renderer builds a placement for each image paint of its coset fill, at the index of that paint"
    )]
    pub(crate) fn texture(&self, coset: usize, image: &Image, point: DVec3) -> Colour {
        let textures = self
            .textures
            .as_ref()
            .expect("internal error: a renderer with an image paint has textures");
        let placement = textures
            .placements
            .get(coset)
            .copied()
            .flatten()
            .expect("internal error: every image paint of the fill has a placement");
        let background = self.options.background;
        textures
            .map
            .map(point)
            .and_then(|flat| placement.inside(placement.position(flat)))
            .map_or(background, |position| {
                Colour::from_linear(sample(
                    image,
                    position,
                    background.linear(),
                    &textures.linear,
                ))
            })
    }

    fn border(
        &self,
        covectors: impl IntoIterator<Item = DVec3>,
        point: DVec3,
        tangents: [DVec3; 2],
    ) -> bool {
        match self.border {
            Border::Pixels { half_width } => on_border(covectors, point, tangents, half_width),
            Border::Intrinsic {
                half_width,
                distance,
            } => on_intrinsic_border(covectors, point, half_width, distance),
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
fn on_border(
    covectors: impl IntoIterator<Item = DVec3>,
    point: DVec3,
    tangents: [DVec3; 2],
    half_width: f64,
) -> bool {
    covectors.into_iter().any(|covector| {
        let value = covector.dot(point);
        let [first, second] = tangents.map(|tangent| covector.dot(tangent));
        let gradient = (first * first + second * second).sqrt();
        !(value.abs() >= half_width * gradient)
    })
}

#[expect(
    clippy::neg_cmp_op_on_partial_ord,
    reason = "the negated comparison is true when a distance is NaN, which must give a border sample"
)]
fn on_intrinsic_border(
    covectors: impl IntoIterator<Item = DVec3>,
    point: DVec3,
    half_width: f64,
    distance: fn(DVec3, DVec3) -> f64,
) -> bool {
    covectors
        .into_iter()
        .any(|covector| !(distance(covector, point) >= half_width))
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
    /// For a hyperbolic domain these are the samples outside the region of its
    /// model, such as those outside the disk, and those whose point cannot be
    /// computed, and for a spherical domain those on or outside the rim of the
    /// ball. It is 0 for a Euclidean domain, whose view covers the whole
    /// image.
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

    pub(crate) fn view(&self) -> &View {
        &self.view
    }
}

#[cfg(test)]
mod tests {
    use escher_domain::{FundamentalDomain, Orientation, PairingKind};
    use escher_geometry::hyperbolic::{covector, from_disk, reflection, wall_through};
    use escher_geometry::{euclidean, hyperbolic, spherical};
    use escher_signature::{Geometry, Signature};
    use glam::{DMat3, DVec2, DVec3};

    use super::{AMBIENT, DIFFUSE, RenderStats, Renderer, on_border, on_intrinsic_border, shade};
    use escher_group::Colouring;

    use crate::ErrorKind;
    use crate::colour::{Colour, CosetFill, Fill, Paint};
    use crate::fold::{PointSize, Wall, fold, fold_carrying};
    use crate::golden_images::{
        ALLOWED_PIXELS, COSET_GOLDEN_IMAGES, EUCLIDEAN_DOMAINS, FOLD_ANGLES, GOLDEN_COSET_FILL,
        GOLDEN_GRID, GOLDEN_IMAGES, GOLDEN_SIZE, IMAGE_FILL_GOLDEN_IMAGES, MODEL_MAPS,
        POLYGON_DOMAINS, REGULAR, WALLPAPER_DOMAINS, arrangements, band_radius, colouring,
        colourings_of, compare, coset_fill, domain_with_aspect, face_picture, find_colouring,
        golden_colouring, image_paints, linear_table, on_circle, reference_walls, renderer,
        whole_face, whole_face_fill,
    };
    use crate::image::sample;
    use crate::options::{BorderUnit, HyperbolicModel, RenderOptions, Shading};
    use crate::reference::{self, ReferenceExt, count_bound, plane_border_distance};
    use crate::view::View;
    use escher_reference::{Reference, Worst, error, int, num, sub, vector};

    const FOLD_TRIPLES: [&str; 7] = [
        "*732",
        "*542",
        "*444",
        "*433",
        "*(12)32",
        "*(100)32",
        "*(100)(100)(100)",
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

    fn side_covectors(walls: &[Wall]) -> impl Iterator<Item = DVec3> + '_ {
        walls.iter().map(Wall::covector)
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
        assert!(!on_border(side_covectors(&walls), inside, tangents, 0.0));
        assert!(on_border(
            side_covectors(&walls),
            DVec3::new(f64::NAN, 0.0, 1.0),
            tangents,
            0.0
        ));
        assert!(on_border(
            side_covectors(&walls),
            inside,
            [DVec3::splat(f64::NAN), DVec3::Y],
            0.0
        ));
        assert!(on_border(
            side_covectors(&walls),
            inside,
            [DVec3::splat(f64::INFINITY), DVec3::Y],
            0.0
        ));
    }

    #[test]
    fn intrinsic_border_marks_a_nan_wall_distance_and_not_a_distance_of_exactly_half_the_width() {
        let walls = [Wall::from_parts(
            DVec3::X,
            DMat3::IDENTITY,
            Orientation::Reversing,
        )];
        let point = DVec3::new(0.25, 0.0, 1.0);
        assert!(!on_intrinsic_border(
            side_covectors(&walls),
            point,
            0.25,
            euclidean::wall_distance
        ));
        assert!(on_intrinsic_border(
            side_covectors(&walls),
            point,
            0.25_f64.next_up(),
            euclidean::wall_distance
        ));
        let distances: [fn(DVec3, DVec3) -> f64; 3] = [
            hyperbolic::wall_distance,
            euclidean::wall_distance,
            spherical::wall_distance,
        ];
        for distance in distances {
            assert!(on_intrinsic_border(
                side_covectors(&walls),
                DVec3::new(f64::NAN, 0.0, 1.0),
                0.0,
                distance
            ));
        }
    }

    #[test]
    fn intrinsic_border_of_the_imaginary_axis_is_the_wedge_of_its_equidistant_curves() {
        const SIZE: u32 = 192;
        const HALF_WIDTH: f64 = 0.1;
        const RANGE: f64 = 1e-6;
        const TOLERANCE: f64 = 1e-10;
        let Some(map) = MODEL_MAPS
            .iter()
            .find(|map| map.model == HyperbolicModel::HalfPlane)
        else {
            panic!("the model maps hold no half-plane");
        };
        let point_of = |position: DVec2| {
            (map.to_disk)(position)
                .and_then(|(disk, _)| from_disk(disk))
                .unwrap_or_else(|| panic!("the half-plane position {position:?} has no point"))
        };
        let normal = match wall_through(point_of(DVec2::Y), point_of(2.0 * DVec2::Y)) {
            Ok(normal) => normal,
            Err(error) => panic!("the wall through i and 2i is rejected: {error}"),
        };
        let walls = [Wall::from_parts(
            covector(normal),
            reflection(normal),
            Orientation::Reversing,
        )];
        let options = match RenderOptions::builder(Fill::One(Colour::WHITE))
            .size(SIZE, SIZE)
            .samples(1)
            .model(HyperbolicModel::HalfPlane)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        let view = View::canonical(&options, Geometry::Hyperbolic);
        let threshold = HALF_WIDTH.sinh();
        let (mut compared, mut marked, mut skipped, mut beyond) = (0, 0, 0, 0);
        let mut largest = (0.0_f64, DVec2::ZERO);
        for row in 0..SIZE {
            for column in 0..SIZE {
                let position = view.position(column, row, 0, 0);
                let model_position = position + map.offset;
                let Some((disk, _)) = (map.to_disk)(model_position) else {
                    panic!("the half-plane position {model_position:?} has no disk position");
                };
                if disk.length() > 1.0 - RANGE {
                    beyond += 1;
                    continue;
                }
                let Some(lift) = view.point(position) else {
                    panic!("the half-plane position {model_position:?} has no point");
                };
                let ratio = model_position.x.abs() / model_position.y;
                let sine = hyperbolic::wall_distance(walls[0].covector(), lift.point).sinh();
                let relative = (sine - ratio).abs() / ratio.max(threshold);
                if relative > largest.0 {
                    largest = (relative, model_position);
                }
                if (ratio - threshold).abs() < TOLERANCE * threshold {
                    skipped += 1;
                    continue;
                }
                let border = on_intrinsic_border(
                    side_covectors(&walls),
                    lift.point,
                    HALF_WIDTH,
                    hyperbolic::wall_distance,
                );
                assert_eq!(
                    border,
                    ratio < threshold,
                    "the half-plane position {model_position:?}, tan |theta| {ratio}, sinh {HALF_WIDTH} {threshold}"
                );
                compared += 1;
                marked += usize::from(border);
            }
        }
        println!(
            "wedge of half width {HALF_WIDTH}: {compared} samples compared, {marked} in the wedge, {skipped} within the tolerance, {beyond} beyond the range; largest relative error of sinh of the distance {:.3e} at {:?}, tolerance {TOLERANCE:e}, margin {:.1}",
            largest.0,
            largest.1,
            TOLERANCE / largest.0
        );
        assert!(8.0 * largest.0 <= TOLERANCE, "relative error {largest:?}");
        assert!(marked > 0 && marked < compared, "{marked} of {compared}");
    }

    #[test]
    fn intrinsic_border_takes_the_same_decision_for_a_point_in_every_model() {
        const WIDTH: f64 = 0.05;
        const CLEARANCE: f64 = 1e-9;
        let domain = domain("*732");
        let walls = reference_walls(&domain);
        let renderer_in = |model: HyperbolicModel| {
            let options = match RenderOptions::builder(Fill::One(Colour::WHITE))
                .border_unit(BorderUnit::Intrinsic)
                .border_width(WIDTH)
                .model(model)
                .build()
            {
                Ok(options) => options,
                Err(error) => panic!("{error}"),
            };
            renderer(&domain, options)
        };
        let decision = |renderer: &Renderer, position: DVec2| {
            let Some(lift) = renderer.view.point(position) else {
                panic!("the screen position {position:?} has no point");
            };
            let (result, tangents) = renderer.fold(&lift, |_| {});
            assert!(
                result.converged(),
                "the screen position {position:?} does not converge"
            );
            renderer.side_border(result.point(), tangents)
        };
        let disk = renderer_in(HyperbolicModel::Disk);
        let models: Vec<_> = MODEL_MAPS
            .iter()
            .map(|map| (map, renderer_in(map.model)))
            .collect();
        let mut reference = Reference::new();
        let clearance = num(CLEARANCE);
        let half_width = num(WIDTH / 2.0);
        let (mut compared, mut marked, mut near) = (0, 0, 0);
        for band in 1..=3 {
            for index in 0..FOLD_ANGLES {
                let position = on_circle(band_radius(band), index, FOLD_ANGLES);
                let name = format!("*732 in band {band} at angle {index} of {FOLD_ANGLES}");
                let folded =
                    reference::fold_exact(&walls, reference::from_disk(position.to_array()), 1500);
                assert!(
                    folded.converged,
                    "{name}: the reference fold does not converge"
                );
                let distances: Vec<_> = walls
                    .iter()
                    .map(|wall| reference.wall_distance(&wall.covector, &folded.point))
                    .collect();
                if distances
                    .iter()
                    .any(|distance| sub(distance, &half_width).abs() < clearance)
                {
                    near += 1;
                    continue;
                }
                let expected = distances.iter().any(|distance| *distance < half_width);
                assert_eq!(decision(&disk, position), expected, "{name} in the disk");
                for (map, renderer) in &models {
                    let model_position =
                        DVec2::from_array(reference.disk_to_model(map.exact, position.to_array()));
                    assert_eq!(
                        decision(renderer, model_position - map.offset),
                        expected,
                        "{name} in the {:?} model at {model_position:?}",
                        map.model
                    );
                }
                compared += 1;
                marked += usize::from(expected);
            }
        }
        println!(
            "*732 with the intrinsic width {WIDTH}: {compared} disk positions give one decision in every model, {marked} of them border samples, {near} within {CLEARANCE:e} of half the width"
        );
        assert!(marked > 0 && marked < compared, "{marked} of {compared}");
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
        const PERTURBED: [&str; 7] = [
            "7-3-2.png",
            "rot-7-3-2.png",
            "6-3-2.png",
            "wp-x-x.png",
            "5-3-2-rotated.png",
            "rot-5-3-2.png",
            "sg-3-s-2.png",
        ];
        let linear = linear_table();
        let images: Vec<_> = GOLDEN_IMAGES
            .iter()
            .filter(|image| PERTURBED.contains(&image.file))
            .collect();
        assert_eq!(images.len(), PERTURBED.len());
        for image in images {
            let text = image.signature;
            let domain = image.domain();
            let options = image.options(&domain);
            let (pixels, _) = renderer(&domain, options.clone()).render();
            let geometry = domain.geometry();
            let walls = domain
                .sides()
                .iter()
                .map(|side| match geometry {
                    _ if side.kind() != PairingKind::Mirror => {
                        let pairing = side.pairing();
                        let moved = DMat3::from_cols(
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
                    let moved_viewing = DMat3::from_cols(
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
        const MEASURED: [[usize; 2]; 6] = [[0, 3], [10, 0], [0, 0], [0, 0], [0, 0], [0, 0]];
        let domains = FOLD_TRIPLES.map(|text| (text.to_owned(), domain(text)));
        assert_border_forms(&domains, &BorderBounds::Summed(MEASURED), 2);
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
            second_band([0, 0], [6, 18]),
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
        assert_border_forms(&domains, &BorderBounds::PerDomain(&MEASURED), 5);
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

    fn assert_border_forms(
        domains: &[(String, FundamentalDomain)],
        measured: &BorderBounds<'_>,
        stride: usize,
    ) {
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
            let radius = band_radius(band);
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
                for index in (0..FOLD_ANGLES).step_by(stride) {
                    let position = on_circle(radius, index, FOLD_ANGLES);
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
                        if on_border(side_covectors(&walls), result.point(), tangents, half_width)
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

    const PLANE_ANGLES: u32 = 512;

    const PLANE_NEAR: f64 = 4.0;

    const PLANE_HALF_WIDTHS: [f64; 2] = [0.5, 1.5];

    const PLANE_DISTANCES: [f64; 4] = [1.0, 4.0, 16.0, 65.0];

    const PLANE_MINIMUM: usize = 1000;

    #[derive(Default)]
    struct PlaneBorders {
        near: usize,
        differing: [usize; 2],
        largest: f64,
        worst: String,
    }

    impl PlaneBorders {
        fn add(&mut self, other: Self) {
            self.near += other.near;
            for (total, count) in self.differing.iter_mut().zip(other.differing) {
                *total += count;
            }
            if other.largest > self.largest {
                self.largest = other.largest;
                self.worst = other.worst;
            }
        }
    }

    fn plane_view() -> View {
        let options = match RenderOptions::builder(Fill::One(Colour::WHITE)).build() {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        View::canonical(&options, Geometry::Euclidean)
    }

    fn plane_border_forms(
        reference: &mut Reference,
        view: &View,
        text: &str,
        domain: &FundamentalDomain,
        radius: f64,
    ) -> PlaneBorders {
        let pixel = view.pixel();
        let mut borders = PlaneBorders::default();
        let walls: Vec<Wall> = domain.sides().iter().map(|&side| Wall::new(side)).collect();
        let covectors: Vec<_> = walls
            .iter()
            .map(|wall| vector(wall.covector().to_array()))
            .collect();
        for index in 0..PLANE_ANGLES {
            let angle = core::f64::consts::TAU * f64::from(index) / f64::from(PLANE_ANGLES);
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
            if exact >= PLANE_NEAR {
                continue;
            }
            borders.near += 1;
            let gradient = gradient_form(&walls, result.point(), tangents);
            let difference = (gradient - exact).abs();
            if difference > borders.largest {
                borders.largest = difference;
                borders.worst = format!(
                    "{text} at angle {index}, gradient form {gradient:.17e}, distance form {exact:.17e}"
                );
            }
            for (count, half_width) in borders.differing.iter_mut().zip(PLANE_HALF_WIDTHS) {
                if on_border(side_covectors(&walls), result.point(), tangents, half_width)
                    != (exact < half_width)
                {
                    *count += 1;
                }
            }
        }
        borders
    }

    #[test]
    fn gradient_form_of_the_border_agrees_with_the_euclidean_distance_form() {
        const MEASURED: [[usize; 2]; 4] = [[0, 0], [0, 0], [0, 0], [0, 0]];
        let mut reference = Reference::new();
        let view = plane_view();
        for (radius, measured) in PLANE_DISTANCES.into_iter().zip(MEASURED) {
            let bounds = measured.map(count_bound);
            let mut borders = PlaneBorders::default();
            for (text, aspect) in EUCLIDEAN_DOMAINS {
                let domain = domain_with_aspect(text, aspect);
                let name = format!("{text} aspect {aspect:?}");
                borders.add(plane_border_forms(
                    &mut reference,
                    &view,
                    &name,
                    &domain,
                    radius,
                ));
            }
            let PlaneBorders {
                near,
                differing,
                largest,
                worst,
            } = borders;
            println!(
                "distance {radius}: {near} samples within {PLANE_NEAR} pixels of a wall, {} and {} classified differently at half widths {} and {}, bounds {} and {}, largest absolute difference {largest:.3e} pixels at {worst}",
                differing[0],
                differing[1],
                PLANE_HALF_WIDTHS[0],
                PLANE_HALF_WIDTHS[1],
                bounds[0],
                bounds[1]
            );
            assert!(
                near >= PLANE_MINIMUM,
                "distance {radius}: only {near} samples near a wall"
            );
            assert!(
                differing[0] <= bounds[0] && differing[1] <= bounds[1],
                "distance {radius}: {differing:?}"
            );
        }
    }

    #[test]
    fn gradient_form_of_the_border_agrees_with_the_euclidean_distance_form_on_the_wallpaper_domains()
     {
        const MEASURED: [usize; 2] = [0, 0];
        let bounds = MEASURED.map(count_bound);
        let mut reference = Reference::new();
        let view = plane_view();
        let domains: Vec<(String, FundamentalDomain)> = WALLPAPER_DOMAINS
            .iter()
            .map(|shape| (shape.name(), shape.domain()))
            .collect();
        for radius in PLANE_DISTANCES {
            let mut total = PlaneBorders::default();
            for (name, domain) in &domains {
                let borders = plane_border_forms(&mut reference, &view, name, domain, radius);
                println!(
                    "distance {radius}, {name}: {} samples within {PLANE_NEAR} pixels of a wall, {} and {} classified differently at half widths {} and {}, bounds {} and {}, largest absolute difference {:.3e} pixels at {}",
                    borders.near,
                    borders.differing[0],
                    borders.differing[1],
                    PLANE_HALF_WIDTHS[0],
                    PLANE_HALF_WIDTHS[1],
                    bounds[0],
                    bounds[1],
                    borders.largest,
                    borders.worst
                );
                assert!(
                    borders.near > 0,
                    "distance {radius}, {name}: no sample near a wall"
                );
                assert!(
                    borders.differing[0] <= bounds[0] && borders.differing[1] <= bounds[1],
                    "distance {radius}, {name}: {:?}, bounds {bounds:?}",
                    borders.differing
                );
                total.add(borders);
            }
            println!(
                "distance {radius}: {} samples of the wallpaper domains within {PLANE_NEAR} pixels of a wall, {} and {} classified differently, largest absolute difference {:.3e} pixels at {}",
                total.near, total.differing[0], total.differing[1], total.largest, total.worst
            );
            assert!(
                total.near >= PLANE_MINIMUM,
                "distance {radius}: only {} samples near a wall",
                total.near
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
                    worst.record(&error(shaded, &exact), &int(1), || {
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

    fn tile(walls: &[Wall], pairings: &[DMat3], point: DVec3, centre: DVec3) -> Option<DVec3> {
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
                        let gradient =
                            on_border(side_covectors(walls), result.point(), tangents, half_width);
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
                let colouring = golden_colouring(&domain, image.colouring?);
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

    #[test]
    fn image_fill_golden_configurations_place_each_image_paint_and_no_solid_paint() {
        for image in &IMAGE_FILL_GOLDEN_IMAGES {
            let domain = image.golden.domain();
            let renderer = renderer(&domain, image.options(&domain));
            let Some(textures) = renderer.textures.as_ref() else {
                panic!("{} has no texture", image.golden.file);
            };
            let solid: Vec<usize> = (0..textures.placements.len())
                .filter(|&index| textures.placements[index].is_none())
                .collect();
            assert_eq!(solid, image.solid, "{}", image.golden.file);
            assert!(
                textures.placements.len() > image.solid.len(),
                "{}",
                image.golden.file
            );
        }
        let domain = domain("*732");
        let solid = coset_renderer(&domain, coset_fill(&domain, colouring(&domain, "2a")));
        assert!(solid.textures.is_none());
    }

    #[test]
    fn image_fill_gives_the_background_where_the_mask_map_or_the_image_has_no_value() {
        let domain = domain("*732");
        let colouring = golden_colouring(&domain, "2a");
        let paints = image_paints(&domain, &colouring, 64, &[]);
        let Paint::Image(image) = paints[0].clone() else {
            panic!("the first paint is solid");
        };
        let fill = match CosetFill::new(&domain, colouring, paints) {
            Ok(fill) => fill,
            Err(error) => panic!("{error}"),
        };
        let background = Colour::from_srgb8(10, 200, 30);
        let options = match RenderOptions::builder(Fill::Cosets(fill))
            .background(background)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("{error}"),
        };
        let renderer = renderer(&domain, options);
        for point in [
            DVec3::ZERO,
            DVec3::new(f64::NAN, 0.0, 1.0),
            DVec3::new(0.0, f64::INFINITY, 1.0),
            DVec3::new(0.0, 0.0, f64::NEG_INFINITY),
            DVec3::new(0.0, 0.0, -1.0),
        ] {
            assert_eq!(renderer.texture(0, &image, point), background, "{point}");
        }
        let Some(textures) = renderer.textures.as_ref() else {
            panic!("the renderer has no texture");
        };
        let placement = textures.placements[0].unwrap_or_else(|| panic!("no placement"));
        let far = hyperbolic::normalize_point(DVec3::new(0.0, -30.0, 31.0))
            .unwrap_or_else(|error| panic!("{error}"));
        let Some(flat) = textures.map.map(far) else {
            panic!("{far} has no mask point");
        };
        assert_eq!(placement.inside(placement.position(flat)), None, "{flat}");
        assert_eq!(renderer.texture(0, &image, far), background, "{far}");
        let centre = hyperbolic::normalize_point(domain.vertices().iter().sum())
            .unwrap_or_else(|error| panic!("{error}"));
        let value = renderer.texture(0, &image, centre);
        let polygon = textures.map.polygon();
        let flat_centre = polygon.iter().sum::<DVec2>() / 3.0;
        assert_eq!(textures.map.map(centre), Some(flat_centre));
        let expected = sample(
            &image,
            placement.position(flat_centre),
            background.linear(),
            &textures.linear,
        );
        assert_eq!(value.linear(), expected);
        let centre = value;
        assert_ne!(centre, background);
    }

    fn same_value_in_every_model(
        name: &str,
        domain: &FundamentalDomain,
        fill: &Fill,
        sample_value: impl Fn(&Renderer, &[u8], DVec3) -> [f64; 3],
    ) {
        const CLEARANCE: f64 = 1e-9;
        const TOLERANCE: f64 = 1.1e-6;
        let walls = reference_walls(domain);
        let renderer_in = |model: HyperbolicModel| {
            let options = match RenderOptions::builder(fill.clone()).model(model).build() {
                Ok(options) => options,
                Err(error) => panic!("{error}"),
            };
            renderer(domain, options)
        };
        let value = |renderer: &Renderer, position: DVec2| {
            let Some(lift) = renderer.view.point(position) else {
                panic!("the screen position {position:?} has no point");
            };
            let mut sides = Vec::new();
            let (result, _) = renderer.fold(&lift, |side| {
                sides.push(u8::try_from(side).unwrap_or(u8::MAX));
            });
            assert!(
                result.converged(),
                "the screen position {position:?} does not converge"
            );
            let found = sample_value(renderer, &sides, result.point());
            (sides, found)
        };
        let disk = renderer_in(HyperbolicModel::Disk);
        let models: Vec<_> = MODEL_MAPS
            .iter()
            .map(|map| (map, renderer_in(map.model)))
            .collect();
        let mut reference = Reference::new();
        let clearance = num(CLEARANCE);
        let (mut compared, mut near, mut largest) = (0, 0, 0.0_f64);
        let mut values = Vec::new();
        for band in 1..=3 {
            for index in 0..FOLD_ANGLES {
                let position = on_circle(band_radius(band), index, FOLD_ANGLES);
                let name = format!("{name} in band {band} at angle {index} of {FOLD_ANGLES}");
                let folded =
                    reference::fold_exact(&walls, reference::from_disk(position.to_array()), 1500);
                assert!(
                    folded.converged,
                    "{name}: the reference fold does not converge"
                );
                if walls
                    .iter()
                    .any(|wall| reference.wall_distance(&wall.covector, &folded.point) < clearance)
                {
                    near += 1;
                    continue;
                }
                let (sides, expected) = value(&disk, position);
                for (map, renderer) in &models {
                    let model_position =
                        DVec2::from_array(reference.disk_to_model(map.exact, position.to_array()));
                    let (other_sides, other) = value(renderer, model_position - map.offset);
                    assert_eq!(other_sides, sides, "{name} in the {:?} model", map.model);
                    for (left, right) in expected.iter().zip(other) {
                        largest = largest.max((left - right).abs());
                    }
                }
                values.push(expected);
                compared += 1;
            }
        }
        values.sort_by(|left, right| {
            left.partial_cmp(right)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        values.dedup();
        println!(
            "{name}: {compared} disk positions take values within {largest:e} of each other in every model, allowed {TOLERANCE:e}, {} distinct values, {near} within {CLEARANCE:e} of a wall",
            values.len()
        );
        assert!(largest <= TOLERANCE, "{largest:e}");
        assert!(
            compared >= 1000 && values.len() > 10,
            "{compared} {}",
            values.len()
        );
    }

    #[test]
    fn image_fill_takes_the_same_value_for_a_point_in_every_model() {
        const IMAGE_SIZE: u32 = 192;
        let domain = domain("*732");
        let colouring = golden_colouring(&domain, "2a");
        let paints = image_paints(&domain, &colouring, IMAGE_SIZE, &[]);
        let fill = match CosetFill::new(&domain, colouring, paints) {
            Ok(fill) => fill,
            Err(error) => panic!("{error}"),
        };
        same_value_in_every_model(
            "*732 with image fills",
            &domain,
            &Fill::Cosets(fill),
            |renderer, sides, point| {
                let Fill::Cosets(fill) = &renderer.options.fill else {
                    panic!("the fill is not a coset fill");
                };
                match fill.paint(sides) {
                    (coset, Paint::Image(image)) => renderer.texture(coset, image, point).linear(),
                    (_, Paint::Solid(_)) => panic!("every paint is an image"),
                }
            },
        );
    }

    #[test]
    fn whole_face_fill_takes_the_same_value_for_a_point_in_every_model() {
        let domain = domain("*732");
        let face = whole_face(&domain, REGULAR);
        let arrangement = find_colouring(&arrangements(&face), "14a");
        let (_, image) = face_picture(&face, &arrangement, 0, GOLDEN_SIZE);
        let fill = whole_face_fill(face, arrangement, vec![Paint::Image(image)]);
        same_value_in_every_model(
            "*732 with a whole-face fill of 14a",
            &domain,
            &Fill::WholeFace(fill),
            |renderer, sides, point| {
                let Fill::WholeFace(fill) = &renderer.options.fill else {
                    panic!("the fill is not a whole-face fill");
                };
                fill.value(0, sides, point, renderer.options.background)
                    .linear()
            },
        );
    }
}
