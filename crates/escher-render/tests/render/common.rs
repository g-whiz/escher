use core::f64::consts::TAU;

use escher_domain::FundamentalDomain;
use escher_geometry::hyperbolic::from_disk;
use escher_geometry::spherical;
use escher_render::{Colour, Fill, RenderOptions, RenderOptionsBuilder, RenderStats, Renderer};
use escher_signature::Signature;
use glam::{DMat3, DVec2, DVec3};
use proptest::test_runner::Config;

use crate::golden_images::{POLYGON_DOMAINS, domain_with_aspect};
use crate::reference::{self, Reference, Vector, Wall, matrix, vector};

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

pub(crate) const EUCLIDEAN_DOMAINS: [(&str, Option<f64>); 7] = [
    ("*632", None),
    ("*442", None),
    ("*333", None),
    ("*2222", Some(1.0)),
    ("*2222", Some(1.618)),
    ("*2222", Some(0.125)),
    ("*2222", Some(8.0)),
];

pub(crate) const EUCLIDEAN_DISTANCES: [f64; 4] = [1.0, 4.0, 16.0, 65.0];

pub(crate) const EUCLIDEAN_RANGE: f64 = 65.0;

pub(crate) const EUCLIDEAN_REPLAY_BOUNDS: [[f64; 4]; 7] = [
    [2e-15, 2e-14, 2e-13, 3e-12],
    [3e-15, 8e-15, 7e-14, 2e-12],
    [2e-15, 1e-14, 9e-14, 2e-12],
    [3e-15, 2e-14, 2e-13, 3e-12],
    [3e-15, 3e-15, 4e-14, 9e-13],
    [5e-16, 5e-15, 2e-13, 2e-12],
    [3e-15, 9e-15, 9e-14, 2e-12],
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

pub(crate) const SPHERICAL_DOMAINS: [&str; 17] = [
    "*332",
    "*432",
    "*532",
    "*222",
    "*322",
    "*522",
    "*(12)22",
    "*(100)22",
    "*(999)22",
    "*(1000)22",
    "*22",
    "*33",
    "*55",
    "*(12)(12)",
    "*(100)(100)",
    "*(999)(999)",
    "*(1000)(1000)",
];

pub(crate) const RIM_DISTANCES: [f64; 7] = [120.0, 24.0, 8.0, 4.0, 2.0, 1.0, 0.5];

pub(crate) const SPHERE_ROTATIONS: [[f64; 3]; 2] = [[0.0; 3], [20.0, 35.0, 0.0]];

pub(crate) const SPHERICAL_REPLAY_BOUNDS: [f64; 17] = [
    3e-15, 3e-15, 4e-15, 3e-15, 3e-15, 3e-15, 4e-15, 2e-14, 4e-14, 5e-14, 8e-16, 2e-15, 2e-15,
    3e-15, 9e-15, 3e-14, 3e-14,
];

pub(crate) const SPHERICAL_IMAGE_BOUNDS: [f64; 2] = [9e-13, 3e-13];

pub(crate) const SPHERICAL_DECISION_COUNTS: [usize; 17] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 0];

pub(crate) fn proptest_config() -> Config {
    Config::default()
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

pub(crate) fn euclidean_name(index: usize) -> String {
    match EUCLIDEAN_DOMAINS[index] {
        (text, None) => text.to_owned(),
        (text, Some(aspect)) => format!("{text} aspect {aspect}"),
    }
}

pub(crate) fn euclidean_domain(index: usize) -> FundamentalDomain {
    let (text, aspect) = EUCLIDEAN_DOMAINS[index];
    domain_with_aspect(text, aspect)
}

pub(crate) fn euclidean_band(distance: f64) -> Option<usize> {
    EUCLIDEAN_DISTANCES
        .iter()
        .position(|&outer| distance <= outer)
}

pub(crate) struct PlaneSample {
    pub(crate) domain: usize,
    pub(crate) distance: usize,
    pub(crate) angle: u32,
    pub(crate) point: DVec3,
}

pub(crate) fn euclidean_fold_set(domain: usize) -> Vec<PlaneSample> {
    let mut samples = Vec::new();
    for (distance, &radius) in EUCLIDEAN_DISTANCES.iter().enumerate() {
        for angle in 0..FOLD_ANGLES {
            let position = on_circle(radius, angle, FOLD_ANGLES);
            samples.push(PlaneSample {
                domain,
                distance,
                angle,
                point: DVec3::new(position.x, position.y, 1.0),
            });
        }
    }
    samples
}

pub(crate) fn describe_plane(sample: &PlaneSample) -> String {
    format!(
        "{} at distance {} at angle {} of {FOLD_ANGLES}",
        euclidean_name(sample.domain),
        EUCLIDEAN_DISTANCES[sample.distance],
        sample.angle
    )
}

pub(crate) struct SphereSample {
    pub(crate) domain: usize,
    pub(crate) rotation: usize,
    pub(crate) band: usize,
    pub(crate) angle: u32,
    pub(crate) point: DVec3,
}

pub(crate) fn spherical_domain(index: usize) -> FundamentalDomain {
    domain_with_aspect(SPHERICAL_DOMAINS[index], None)
}

fn viewing_matrix([yaw, pitch, roll]: [f64; 3]) -> DMat3 {
    let [yaw, pitch, roll] = [yaw, pitch, roll].map(f64::to_radians);
    let (sin, cos) = pitch.sin_cos();
    let turn_pitch = DMat3::from_cols(
        DVec3::X,
        DVec3::new(0.0, cos, -sin),
        DVec3::new(0.0, sin, cos),
    );
    spherical::rotation(roll) * turn_pitch * spherical::boost(yaw)
}

pub(crate) fn spherical_fold_set(domain: usize) -> Vec<SphereSample> {
    let pixel = 2.0 / (15.0 / 16.0 * 512.0);
    let mut samples = Vec::new();
    for (rotation, &angles) in SPHERE_ROTATIONS.iter().enumerate() {
        let inverse = viewing_matrix(angles).transpose();
        for (band, &distance) in RIM_DISTANCES.iter().enumerate() {
            for angle in 0..FOLD_ANGLES {
                let position = on_circle(1.0 - distance * pixel, angle, FOLD_ANGLES);
                let screen = DVec3::new(
                    position.x,
                    position.y,
                    (1.0 - position.length_squared()).sqrt(),
                );
                samples.push(SphereSample {
                    domain,
                    rotation,
                    band,
                    angle,
                    point: inverse * screen,
                });
            }
        }
    }
    samples
}

pub(crate) fn describe_sphere(sample: &SphereSample) -> String {
    format!(
        "{} with rotation {:?} at {} pixels from the rim at angle {} of {FOLD_ANGLES}",
        SPHERICAL_DOMAINS[sample.domain],
        SPHERE_ROTATIONS[sample.rotation],
        RIM_DISTANCES[sample.band],
        sample.angle
    )
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

pub(crate) const POLYGON_REPLAY_BOUNDS: [[f64; 6]; POLYGON_DOMAINS.len()] = [
    [4e-13, 4e-11, 4e-9, 4e-7, 4e-5, 4e-3],
    [1e-12, 1e-10, 1e-8, 2e-6, 1e-4, 1e-2],
    [1e-12, 2e-10, 1e-8, 1e-6, 1e-4, 1e-2],
    [4e-12, 4e-10, 2e-8, 4e-6, 4e-4, 2e-2],
    [4e-13, 4e-11, 4e-9, 4e-7, 4e-5, 4e-3],
    [1e-12, 1e-10, 1e-8, 1e-6, 1e-4, 1e-2],
    [1e-11, 4e-10, 1e-7, 4e-6, 4e-4, 4e-2],
    [4e-13, 4e-11, 4e-9, 4e-7, 4e-5, 4e-3],
    [1e-12, 1e-10, 1e-8, 1e-6, 4e-5, 1e-2],
    [4e-12, 2e-10, 4e-8, 4e-6, 4e-4, 4e-2],
    [4e-13, 4e-11, 4e-9, 4e-7, 2e-5, 4e-3],
    [4e-11, 4e-9, 4e-7, 4e-5, 4e-3, 4e-1],
    [1e-12, 2e-10, 1e-8, 1e-6, 1e-4, 1e-2],
    [4e-11, 4e-9, 2e-7, 4e-5, 2e-3, 2e-1],
    [1e-11, 1e-9, 1e-7, 1e-5, 1e-3, 2e-1],
    [4e-12, 4e-10, 4e-8, 4e-6, 4e-4, 4e-2],
    [2e-12, 4e-10, 4e-8, 2e-6, 4e-4, 4e-2],
    [2e-12, 2e-10, 4e-8, 4e-6, 2e-4, 4e-2],
    [1e-12, 1e-10, 1e-8, 1e-6, 1e-4, 1e-2],
    [2e-10, 1e-8, 1e-6, 1e-4, 2e-2, 1.0],
];

pub(crate) const POLYGON_DECISION_COUNTS: [[usize; 6]; POLYGON_DOMAINS.len()] = [
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0, 0, 0, 0, 0, 2],
    [0; 6],
    [0, 0, 0, 0, 0, 1],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0; 6],
    [0, 0, 0, 0, 0, 1],
];

pub(crate) struct PolygonSample {
    pub(crate) domain: usize,
    pub(crate) band: usize,
    pub(crate) angle: u32,
    pub(crate) point: DVec3,
}

pub(crate) fn polygon_fold_set(domain: usize) -> Vec<PolygonSample> {
    let mut samples = Vec::new();
    for (band, &exponent) in BANDS.iter().enumerate() {
        for angle in 0..FOLD_ANGLES {
            let position = on_circle(band_radius(exponent), angle, FOLD_ANGLES);
            let Some(point) = from_disk(position) else {
                panic!("{position:?} of band {exponent} is outside the disk");
            };
            samples.push(PolygonSample {
                domain,
                band,
                angle,
                point,
            });
        }
    }
    samples
}

pub(crate) fn describe_polygon(sample: &PolygonSample) -> String {
    format!(
        "{} in band {} at angle {} of {FOLD_ANGLES}",
        POLYGON_DOMAINS[sample.domain].name(),
        BANDS[sample.band],
        sample.angle
    )
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

pub(crate) fn rotation_text(starred: &str) -> &str {
    starred
        .strip_prefix('*')
        .unwrap_or_else(|| panic!("{starred} has no star"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Measure {
    Hyperbolic,
    Euclidean,
    Spherical,
}

impl Measure {
    pub(crate) fn distance(
        self,
        reference: &mut Reference,
        x: &Vector,
        y: &Vector,
    ) -> astro_float_num::BigFloat {
        match self {
            Self::Hyperbolic => reference.distance(x, y),
            Self::Euclidean => reference::plane_distance(x, y),
            Self::Spherical if reference::dot(x, y).is_positive() => reference.angle(x, y),
            Self::Spherical => reference.pi(),
        }
    }

    pub(crate) fn wall_distance(
        self,
        reference: &mut Reference,
        covector: &Vector,
        point: &Vector,
    ) -> astro_float_num::BigFloat {
        match self {
            Self::Hyperbolic => reference.wall_distance(covector, point),
            Self::Euclidean => reference::plane_wall_distance(covector, point),
            Self::Spherical => reference.sphere_wall_distance(covector, point),
        }
    }
}

pub(crate) struct RotationSample {
    pub(crate) group: usize,
    pub(crate) point: DVec3,
    pub(crate) name: String,
}

pub(crate) struct RotationCase {
    pub(crate) name: String,
    pub(crate) domain: FundamentalDomain,
    pub(crate) reflection: FundamentalDomain,
    pub(crate) measure: Measure,
    pub(crate) groups: Vec<String>,
    pub(crate) bounds: Vec<f64>,
    pub(crate) reflection_bounds: Vec<f64>,
    pub(crate) decisions: Vec<usize>,
    pub(crate) oracle: Vec<usize>,
    pub(crate) range: usize,
    pub(crate) samples: Vec<RotationSample>,
}

pub(crate) const ROTATION_REPLAY_BOUNDS: [[f64; 6]; 7] = [
    [3e-13, 3e-11, 3e-9, 3e-7, 4e-5, 4e-3],
    [3e-13, 4e-11, 4e-9, 3e-7, 4e-5, 4e-3],
    [5e-13, 6e-11, 7e-9, 8e-7, 6e-5, 5e-3],
    [3e-13, 5e-11, 4e-9, 4e-7, 4e-5, 4e-3],
    [3e-13, 3e-11, 4e-9, 4e-7, 4e-5, 4e-3],
    [2e-11, 2e-9, 2e-7, 6e-5, 3e-3, 4e-1],
    [3e-12, 3e-10, 2e-7, 8e-6, 3e-4, 2e-1],
];

pub(crate) const ROTATION_EUCLIDEAN_REPLAY_BOUNDS: [[f64; 4]; 7] = [
    [3e-15, 2e-14, 9e-14, 2e-12],
    [3e-15, 9e-15, 2e-13, 2e-12],
    [3e-15, 2e-14, 1e-13, 2e-12],
    [9e-16, 4e-15, 2e-14, 6e-14],
    [2e-15, 4e-15, 4e-14, 9e-13],
    [3e-16, 5e-15, 5e-14, 8e-13],
    [2e-15, 2e-14, 9e-14, 2e-12],
];

pub(crate) const ROTATION_SPHERICAL_REPLAY_BOUNDS: [f64; 17] = [
    3e-15, 3e-15, 4e-15, 2e-15, 2e-15, 3e-15, 3e-15, 8e-15, 3e-14, 3e-14, 6e-16, 1e-15, 2e-15,
    3e-15, 6e-15, 2e-14, 3e-14,
];

pub(crate) const ROTATION_DECISION_COUNTS: [[usize; 6]; 7] = [
    [0; 6],
    [0, 0, 0, 0, 0, 1],
    [0; 6],
    [0; 6],
    [0; 6],
    [0, 0, 0, 0, 0, 2],
    [0; 6],
];

pub(crate) const ROTATION_EUCLIDEAN_DECISION_COUNTS: [[usize; 4]; 7] =
    [[1, 0, 0, 0], [0; 4], [0; 4], [0; 4], [0; 4], [0; 4], [0; 4]];

pub(crate) const ROTATION_SPHERICAL_DECISION_COUNTS: [usize; 17] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0];

pub(crate) const ROTATION_ORACLE_COUNTS: [[usize; 6]; 7] = [[0; 6]; 7];

pub(crate) const ROTATION_EUCLIDEAN_ORACLE_COUNTS: [[usize; 4]; 7] = [[0; 4]; 7];

pub(crate) const ROTATION_SPHERICAL_ORACLE_COUNTS: [usize; 17] = [0; 17];

pub(crate) const ROTATION_RANGES: [usize; 7] = [6; 7];

pub(crate) fn rotation_domain(starred: &str, aspect: Option<f64>) -> FundamentalDomain {
    domain_with_aspect(rotation_text(starred), aspect)
}

pub(crate) fn hyperbolic_rotation_cases() -> Vec<RotationCase> {
    FOLD_TRIPLES
        .iter()
        .enumerate()
        .map(|(index, &triple)| {
            let starred = text(triple);
            let name = rotation_text(&starred).to_owned();
            let samples = fold_set(triple)
                .into_iter()
                .map(|sample| RotationSample {
                    group: sample.band,
                    point: sample.point,
                    name: format!(
                        "{name} in band {} at angle {} of {FOLD_ANGLES}",
                        BANDS[sample.band], sample.angle
                    ),
                })
                .collect();
            RotationCase {
                domain: rotation_domain(&starred, None),
                reflection: domain(triple),
                measure: Measure::Hyperbolic,
                groups: BANDS.iter().map(|band| format!("band {band}")).collect(),
                bounds: ROTATION_REPLAY_BOUNDS[index].to_vec(),
                reflection_bounds: REPLAY_BOUNDS[index].to_vec(),
                decisions: ROTATION_DECISION_COUNTS[index].to_vec(),
                oracle: ROTATION_ORACLE_COUNTS[index].to_vec(),
                range: ROTATION_RANGES[index],
                samples,
                name,
            }
        })
        .collect()
}

pub(crate) fn euclidean_rotation_cases() -> Vec<RotationCase> {
    (0..EUCLIDEAN_DOMAINS.len())
        .map(|index| {
            let (starred, aspect) = EUCLIDEAN_DOMAINS[index];
            let name = euclidean_name(index)
                .strip_prefix('*')
                .unwrap_or_default()
                .to_owned();
            let samples = euclidean_fold_set(index)
                .into_iter()
                .map(|sample| RotationSample {
                    group: sample.distance,
                    point: sample.point,
                    name: format!(
                        "{name} at distance {} at angle {} of {FOLD_ANGLES}",
                        EUCLIDEAN_DISTANCES[sample.distance], sample.angle
                    ),
                })
                .collect();
            RotationCase {
                domain: rotation_domain(starred, aspect),
                reflection: euclidean_domain(index),
                measure: Measure::Euclidean,
                groups: EUCLIDEAN_DISTANCES
                    .iter()
                    .map(|distance| format!("distance {distance}"))
                    .collect(),
                bounds: ROTATION_EUCLIDEAN_REPLAY_BOUNDS[index].to_vec(),
                reflection_bounds: EUCLIDEAN_REPLAY_BOUNDS[index].to_vec(),
                decisions: ROTATION_EUCLIDEAN_DECISION_COUNTS[index].to_vec(),
                oracle: ROTATION_EUCLIDEAN_ORACLE_COUNTS[index].to_vec(),
                range: EUCLIDEAN_DISTANCES.len(),
                samples,
                name,
            }
        })
        .collect()
}

pub(crate) fn spherical_rotation_cases(
    indices: impl IntoIterator<Item = usize>,
) -> Vec<RotationCase> {
    indices
        .into_iter()
        .map(|index| {
            let starred = SPHERICAL_DOMAINS[index];
            let name = rotation_text(starred).to_owned();
            let samples = spherical_fold_set(index)
                .into_iter()
                .map(|sample| RotationSample {
                    group: 0,
                    point: sample.point,
                    name: format!(
                        "{name} with rotation {:?} at {} pixels from the rim at angle {} of {FOLD_ANGLES}",
                        SPHERE_ROTATIONS[sample.rotation],
                        RIM_DISTANCES[sample.band],
                        sample.angle
                    ),
                })
                .collect();
            RotationCase {
                domain: rotation_domain(starred, None),
                reflection: spherical_domain(index),
                measure: Measure::Spherical,
                groups: vec!["all positions".to_owned()],
                bounds: vec![ROTATION_SPHERICAL_REPLAY_BOUNDS[index]],
                reflection_bounds: vec![SPHERICAL_REPLAY_BOUNDS[index]],
                decisions: vec![ROTATION_SPHERICAL_DECISION_COUNTS[index]],
                oracle: vec![ROTATION_SPHERICAL_ORACLE_COUNTS[index]],
                range: 1,
                samples,
                name,
            }
        })
        .collect()
}

pub(crate) fn render_in_bands(
    name: &str,
    domain: &FundamentalDomain,
    options: RenderOptions,
) -> RenderStats {
    let (width, height) = (options.width(), options.height());
    let renderer = Renderer::new(domain, options)
        .unwrap_or_else(|error| panic!("{name}: the renderer is rejected: {error}"));
    let band_rows = height.div_ceil(8);
    let row_bytes = 3 * usize::try_from(width).unwrap_or(usize::MAX);
    let band_bytes = row_bytes * usize::try_from(band_rows).unwrap_or(usize::MAX);
    let mut pixels = vec![0; row_bytes * usize::try_from(height).unwrap_or(usize::MAX)];
    std::thread::scope(|scope| {
        let renderer = &renderer;
        let bands: Vec<_> = pixels
            .chunks_mut(band_bytes)
            .zip((0..height).step_by(usize::try_from(band_rows).unwrap_or(usize::MAX)))
            .map(|(band, first)| scope.spawn(move || renderer.render_rows(first, band)))
            .collect();
        bands
            .into_iter()
            .fold(RenderStats::default(), |stats, band| {
                match band.join().expect("a band of rows returns") {
                    Ok(band) => stats.merged(band),
                    Err(error) => panic!("{name}: {error}"),
                }
            })
    })
}
