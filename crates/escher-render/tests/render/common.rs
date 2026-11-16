use core::f64::consts::TAU;

use escher_domain::FundamentalDomain;
use escher_geometry::hyperbolic::from_disk;
use escher_render::{Colour, Fill, RenderOptions, RenderOptionsBuilder};
use escher_signature::Signature;
use glam::{DVec2, DVec3};
use proptest::test_runner::Config;

use crate::reference::{Wall, matrix, vector};

pub(crate) const PLAN_TRIPLES: [[u32; 3]; 6] = [
    [7, 3, 2],
    [5, 4, 2],
    [4, 4, 4],
    [4, 3, 3],
    [12, 3, 2],
    [1000, 3, 2],
];

pub(crate) const FOLD_TRIPLES: [[u32; 3]; 7] = [
    [7, 3, 2],
    [5, 4, 2],
    [4, 4, 4],
    [4, 3, 3],
    [12, 3, 2],
    [1000, 3, 2],
    [1000, 1000, 1000],
];

pub(crate) const BANDS: [i32; 6] = [1, 2, 3, 4, 5, 6];

pub(crate) const FOLD_ANGLES: u32 = 512;

pub(crate) const RIM_ANGLES: u32 = 64;

pub(crate) const DEFAULT_CAP: u32 = 1500;

pub(crate) const TOLERANCE: f64 = 4e-15;

pub(crate) const VERTEX_TRIPLES: [[u32; 3]; 25] = [
    [38, 20, 4],
    [995, 983, 970],
    [994, 991, 970],
    [997, 996, 970],
    [997, 980, 973],
    [992, 990, 979],
    [995, 987, 976],
    [928, 4, 3],
    [999, 7, 6],
    [998, 4, 4],
    [1000, 4, 4],
    [1000, 1000, 2],
    [1000, 1000, 1000],
    [868, 4, 4],
    [923, 6, 5],
    [998, 981, 961],
    [998, 998, 958],
    [997, 931, 906],
    [992, 991, 975],
    [989, 976, 974],
    [978, 977, 971],
    [985, 985, 971],
    [973, 972, 964],
    [979, 979, 971],
    [987, 984, 970],
];

pub(crate) const REPLAY_BOUNDS: [[f64; 6]; 7] = [
    [4e-13, 4e-11, 3e-9, 4e-7, 4e-5, 4e-3],
    [3e-13, 3e-11, 3e-9, 4e-7, 3e-5, 3e-3],
    [3e-13, 3e-11, 3e-9, 2e-7, 3e-5, 3e-3],
    [3e-13, 3e-11, 2e-9, 3e-7, 3e-5, 3e-3],
    [3e-13, 3e-11, 4e-9, 4e-7, 3e-5, 4e-3],
    [2e-11, 2e-10, 5e-8, 3e-6, 4e-4, 3e-2],
    [5e-13, 9e-11, 8e-9, 2e-6, 2e-4, 2e-2],
];

pub(crate) fn proptest_config() -> Config {
    if cfg!(miri) {
        let mut config = Config::with_cases(8);
        config.failure_persistence = None;
        config
    } else {
        Config::default()
    }
}

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

pub(crate) fn domain(orders: [u32; 3]) -> FundamentalDomain {
    let text = text(orders);
    let signature: Signature = match text.parse() {
        Ok(signature) => signature,
        Err(error) => panic!("{text:?} is rejected: {error}"),
    };
    match FundamentalDomain::build(&signature) {
        Ok(domain) => domain,
        Err(error) => panic!("{text} has no domain: {error}"),
    }
}

pub(crate) fn options(builder: RenderOptionsBuilder) -> RenderOptions {
    match builder.build() {
        Ok(options) => options,
        Err(error) => panic!("the options are rejected: {error}"),
    }
}

pub(crate) fn one_fill() -> Fill {
    Fill::One(Colour::from_srgb8(70, 130, 180))
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

pub(crate) fn band_radius(band: i32) -> f64 {
    1.0 - 10.0_f64.powi(-band)
}

pub(crate) fn on_circle(radius: f64, index: u32, count: u32) -> DVec2 {
    let angle = TAU * f64::from(index) / f64::from(count);
    DVec2::new(radius * angle.cos(), radius * angle.sin())
}

pub(crate) fn pixel_position(size: u32, column: u32, row: u32) -> DVec2 {
    let pixel = 2.0 / (15.0 / 16.0 * f64::from(size));
    let half = f64::from(size) / 2.0;
    DVec2::new(
        (f64::from(column) + 0.5 - half) * pixel,
        (half - f64::from(row) - 0.5) * pixel,
    )
}

pub(crate) struct Sample {
    pub(crate) triple: usize,
    pub(crate) band: usize,
    pub(crate) angle: u32,
    pub(crate) point: DVec3,
}

pub(crate) fn fold_set(orders: [u32; 3]) -> Vec<Sample> {
    let triple = FOLD_TRIPLES
        .iter()
        .position(|&triple| triple == orders)
        .unwrap_or_else(|| panic!("{orders:?} is not in the fold set"));
    let mut samples = Vec::new();
    for (band, &exponent) in BANDS.iter().enumerate() {
        for angle in 0..FOLD_ANGLES {
            let position = on_circle(band_radius(exponent), angle, FOLD_ANGLES);
            let Some(point) = from_disk(position) else {
                panic!("{position:?} of band {exponent} is outside the disk");
            };
            samples.push(Sample {
                triple,
                band,
                angle,
                point,
            });
        }
    }
    samples
}

pub(crate) fn describe(sample: &Sample) -> String {
    format!(
        "{} in band {} at angle {} of {FOLD_ANGLES}",
        text(FOLD_TRIPLES[sample.triple]),
        BANDS[sample.band],
        sample.angle
    )
}

pub(crate) fn rim_set() -> Vec<DVec2> {
    let mut positions = Vec::new();
    for exponent in 7..=15 {
        for angle in 0..RIM_ANGLES {
            positions.push(on_circle(1.0 - 10.0_f64.powi(-exponent), angle, RIM_ANGLES));
        }
    }
    let below_one = 1.0_f64.next_down();
    positions.push(DVec2::new(below_one, 0.0));
    positions.push(DVec2::new(0.0, below_one));
    positions
}

pub(crate) struct Decoded {
    pub(crate) output: png::OutputInfo,
    pub(crate) pixels: Vec<u8>,
    pub(crate) info: png::Info<'static>,
}

pub(crate) fn decode_png(file: &[u8]) -> Result<Decoded, png::DecodingError> {
    let decoder = png::Decoder::new(std::io::Cursor::new(file));
    let mut reader = decoder.read_info()?;
    let size = reader.output_buffer_size().unwrap_or(0);
    let mut pixels = vec![0; size];
    let output = reader.next_frame(&mut pixels)?;
    pixels.truncate(output.buffer_size());
    let info = reader.info().clone();
    Ok(Decoded {
        output,
        pixels,
        info,
    })
}
