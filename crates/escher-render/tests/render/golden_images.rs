use crate::reference::Reference;
use crate::{Colour, Fill, RenderOptions};

pub(crate) struct GoldenImage {
    pub(crate) signature: &'static str,
    pub(crate) file: &'static str,
}

pub(crate) const GOLDEN_IMAGES: [GoldenImage; 3] = [
    GoldenImage {
        signature: "*732",
        file: "7-3-2.png",
    },
    GoldenImage {
        signature: "*542",
        file: "5-4-2.png",
    },
    GoldenImage {
        signature: "*433",
        file: "4-3-3.png",
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
