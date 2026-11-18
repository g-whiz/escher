use escher_domain::{FundamentalDomain, ShapeParameters};
use escher_group::{Colouring, ColouringId, IndexLimit, SearchBudget, colourings_of_index};
use escher_signature::Signature;
use glam::{DMat3, DVec2};

use crate::reference::Reference;
use crate::{Colour, CosetFill, Fill, RenderOptions, Renderer, Shading};

pub(crate) struct GoldenImage {
    pub(crate) signature: &'static str,
    pub(crate) aspect: Option<f64>,
    pub(crate) rotation: [f64; 3],
    pub(crate) shading: Shading,
    pub(crate) colouring: Option<&'static str>,
    pub(crate) file: &'static str,
}

impl GoldenImage {
    pub(crate) fn domain(&self) -> FundamentalDomain {
        domain_with_aspect(self.signature, self.aspect)
    }

    pub(crate) fn options(&self, domain: &FundamentalDomain) -> RenderOptions {
        let [even, odd] =
            GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
        let fill = match self.colouring {
            None => Fill::Parity([even, odd]),
            Some(id) => Fill::Cosets(coset_fill(domain, colouring(domain, id))),
        };
        let [yaw, pitch, roll] = self.rotation.map(f64::to_radians);
        match RenderOptions::builder(fill)
            .size(GOLDEN_SIZE, GOLDEN_SIZE)
            .samples(GOLDEN_GRID)
            .border_width(1.0)
            .rotation(yaw, pitch, roll)
            .shading(self.shading)
            .build()
        {
            Ok(options) => options,
            Err(error) => panic!("the golden options of {} are rejected: {error}", self.file),
        }
    }
}

const fn golden(signature: &'static str, aspect: Option<f64>, file: &'static str) -> GoldenImage {
    GoldenImage {
        signature,
        aspect,
        rotation: [0.0; 3],
        shading: Shading::Phong,
        colouring: None,
        file,
    }
}

const fn coset(signature: &'static str, id: &'static str, file: &'static str) -> GoldenImage {
    GoldenImage {
        signature,
        aspect: None,
        rotation: [0.0; 3],
        shading: Shading::Phong,
        colouring: Some(id),
        file,
    }
}

pub(crate) const GOLDEN_IMAGES: [GoldenImage; 20] = [
    golden("*732", None, "7-3-2.png"),
    golden("*542", None, "5-4-2.png"),
    golden("*433", None, "4-3-3.png"),
    golden("*632", None, "6-3-2.png"),
    golden("*442", None, "4-4-2.png"),
    golden("*333", None, "3-3-3.png"),
    golden("*2222", Some(1.618), "2-2-2-2.png"),
    golden("*532", None, "5-3-2.png"),
    golden("*432", None, "4-3-2.png"),
    golden("*332", None, "3-3-2.png"),
    golden("*722", None, "7-2-2.png"),
    golden("*55", None, "5-5.png"),
    GoldenImage {
        signature: "*532",
        aspect: None,
        rotation: [20.0, 35.0, 0.0],
        shading: Shading::Phong,
        colouring: None,
        file: "5-3-2-rotated.png",
    },
    GoldenImage {
        signature: "*532",
        aspect: None,
        rotation: [0.0; 3],
        shading: Shading::Flat,
        colouring: None,
        file: "5-3-2-flat.png",
    },
    golden("732", None, "rot-7-3-2.png"),
    golden("442", None, "rot-4-4-2.png"),
    golden("2222", Some(1.618), "rot-2-2-2-2.png"),
    golden("532", None, "rot-5-3-2.png"),
    golden("722", None, "rot-7-2-2.png"),
    golden("55", None, "rot-5-5.png"),
];

pub(crate) const COSET_GOLDEN_IMAGES: [GoldenImage; 6] = [
    coset("*642", "3a", "cos-6-4-2-3a.png"),
    coset("*732", "8a", "cos-7-3-2-8a.png"),
    coset("432", "3a", "cos-rot-4-3-2-3a.png"),
    coset("532", "5a", "cos-rot-5-3-2-5a.png"),
    coset("*2222", "4a", "cos-2-2-2-2-4a.png"),
    coset("*532", "5a", "cos-5-3-2-5a.png"),
];

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
    let presentation = domain
        .side_presentation()
        .unwrap_or_else(|error| panic!("the domain has no side presentation: {error}"));
    let limit = IndexLimit::new(colours)
        .unwrap_or_else(|error| panic!("{colours} colours are too many: {error}"));
    let found = colourings_of_index(&presentation, limit, SearchBudget::DEFAULT)
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
    let colours = id
        .parse::<ColouringId>()
        .unwrap_or_else(|error| panic!("{id:?} is rejected: {error}"))
        .colours();
    find_colouring(&colourings_of(domain, colours), id)
}

pub(crate) fn coset_fill(domain: &FundamentalDomain, colouring: Colouring) -> CosetFill {
    let id = colouring.id();
    let colours = GOLDEN_COSET_FILL
        .iter()
        .take(usize::try_from(colouring.colours()).unwrap_or(usize::MAX))
        .map(|&[red, green, blue]| Colour::from_srgb8(red, green, blue))
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
    let signature: Signature = text
        .parse()
        .unwrap_or_else(|error| panic!("{text:?} is rejected: {error}"));
    let mut settings = ShapeParameters::new();
    if let Some(aspect) = aspect {
        settings.set("aspect", aspect);
    }
    FundamentalDomain::build_with(&signature, &settings)
        .unwrap_or_else(|error| panic!("{text} with aspect {aspect:?} has no domain: {error}"))
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
