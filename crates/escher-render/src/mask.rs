use core::f64::consts::PI;
use std::collections::VecDeque;

use escher_domain::{FundamentalDomain, PairingKind};
use escher_geometry::{euclidean, hyperbolic, spherical};
use escher_group::{Colouring, Letter};
use escher_signature::Geometry;
use glam::{DVec2, DVec3};

use crate::colour::{Colour, checked_presentation, encode};
use crate::error::{Error, MASKS, RENDER_MASK, Result};
use crate::image::Placement;
use crate::options::check_size;

pub(crate) const MASK_GRID: u32 = 4;

const BORDER_FRACTION: f64 = 1.0 / 64.0;

const MARK_FRACTION: f64 = 0.25;

const TICK_LENGTH: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tick {
    Absent,
    OnEdgeZero,
}

/// The polygon masks of a colouring of a fundamental domain, one for each
/// colour.
///
/// Every mask shows the same flat polygon with straight edges, which stands
/// for the domain: one vertex for each vertex of the domain and one edge for
/// each side, in the same order. The polygon is white on black, centred and
/// fitted in the image with a margin of 1/32 of its shorter side, and keeps
/// its proportions at every size. A
/// [`Paint::Image`](crate::Paint::Image) reads its image where the polygon
/// lies in a mask of the size of the image, so an image painted over the
/// polygon of a mask fills every tile of that colour.
///
/// Inside each edge the mask of a colour draws a border of 1/64 of the
/// shorter side of the image, in a code colour. Edge `j` of colour `a` meets
/// edge `p` of colour `b` when a tile of colour `a` has a tile of colour `b`
/// across its side `j`, with `p` the partner of `j`. Edges that meet, and all
/// the edges joined to them by further meetings, share one code colour in the
/// masks of every colour, so the artwork along all of them must agree. The
/// quarter of an edge at its marked end is black, and wherever two edges
/// meet, their marked ends meet. When the edges of one code colour cannot be
/// marked at one end each that way, each of them is marked at both ends, and
/// the artwork along them must match itself reversed.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_group::{ColouringSet, IndexLimit, SearchBudget, colourings_of_index};
/// use escher_render::{Masks, encode_png};
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*642".parse::<Signature>()?)?;
/// let presentation = domain.side_presentation()?;
/// let found = colourings_of_index(&presentation, ColouringSet::ValidClasses, IndexLimit::new(4)?, SearchBudget::DEFAULT)?;
/// let colouring = found.find("4a".parse()?).ok_or("4a is missing")?;
/// let masks = Masks::new(&domain, colouring)?;
/// assert_eq!(masks.count(), 4);
/// let pixels = masks.render(1, 32, 32)?;
/// let file = encode_png(32, 32, &pixels)?;
/// assert!(file.starts_with(b"\x89PNG"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Masks {
    polygon: Vec<DVec2>,
    colours: usize,
    edges: Vec<EdgeStyle>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EdgeStyle {
    pub(crate) class: usize,
    pub(crate) code: [u8; 3],
    pub(crate) marks: [bool; 2],
}

impl Masks {
    /// Returns the masks of a colouring of a domain.
    ///
    /// It accepts the colouring under the same conditions as
    /// [`CosetFill::new`](crate::CosetFill::new): the domain has a side
    /// presentation, the table of the colouring has one generator for each
    /// side, and tracing each relator from each coset returns to that coset.
    /// It then builds the flat polygon of the domain, and finds from the table
    /// and the pairings of the sides which edges meet, their code colours, and
    /// their marked ends. It keeps no reference to the domain.
    ///
    /// The flat polygon of a Euclidean domain is the domain itself. A
    /// hyperbolic or spherical triangle of mirrors becomes a flat triangle
    /// that keeps a right angle where it has one, and the angle of the next
    /// smallest corner order. A hyperbolic polygon of mirrors with four or
    /// more sides, and its rotation domain, becomes its picture in the Klein
    /// model, centred on the domain. A rotation domain built from a triangle
    /// becomes the flat triangle of that triangle together with its mirror
    /// image. A domain with fewer than three sides, and a domain whose flat
    /// triangle would have a corner of zero, such as `*222`, has no polygon.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidOption`](crate::ErrorKind::InvalidOption), for the
    /// first of these causes: the domain has no side presentation, the number
    /// of generators of the colouring differs from the number of sides of the
    /// domain, a relator of the side presentation does not return some coset
    /// of the colouring to itself, or the domain has no flat polygon.
    pub fn new(domain: &FundamentalDomain, colouring: &Colouring) -> Result<Self> {
        checked_presentation(domain, colouring).map_err(|error| error.during(MASKS))?;
        let polygon = MaskMap::new(domain)
            .ok_or_else(|| Error::no_mask_polygon().during(MASKS))?
            .polygon;
        Ok(Self {
            polygon,
            colours: colouring.table().index(),
            edges: edge_styles(domain, colouring),
        })
    }

    /// Returns the number of masks, which is the number of colours of the
    /// colouring.
    #[must_use]
    pub fn count(&self) -> u32 {
        u32::try_from(self.colours).unwrap_or(u32::MAX)
    }

    /// Returns the mask of a colour, counted from 0, at the given size.
    ///
    /// The buffer holds 8-bit sRGB, 3 bytes per pixel, row by row from the
    /// top, in the layout of [`Renderer::render`](crate::Renderer::render), so
    /// [`encode_png`](crate::encode_png) encodes it. Each pixel is the mean, in
    /// linear light, of a 4 by 4 grid of samples, so the edges and the borders
    /// are antialiased. The mask reads no render option and starts no thread.
    ///
    /// # Errors
    ///
    /// Returns an error of kind
    /// [`ErrorKind::InvalidOption`](crate::ErrorKind::InvalidOption) for a
    /// colour at or above [`count`](Masks::count), and the errors of
    /// [`RenderOptionsBuilder::size`](crate::RenderOptionsBuilder::size) for
    /// the size, with their kinds.
    pub fn render(&self, colour: u32, width: u32, height: u32) -> Result<Vec<u8>> {
        let index = usize::try_from(colour)
            .ok()
            .filter(|&index| index < self.colours)
            .ok_or_else(|| Error::mask_colour(colour, self.count()))?;
        check_size(width, height).map_err(|error| error.during(RENDER_MASK))?;
        Ok(self.mask(index, width, height, MASK_GRID))
    }

    fn edge_styles_of(&self, colour: usize) -> &[EdgeStyle] {
        let sides = self.polygon.len();
        self.edges
            .get(colour * sides..(colour + 1) * sides)
            .unwrap_or_default()
    }

    fn mask(&self, colour: usize, width: u32, height: u32, grid: u32) -> Vec<u8> {
        draw_mask(
            &self.polygon,
            Placement::new(&self.polygon, width, height),
            self.edge_styles_of(colour),
            Tick::Absent,
            width,
            height,
            grid,
        )
    }
}

pub(crate) fn draw_mask(
    polygon: &[DVec2],
    placement: Placement,
    styles: &[EdgeStyle],
    tick: Tick,
    width: u32,
    height: u32,
    grid: u32,
) -> Vec<u8> {
    let edges: Vec<Edge> = styles
        .iter()
        .enumerate()
        .map(|(side, style)| Edge::new(polygon, side, style))
        .collect();
    let border = BORDER_FRACTION * f64::from(width.min(height));
    let weight = f64::from(grid * grid);
    let row_bytes = 3 * usize::try_from(width).unwrap_or(usize::MAX);
    let rows = usize::try_from(height).unwrap_or(usize::MAX);
    let depth = border / placement.scale();
    let strip = match tick {
        Tick::Absent => None,
        Tick::OnEdgeZero => Some(TickStrip::new(&edges[0], depth)),
    };
    let mut pixels = vec![0; row_bytes * rows];
    for (row, bytes) in (0..).zip(pixels.chunks_exact_mut(row_bytes)) {
        for (column, pixel) in (0..).zip(bytes.as_chunks_mut::<3>().0) {
            let mut sum = [0.0; 3];
            for down in 0..grid {
                for across in 0..grid {
                    let position = DVec2::new(
                        f64::from(column) + (f64::from(across) + 0.5) / f64::from(grid),
                        f64::from(row) + (f64::from(down) + 0.5) / f64::from(grid),
                    );
                    let point = placement.point(position);
                    let value = if strip.is_some_and(|strip| strip.contains(&edges, point)) {
                        Colour::BLACK
                    } else {
                        sample_colour(&edges, point, depth)
                    };
                    for (total, channel) in sum.iter_mut().zip(value.linear()) {
                        *total += channel;
                    }
                }
            }
            *pixel = sum.map(|total| encode(total / weight));
        }
    }
    pixels
}

struct Edge {
    start: DVec2,
    direction: DVec2,
    length: f64,
    code: Colour,
    marks: [bool; 2],
}

impl Edge {
    fn new(polygon: &[DVec2], side: usize, style: &EdgeStyle) -> Self {
        let count = polygon.len();
        let start = polygon[(side + count - 1) % count];
        let direction = polygon[side] - start;
        let [red, green, blue] = style.code;
        Self {
            start,
            direction,
            length: direction.length(),
            code: Colour::from_srgb8(red, green, blue),
            marks: style.marks,
        }
    }

    fn distance(&self, point: DVec2) -> f64 {
        self.direction.perp_dot(point - self.start) / self.length
    }

    fn fraction(&self, point: DVec2) -> f64 {
        (point - self.start).dot(self.direction) / (self.length * self.length)
    }
}

fn sample_colour(edges: &[Edge], point: DVec2, border: f64) -> Colour {
    let mut nearest: Option<(f64, &Edge)> = None;
    for edge in edges {
        let distance = edge.distance(point);
        if distance < 0.0 {
            return Colour::BLACK;
        }
        if nearest.is_none_or(|(least, _)| distance < least) {
            nearest = Some((distance, edge));
        }
    }
    match nearest {
        Some((distance, edge)) if distance < border => {
            let fraction = edge.fraction(point);
            let [start, end] = edge.marks;
            if (start && fraction <= MARK_FRACTION) || (end && fraction >= 1.0 - MARK_FRACTION) {
                Colour::BLACK
            } else {
                edge.code
            }
        }
        _ => Colour::WHITE,
    }
}

#[derive(Clone, Copy)]
struct TickStrip<'edge> {
    edge: &'edge Edge,
    depth: f64,
    half_width: f64,
}

impl<'edge> TickStrip<'edge> {
    fn new(edge: &'edge Edge, border: f64) -> Self {
        Self {
            edge,
            depth: TICK_LENGTH * border,
            half_width: border.min(edge.length / 4.0) / 2.0,
        }
    }

    fn contains(&self, edges: &[Edge], point: DVec2) -> bool {
        self.edge.distance(point) < self.depth
            && ((self.edge.fraction(point) - 0.5) * self.edge.length).abs() <= self.half_width
            && edges.iter().all(|other| other.distance(point) >= 0.0)
    }
}

#[expect(
    clippy::expect_used,
    reason = "the masks check the colouring against the side presentation of the domain, so the table has one generator for each side, at most 256 of them, and an image in every column for every coset"
)]
fn neighbour_pairs(colouring: &Colouring, side: usize, sides: usize) -> Vec<(usize, usize)> {
    let table = colouring.table();
    let colours = table.index();
    let letter =
        |side: usize| u8::try_from(side).expect("internal error: a side index fits a byte");
    let image = |coset: usize, generator: usize| {
        u32::try_from(coset)
            .ok()
            .and_then(|coset| table.image(coset, Letter::generator(letter(generator))))
            .and_then(|image| usize::try_from(image).ok())
            .expect("internal error: every coset has an image under every generator")
    };
    let neighbour = colouring
        .colour_of(&[letter(side)])
        .and_then(|colour| usize::try_from(colour).ok())
        .expect("internal error: every side has a colour across it");
    let mut reached = vec![false; colours * colours];
    reached[neighbour] = true;
    let mut pairs = vec![(0, neighbour)];
    let mut next = 0;
    while let Some(&(first, second)) = pairs.get(next) {
        next += 1;
        for generator in 0..sides {
            let pair = (image(first, generator), image(second, generator));
            let slot = &mut reached[pair.0 * colours + pair.1];
            if !*slot {
                *slot = true;
                pairs.push(pair);
            }
        }
    }
    pairs
}

fn edge_styles(domain: &FundamentalDomain, colouring: &Colouring) -> Vec<EdgeStyle> {
    let sides = domain.sides();
    let count = sides.len();
    let colours = colouring.table().index();
    let mut meetings: Vec<Vec<usize>> = vec![Vec::new(); colours * count];
    for (side, data) in sides.iter().enumerate() {
        let partner = data.partner();
        for (first, second) in neighbour_pairs(colouring, side, count) {
            meetings[first * count + side].push(second * count + partner);
            meetings[second * count + partner].push(first * count + side);
        }
    }
    let preserving: Vec<bool> = sides
        .iter()
        .map(|side| {
            matches!(
                side.kind(),
                PairingKind::Rotation | PairingKind::Translation
            )
        })
        .collect();
    seam_classes(&meetings, &preserving)
}

#[expect(
    clippy::expect_used,
    reason = "the walk over the meetings gives every edge a class before the styles are read"
)]
pub(crate) fn seam_classes(meetings: &[Vec<usize>], preserving: &[bool]) -> Vec<EdgeStyle> {
    let count = preserving.len();
    let edges = meetings.len();
    let mut classes: Vec<Option<usize>> = vec![None; edges];
    let mut marked: Vec<[bool; 2]> = vec![[false; 2]; edges];
    let mut doubled: Vec<bool> = Vec::new();
    for first in 0..edges {
        if classes[first].is_some() {
            continue;
        }
        let class = doubled.len();
        let mut both = false;
        classes[first] = Some(class);
        marked[first] = [true, false];
        let mut queue = VecDeque::from([first]);
        while let Some(edge) = queue.pop_front() {
            let at_start = marked[edge][0];
            let target = if at_start == preserving[edge % count] {
                [false, true]
            } else {
                [true, false]
            };
            for &other in &meetings[edge] {
                if classes[other].is_none() {
                    classes[other] = Some(class);
                    marked[other] = target;
                    queue.push_back(other);
                } else if marked[other] != target {
                    both = true;
                }
            }
        }
        doubled.push(both);
    }
    let total = doubled.len();
    classes
        .iter()
        .zip(marked)
        .map(|(class, marks)| {
            let class = class.expect("internal error: the walk gives every edge a class");
            EdgeStyle {
                class,
                code: code_colour(class, total),
                marks: if doubled[class] { [true, true] } else { marks },
            }
        })
        .collect()
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the floor of a hue value from 0 to 6 and the rounded product of a channel from 0 to 1 and 255 fit a byte"
)]
fn code_colour(class: usize, total: usize) -> [u8; 3] {
    let hue = 6.0 * to_f64(class) / to_f64(total);
    let sector = hue.floor();
    let rising = hue - sector;
    let falling = 1.0 - rising;
    let channels = match sector as u8 {
        0 => [1.0, rising, 0.0],
        1 => [falling, 1.0, 0.0],
        2 => [0.0, 1.0, rising],
        3 => [0.0, falling, 1.0],
        4 => [rising, 0.0, 1.0],
        _ => [1.0, 0.0, falling],
    };
    channels.map(|channel: f64| (255.0 * channel).round() as u8)
}

fn to_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MaskMap {
    geometry: Geometry,
    centre: DVec3,
    chart_centre: DVec3,
    covectors: Vec<DVec3>,
    vertices: Vec<DVec3>,
    charts: Vec<DVec3>,
    lengths: Vec<f64>,
    polygon: Vec<DVec2>,
    flat_centre: DVec2,
}

impl MaskMap {
    pub(crate) fn new(domain: &FundamentalDomain) -> Option<Self> {
        Self::with_polygon(domain, |centre| mask_polygon(domain, centre))
    }

    pub(crate) fn onto(domain: &FundamentalDomain, polygon: Vec<DVec2>) -> Option<Self> {
        Self::with_polygon(domain, |_| Some(polygon))
    }

    pub(crate) fn region(
        geometry: Geometry,
        vertices: Vec<DVec3>,
        covectors: Vec<DVec3>,
        polygon: Vec<DVec2>,
    ) -> Option<Self> {
        let centre = vertex_centre(geometry, &vertices)?;
        Self::build(geometry, vertices, covectors, centre, polygon)
    }

    fn with_polygon(
        domain: &FundamentalDomain,
        polygon_of: impl FnOnce(DVec3) -> Option<Vec<DVec2>>,
    ) -> Option<Self> {
        let geometry = domain.geometry();
        let vertices = domain.vertices().to_vec();
        let centre = vertex_centre(geometry, &vertices)?;
        let polygon = polygon_of(centre)?;
        let covectors = domain.sides().iter().map(|side| side.covector()).collect();
        Self::build(geometry, vertices, covectors, centre, polygon)
    }

    fn build(
        geometry: Geometry,
        vertices: Vec<DVec3>,
        covectors: Vec<DVec3>,
        centre: DVec3,
        polygon: Vec<DVec2>,
    ) -> Option<Self> {
        let count = vertices.len();
        let flat_centre = polygon.iter().sum::<DVec2>() / to_f64(count);
        let lengths = (0..count)
            .map(|side| {
                distance(
                    geometry,
                    vertices[(side + count - 1) % count],
                    vertices[side],
                )
            })
            .collect();
        let mut map = Self {
            geometry,
            centre,
            chart_centre: DVec3::ZERO,
            covectors,
            vertices,
            charts: Vec::new(),
            lengths,
            polygon,
            flat_centre,
        };
        map.chart_centre = map.chart(centre)?;
        map.charts = map
            .vertices
            .iter()
            .map(|&vertex| map.chart(vertex))
            .collect::<Option<_>>()?;
        Some(map)
    }

    pub(crate) fn polygon(&self) -> &[DVec2] {
        &self.polygon
    }

    fn turn(&self, first: DVec3, second: DVec3) -> f64 {
        match self.geometry {
            Geometry::Hyperbolic | Geometry::Euclidean => {
                first.truncate().perp_dot(second.truncate())
            }
            Geometry::Spherical => self.centre.dot(first.cross(second)),
        }
    }

    fn chart(&self, point: DVec3) -> Option<DVec3> {
        let divisor = match self.geometry {
            Geometry::Hyperbolic | Geometry::Euclidean => point.z,
            Geometry::Spherical => self.centre.dot(point),
        };
        if divisor.is_nan() || divisor <= 0.0 {
            return None;
        }
        let chart = point / divisor;
        chart.is_finite().then_some(chart)
    }

    pub(crate) fn map(&self, point: DVec3) -> Option<DVec2> {
        let chart = self.chart(point)?;
        if chart == self.chart_centre {
            return Some(self.flat_centre);
        }
        let ray = chart - self.chart_centre;
        let count = self.vertices.len();
        let side = (0..count).find(|&side| {
            let start = self.charts[(side + count - 1) % count] - self.chart_centre;
            let end = self.charts[side] - self.chart_centre;
            self.turn(start, ray) >= 0.0 && self.turn(ray, end) > 0.0
        })?;
        let covector = self.covectors[side];
        let rate = covector.dot(ray);
        if rate.is_nan() || rate <= 0.0 {
            return None;
        }
        let parameter = -covector.dot(self.chart_centre) / rate;
        let boundary = self.chart_centre + parameter * ray;
        let boundary = match self.geometry {
            Geometry::Hyperbolic => hyperbolic::normalize_point(boundary).ok()?,
            Geometry::Spherical => spherical::normalize_point(boundary).ok()?,
            Geometry::Euclidean => boundary,
        };
        let previous = (side + count - 1) % count;
        let fraction = (distance(self.geometry, self.vertices[previous], boundary)
            / self.lengths[side])
            .clamp(0.0, 1.0);
        let flat =
            self.polygon[previous] + fraction * (self.polygon[side] - self.polygon[previous]);
        Some(self.flat_centre + (flat - self.flat_centre) / parameter)
    }
}

fn vertex_centre(geometry: Geometry, vertices: &[DVec3]) -> Option<DVec3> {
    let sum: DVec3 = vertices.iter().sum();
    match geometry {
        Geometry::Hyperbolic => hyperbolic::normalize_point(sum).ok(),
        Geometry::Spherical => spherical::normalize_point(sum).ok(),
        Geometry::Euclidean => Some(sum / to_f64(vertices.len())),
    }
}

fn distance(geometry: Geometry, first: DVec3, second: DVec3) -> f64 {
    match geometry {
        Geometry::Hyperbolic => hyperbolic::distance(first, second),
        Geometry::Euclidean => euclidean::distance(first, second),
        Geometry::Spherical => spherical::distance(first, second),
    }
}

fn mask_polygon(domain: &FundamentalDomain, centre: DVec3) -> Option<Vec<DVec2>> {
    let sides = domain.sides();
    let mirrors = sides
        .iter()
        .all(|side| matches!(side.kind(), PairingKind::Mirror));
    let vertices = domain.vertices();
    let angles = domain.corner_angles();
    match (domain.geometry(), sides.len(), mirrors) {
        (Geometry::Euclidean, 3.., _) => Some(
            vertices
                .iter()
                .map(|vertex| DVec2::new(vertex.x, vertex.y))
                .collect(),
        ),
        (Geometry::Hyperbolic | Geometry::Spherical, 3, true) => {
            flat_triangle([angles[0], angles[1], angles[2]]).map(Vec::from)
        }
        (Geometry::Hyperbolic, 4.., true) | (Geometry::Hyperbolic, 6.., false) => {
            let recentre = hyperbolic::inverse(hyperbolic::translation(centre));
            Some(
                vertices
                    .iter()
                    .map(|&vertex| {
                        let moved = recentre * vertex;
                        DVec2::new(moved.x / moved.z, moved.y / moved.z)
                    })
                    .collect(),
            )
        }
        (Geometry::Hyperbolic | Geometry::Spherical, 4, false) => {
            let [first, second, third] =
                flat_triangle([angles[0] / 2.0, angles[2] / 2.0, angles[3]])?;
            Some(vec![first, reflect(third, first, second), second, third])
        }
        _ => None,
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the rounded quotient of pi and a corner angle is a corner order from 1 to a few hundred, and a cast saturates any other value"
)]
pub(crate) fn corner_order(angle: f64) -> u32 {
    (PI / angle).round() as u32
}

fn flat_triangle(angles: [f64; 3]) -> Option<[DVec2; 3]> {
    let orders = angles.map(corner_order);
    let [first, second, third] = orders.map(|order| PI / f64::from(order));
    let [vertex_0, vertex_1, vertex_2] = match orders {
        [_, 3..=u32::MAX, 2] => [PI - second - third, second, third],
        [_, 2, 2] => [first, PI - first - third, third],
        [_, _, 3..=u32::MAX] => [first, second, PI - first - second],
        _ => return None,
    };
    if !(vertex_0 > 0.0 && vertex_1 > 0.0 && vertex_2 > 0.0) {
        return None;
    }
    let reach = vertex_0.sin() / vertex_1.sin();
    let (sin, cos) = vertex_2.sin_cos();
    Some([DVec2::X, DVec2::new(reach * cos, reach * sin), DVec2::ZERO])
}

fn reflect(point: DVec2, start: DVec2, end: DVec2) -> DVec2 {
    let direction = end - start;
    let foot = start + direction * ((point - start).dot(direction) / direction.length_squared());
    2.0 * foot - point
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use std::sync::LazyLock;

    use escher_domain::{FundamentalDomain, PairingKind};
    use escher_geometry::{hyperbolic, spherical};
    use escher_group::{Colouring, ColouringSet, IndexLimit, SearchBudget, colourings_of_index};
    use escher_signature::Geometry;
    use glam::{DVec2, DVec3};
    use proptest::prelude::{ProptestConfig, any, prop, prop_assert, proptest};

    use super::{
        EdgeStyle, MASK_GRID, MaskMap, Masks, Tick, code_colour, draw_mask, neighbour_pairs,
        seam_classes,
    };
    use crate::colour::{Colour, CosetFill, Fill, Paint, decode_table};
    use crate::fold::fold;
    use crate::golden_images::SEVERAL_SET;
    use crate::golden_images::{
        GOLDEN_SIZE, MASK_GOLDEN_IMAGES, arrangements, arrangements_with, colouring,
        domain_with_settings, exact_face, exact_mask, exact_pictures, face_triangle_mask,
        golden_colouring, gradient, least_arrangements, marks_text, mask_domains, mask_rule,
        picture_arrangements, picture_tilings, renderer, several_face, several_firsts,
        signature_orders, texel_index, type_rules, whole_face,
    };
    use crate::image::{Image, Placement, sample};
    use crate::options::RenderOptions;
    use crate::reference::{ExactMask, ExactPlacement, Flat, MaskRule, RegionShape};
    use crate::whole_face::{
        EdgeMeeting, PICTURE_COUNTS, WHOLE_FACE_SET, face_mask_styles, uniform_firsts,
    };
    use escher_reference::{Reference, num, vector};

    static DOMAINS: LazyLock<Vec<(String, FundamentalDomain)>> = LazyLock::new(mask_domains);

    fn mask_map(name: &str, domain: &FundamentalDomain) -> MaskMap {
        MaskMap::new(domain).unwrap_or_else(|| panic!("{name} has no mask polygon"))
    }

    fn extent(polygon: &[DVec2]) -> f64 {
        let (low, high) = polygon.iter().fold(
            (DVec2::INFINITY, DVec2::NEG_INFINITY),
            |(low, high), &vertex| (low.min(vertex), high.max(vertex)),
        );
        (high - low).max_element()
    }

    fn approximate(reference: &mut Reference, flat: &[astro_float_num::BigFloat; 2]) -> DVec2 {
        DVec2::new(
            reference.approximate(&flat[0]),
            reference.approximate(&flat[1]),
        )
    }

    fn exact_point(reference: &mut Reference, point: &escher_reference::Vector) -> DVec3 {
        DVec3::new(
            reference.approximate(&point[0]),
            reference.approximate(&point[1]),
            reference.approximate(&point[2]),
        )
    }

    fn uncharted(geometry: Geometry, chart: DVec3) -> DVec3 {
        let point = match geometry {
            Geometry::Hyperbolic => hyperbolic::normalize_point(chart).ok(),
            Geometry::Spherical => spherical::normalize_point(chart).ok(),
            Geometry::Euclidean => Some(chart),
        };
        point.unwrap_or_else(|| panic!("the chart position {chart} has no point"))
    }

    fn table_angles(orders: [u32; 3]) -> [f64; 3] {
        let pi = core::f64::consts::PI;
        let [first, second, third] = orders.map(|order| pi / f64::from(order));
        match orders {
            [_, second_order, 2] if second_order >= 3 => [pi - second - third, second, third],
            [_, 2, 2] => [first, pi - first - third, third],
            _ => [first, second, pi - first - second],
        }
    }

    fn corner_angle(previous: DVec2, here: DVec2, next: DVec2) -> f64 {
        let (back, ahead) = (previous - here, next - here);
        back.perp_dot(ahead).abs().atan2(back.dot(ahead))
    }

    const POLYGON_TOLERANCE: f64 = 2.2e-14;

    fn check_shape(
        name: &str,
        domain: &FundamentalDomain,
        polygon: &[DVec2],
        worst: &mut [f64; 4],
    ) {
        let count = domain.vertices().len();
        assert_eq!(polygon.len(), count, "{name}");
        let size = extent(polygon);
        let at = |index: usize| polygon[index % count];
        let mut area = 0.0;
        for vertex in 0..count {
            let (previous, here, next) = (at(vertex + count - 1), at(vertex), at(vertex + 1));
            let turn = (here - previous).perp_dot(next - here)
                / ((here - previous).length() * (next - here).length());
            worst[0] = worst[0].max(-turn);
            assert!(
                turn >= -POLYGON_TOLERANCE,
                "{name}: vertex {vertex} turns by {turn:e}"
            );
            area += previous.perp_dot(here);
        }
        assert!(area > 0.0, "{name}: the polygon is clockwise");
        let length = |side: usize| (at(side) - at(side + count - 1)).length();
        for (side, data) in domain.sides().iter().enumerate() {
            let gap = (length(side) - length(data.partner())).abs() / size;
            worst[1] = worst[1].max(gap);
            assert!(
                gap <= POLYGON_TOLERANCE,
                "{name}: edge {side} and edge {} differ by {gap:e}",
                data.partner()
            );
        }
    }

    fn check_rule(
        reference: &mut Reference,
        name: &str,
        domain: &FundamentalDomain,
        polygon: &[DVec2],
        worst: &mut [f64; 4],
    ) -> usize {
        let size = extent(polygon);
        let rule = mask_rule(domain);
        if rule == MaskRule::Plane {
            for (vertex, flat) in domain.vertices().iter().zip(polygon) {
                assert_eq!(*flat, DVec2::new(vertex.x, vertex.y), "{name}");
            }
            return 0;
        }
        let exact = exact_mask(reference, domain);
        for (vertex, (flat, expected)) in polygon.iter().zip(&exact.polygon).enumerate() {
            let gap = (*flat - approximate(reference, expected))
                .abs()
                .max_element()
                / size;
            worst[2] = worst[2].max(gap);
            assert!(
                gap <= POLYGON_TOLERANCE,
                "{name}: vertex {vertex} is off by {gap:e}"
            );
        }
        let (index, orders, triangle) = match rule {
            MaskRule::Triangle(orders) => (1, orders, [polygon[0], polygon[1], polygon[2]]),
            MaskRule::Double(orders) => {
                let mirrored = super::reflect(polygon[3], polygon[0], polygon[2]);
                let gap = (mirrored - polygon[1]).abs().max_element() / size;
                worst[2] = worst[2].max(gap);
                assert!(
                    gap <= POLYGON_TOLERANCE,
                    "{name}: the double is not symmetric, {gap:e}"
                );
                (3, orders, [polygon[0], polygon[2], polygon[3]])
            }
            MaskRule::Klein | MaskRule::Plane => return 2,
            MaskRule::FaceTriangle { .. } | MaskRule::FaceRegion { .. } => {
                panic!("{name}: a domain has no face triangle or face region rule")
            }
        };
        for (vertex, expected) in table_angles(orders).into_iter().enumerate() {
            let angle = corner_angle(
                triangle[(vertex + 2) % 3],
                triangle[vertex],
                triangle[(vertex + 1) % 3],
            );
            worst[3] = worst[3].max((angle - expected).abs());
            assert!(
                (angle - expected).abs() <= POLYGON_TOLERANCE,
                "{name}: the flat angle {vertex} is {angle}, not {expected}"
            );
        }
        index
    }

    fn check_centres(name: &str, domain: &FundamentalDomain, map: &MaskMap) {
        let polygon = map.polygon();
        let count = polygon.len();
        for (side, data) in domain.sides().iter().enumerate() {
            assert!(
                data.covector().dot(map.centre) < 0.0,
                "{name}: the centre is not strictly inside side {side}"
            );
            let start = polygon[(side + count - 1) % count];
            assert!(
                (polygon[side] - start).perp_dot(map.flat_centre - start) > 0.0,
                "{name}: the flat centre is not strictly inside edge {side}"
            );
        }
        if domain.geometry() == Geometry::Spherical {
            for (index, vertex) in domain.vertices().iter().enumerate() {
                assert!(
                    map.centre.dot(*vertex) > 0.0,
                    "{name}: vertex {index} lies outside the chart"
                );
            }
        }
    }

    #[test]
    fn polygon_rules_give_a_convex_polygon_with_equal_paired_edges_around_both_centres() {
        let mut reference = Reference::new();
        let mut worst = [0.0_f64; 4];
        let mut rules = [0_usize; 4];
        for (name, domain) in DOMAINS.iter() {
            let map = mask_map(name, domain);
            check_shape(name, domain, map.polygon(), &mut worst);
            rules[check_rule(&mut reference, name, domain, map.polygon(), &mut worst)] += 1;
            check_centres(name, domain, &map);
        }
        println!(
            "{} domains, rules {rules:?}: turn {:e}, paired edges {:e}, vertices {:e}, angles {:e}, allowed {POLYGON_TOLERANCE:e}",
            DOMAINS.len(),
            worst[0],
            worst[1],
            worst[2],
            worst[3]
        );
        assert!(rules.iter().all(|&count| count >= 5), "{rules:?}");
    }

    const OUTSIDE_TOLERANCE: f64 = 2.2e-14;

    const ROUND_TRIP_TOLERANCE: f64 = 1.2e-9;

    const VERTEX_ROUND_TRIP_TOLERANCE: f64 = 2.1e-4;

    fn chart_point(map: &MaskMap, weights: &[f64]) -> DVec3 {
        let charts: Vec<DVec3> = map
            .vertices
            .iter()
            .map(|&vertex| match map.geometry {
                Geometry::Hyperbolic | Geometry::Euclidean => vertex / vertex.z,
                Geometry::Spherical => vertex / map.centre.dot(vertex),
            })
            .collect();
        let total: f64 = weights.iter().take(charts.len()).sum();
        let chart = charts
            .iter()
            .zip(weights)
            .fold(DVec3::ZERO, |sum, (&chart, &weight)| sum + weight * chart)
            / total;
        uncharted(map.geometry, chart)
    }

    fn inside_polygon(polygon: &[DVec2], point: DVec2) -> f64 {
        let count = polygon.len();
        (0..count)
            .map(|side| {
                let start = polygon[(side + count - 1) % count];
                let edge = polygon[side] - start;
                edge.perp_dot(point - start) / edge.length()
            })
            .fold(f64::INFINITY, f64::min)
    }

    fn round_trip_error(
        reference: &mut Reference,
        exact: &ExactMask,
        map: &MaskMap,
        point: DVec3,
    ) -> Result<(f64, f64), String> {
        let flat = map
            .map(point)
            .ok_or_else(|| format!("{point} has no mask point"))?;
        let size = extent(map.polygon());
        let outside = -inside_polygon(map.polygon(), flat) / size;
        let back = exact.inverse(reference, &[num(flat.x), num(flat.y)]);
        let error = exact.distance(reference, &back, &vector(point.to_array()));
        Ok((outside, reference.approximate(&error)))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]
        #[test]
        fn mask_map_carries_a_point_of_the_domain_into_the_polygon_and_back(
            index in any::<prop::sample::Index>(),
            weights in prop::collection::vec(1e-3..1.0_f64, 14),
        ) {
            let (name, domain) = index.get(&DOMAINS);
            let map = mask_map(name, domain);
            let point = chart_point(&map, &weights);
            let mut reference = Reference::new();
            let exact = exact_mask(&mut reference, domain);
            let (outside, error) = round_trip_error(&mut reference, &exact, &map, point)
                .unwrap_or_else(|failure| panic!("{name}: {failure}"));
            prop_assert!(outside <= OUTSIDE_TOLERANCE, "{}: {} lies {:e} outside", name, point, outside);
            prop_assert!(error <= ROUND_TRIP_TOLERANCE, "{}: {} returns {:e} away", name, point, error);
        }
    }

    #[derive(Default)]
    struct Worst {
        outside: f64,
        error: f64,
        centre: f64,
        compared: usize,
    }

    fn vertex_round_trips(
        reference: &mut Reference,
        name: &str,
        map: &MaskMap,
        exact: &ExactMask,
        worst: &mut Worst,
    ) {
        assert_eq!(map.map(map.centre), Some(map.flat_centre), "{name}");
        let (_, centre_error) = round_trip_error(reference, exact, map, map.centre)
            .unwrap_or_else(|failure| panic!("{name}: {failure}"));
        worst.centre = worst.centre.max(centre_error);
        assert!(
            centre_error <= ROUND_TRIP_TOLERANCE,
            "{name}: the centre returns {centre_error:e} away"
        );
        for (index, &vertex) in map.vertices.iter().enumerate() {
            let (outside, error) = round_trip_error(reference, exact, map, vertex)
                .unwrap_or_else(|failure| panic!("{name}: {failure}"));
            worst.outside = worst.outside.max(outside);
            worst.error = worst.error.max(error);
            assert!(
                outside <= OUTSIDE_TOLERANCE && error <= VERTEX_ROUND_TRIP_TOLERANCE,
                "{name}: vertex {index} lies {outside:e} outside and returns {error:e} away"
            );
            worst.compared += 1;
        }
    }

    fn report_vertex_round_trips(worst: &Worst) {
        println!(
            "{} vertices: {:e} outside the polygon, allowed {OUTSIDE_TOLERANCE:e}, {:e} away after the round trip, allowed {VERTEX_ROUND_TRIP_TOLERANCE:e}, centres {:e} away after the round trip, allowed {ROUND_TRIP_TOLERANCE:e}",
            worst.compared, worst.outside, worst.error, worst.centre
        );
    }

    #[test]
    fn mask_map_carries_the_vertices_and_the_centre_to_their_flat_points_and_back() {
        let mut reference = Reference::new();
        let mut worst = Worst::default();
        for (name, domain) in DOMAINS.iter() {
            let map = mask_map(name, domain);
            let exact = exact_mask(&mut reference, domain);
            vertex_round_trips(&mut reference, name, &map, &exact, &mut worst);
        }
        report_vertex_round_trips(&worst);
        assert!(worst.compared >= 300, "{}", worst.compared);
    }

    const EDGE_FRACTION_TOLERANCE: f64 = 1.25e-5;

    fn edge_fraction_gaps(
        reference: &mut Reference,
        name: &str,
        map: &MaskMap,
        exact: &ExactMask,
        worst: &mut f64,
    ) -> usize {
        let polygon = map.polygon();
        let size = extent(polygon);
        let count = polygon.len();
        for side in 0..count {
            let start = polygon[(side + count - 1) % count];
            for quarters in 0..=4 {
                let fraction = f64::from(quarters) / 4.0;
                let on_side = exact.along_side(reference, side, &num(fraction));
                let point = exact_point(reference, &on_side);
                let flat = map
                    .map(point)
                    .unwrap_or_else(|| panic!("{name}: side {side} has no mask point"));
                let expected = start + fraction * (polygon[side] - start);
                let gap = (flat - expected).abs().max_element() / size;
                *worst = worst.max(gap);
                assert!(
                    gap <= EDGE_FRACTION_TOLERANCE,
                    "{name}: side {side} at {fraction} maps {gap:e} away from its edge"
                );
            }
        }
        5 * count
    }

    #[test]
    fn mask_map_sends_each_side_to_its_edge_in_proportion_to_distance_along_the_side() {
        const TOLERANCE: f64 = EDGE_FRACTION_TOLERANCE;
        let mut reference = Reference::new();
        let (mut worst, mut compared) = (0.0_f64, 0);
        for (name, domain) in DOMAINS.iter() {
            let map = mask_map(name, domain);
            let exact = exact_mask(&mut reference, domain);
            compared += edge_fraction_gaps(&mut reference, name, &map, &exact, &mut worst);
        }
        println!(
            "{compared} points of the sides: {worst:e} from their edges, allowed {TOLERANCE:e}"
        );
        assert!(compared >= 1500, "{compared}");
    }

    struct FaceTriangle {
        name: String,
        text: &'static str,
        marks: [bool; 3],
        domain: FundamentalDomain,
        map: MaskMap,
    }

    static FACE_TRIANGLES: LazyLock<Vec<FaceTriangle>> = LazyLock::new(|| {
        WHOLE_FACE_SET
            .iter()
            .map(|&(text, marks)| {
                let domain = domain_with_settings(text, &[]);
                let name = format!("{text} {}", marks_text(marks));
                let map = whole_face(&domain, marks)
                    .face_map()
                    .cloned()
                    .unwrap_or_else(|| panic!("{name} has no face mask map"));
                FaceTriangle {
                    name,
                    text,
                    marks,
                    domain,
                    map,
                }
            })
            .collect()
    });

    impl FaceTriangle {
        fn exact(&self, reference: &mut Reference) -> ExactMask {
            face_triangle_mask(reference, self.text, &self.domain, self.marks)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]
        #[test]
        fn face_triangle_map_carries_a_point_of_the_domain_into_the_triangle_and_back(
            index in any::<prop::sample::Index>(),
            weights in prop::collection::vec(1e-3..1.0_f64, 3),
        ) {
            let scene = index.get(&FACE_TRIANGLES);
            let point = chart_point(&scene.map, &weights);
            let mut reference = Reference::new();
            let exact = scene.exact(&mut reference);
            let (outside, error) = round_trip_error(&mut reference, &exact, &scene.map, point)
                .unwrap_or_else(|failure| panic!("{}: {failure}", scene.name));
            prop_assert!(outside <= OUTSIDE_TOLERANCE, "{}: {} lies {:e} outside", scene.name, point, outside);
            prop_assert!(error <= ROUND_TRIP_TOLERANCE, "{}: {} returns {:e} away", scene.name, point, error);
        }
    }

    #[test]
    fn face_triangle_map_carries_the_vertices_and_the_centre_to_their_flat_points_and_back() {
        let mut reference = Reference::new();
        let mut worst = Worst::default();
        for scene in FACE_TRIANGLES.iter() {
            let exact = scene.exact(&mut reference);
            vertex_round_trips(&mut reference, &scene.name, &scene.map, &exact, &mut worst);
        }
        report_vertex_round_trips(&worst);
        assert_eq!(worst.compared, 3 * WHOLE_FACE_SET.len());
    }

    #[test]
    fn face_triangle_map_sends_each_side_to_its_edge_in_proportion_to_distance_along_the_side() {
        let mut reference = Reference::new();
        let (mut worst, mut compared) = (0.0_f64, 0);
        for scene in FACE_TRIANGLES.iter() {
            let exact = scene.exact(&mut reference);
            compared +=
                edge_fraction_gaps(&mut reference, &scene.name, &scene.map, &exact, &mut worst);
        }
        println!(
            "{compared} points of the sides: {worst:e} from their edges, allowed {EDGE_FRACTION_TOLERANCE:e}"
        );
        assert_eq!(compared, 15 * WHOLE_FACE_SET.len());
    }

    struct FaceRegion {
        name: String,
        map: MaskMap,
        sides: u32,
        shape: RegionShape,
    }

    static FACE_REGIONS: LazyLock<Vec<FaceRegion>> = LazyLock::new(|| {
        SEVERAL_SET
            .iter()
            .flat_map(|&(text, marks)| {
                let (_, face) = several_face(text, marks);
                type_rules(signature_orders(text), marks)
                    .into_iter()
                    .enumerate()
                    .map(|(kind, rule)| {
                        let name = format!("{text} {} type {}", marks_text(marks), rule.corner);
                        let map = face
                            .type_map(kind)
                            .cloned()
                            .unwrap_or_else(|| panic!("{name} has no region map"));
                        let shape = if rule.both() {
                            RegionShape::BothOn
                        } else if rule.centre == (rule.corner + 1) % 3 {
                            RegionShape::CentreAfter
                        } else {
                            RegionShape::CentreBefore
                        };
                        FaceRegion {
                            name,
                            map,
                            sides: rule.sides,
                            shape,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    });

    impl FaceRegion {
        fn exact(&self, reference: &mut Reference) -> ExactMask {
            let vertices: Vec<[f64; 3]> = self.map.vertices.iter().map(DVec3::to_array).collect();
            let covectors: Vec<[f64; 3]> = self.map.covectors.iter().map(DVec3::to_array).collect();
            ExactMask::new(
                reference,
                crate::reference::Measure::of(self.map.geometry),
                &vertices,
                &covectors,
                MaskRule::FaceRegion {
                    sides: self.sides,
                    shape: self.shape,
                },
            )
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]
        #[test]
        fn region_map_carries_a_point_of_the_region_into_the_flat_region_and_back(
            index in any::<prop::sample::Index>(),
            weights in prop::collection::vec(1e-3..1.0_f64, 4),
        ) {
            let region = index.get(&FACE_REGIONS);
            let point = chart_point(&region.map, &weights);
            let mut reference = Reference::new();
            let exact = region.exact(&mut reference);
            let (outside, error) = round_trip_error(&mut reference, &exact, &region.map, point)
                .unwrap_or_else(|failure| panic!("{}: {failure}", region.name));
            prop_assert!(outside <= OUTSIDE_TOLERANCE, "{}: {} lies {:e} outside", region.name, point, outside);
            prop_assert!(error <= ROUND_TRIP_TOLERANCE, "{}: {} returns {:e} away", region.name, point, error);
        }
    }

    #[test]
    fn region_map_carries_the_vertices_and_the_centre_to_their_flat_points_and_back() {
        let mut reference = Reference::new();
        let mut worst = Worst::default();
        for region in FACE_REGIONS.iter() {
            let exact = region.exact(&mut reference);
            vertex_round_trips(
                &mut reference,
                &region.name,
                &region.map,
                &exact,
                &mut worst,
            );
        }
        report_vertex_round_trips(&worst);
        assert_eq!(worst.compared, REGION_VERTICES);
    }

    #[test]
    fn region_map_sends_each_side_to_its_edge_in_proportion_to_distance_along_the_side() {
        let mut reference = Reference::new();
        let (mut worst, mut compared) = (0.0_f64, 0);
        for region in FACE_REGIONS.iter() {
            let exact = region.exact(&mut reference);
            compared += edge_fraction_gaps(
                &mut reference,
                &region.name,
                &region.map,
                &exact,
                &mut worst,
            );
        }
        println!(
            "{compared} points of the sides: {worst:e} from their edges, allowed {EDGE_FRACTION_TOLERANCE:e}"
        );
        assert_eq!(compared, 5 * REGION_VERTICES);
    }

    #[test]
    fn each_region_is_convex_counterclockwise_and_holds_its_centre_inside_every_side() {
        let mut reference = Reference::new();
        let mut sides = 0;
        for region in FACE_REGIONS.iter() {
            let exact = region.exact(&mut reference);
            let charts: Vec<escher_reference::Vector> = exact
                .vertices
                .iter()
                .map(|vertex| exact.chart(vertex))
                .collect();
            let centre = exact.chart(&exact.centre);
            let count = charts.len();
            for side in 0..count {
                let [start, end, next] =
                    [side, side + 1, side + 2].map(|index| &charts[index % count]);
                let run = escher_reference::difference(end, start);
                let turn = exact.turn(&run, &escher_reference::difference(next, end));
                let inside = exact.turn(&run, &escher_reference::difference(&centre, start));
                assert!(
                    turn.is_positive() && inside.is_positive(),
                    "{}: side {side} turns {turn} and holds the centre at {inside}",
                    region.name
                );
                sides += 1;
            }
        }
        println!("{sides} sides");
        assert_eq!(sides, REGION_VERTICES);
    }

    const REGION_VERTICES: usize = 159;

    #[test]
    fn mask_map_of_a_euclidean_domain_is_the_identity_of_the_plane() {
        const TOLERANCE: f64 = 8e-15;
        let (mut worst, mut compared) = (0.0_f64, 0);
        for (name, domain) in DOMAINS
            .iter()
            .filter(|(_, domain)| domain.geometry() == Geometry::Euclidean)
        {
            let map = mask_map(name, domain);
            let size = extent(map.polygon());
            let count = map.vertices.len();
            let mut points = vec![map.centre];
            for first in 0..count {
                for second in 0..count {
                    for weight in [0.1, 0.5, 0.9] {
                        let mut weights = vec![0.05; count];
                        weights[first] += weight;
                        weights[second] += 1.0 - weight;
                        points.push(chart_point(&map, &weights));
                    }
                }
                points.push(map.vertices[first]);
            }
            for point in points {
                let flat = map
                    .map(point)
                    .unwrap_or_else(|| panic!("{name}: {point} has no mask point"));
                let gap = (flat - point.truncate()).abs().max_element() / size;
                worst = worst.max(gap);
                assert!(gap <= TOLERANCE, "{name}: {point} maps {gap:e} away");
                compared += 1;
            }
        }
        println!("{compared} Euclidean points: {worst:e} from themselves, allowed {TOLERANCE:e}");
        assert!(compared >= 1000, "{compared}");
    }

    #[test]
    fn mask_map_gives_no_point_where_the_ray_from_the_centre_underflows() {
        let cases = [
            (
                "*2222 aspect 1",
                [DVec3::new(5e-324, 0.0, 0.0), DVec3::new(0.0, -5e-324, 0.0)],
            ),
            (
                "o aspect 1 skew 0.25",
                [DVec3::new(0.0, 5e-324, 0.0), DVec3::new(0.0, -5e-324, 0.0)],
            ),
            (
                "*22222 cornerpair1.length 3.5 cornerpair2.length 3.5",
                [DVec3::new(1e-310, 0.0, 0.0), DVec3::new(0.0, -1e-310, 0.0)],
            ),
            (
                "*522",
                [DVec3::new(1e-310, 0.0, 0.0), DVec3::new(-1e-310, 0.0, 0.0)],
            ),
        ];
        for (name, steps) in cases {
            let Some((_, domain)) = DOMAINS.iter().find(|(other, _)| other == name) else {
                panic!("{name} is not a mask domain");
            };
            let map = mask_map(name, domain);
            for step in steps {
                let point = map.chart_centre + step;
                assert_ne!(point, map.chart_centre, "{name}: {step}");
                assert_eq!(uncharted(domain.geometry(), point), point, "{name}: {step}");
                assert_eq!(map.map(point), None, "{name}: {point}");
            }
        }
    }

    fn valid_colourings(domain: &FundamentalDomain, most: u32) -> Vec<Colouring> {
        let presentation = domain
            .side_presentation()
            .unwrap_or_else(|error| panic!("{error}"));
        (1..=most)
            .rev()
            .find_map(|colours| {
                let limit = IndexLimit::new(colours).unwrap_or_else(|error| panic!("{error}"));
                let found = colourings_of_index(
                    &presentation,
                    ColouringSet::ValidClasses,
                    limit,
                    SearchBudget::DEFAULT,
                )
                .unwrap_or_else(|error| panic!("{error}"));
                found.is_complete().then(|| found.list().to_vec())
            })
            .unwrap_or_default()
    }

    fn masks_of(name: &str, domain: &FundamentalDomain, colouring: &Colouring) -> Masks {
        Masks::new(domain, colouring)
            .unwrap_or_else(|error| panic!("{name} {}: {error}", colouring.id()))
    }

    fn pixel(pixels: &[u8], width: u32, column: u32, row: u32) -> [u8; 3] {
        let start = usize::try_from(3 * (row * width + column)).unwrap_or(usize::MAX);
        [pixels[start], pixels[start + 1], pixels[start + 2]]
    }

    #[test]
    fn mask_shows_the_polygon_with_a_border_in_the_code_colour_of_each_edge_and_its_marks() {
        let mut scenes: Vec<(String, FundamentalDomain, Colouring)> = MASK_GOLDEN_IMAGES
            .iter()
            .map(|golden| {
                let (domain, colouring, _) = golden.masks();
                (golden.signature.to_owned(), domain, colouring)
            })
            .collect();
        for (signature, id) in [("3333", "4a"), ("xx", "4c"), ("3*2", "4a"), ("*732", "2a")] {
            let domain = domain_with_settings(signature, &[]);
            let colouring = golden_colouring(&domain, id);
            scenes.push((signature.to_owned(), domain, colouring));
        }
        let mut content = Content::default();
        for (name, domain, colouring) in &scenes {
            let masks = masks_of(name, domain, colouring);
            for [width, height] in MASK_SIZES {
                for colour in 0..masks.colours {
                    let pixels = masks.mask(colour, width, height, 1);
                    let name = format!("{name} {} colour {colour}", colouring.id());
                    check_mask_content(
                        &name,
                        (
                            &masks.polygon,
                            Placement::new(&masks.polygon, width, height),
                        ),
                        masks.edge_styles_of(colour),
                        Tick::Absent,
                        &pixels,
                        [width, height],
                        &mut content,
                    );
                }
            }
        }
        content.report();
        assert!(
            content.white >= 10_000
                && content.black >= 10_000
                && content.coded >= 2000
                && content.marked >= 500
        );
    }

    const MASK_SIZES: [[u32; 2]; 2] = [[96, 96], [128, 64]];

    #[derive(Default)]
    struct Content {
        white: usize,
        black: usize,
        coded: usize,
        marked: usize,
    }

    impl Content {
        fn report(&self) {
            println!(
                "{} white, {} black, {} code colour, and {} mark pixels",
                self.white, self.black, self.coded, self.marked
            );
        }
    }

    fn near_tick(polygon: &[DVec2], point: DVec2, border: f64, pixel_size: f64) -> bool {
        let count = polygon.len();
        let start = polygon[count - 1];
        let edge = polygon[0] - start;
        let length = edge.length();
        let along = (point - start).dot(edge) / length - length / 2.0;
        let width = border.min(length / 4.0);
        edge.perp_dot(point - start) / length < 4.0 * border + pixel_size
            && along.abs() <= width / 2.0 + pixel_size
    }

    fn check_mask_content(
        name: &str,
        (polygon, placement): (&[DVec2], Placement),
        styles: &[EdgeStyle],
        tick: Tick,
        pixels: &[u8],
        [width, height]: [u32; 2],
        content: &mut Content,
    ) {
        let count = polygon.len();
        let pixel_size = 1.0 / placement.scale();
        let border = f64::from(width.min(height)) / 64.0 * pixel_size;
        for row in 0..height {
            for column in 0..width {
                let centre = DVec2::new(f64::from(column) + 0.5, f64::from(row) + 0.5);
                let point = placement.point(centre);
                let distances: Vec<(f64, f64)> = (0..count)
                    .map(|side| {
                        let start = polygon[(side + count - 1) % count];
                        let edge = polygon[side] - start;
                        (
                            edge.perp_dot(point - start) / edge.length(),
                            (point - start).dot(edge) / edge.length_squared(),
                        )
                    })
                    .collect();
                let nearest = distances
                    .iter()
                    .map(|&(distance, _)| distance)
                    .fold(f64::INFINITY, f64::min);
                let found = pixel(pixels, width, column, row);
                let name = format!("{name} at {width}x{height} column {column} row {row}");
                if tick == Tick::OnEdgeZero
                    && nearest >= -pixel_size
                    && near_tick(polygon, point, border, pixel_size)
                {
                    continue;
                }
                if nearest < -pixel_size {
                    assert_eq!(found, [0; 3], "{name}");
                    content.black += 1;
                } else if nearest > border + pixel_size {
                    assert_eq!(found, [255; 3], "{name}");
                    content.white += 1;
                } else if let Some(side) = (0..count).find(|&side| {
                    let (distance, fraction) = distances[side];
                    let length = (polygon[side] - polygon[(side + count - 1) % count]).length();
                    let clear = pixel_size / length;
                    distance > 1e-9 * pixel_size
                        && distance < border - pixel_size
                        && (0..count)
                            .all(|other| other == side || distances[other].0 > border + pixel_size)
                        && (fraction - 0.25).abs() > clear
                        && (fraction - 0.75).abs() > clear
                }) {
                    let fraction = distances[side].1;
                    let style = styles[side];
                    let at_mark =
                        (style.marks[0] && fraction < 0.25) || (style.marks[1] && fraction > 0.75);
                    if at_mark {
                        assert_eq!(found, [0; 3], "{name}: edge {side}");
                        content.marked += 1;
                    } else {
                        assert_eq!(found, style.code, "{name}: edge {side}");
                        content.coded += 1;
                    }
                }
            }
        }
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "the test checks the styles, the content, and the tick of the face mask of every arrangement of the whole-face set"
    )]
    fn face_mask_shows_the_flat_face_with_one_code_colour_for_each_pair_of_edges_and_its_marks() {
        let mut content = Content::default();
        let mut reference = Reference::new();
        let (mut masks, mut alone, mut both) = (0, 0, 0);
        for &(text, marks) in &WHOLE_FACE_SET {
            let domain = domain_with_settings(text, &[]);
            let face = whole_face(&domain, marks);
            let exact = exact_face(&mut reference, text, &domain, marks);
            for arrangement in arrangements(&face) {
                let name = format!("{text} {} {}", marks_text(marks), arrangement.id());
                let edges = face
                    .edges(&arrangement)
                    .unwrap_or_else(|error| panic!("{name}: {error}"));
                let styles =
                    face_mask_styles(&edges, &uniform_firsts(edges.len(), face.polygon().len()));
                let total = styles
                    .iter()
                    .map(|style| style.class)
                    .max()
                    .map_or(0, |last| last + 1);
                let mut next_class = 0;
                for (edge, meeting) in edges.iter().enumerate() {
                    let partner = usize::try_from(meeting.partner()).unwrap_or(usize::MAX);
                    let style = styles[edge];
                    assert_eq!(
                        style.code,
                        code_colour(style.class, total),
                        "{name}: edge {edge}"
                    );
                    assert_eq!(styles[partner].class, style.class, "{name}: edge {edge}");
                    if style.class == next_class {
                        next_class += 1;
                    }
                    assert!(
                        style.class < next_class,
                        "{name}: edge {edge} opens class {}",
                        style.class
                    );
                    if partner == edge {
                        let sharing = styles
                            .iter()
                            .filter(|other| other.class == style.class)
                            .count();
                        assert_eq!(sharing, 1, "{name}: edge {edge} shares its class");
                        let wanted = if meeting.mirrored() {
                            [true, false]
                        } else {
                            [true, true]
                        };
                        assert_eq!(style.marks, wanted, "{name}: edge {edge}");
                        alone += 1;
                        both += usize::from(!meeting.mirrored());
                    }
                }
                for size in MASK_SIZES {
                    let pixels = draw_mask(
                        face.polygon(),
                        Placement::new(face.polygon(), size[0], size[1]),
                        &styles,
                        Tick::OnEdgeZero,
                        size[0],
                        size[1],
                        1,
                    );
                    check_mask_content(
                        &name,
                        (
                            face.polygon(),
                            Placement::new(face.polygon(), size[0], size[1]),
                        ),
                        &styles,
                        Tick::OnEdgeZero,
                        &pixels,
                        size,
                        &mut content,
                    );
                }
                let pixels = draw_mask(
                    face.polygon(),
                    Placement::new(face.polygon(), TICK_SIZE, TICK_SIZE),
                    &styles,
                    Tick::OnEdgeZero,
                    TICK_SIZE,
                    TICK_SIZE,
                    1,
                );
                let placement = ExactPlacement::new(&exact.polygon, TICK_SIZE, TICK_SIZE);
                check_tick(
                    &mut reference,
                    &name,
                    (&exact.polygon, &placement),
                    &styles,
                    &pixels,
                );
                masks += 1;
            }
        }
        content.report();
        println!("{masks} face masks, {alone} edges paired with themselves, {both} of them turned");
        assert_eq!((masks, alone, both), (114, 250, 120));
        assert!(
            content.white >= 500_000
                && content.black >= 500_000
                && content.coded >= 10_000
                && content.marked >= 4000
        );
    }

    const TICK_SIZE: u32 = 512;

    fn assert_picture_styles(
        name: &str,
        edges: &[EdgeMeeting],
        styles: &[EdgeStyle],
        firsts: &[usize],
    ) -> [usize; 3] {
        let picture_of = |edge: usize| firsts.iter().rposition(|&first| first <= edge).unwrap_or(0);
        let total = styles
            .iter()
            .map(|style| style.class)
            .max()
            .map_or(0, |last| last + 1);
        let mut next_class = 0;
        let mut kinds = [0; 3];
        for (edge, meeting) in edges.iter().enumerate() {
            let other = firsts[usize::try_from(meeting.picture()).unwrap_or(usize::MAX)]
                + usize::try_from(meeting.partner()).unwrap_or(usize::MAX);
            let style = styles[edge];
            let picture = picture_of(edge);
            let label = format!(
                "{name}: edge {} of picture {picture}",
                edge - firsts[picture]
            );
            assert_eq!(style.code, code_colour(style.class, total), "{label}");
            assert_eq!(styles[other].class, style.class, "{label}");
            if style.class == next_class {
                next_class += 1;
            }
            assert!(
                style.class < next_class,
                "{label} opens class {}",
                style.class
            );
            let sharing = styles
                .iter()
                .filter(|another| another.class == style.class)
                .count();
            let turned = !meeting.mirrored();
            if other == edge {
                assert_eq!(sharing, 1, "{label} shares its class");
                assert_eq!(style.marks, [true, turned], "{label}");
                kinds[0] += 1;
            } else {
                assert_eq!(sharing, 2, "{label}");
                let wanted = if other > edge {
                    [true, false]
                } else {
                    [!turned, turned]
                };
                assert_eq!(style.marks, wanted, "{label}");
                kinds[1 + usize::from(picture_of(other) != picture)] += 1;
            }
        }
        kinds
    }

    #[test]
    fn face_masks_of_several_pictures_share_one_code_colour_for_each_meeting_and_mark_its_ends() {
        let mut kinds = [0; 3];
        let mut arrangements = 0;
        for &(text, marks) in &WHOLE_FACE_SET {
            let face = whole_face(&domain_with_settings(text, &[]), marks);
            let order = usize::try_from(face.index() / 2).unwrap_or(usize::MAX);
            for pictures in PICTURE_COUNTS {
                for arrangement in arrangements_with(&face, pictures) {
                    let name = format!("{text} {} {}", marks_text(marks), arrangement.id());
                    let edges = face
                        .edges(&arrangement)
                        .unwrap_or_else(|error| panic!("{name}: {error}"));
                    let firsts = uniform_firsts(edges.len(), order);
                    let styles = face_mask_styles(&edges, &firsts);
                    let found = assert_picture_styles(&name, &edges, &styles, &firsts);
                    kinds = [0, 1, 2].map(|kind| kinds[kind] + found[kind]);
                    let refused = face
                        .render_mask(&arrangement, pictures, 8, 8)
                        .err()
                        .map(|error| error.to_string());
                    assert_eq!(
                        refused.as_deref(),
                        Some(
                            format!(
                                "cannot render face mask: the picture {pictures} is not below the {pictures} pictures of the arrangement"
                            )
                            .as_str()
                        ),
                        "{name}"
                    );
                    arrangements += 1;
                }
            }
        }
        println!(
            "{arrangements} arrangements, edges (paired with themselves, paired in one picture, paired across two pictures): {kinds:?}"
        );
        assert_eq!((arrangements, kinds), (3399, [16670, 10058, 29884]));
    }

    #[test]
    fn face_masks_of_several_face_types_share_one_code_colour_for_each_meeting_and_mark_its_ends() {
        let mut kinds = [0; 3];
        let mut arrangements = 0;
        for &(text, marks) in &SEVERAL_SET {
            let (_, face) = several_face(text, marks);
            for arrangement in least_arrangements(&face) {
                let name = format!("{text} {} {}", marks_text(marks), arrangement.id());
                let edges = face
                    .edges(&arrangement)
                    .unwrap_or_else(|error| panic!("{name}: {error}"));
                let firsts = several_firsts(&face, &arrangement);
                let styles = face_mask_styles(&edges, &firsts);
                let found = assert_picture_styles(&name, &edges, &styles, &firsts);
                kinds = [0, 1, 2].map(|kind| kinds[kind] + found[kind]);
                let pictures = u32::try_from(firsts.len()).unwrap_or(u32::MAX);
                let refused = face
                    .render_mask(&arrangement, pictures, 8, 8)
                    .err()
                    .map(|error| error.to_string());
                assert_eq!(
                    refused,
                    Some(format!(
                        "cannot render face mask: the picture {pictures} is not below the {pictures} pictures of the arrangement"
                    )),
                    "{name}"
                );
                arrangements += 1;
            }
        }
        println!(
            "{arrangements} arrangements, edges (paired with themselves, paired in one picture, paired across two pictures): {kinds:?}"
        );
        assert_eq!((arrangements, kinds), (64, [100, 82, 2818]));
    }

    #[test]
    fn face_mask_of_each_picture_shows_its_turned_face_with_its_borders_marks_and_tick() {
        let mut content = Content::default();
        let mut reference = Reference::new();
        let mut masks = 0;
        for tiling in picture_tilings() {
            let domain = domain_with_settings(tiling.signature, &[]);
            let face = whole_face(&domain, tiling.marks);
            let order = usize::try_from(face.index() / 2).unwrap_or(usize::MAX);
            for arrangement in picture_arrangements(&face, tiling) {
                let name = format!("{} {}", tiling.name(), arrangement.id());
                let edges = face
                    .edges(&arrangement)
                    .unwrap_or_else(|error| panic!("{name}: {error}"));
                let styles = face_mask_styles(&edges, &uniform_firsts(edges.len(), order));
                let exact = exact_pictures(
                    &mut reference,
                    tiling.signature,
                    &domain,
                    tiling.marks,
                    arrangement.table(),
                );
                let polygons = face.picture_polygons(&arrangement);
                assert_eq!(polygons.len(), exact.len(), "{name}");
                let exact_polygons: Vec<Vec<Flat>> = exact
                    .iter()
                    .map(|picture| picture.polygon.clone())
                    .collect();
                let exact_placements =
                    ExactPlacement::arrangement(&exact_polygons, TICK_SIZE, TICK_SIZE);
                for (picture, (polygon, exact)) in polygons.iter().zip(&exact).enumerate() {
                    let name = format!("{name} picture {picture}");
                    let styles = &styles[picture * order..(picture + 1) * order];
                    for size in MASK_SIZES {
                        let placement =
                            Placement::arrangement(&polygons, size[0], size[1])[picture];
                        let pixels = draw_mask(
                            polygon,
                            placement,
                            styles,
                            Tick::OnEdgeZero,
                            size[0],
                            size[1],
                            1,
                        );
                        check_mask_content(
                            &name,
                            (polygon, placement),
                            styles,
                            Tick::OnEdgeZero,
                            &pixels,
                            size,
                            &mut content,
                        );
                    }
                    let pixels = draw_mask(
                        polygon,
                        Placement::arrangement(&polygons, TICK_SIZE, TICK_SIZE)[picture],
                        styles,
                        Tick::OnEdgeZero,
                        TICK_SIZE,
                        TICK_SIZE,
                        1,
                    );
                    check_tick(
                        &mut reference,
                        &name,
                        (&exact.polygon, &exact_placements[picture]),
                        styles,
                        &pixels,
                    );
                    masks += 1;
                }
            }
        }
        content.report();
        println!("{masks} face masks");
        assert_eq!(masks, 32);
        assert!(
            content.white >= 120_000
                && content.black >= 120_000
                && content.coded >= 2500
                && content.marked >= 800
        );
    }

    fn check_tick(
        reference: &mut Reference,
        name: &str,
        (polygon, placement): (&[Flat], &ExactPlacement),
        styles: &[EdgeStyle],
        pixels: &[u8],
    ) {
        let border = f64::from(TICK_SIZE) / 64.0;
        let count = polygon.len();
        for (edge, style) in styles.iter().enumerate() {
            let ends = [&polygon[(edge + count - 1) % count], &polygon[edge]];
            for (depth, beyond_border) in [(0.5, false), (2.0, true), (3.0, true)] {
                let [across, down] = placement.inside_edge(reference, ends, 0.5, depth * border);
                let found = pixel(
                    pixels,
                    TICK_SIZE,
                    texel_index(across, TICK_SIZE),
                    texel_index(down, TICK_SIZE),
                );
                let wanted = match (edge, beyond_border) {
                    (0, _) => [0; 3],
                    (_, false) => style.code,
                    (_, true) => [u8::MAX; 3],
                };
                assert_eq!(
                    found, wanted,
                    "{name}: edge {edge} at {depth} border depths"
                );
            }
        }
    }

    fn is_normal(colouring: &Colouring, sides: usize) -> bool {
        let table = colouring.table();
        let colours = table.index();
        let generators: Vec<Vec<u32>> = (0..sides)
            .map(|side| {
                (0..colours)
                    .map(|coset| {
                        let letter =
                            escher_group::Letter::generator(u8::try_from(side).unwrap_or(u8::MAX));
                        table
                            .image(u32::try_from(coset).unwrap_or(u32::MAX), letter)
                            .unwrap_or_else(|| panic!("{} has a gap", colouring.id()))
                    })
                    .collect()
            })
            .collect();
        let identity: Vec<u32> = (0..u32::try_from(colours).unwrap_or(u32::MAX)).collect();
        let mut group: HashSet<Vec<u32>> = HashSet::from([identity.clone()]);
        let mut queue = vec![identity];
        while let Some(element) = queue.pop() {
            for generator in &generators {
                let product: Vec<u32> = element
                    .iter()
                    .map(|&coset| generator[usize::try_from(coset).unwrap_or(usize::MAX)])
                    .collect();
                if group.insert(product.clone()) {
                    if group.len() > colours {
                        return false;
                    }
                    queue.push(product);
                }
            }
        }
        group.len() == colours
    }

    fn end_images(domain: &FundamentalDomain, side: usize) -> [usize; 2] {
        let sides = domain.sides();
        let vertices = domain.vertices();
        let count = sides.len();
        let partner = sides[side].partner();
        let pairing = sides[side].pairing();
        let ends = [vertices[(side + count - 1) % count], vertices[side]];
        let targets = [vertices[(partner + count - 1) % count], vertices[partner]];
        ends.map(|end| {
            let image = pairing * end;
            let gaps = targets
                .map(|target| (image - target).abs().max_element() / target.abs().max_element());
            let near = usize::from(gaps[1] < gaps[0]);
            assert!(
                gaps[near] < 1e-9 && gaps[1 - near] > 1e-6,
                "the pairing of side {side} maps an end {gaps:?} from the ends of side {partner}"
            );
            near
        })
    }

    fn carrying_contradicts(
        meetings: &[(usize, usize, usize, usize)],
        ends: &[[usize; 2]],
        edges: usize,
        first: usize,
    ) -> bool {
        let count = ends.len();
        let mut marked: Vec<Option<usize>> = vec![None; edges];
        marked[first] = Some(0);
        let mut queue = vec![first];
        while let Some(edge) = queue.pop() {
            let end = marked[edge].unwrap_or_else(|| panic!("edge {edge} has no mark"));
            for &(one, side, other, partner) in meetings {
                let (to, to_end) = if one * count + side == edge {
                    (other * count + partner, ends[side][end])
                } else if other * count + partner == edge {
                    (one * count + side, ends[partner][end])
                } else {
                    continue;
                };
                match marked[to] {
                    None => {
                        marked[to] = Some(to_end);
                        queue.push(to);
                    }
                    Some(existing) if existing != to_end => return true,
                    Some(_) => {}
                }
            }
        }
        false
    }

    #[test]
    fn seam_classes_join_edges_that_meet_with_one_code_colour_and_marked_ends_together() {
        const COLOURS: u32 = 4;
        let (mut colourings, mut largest, mut both, mut normal_classes) = (0, 0, 0, 0);
        let mut largest_at = String::new();
        for (name, domain) in DOMAINS.iter() {
            let sides = domain.sides();
            let count = sides.len();
            let ends: Vec<[usize; 2]> = (0..count).map(|side| end_images(domain, side)).collect();
            let mut list = valid_colourings(domain, COLOURS);
            list.push(colouring(domain, "1a"));
            for colouring in &list {
                let id = colouring.id();
                let masks = masks_of(name, domain, colouring);
                let styles = &masks.edges;
                let total = styles.iter().map(|style| style.class).max().unwrap_or(0) + 1;
                let firsts: Vec<usize> = (0..total)
                    .map(|class| {
                        styles
                            .iter()
                            .position(|style| style.class == class)
                            .unwrap_or_else(|| panic!("{name} {id}: class {class} is empty"))
                    })
                    .collect();
                assert!(
                    firsts.is_sorted(),
                    "{name} {id}: the classes are out of order"
                );
                let mut meetings = Vec::new();
                for (side, data) in sides.iter().enumerate() {
                    let partner = data.partner();
                    for (first, second) in neighbour_pairs(colouring, side, count) {
                        let (one, other) = (
                            styles[first * count + side],
                            styles[second * count + partner],
                        );
                        assert_eq!(
                            one.class, other.class,
                            "{name} {id}: edge {side} of colour {first} meets edge {partner} of colour {second}"
                        );
                        if one.marks != [true, true] {
                            let marked = usize::from(one.marks[1]);
                            let target = usize::from(other.marks[1]);
                            assert_eq!(
                                ends[side][marked], target,
                                "{name} {id}: the marked end of edge {side} of colour {first} meets an unmarked end of edge {partner} of colour {second}"
                            );
                        }
                        meetings.push((first, side, second, partner));
                    }
                }
                let normal = is_normal(colouring, count);
                for class in 0..total {
                    let members: Vec<usize> = (0..styles.len())
                        .filter(|&edge| styles[edge].class == class)
                        .collect();
                    for &edge in &members {
                        assert_eq!(styles[edge].code, code_colour(class, total), "{name} {id}");
                    }
                    let doubled = members
                        .iter()
                        .all(|&edge| styles[edge].marks == [true, true]);
                    assert!(
                        doubled
                            || members
                                .iter()
                                .all(|&edge| styles[edge].marks[0] != styles[edge].marks[1]),
                        "{name} {id}: class {class} mixes its marks"
                    );
                    assert_eq!(
                        doubled,
                        carrying_contradicts(&meetings, &ends, styles.len(), members[0]),
                        "{name} {id}: class {class}"
                    );
                    both += usize::from(doubled);
                    if members.len() > largest {
                        largest = members.len();
                        largest_at = format!("{name} {id} class {class}");
                    }
                    if normal {
                        normal_classes += 1;
                        assert!(
                            members.len() <= 2,
                            "{name} {id}: class {class} has {} edges",
                            members.len()
                        );
                    }
                }
                colourings += 1;
            }
        }
        println!(
            "{colourings} colourings: the largest seam class has {largest} edges ({largest_at}), {both} classes are marked at both ends, {normal_classes} classes of normal colourings"
        );
        assert!(colourings >= 1000, "{colourings}");
    }

    #[test]
    fn seam_classes_mark_every_edge_of_a_class_at_both_ends_when_an_edge_is_reached_with_both_ends()
    {
        let meetings = [vec![1], vec![0], vec![2], Vec::new()];
        let styles = seam_classes(&meetings, &[false, true]);
        let classes: Vec<usize> = styles.iter().map(|style| style.class).collect();
        let marks: Vec<[bool; 2]> = styles.iter().map(|style| style.marks).collect();
        assert_eq!(classes, [0, 0, 1, 2]);
        assert_eq!(
            marks,
            [[true, true], [true, true], [true, false], [true, false]]
        );
    }

    #[test]
    fn golden_mask_is_drawn_with_the_four_by_four_sample_grid() {
        for golden in &MASK_GOLDEN_IMAGES {
            let (_, _, masks) = golden.masks();
            let colour = usize::try_from(golden.colour).unwrap_or(usize::MAX);
            let pixels = golden.pixels();
            assert_eq!(
                pixels,
                masks.mask(colour, GOLDEN_SIZE, GOLDEN_SIZE, MASK_GRID),
                "{}",
                golden.file
            );
            let between = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .filter(|pixel| {
                    **pixel != [0; 3]
                        && **pixel != [255; 3]
                        && masks.edges.iter().all(|style| **pixel != style.code)
                })
                .count();
            assert!(
                between >= 100,
                "{}: {between} antialiased pixels",
                golden.file
            );
        }
    }

    fn seam_samples(domain: &FundamentalDomain) -> Vec<DVec3> {
        const STEPS: u32 = 48;
        let step = |index: u32| 2.0 * (f64::from(index) + 0.5) / f64::from(STEPS) - 1.0;
        let mut points = Vec::new();
        for row in 0..STEPS {
            for column in 0..STEPS {
                let position = DVec2::new(step(column), step(row));
                match domain.geometry() {
                    Geometry::Hyperbolic if position.length() < 0.9 => {
                        points.extend(hyperbolic::from_disk(0.9 * position));
                    }
                    Geometry::Euclidean => points.push(DVec3::from((4.0 * position, 1.0))),
                    Geometry::Spherical if position.length() < 1.0 => {
                        let height = (1.0 - position.length_squared()).sqrt();
                        points.push(DVec3::from((position, height)));
                        points.push(DVec3::from((position, -height)));
                    }
                    Geometry::Hyperbolic | Geometry::Spherical => {}
                }
            }
        }
        points
    }

    fn folded_colour(
        domain: &FundamentalDomain,
        colouring: &Colouring,
        point: DVec3,
    ) -> Option<(usize, Vec<usize>, DVec3)> {
        let mut sides = Vec::new();
        let result = fold(domain, point, 1500, |side| sides.push(side));
        if !result.converged() {
            return None;
        }
        let bytes: Vec<u8> = sides
            .iter()
            .map(|&side| u8::try_from(side).unwrap_or(u8::MAX))
            .collect();
        let colour = usize::try_from(colouring.colour_of(&bytes)?).ok()?;
        Some((colour, sides, result.point()))
    }

    const SEAM_TOLERANCE: f64 = 1e-6;

    const SEAM_IMAGE: u32 = 96;

    fn seam_scenes() -> Vec<(String, FundamentalDomain, Colouring)> {
        let mut scenes = Vec::new();
        for (signature, id) in [
            ("*732", "2a"),
            ("*642", "4a"),
            ("o", "5c"),
            ("22x", "6e"),
            ("4*2", "4c"),
            ("3*2", "4a"),
        ] {
            let domain = domain_with_settings(signature, &[]);
            let colouring = golden_colouring(&domain, id);
            scenes.push((format!("{signature} {id}"), domain, colouring));
        }
        let rotation = domain_with_settings("732", &[]);
        let first = valid_colourings(&rotation, 7)
            .into_iter()
            .find(|colouring| colouring.colours() >= 2)
            .unwrap_or_else(|| panic!("732 has no valid colouring up to 7 colours"));
        scenes.insert(2, (format!("732 {}", first.id()), rotation, first));
        scenes
    }

    fn observed_pairs(
        name: &str,
        domain: &FundamentalDomain,
        colouring: &Colouring,
        map: &MaskMap,
    ) -> (Vec<BTreeSet<(usize, usize)>>, usize) {
        const OFFSET: f64 = 1e-3;
        let sides = domain.sides();
        let count = sides.len();
        let near = |side: usize, offset: f64| {
            let middle = (map.charts[(side + count - 1) % count] + map.charts[side]) / 2.0;
            uncharted(map.geometry, middle + offset * (middle - map.chart_centre))
        };
        let mut observed: Vec<BTreeSet<(usize, usize)>> = vec![BTreeSet::new(); count];
        let mut tiles = HashSet::new();
        for point in seam_samples(domain) {
            let Some((colour, path, _)) = folded_colour(domain, colouring, point) else {
                continue;
            };
            if !tiles.insert(path.clone()) {
                continue;
            }
            let place = |point: DVec3| {
                path.iter().rev().fold(point, |moved, &side| {
                    sides[sides[side].partner()].pairing() * moved
                })
            };
            for (side, pairs) in observed.iter_mut().enumerate() {
                let inside = folded_colour(domain, colouring, place(near(side, -OFFSET)));
                let outside = folded_colour(domain, colouring, place(near(side, OFFSET)));
                if let (Some((same, ..)), Some((across, ..))) = (inside, outside) {
                    assert_eq!(
                        same, colour,
                        "{name}: the tile {path:?} changes colour inside side {side}"
                    );
                    pairs.insert((colour, across));
                }
            }
        }
        (observed, tiles.len())
    }

    fn across_fraction(one: [bool; 2], other: [bool; 2], kind: PairingKind, fraction: f64) -> f64 {
        if one == [true, true] {
            return if matches!(kind, PairingKind::Rotation | PairingKind::Translation) {
                1.0 - fraction
            } else {
                fraction
            };
        }
        let from_mark = if one[0] { fraction } else { 1.0 - fraction };
        if other[0] { from_mark } else { 1.0 - from_mark }
    }

    fn seam_values(
        reference: &mut Reference,
        name: &str,
        domain: &FundamentalDomain,
        colouring: &Colouring,
        map: &MaskMap,
    ) -> (usize, f64) {
        let sides = domain.sides();
        let count = sides.len();
        let colours = usize::try_from(colouring.colours()).unwrap_or(usize::MAX);
        let images: Vec<Image> = (0..colours)
            .map(|colour| gradient(colour, SEAM_IMAGE))
            .collect();
        let fill = CosetFill::new(
            domain,
            colouring.clone(),
            images.iter().cloned().map(Paint::Image).collect(),
        )
        .unwrap_or_else(|error| panic!("{name}: {error}"));
        let options = RenderOptions::builder(Fill::Cosets(fill))
            .build()
            .unwrap_or_else(|error| panic!("{error}"));
        let render = renderer(domain, options);
        let placement = Placement::new(map.polygon(), SEAM_IMAGE, SEAM_IMAGE);
        let table = decode_table();
        let masks = masks_of(name, domain, colouring);
        let exact = exact_mask(reference, domain);
        let polygon = map.polygon();
        let value_on_edge = |colour: usize, side: usize, fraction: f64| {
            let start = polygon[(side + count - 1) % count];
            let flat = start + fraction * (polygon[side] - start);
            sample(
                &images[colour],
                placement.position(flat),
                Colour::WHITE.linear(),
                &table,
            )
        };
        let (mut values, mut worst) = (0, 0.0_f64);
        for (side, data) in sides.iter().enumerate() {
            let partner = data.partner();
            for quarters in 1..=3 {
                let fraction = f64::from(quarters) / 4.0;
                let on_side = exact.along_side(reference, side, &num(fraction));
                let chart = map
                    .chart(exact_point(reference, &on_side))
                    .unwrap_or_else(|| panic!("{name}: side {side} leaves the chart"));
                let step = 1e-7 * (chart - map.chart_centre);
                let inside = uncharted(map.geometry, chart - step);
                let Some((_, path, folded)) =
                    folded_colour(domain, colouring, uncharted(map.geometry, chart + step))
                else {
                    panic!("{name}: the point across side {side} does not fold");
                };
                assert_eq!(path, [side], "{name}: the point across side {side}");
                for (first, second) in neighbour_pairs(colouring, side, count) {
                    let along = across_fraction(
                        masks.edges[first * count + side].marks,
                        masks.edges[second * count + partner].marks,
                        data.kind(),
                        fraction,
                    );
                    for (value, expected) in [
                        (
                            render.texture(first, &images[first], inside),
                            value_on_edge(first, side, fraction),
                        ),
                        (
                            render.texture(second, &images[second], folded),
                            value_on_edge(second, partner, along),
                        ),
                    ] {
                        let gap = value
                            .linear()
                            .iter()
                            .zip(expected)
                            .map(|(left, right)| (left - right).abs())
                            .fold(0.0, f64::max);
                        worst = worst.max(gap);
                        assert!(
                            gap <= SEAM_TOLERANCE,
                            "{name}: side {side} at {fraction} for colours {first} and {second} differs by {gap:e}"
                        );
                        values += 1;
                    }
                }
            }
        }
        (values, worst)
    }

    #[test]
    fn seams_pair_the_colours_of_the_closure_and_match_the_masks_laid_edge_to_edge() {
        let mut reference = Reference::new();
        let (mut tiles, mut pairs, mut values, mut worst) = (0, 0, 0, 0.0_f64);
        for (name, domain, colouring) in seam_scenes() {
            let map = mask_map(&name, &domain);
            let count = domain.sides().len();
            let (observed, seen) = observed_pairs(&name, &domain, &colouring, &map);
            for (side, found) in observed.iter().enumerate() {
                let closure: BTreeSet<(usize, usize)> = neighbour_pairs(&colouring, side, count)
                    .into_iter()
                    .collect();
                assert_eq!(*found, closure, "{name}: the pairs across side {side}");
                pairs += closure.len();
            }
            tiles += seen;
            let (compared, gap) = seam_values(&mut reference, &name, &domain, &colouring, &map);
            values += compared;
            worst = worst.max(gap);
        }
        println!(
            "{tiles} tiles give every pair of the closure, {pairs} pairs; {values} render values match the masks laid edge to edge within {worst:e}, allowed {SEAM_TOLERANCE:e}"
        );
        assert!(tiles >= 1000 && values >= 2000, "{tiles} {values}");
    }

    #[test]
    fn code_colours_differ_pairwise_and_are_neither_white_nor_black() {
        for total in 1..=896 {
            let mut seen = HashSet::new();
            for class in 0..total {
                let code = code_colour(class, total);
                assert!(
                    code != [0; 3] && code != [255; 3],
                    "{class} of {total}: {code:?}"
                );
                assert!(seen.insert(code), "{class} of {total} repeats {code:?}");
            }
        }
        assert_eq!(code_colour(0, 6), [255, 0, 0]);
        assert_eq!(code_colour(1, 6), [255, 255, 0]);
        assert_eq!(code_colour(2, 6), [0, 255, 0]);
        assert_eq!(code_colour(3, 6), [0, 255, 255]);
        assert_eq!(code_colour(4, 6), [0, 0, 255]);
        assert_eq!(code_colour(5, 6), [255, 0, 255]);
        assert_eq!(code_colour(1, 12), [255, 128, 0]);
    }
}
