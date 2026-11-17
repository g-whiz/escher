use escher_domain::{FundamentalDomain, ShapeParameters};
use escher_signature::Signature;
use glam::{DMat3, DVec2};

use crate::reference::Reference;
use crate::{Colour, Fill, RenderOptions};

pub(crate) struct GoldenImage {
    pub(crate) signature: &'static str,
    pub(crate) aspect: Option<f64>,
    pub(crate) file: &'static str,
}

impl GoldenImage {
    pub(crate) fn domain(&self) -> FundamentalDomain {
        domain_with_aspect(self.signature, self.aspect)
    }
}

pub(crate) const GOLDEN_IMAGES: [GoldenImage; 7] = [
    GoldenImage {
        signature: "*732",
        aspect: None,
        file: "7-3-2.png",
    },
    GoldenImage {
        signature: "*542",
        aspect: None,
        file: "5-4-2.png",
    },
    GoldenImage {
        signature: "*433",
        aspect: None,
        file: "4-3-3.png",
    },
    GoldenImage {
        signature: "*632",
        aspect: None,
        file: "6-3-2.png",
    },
    GoldenImage {
        signature: "*442",
        aspect: None,
        file: "4-4-2.png",
    },
    GoldenImage {
        signature: "*333",
        aspect: None,
        file: "3-3-3.png",
    },
    GoldenImage {
        signature: "*2222",
        aspect: Some(1.618),
        file: "2-2-2-2.png",
    },
];

pub(crate) const GOLDEN_FILL: [[u8; 3]; 2] = [[245, 222, 179], [70, 130, 180]];

pub(crate) const GOLDEN_SIZE: u32 = 192;

pub(crate) const GOLDEN_GRID: u32 = 4;

pub(crate) const ALLOWED_PIXELS: usize = 16;

pub(crate) fn golden_options() -> RenderOptions {
    let [even, odd] = GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
    match RenderOptions::builder(Fill::Parity([even, odd]))
        .size(GOLDEN_SIZE, GOLDEN_SIZE)
        .samples(GOLDEN_GRID)
        .border_width(1.0)
        .build()
    {
        Ok(options) => options,
        Err(error) => panic!("the golden options are rejected: {error}"),
    }
}

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
