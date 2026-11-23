use escher_domain::{FundamentalDomain, PairingKind, ShapeParameters, parameters};
use escher_geometry::hyperbolic::{disk_from_band, disk_from_half_plane, disk_from_klein};
use escher_group::{
    Colouring, ColouringId, ColouringSet, IndexLimit, SearchBudget, colourings_of_index,
};
use escher_signature::{Geometry, Signature};
use glam::{DMat2, DMat3, DVec2, DVec3};

use crate::reference::{self, ExactMask, MaskRule, Measure, ReferenceExt, Wall};
use crate::{
    BorderUnit, Colour, CosetFill, Fill, HyperbolicModel, Image, Masks, Paint, RenderOptions,
    RenderOptionsBuilder, Renderer, Shading, encode_png,
};
use escher_reference::{Reference, matrix, vector};

pub(crate) struct GoldenImage {
    pub(crate) signature: &'static str,
    pub(crate) settings: &'static [(&'static str, f64)],
    pub(crate) rotation: [f64; 3],
    pub(crate) shading: Shading,
    pub(crate) model: HyperbolicModel,
    pub(crate) colouring: Option<&'static str>,
    pub(crate) border_unit: BorderUnit,
    pub(crate) border_width: f64,
    pub(crate) file: &'static str,
}

impl GoldenImage {
    pub(crate) fn domain(&self) -> FundamentalDomain {
        domain_with_settings(self.signature, self.settings)
    }

    pub(crate) fn builder_without_unit(&self, domain: &FundamentalDomain) -> RenderOptionsBuilder {
        let [even, odd] =
            GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
        let fill = match self.colouring {
            None => Fill::Parity([even, odd]),
            Some(id) => Fill::Cosets(coset_fill(domain, golden_colouring(domain, id))),
        };
        self.builder_with_fill(fill)
    }

    pub(crate) fn builder_with_fill(&self, fill: Fill) -> RenderOptionsBuilder {
        let [yaw, pitch, roll] = self.rotation.map(f64::to_radians);
        RenderOptions::builder(fill)
            .size(GOLDEN_SIZE, GOLDEN_SIZE)
            .samples(GOLDEN_GRID)
            .border_width(self.border_width)
            .rotation(yaw, pitch, roll)
            .shading(self.shading)
            .model(self.model)
    }

    pub(crate) fn options(&self, domain: &FundamentalDomain) -> RenderOptions {
        match self
            .builder_without_unit(domain)
            .border_unit(self.border_unit)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("the golden options of {} are rejected: {error}", self.file),
        }
    }
}

const fn golden(
    signature: &'static str,
    settings: &'static [(&'static str, f64)],
    file: &'static str,
) -> GoldenImage {
    GoldenImage {
        signature,
        settings,
        rotation: [0.0; 3],
        shading: Shading::Phong,
        model: HyperbolicModel::Disk,
        colouring: None,
        border_unit: BorderUnit::Pixels,
        border_width: 1.0,
        file,
    }
}

pub(crate) const GOLDEN_INTRINSIC_WIDTH: f64 = 0.05;

const fn intrinsic(signature: &'static str, file: &'static str) -> GoldenImage {
    GoldenImage {
        border_unit: BorderUnit::Intrinsic,
        border_width: GOLDEN_INTRINSIC_WIDTH,
        ..golden(signature, &[], file)
    }
}

const fn in_model(
    signature: &'static str,
    model: HyperbolicModel,
    file: &'static str,
) -> GoldenImage {
    GoldenImage {
        model,
        ..golden(signature, &[], file)
    }
}

const fn coset(signature: &'static str, id: &'static str, file: &'static str) -> GoldenImage {
    coset_with(signature, &[], id, file)
}

const fn coset_with(
    signature: &'static str,
    settings: &'static [(&'static str, f64)],
    id: &'static str,
    file: &'static str,
) -> GoldenImage {
    GoldenImage {
        signature,
        settings,
        rotation: [0.0; 3],
        shading: Shading::Phong,
        model: HyperbolicModel::Disk,
        colouring: Some(id),
        border_unit: BorderUnit::Pixels,
        border_width: 1.0,
        file,
    }
}

pub(crate) const GOLDEN_IMAGES: [GoldenImage; 39] = [
    golden("*732", &[], "7-3-2.png"),
    golden("*542", &[], "5-4-2.png"),
    golden("*433", &[], "4-3-3.png"),
    golden("*632", &[], "6-3-2.png"),
    golden("*442", &[], "4-4-2.png"),
    golden("*333", &[], "3-3-3.png"),
    golden("*2222", &[("aspect", 1.618)], "2-2-2-2.png"),
    golden("*532", &[], "5-3-2.png"),
    golden("*432", &[], "4-3-2.png"),
    golden("*332", &[], "3-3-2.png"),
    golden("*722", &[], "7-2-2.png"),
    golden("*55", &[], "5-5.png"),
    GoldenImage {
        signature: "*532",
        settings: &[],
        rotation: [20.0, 35.0, 0.0],
        shading: Shading::Phong,
        model: HyperbolicModel::Disk,
        colouring: None,
        border_unit: BorderUnit::Pixels,
        border_width: 1.0,
        file: "5-3-2-rotated.png",
    },
    GoldenImage {
        signature: "*532",
        settings: &[],
        rotation: [0.0; 3],
        shading: Shading::Flat,
        model: HyperbolicModel::Disk,
        colouring: None,
        border_unit: BorderUnit::Pixels,
        border_width: 1.0,
        file: "5-3-2-flat.png",
    },
    golden("732", &[], "rot-7-3-2.png"),
    golden("442", &[], "rot-4-4-2.png"),
    golden("2222", &[("aspect", 1.618)], "rot-2-2-2-2.png"),
    golden("532", &[], "rot-5-3-2.png"),
    golden("722", &[], "rot-7-2-2.png"),
    golden("55", &[], "rot-5-5.png"),
    golden("*3333", &[], "3-3-3-3.png"),
    GoldenImage {
        signature: "*3333",
        settings: &[("slice1.length", 2.5)],
        rotation: [0.0; 3],
        shading: Shading::Phong,
        model: HyperbolicModel::Disk,
        colouring: None,
        border_unit: BorderUnit::Pixels,
        border_width: 1.0,
        file: "3-3-3-3-slice-2.5.png",
    },
    golden("*22222", &[], "2-2-2-2-2.png"),
    golden("*5432", &[], "5-4-3-2.png"),
    golden("3333", &[], "rot-3-3-3-3.png"),
    golden("o", &[("skew", 0.25)], "wp-o-skew-0.25.png"),
    golden("**", &[("aspect", 2.0)], "wp-s-s-aspect-2.png"),
    golden("xx", &[], "wp-x-x.png"),
    golden("2*22", &[], "wp-2-s-2-2.png"),
    golden("4*2", &[], "wp-4-s-2.png"),
    golden("3*3", &[], "wp-3-s-3.png"),
    golden("2222", &[("skew", 0.25)], "rot-2-2-2-2-skew-0.25.png"),
    golden("3*2", &[], "sg-3-s-2.png"),
    in_model("*732", HyperbolicModel::HalfPlane, "7-3-2-half-plane.png"),
    in_model("*732", HyperbolicModel::Band, "7-3-2-band.png"),
    in_model("*732", HyperbolicModel::Klein, "7-3-2-klein.png"),
    intrinsic("*732", "7-3-2-intrinsic.png"),
    intrinsic("*632", "6-3-2-intrinsic.png"),
    intrinsic("*532", "5-3-2-intrinsic.png"),
];

pub(crate) struct ModelMap {
    pub(crate) model: HyperbolicModel,
    pub(crate) exact: reference::Model,
    pub(crate) to_disk: fn(DVec2) -> Option<(DVec2, DMat2)>,
    pub(crate) offset: DVec2,
    pub(crate) fills_height: bool,
}

impl ModelMap {
    pub(crate) fn pixel(&self, width: u32, height: u32) -> f64 {
        if self.fills_height {
            2.0 / f64::from(height)
        } else {
            disk_pixel(width, height)
        }
    }

    pub(crate) fn position(&self, width: u32, height: u32, column: u32, row: u32) -> DVec2 {
        pixel_centre(width, height, column, row, self.pixel(width, height)) + self.offset
    }
}

pub(crate) fn disk_pixel(width: u32, height: u32) -> f64 {
    2.0 / (15.0 / 16.0 * f64::from(width.min(height)))
}

pub(crate) fn pixel_centre(width: u32, height: u32, column: u32, row: u32, pixel: f64) -> DVec2 {
    DVec2::new(
        (f64::from(column) + 0.5 - f64::from(width) / 2.0) * pixel,
        (f64::from(height) / 2.0 - (f64::from(row) + 0.5)) * pixel,
    )
}

pub(crate) const FOLD_ANGLES: u32 = 512;

pub(crate) fn band_radius(band: i32) -> f64 {
    1.0 - 10.0_f64.powi(-band)
}

pub(crate) fn on_circle(radius: f64, index: u32, count: u32) -> DVec2 {
    let angle = core::f64::consts::TAU * f64::from(index) / f64::from(count);
    DVec2::new(radius * angle.cos(), radius * angle.sin())
}

pub(crate) fn reference_walls(domain: &FundamentalDomain) -> Vec<Wall> {
    domain
        .sides()
        .iter()
        .map(|side| Wall {
            covector: vector(side.covector().to_array()),
            pairing: matrix(side.pairing().to_cols_array()),
        })
        .collect()
}

pub(crate) const MODEL_MAPS: [ModelMap; 3] = [
    ModelMap {
        model: HyperbolicModel::HalfPlane,
        exact: reference::Model::HalfPlane,
        to_disk: disk_from_half_plane,
        offset: DVec2::Y,
        fills_height: true,
    },
    ModelMap {
        model: HyperbolicModel::Band,
        exact: reference::Model::Band,
        to_disk: disk_from_band,
        offset: DVec2::ZERO,
        fills_height: true,
    },
    ModelMap {
        model: HyperbolicModel::Klein,
        exact: reference::Model::Klein,
        to_disk: disk_from_klein,
        offset: DVec2::ZERO,
        fills_height: false,
    },
];

pub(crate) const COSET_GOLDEN_IMAGES: [GoldenImage; 18] = [
    coset("*642", "4a", "cos-6-4-2-4a.png"),
    coset("*732", "8a", "cos-7-3-2-8a.png"),
    coset("432", "3a", "cos-rot-4-3-2-3a.png"),
    coset("532", "5a", "cos-rot-5-3-2-5a.png"),
    coset("*2222", "4a", "cos-2-2-2-2-4a.png"),
    coset("*532", "5a", "cos-5-3-2-5a.png"),
    coset("*22222", "8ekf", "cos-2-2-2-2-2-8ekf.png"),
    coset_with("o", &[("skew", 0.25)], "5c", "cos-wp-o-5c.png"),
    coset("**", "6g", "cos-wp-s-s-6g.png"),
    coset("xx", "6f", "cos-wp-x-x-6f.png"),
    coset("*x", "4c", "cos-wp-s-x-4c.png"),
    coset("22*", "6h", "cos-wp-2-2-s-6h.png"),
    coset("22x", "6e", "cos-wp-2-2-x-6e.png"),
    coset("2*22", "4d", "cos-wp-2-s-2-2-4d.png"),
    coset("4*2", "4c", "cos-wp-4-s-2-4c.png"),
    coset("3*3", "3a", "cos-wp-3-s-3-3a.png"),
    coset("3*2", "4a", "cos-sg-3-s-2-4a.png"),
    coset("2*5", "5a", "cos-sg-2-s-5-5a.png"),
];

pub(crate) struct MaskGolden {
    pub(crate) signature: &'static str,
    pub(crate) colouring: &'static str,
    pub(crate) colour: u32,
    pub(crate) file: &'static str,
}

impl MaskGolden {
    pub(crate) fn masks(&self) -> (FundamentalDomain, Colouring, Masks) {
        let domain = domain_with_settings(self.signature, &[]);
        let colouring = golden_colouring(&domain, self.colouring);
        let masks = masks(&domain, &colouring);
        (domain, colouring, masks)
    }

    pub(crate) fn pixels(&self) -> Vec<u8> {
        mask(&self.masks().2, self.colour, GOLDEN_SIZE, GOLDEN_SIZE)
    }
}

pub(crate) const MASK_GOLDEN_IMAGES: [MaskGolden; 1] = [MaskGolden {
    signature: "*642",
    colouring: "4a",
    colour: 1,
    file: "mask-6-4-2-4a-1.png",
}];

pub(crate) struct ImageFillGolden {
    pub(crate) golden: GoldenImage,
    pub(crate) solid: &'static [usize],
}

impl ImageFillGolden {
    pub(crate) fn fill(&self, domain: &FundamentalDomain) -> CosetFill {
        let id = self
            .golden
            .colouring
            .unwrap_or_else(|| panic!("{} has no colouring", self.golden.file));
        let colouring = golden_colouring(domain, id);
        let paints = image_paints(domain, &colouring, GOLDEN_SIZE, self.solid);
        CosetFill::new(domain, colouring, paints).unwrap_or_else(|error| {
            panic!(
                "the image fill of {} is rejected: {error}",
                self.golden.file
            )
        })
    }

    pub(crate) fn options(&self, domain: &FundamentalDomain) -> RenderOptions {
        self.golden
            .builder_with_fill(Fill::Cosets(self.fill(domain)))
            .build()
            .unwrap_or_else(|error| {
                panic!(
                    "the golden options of {} are rejected: {error}",
                    self.golden.file
                )
            })
    }
}

pub(crate) const IMAGE_FILL_GOLDEN_IMAGES: [ImageFillGolden; 3] = [
    ImageFillGolden {
        golden: coset("*642", "4a", "img-6-4-2-4a.png"),
        solid: &[],
    },
    ImageFillGolden {
        golden: coset("*632", "2a", "img-6-3-2-2a.png"),
        solid: &[1],
    },
    ImageFillGolden {
        golden: coset("*532", "5a", "img-5-3-2-5a.png"),
        solid: &[],
    },
];

pub(crate) fn masks(domain: &FundamentalDomain, colouring: &Colouring) -> Masks {
    Masks::new(domain, colouring)
        .unwrap_or_else(|error| panic!("the masks of {} are rejected: {error}", colouring.id()))
}

pub(crate) fn mask(masks: &Masks, colour: u32, width: u32, height: u32) -> Vec<u8> {
    masks
        .render(colour, width, height)
        .unwrap_or_else(|error| panic!("the mask of colour {colour} is rejected: {error}"))
}

const MARK: [u8; 3] = [20, 20, 20];

const MARK_BARS: [[f64; 4]; 3] = [
    [0.2, 0.4, 0.2, 0.8],
    [0.2, 0.8, 0.2, 0.35],
    [0.2, 0.6, 0.45, 0.6],
];

pub(crate) fn painted(mask: &[u8], width: u32, height: u32, colour: [u8; 3]) -> Vec<u8> {
    let columns = usize::try_from(width).unwrap_or(usize::MAX);
    let inside = |index: usize| mask.get(3 * index..3 * index + 3) == Some(&[u8::MAX; 3][..]);
    let cells: Vec<(usize, usize)> = (0..mask.len() / 3)
        .filter(|&index| inside(index))
        .map(|index| (index % columns, index / columns))
        .collect();
    assert!(
        !cells.is_empty(),
        "the mask of a {width}x{height} image is black"
    );
    let low = cells
        .iter()
        .fold((usize::MAX, usize::MAX), |(column, row), &(x, y)| {
            (column.min(x), row.min(y))
        });
    let high = cells
        .iter()
        .fold((0, 0), |(column, row), &(x, y)| (column.max(x), row.max(y)));
    let span =
        |low: usize, high: usize| f64::from(u32::try_from(high - low + 1).unwrap_or(u32::MAX));
    let relative = |value: usize, low: usize, high: usize| {
        (f64::from(u32::try_from(value - low).unwrap_or(u32::MAX)) + 0.5) / span(low, high)
    };
    let mut pixels: Vec<u8> = colour.repeat(mask.len() / 3);
    for &(column, row) in &cells {
        let across = relative(column, low.0, high.0);
        let down = relative(row, low.1, high.1);
        let marked = MARK_BARS.iter().any(|&[left, right, top, bottom]| {
            (left..right).contains(&across) && (top..bottom).contains(&down)
        });
        if marked {
            let start = 3 * (row * columns + column);
            pixels[start..start + 3].copy_from_slice(&MARK);
        }
    }
    pixels
}

pub(crate) fn image_of(width: u32, height: u32, pixels: &[u8]) -> Image {
    let file = encode_png(width, height, pixels)
        .unwrap_or_else(|error| panic!("a {width}x{height} test image is rejected: {error}"));
    Image::decode_png(file.as_slice())
        .unwrap_or_else(|error| panic!("a {width}x{height} test image cannot be decoded: {error}"))
}

pub(crate) fn painted_image(
    masks: &Masks,
    colour: u32,
    width: u32,
    height: u32,
) -> (Vec<u8>, Image) {
    let index = usize::try_from(colour).unwrap_or(usize::MAX);
    let paint = GOLDEN_COSET_FILL[index % GOLDEN_COSET_FILL.len()];
    let pixels = painted(&mask(masks, colour, width, height), width, height, paint);
    let image = image_of(width, height, &pixels);
    (pixels, image)
}

pub(crate) fn image_paints(
    domain: &FundamentalDomain,
    colouring: &Colouring,
    size: u32,
    solid: &[usize],
) -> Vec<Paint> {
    let masks = masks(domain, colouring);
    (0..masks.count())
        .map(|colour| {
            let index = usize::try_from(colour).unwrap_or(usize::MAX);
            if solid.contains(&index) {
                let [red, green, blue] = GOLDEN_COSET_FILL[index];
                Paint::Solid(Colour::from_srgb8(red, green, blue))
            } else {
                Paint::Image(painted_image(&masks, colour, size, size).1)
            }
        })
        .collect()
}

pub(crate) const GOLDEN_FILL: [[u8; 3]; 2] = [[245, 222, 179], [70, 130, 180]];

pub(crate) const GOLDEN_COSET_FILL: [[u8; 3]; 8] = [
    [245, 222, 179],
    [70, 130, 180],
    [220, 20, 60],
    [46, 139, 87],
    [255, 215, 0],
    [106, 90, 205],
    [255, 140, 0],
    [64, 224, 208],
];

pub(crate) fn colourings_of(domain: &FundamentalDomain, colours: u32) -> Vec<Colouring> {
    colourings_in(domain, colours, ColouringSet::All)
}

fn colourings_in(domain: &FundamentalDomain, colours: u32, set: ColouringSet) -> Vec<Colouring> {
    let presentation = domain
        .side_presentation()
        .unwrap_or_else(|error| panic!("the domain has no side presentation: {error}"));
    let limit = IndexLimit::new(colours)
        .unwrap_or_else(|error| panic!("{colours} colours are too many: {error}"));
    let found = colourings_of_index(&presentation, set, limit, SearchBudget::DEFAULT)
        .unwrap_or_else(|error| panic!("the search for {colours} colours fails: {error}"));
    assert!(
        found.is_complete(),
        "the search for {colours} colours is not complete"
    );
    found.list().to_vec()
}

pub(crate) fn find_colouring(colourings: &[Colouring], id: &str) -> Colouring {
    let id: ColouringId = id
        .parse()
        .unwrap_or_else(|error| panic!("{id:?} is rejected: {error}"));
    colourings
        .iter()
        .find(|colouring| colouring.id() == id)
        .cloned()
        .unwrap_or_else(|| panic!("the domain has no colouring {id}"))
}

pub(crate) fn colouring(domain: &FundamentalDomain, id: &str) -> Colouring {
    colouring_in(domain, id, ColouringSet::All)
}

pub(crate) fn golden_colouring(domain: &FundamentalDomain, id: &str) -> Colouring {
    colouring_in(domain, id, ColouringSet::ValidClasses)
}

fn colouring_in(domain: &FundamentalDomain, id: &str, set: ColouringSet) -> Colouring {
    let colours = id
        .parse::<ColouringId>()
        .unwrap_or_else(|error| panic!("{id:?} is rejected: {error}"))
        .colours();
    find_colouring(&colourings_in(domain, colours, set), id)
}

pub(crate) fn coset_fill(domain: &FundamentalDomain, colouring: Colouring) -> CosetFill {
    let id = colouring.id();
    let colours = GOLDEN_COSET_FILL
        .iter()
        .take(usize::try_from(colouring.colours()).unwrap_or(usize::MAX))
        .map(|&[red, green, blue]| Paint::Solid(Colour::from_srgb8(red, green, blue)))
        .collect();
    CosetFill::new(domain, colouring, colours)
        .unwrap_or_else(|error| panic!("the coset fill of {id} is rejected: {error}"))
}

pub(crate) fn renderer(domain: &FundamentalDomain, options: RenderOptions) -> Renderer {
    Renderer::new(domain, options)
        .unwrap_or_else(|error| panic!("the renderer is rejected: {error}"))
}

pub(crate) const GOLDEN_SIZE: u32 = 192;

pub(crate) const GOLDEN_GRID: u32 = 4;

pub(crate) const ALLOWED_PIXELS: usize = 16;

pub(crate) fn linear_table() -> Vec<f64> {
    let mut reference = Reference::new();
    (0..=u8::MAX)
        .map(|byte| {
            let linear = reference.srgb_decode(byte);
            reference.approximate(&linear)
        })
        .collect()
}

#[derive(Debug)]
pub(crate) struct Comparison {
    pub(crate) differing: usize,
    pub(crate) within_limit: bool,
}

impl Comparison {
    pub(crate) fn within_allowance(&self) -> bool {
        self.within_limit && self.differing <= ALLOWED_PIXELS
    }
}

pub(crate) fn compare(first: &[u8], second: &[u8], grid: u32, linear: &[f64]) -> Comparison {
    let weight = 1.0 / f64::from(grid * grid);
    let value = |byte: u8| linear[usize::from(byte)];
    let mut comparison = Comparison {
        differing: 0,
        within_limit: first.len() == second.len(),
    };
    for (left, right) in first
        .as_chunks::<3>()
        .0
        .iter()
        .zip(second.as_chunks::<3>().0)
    {
        if left == right {
            continue;
        }
        comparison.differing += 1;
        for (&left, &right) in left.iter().zip(right) {
            let larger = left.max(right);
            let step = value(larger) - value(larger.saturating_sub(1));
            if (value(left) - value(right)).abs() > weight + step {
                comparison.within_limit = false;
            }
        }
    }
    comparison
}

pub(crate) fn domain_with_aspect(text: &str, aspect: Option<f64>) -> FundamentalDomain {
    let settings: Vec<(&str, f64)> = aspect
        .map(|aspect| ("aspect", aspect))
        .into_iter()
        .collect();
    domain_with_settings(text, &settings)
}

pub(crate) fn domain_with_settings(text: &str, settings: &[(&str, f64)]) -> FundamentalDomain {
    let signature: Signature = text
        .parse()
        .unwrap_or_else(|error| panic!("{text:?} is rejected: {error}"));
    let mut parameters = ShapeParameters::new();
    for &(name, value) in settings {
        parameters.set(name, value);
    }
    FundamentalDomain::build_with(&signature, &parameters)
        .unwrap_or_else(|error| panic!("{text} with {settings:?} has no domain: {error}"))
}

pub(crate) const FOLD_TRIPLES: [[u32; 3]; 7] = [
    [7, 3, 2],
    [5, 4, 2],
    [4, 4, 4],
    [4, 3, 3],
    [12, 3, 2],
    [100, 3, 2],
    [100, 100, 100],
];

pub(crate) const EUCLIDEAN_DOMAINS: [(&str, Option<f64>); 7] = [
    ("*632", None),
    ("*442", None),
    ("*333", None),
    ("*2222", Some(1.0)),
    ("*2222", Some(1.618)),
    ("*2222", Some(0.125)),
    ("*2222", Some(8.0)),
];

pub(crate) const SPHERICAL_DOMAINS: [&str; 13] = [
    "*332",
    "*432",
    "*532",
    "*222",
    "*322",
    "*522",
    "*(12)22",
    "*(100)22",
    "*22",
    "*33",
    "*55",
    "*(12)(12)",
    "*(100)(100)",
];

pub(crate) const SPHERICAL_GROUP_DOMAINS: [&str; 19] = [
    "3*2", "*", "x", "1", "2*", "3*", "5*", "(12)*", "(100)*", "2x", "3x", "5x", "(12)x", "(100)x",
    "2*2", "2*3", "2*5", "2*(12)", "2*(50)",
];

pub(crate) fn text(orders: [u32; 3]) -> String {
    let corners: String = orders
        .into_iter()
        .map(|order| {
            if order < 10 {
                order.to_string()
            } else {
                format!("({order})")
            }
        })
        .collect();
    format!("*{corners}")
}

pub(crate) fn rotation_text(starred: &str) -> &str {
    starred
        .strip_prefix('*')
        .unwrap_or_else(|| panic!("{starred} has no star"))
}

pub(crate) const FLATLESS: [&str; 3] = ["*222", "222", "2*"];

pub(crate) fn has_mask_polygon(signature: &str, domain: &FundamentalDomain) -> bool {
    domain.sides().len() >= 3 && !FLATLESS.contains(&signature)
}

pub(crate) fn mask_domains() -> Vec<(String, FundamentalDomain)> {
    let mut domains: Vec<(String, FundamentalDomain)> = Vec::new();
    let mut add = |name: String, domain: FundamentalDomain| {
        let signature = name.split(' ').next().unwrap_or_default().to_owned();
        if has_mask_polygon(&signature, &domain) {
            domains.push((name, domain));
        }
    };
    for signature in PLAN_TRIPLES {
        add(signature.to_owned(), domain_with_settings(signature, &[]));
    }
    for (signature, aspect) in EUCLIDEAN_DOMAINS {
        let suffix = aspect.map_or(String::new(), |aspect| format!(" aspect {aspect}"));
        add(
            format!("{signature}{suffix}"),
            domain_with_aspect(signature, aspect),
        );
        let rotation = rotation_text(signature);
        add(
            format!("{rotation}{suffix}"),
            domain_with_aspect(rotation, aspect),
        );
    }
    for signature in SPHERICAL_DOMAINS {
        add(signature.to_owned(), domain_with_settings(signature, &[]));
        let rotation = rotation_text(signature);
        add(rotation.to_owned(), domain_with_settings(rotation, &[]));
    }
    for triple in FOLD_TRIPLES {
        let rotation = text(triple)[1..].to_owned();
        let domain = domain_with_settings(&rotation, &[]);
        add(rotation, domain);
    }
    for shape in POLYGON_DOMAINS.iter().chain(&WALLPAPER_DOMAINS) {
        add(shape.name(), shape.domain());
    }
    for signature in SPHERICAL_GROUP_DOMAINS {
        add(signature.to_owned(), domain_with_settings(signature, &[]));
    }
    domains
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the assertion keeps the rounded order between 1 and 200"
)]
fn corner_order(angle: f64) -> u32 {
    let order = (core::f64::consts::PI / angle).round();
    assert!(
        (1.0..=200.0).contains(&order),
        "the corner angle {angle} has no order"
    );
    order as u32
}

pub(crate) fn mask_rule(domain: &FundamentalDomain) -> MaskRule {
    let sides = domain.sides();
    let mirrors = sides
        .iter()
        .all(|side| matches!(side.kind(), PairingKind::Mirror));
    let angles = domain.corner_angles();
    match (domain.geometry(), sides.len(), mirrors) {
        (Geometry::Euclidean, _, _) => MaskRule::Plane,
        (_, 3, true) => MaskRule::Triangle([angles[0], angles[1], angles[2]].map(corner_order)),
        (Geometry::Hyperbolic, 4.., true) | (Geometry::Hyperbolic, 6.., false) => MaskRule::Klein,
        (_, 4, false) => {
            MaskRule::Double([angles[0] / 2.0, angles[2] / 2.0, angles[3]].map(corner_order))
        }
        (geometry, count, _) => panic!("a {geometry:?} domain with {count} sides has no mask rule"),
    }
}

pub(crate) fn exact_mask(reference: &mut Reference, domain: &FundamentalDomain) -> ExactMask {
    let vertices: Vec<[f64; 3]> = domain.vertices().iter().map(DVec3::to_array).collect();
    let covectors: Vec<[f64; 3]> = domain
        .sides()
        .iter()
        .map(|side| side.covector().to_array())
        .collect();
    ExactMask::new(
        reference,
        Measure::of(domain.geometry()),
        &vertices,
        &covectors,
        mask_rule(domain),
    )
}

pub(crate) const PLAN_TRIPLES: [&str; 6] = ["*732", "*542", "*444", "*433", "*(12)32", "*(100)32"];

#[derive(Clone, Copy, Debug)]
pub(crate) enum Shape {
    Default,
    Named(&'static [(&'static str, f64)]),
    Every(f64),
}

pub(crate) struct PolygonShape {
    pub(crate) signature: &'static str,
    pub(crate) shape: Shape,
}

const fn polygon(signature: &'static str, shape: Shape) -> PolygonShape {
    PolygonShape { signature, shape }
}

pub(crate) const POLYGON_DOMAINS: [PolygonShape; 20] = [
    polygon("*3333", Shape::Default),
    polygon("*3333", Shape::Named(&[("slice1.length", 0.5)])),
    polygon("*3333", Shape::Named(&[("slice1.length", 2.5)])),
    polygon("*3333", Shape::Named(&[("slice1.length", 3.5)])),
    polygon("*22222", Shape::Default),
    polygon(
        "*22222",
        Shape::Named(&[("cornerpair1.length", 0.5), ("cornerpair2.length", 0.5)]),
    ),
    polygon(
        "*22222",
        Shape::Named(&[("cornerpair1.length", 3.5), ("cornerpair2.length", 3.5)]),
    ),
    polygon("*5432", Shape::Default),
    polygon("*5432", Shape::Named(&[("slice1.length", 0.5)])),
    polygon("*5432", Shape::Named(&[("slice1.length", 3.5)])),
    polygon("*3222", Shape::Default),
    polygon("*(100)2222", Shape::Default),
    polygon("*22222222", Shape::Default),
    polygon("*22222222", Shape::Every(0.5)),
    polygon("*22222222", Shape::Every(3.5)),
    polygon("3333", Shape::Default),
    polygon("22222", Shape::Default),
    polygon("5432", Shape::Default),
    polygon("3222", Shape::Default),
    polygon("22222222", Shape::Default),
];

const fn wallpaper(
    signature: &'static str,
    settings: &'static [(&'static str, f64)],
) -> PolygonShape {
    PolygonShape {
        signature,
        shape: Shape::Named(settings),
    }
}

pub(crate) const WALLPAPER_DOMAINS: [PolygonShape; 27] = [
    wallpaper("o", &[("aspect", 1.0), ("skew", 0.0)]),
    wallpaper("o", &[("aspect", 1.0), ("skew", 0.25)]),
    wallpaper("o", &[("aspect", 8.0), ("skew", -0.5)]),
    wallpaper("o", &[("aspect", 0.125), ("skew", 0.5)]),
    wallpaper("**", &[("aspect", 1.0)]),
    wallpaper("**", &[("aspect", 0.125)]),
    wallpaper("**", &[("aspect", 8.0)]),
    wallpaper("xx", &[("aspect", 1.0)]),
    wallpaper("xx", &[("aspect", 0.125)]),
    wallpaper("xx", &[("aspect", 8.0)]),
    wallpaper("*x", &[("aspect", 1.0)]),
    wallpaper("*x", &[("aspect", 0.125)]),
    wallpaper("*x", &[("aspect", 8.0)]),
    wallpaper("22*", &[("aspect", 1.0)]),
    wallpaper("22*", &[("aspect", 0.125)]),
    wallpaper("22*", &[("aspect", 8.0)]),
    wallpaper("22x", &[("aspect", 1.0)]),
    wallpaper("22x", &[("aspect", 0.125)]),
    wallpaper("22x", &[("aspect", 8.0)]),
    wallpaper("2*22", &[("aspect", 1.0)]),
    wallpaper("2*22", &[("aspect", 0.125)]),
    wallpaper("2*22", &[("aspect", 8.0)]),
    polygon("4*2", Shape::Default),
    polygon("3*3", Shape::Default),
    wallpaper("2222", &[("aspect", 1.0), ("skew", 0.25)]),
    wallpaper("2222", &[("aspect", 8.0), ("skew", -0.5)]),
    wallpaper("2222", &[("aspect", 0.125), ("skew", 0.5)]),
];

impl PolygonShape {
    pub(crate) fn name(&self) -> String {
        match self.shape {
            Shape::Default => self.signature.to_owned(),
            Shape::Named(settings) => {
                let settings: Vec<String> = settings
                    .iter()
                    .map(|(name, value)| format!("{name} {value}"))
                    .collect();
                format!("{} {}", self.signature, settings.join(" "))
            }
            Shape::Every(value) => format!("{} every length {value}", self.signature),
        }
    }

    pub(crate) fn domain(&self) -> FundamentalDomain {
        match self.shape {
            Shape::Default => domain_with_settings(self.signature, &[]),
            Shape::Named(settings) => domain_with_settings(self.signature, settings),
            Shape::Every(value) => {
                let signature: Signature = self
                    .signature
                    .parse()
                    .unwrap_or_else(|error| panic!("{:?} is rejected: {error}", self.signature));
                let names: Vec<String> = parameters(&signature)
                    .iter()
                    .map(|spec| spec.name().to_owned())
                    .collect();
                assert!(!names.is_empty(), "{} has no parameter", self.signature);
                let settings: Vec<(&str, f64)> =
                    names.iter().map(|name| (name.as_str(), value)).collect();
                domain_with_settings(self.signature, &settings)
            }
        }
    }
}

pub(crate) const TRANSLATION_TOLERANCE: f64 = 6e-15;

pub(crate) struct Translation {
    pub(crate) offset: DVec2,
    pub(crate) linear_defect: f64,
    pub(crate) last_row_defect: f64,
}

pub(crate) fn translation(domain: &FundamentalDomain, word: &[usize]) -> Translation {
    let product = word.iter().fold(DMat3::IDENTITY, |product, &index| {
        product * domain.sides()[index].pairing()
    });
    let linear = [
        product.x_axis.x - 1.0,
        product.x_axis.y,
        product.y_axis.x,
        product.y_axis.y - 1.0,
    ];
    let last_row = [product.x_axis.z, product.y_axis.z, product.z_axis.z - 1.0];
    let largest = |values: &[f64]| {
        values
            .iter()
            .fold(0.0_f64, |largest, value| largest.max(value.abs()))
    };
    Translation {
        offset: DVec2::new(product.z_axis.x, product.z_axis.y),
        linear_defect: largest(&linear),
        last_row_defect: largest(&last_row),
    }
}

pub(crate) fn shifted_pairs(
    pixels: &[u8],
    width: u32,
    height: u32,
    columns: i64,
    rows: i64,
) -> (Vec<u8>, Vec<u8>) {
    let mut here = Vec::new();
    let mut there = Vec::new();
    for row in 0..i64::from(height) {
        for column in 0..i64::from(width) {
            let (other_column, other_row) = (column + columns, row + rows);
            let inside = (0..i64::from(width)).contains(&other_column)
                && (0..i64::from(height)).contains(&other_row);
            if !inside {
                continue;
            }
            let index = |column: i64, row: i64| {
                usize::try_from(3 * (row * i64::from(width) + column)).unwrap_or(usize::MAX)
            };
            here.extend_from_slice(&pixels[index(column, row)..index(column, row) + 3]);
            there.extend_from_slice(
                &pixels[index(other_column, other_row)..index(other_column, other_row) + 3],
            );
        }
    }
    (here, there)
}
