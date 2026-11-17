use astro_float_num::BigFloat;
use escher_domain::{Check, FundamentalDomain, ShapeParameters};
use escher_signature::Signature;
use glam::DVec3;
use proptest::test_runner::Config;

use crate::reference::{Vector, vector};

pub(crate) const EXAMPLE_TRIPLES: [[u32; 3]; 6] = [
    [7, 3, 2],
    [5, 4, 2],
    [4, 4, 4],
    [4, 3, 3],
    [12, 3, 2],
    [1000, 3, 2],
];

pub(crate) const WORST_TRIPLES: [[u32; 3]; 24] = [
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

pub(crate) const SMALL_ORDER: u32 = 12;

pub(crate) fn triangles() -> impl Iterator<Item = [u32; 3]> {
    EXAMPLE_TRIPLES.into_iter().chain(WORST_TRIPLES)
}

pub(crate) fn small_triangles() -> Vec<[u32; 3]> {
    let mut triples = Vec::new();
    for first in 2..=SMALL_ORDER {
        for second in 2..=first {
            for third in 2..=second {
                if is_hyperbolic([first, second, third]) {
                    triples.push([first, second, third]);
                }
            }
        }
    }
    triples
}

pub(crate) fn is_small(orders: [u32; 3]) -> bool {
    orders.iter().all(|&order| order <= SMALL_ORDER)
}

pub(crate) fn all_triangles() -> Vec<[u32; 3]> {
    let mut triples = small_triangles();
    triples.extend(triangles().filter(|&orders| !is_small(orders)));
    triples
}

pub(crate) fn proptest_config() -> Config {
    if cfg!(miri) {
        let mut config = Config::with_cases(8);
        config.failure_persistence = None;
        config
    } else {
        Config::default()
    }
}

pub(crate) fn order_text(order: u32) -> String {
    if order < 10 {
        order.to_string()
    } else {
        format!("({order})")
    }
}

pub(crate) fn text(orders: [u32; 3]) -> String {
    let corners: String = orders.into_iter().map(order_text).collect();
    format!("*{corners}")
}

pub(crate) fn cone_text(orders: [u32; 3]) -> String {
    orders.into_iter().map(order_text).collect()
}

pub(crate) fn built(text: &str) -> FundamentalDomain {
    match FundamentalDomain::build(&signature(text)) {
        Ok(domain) => domain,
        Err(error) => panic!("{text} has no domain: {error}"),
    }
}

pub(crate) fn signature(text: &str) -> Signature {
    match text.parse() {
        Ok(signature) => signature,
        Err(error) => panic!("{text:?} is rejected: {error}"),
    }
}

pub(crate) fn domain(orders: [u32; 3]) -> FundamentalDomain {
    built(&text(orders))
}

pub(crate) fn kite(orders: [u32; 3]) -> FundamentalDomain {
    built(&cone_text(orders))
}

pub(crate) fn is_hyperbolic(orders: [u32; 3]) -> bool {
    let [p, q, r] = orders.map(u64::from);
    q * r + p * r + p * q < p * q * r
}

pub(crate) fn big(value: DVec3) -> Vector {
    vector(value.to_array())
}

pub(crate) fn largest_coordinate_error(value: DVec3, exact: &Vector) -> BigFloat {
    let computed = big(value);
    let mut largest = crate::reference::int(0);
    for (computed, exact) in computed.iter().zip(exact) {
        largest = crate::reference::max(&largest, &crate::reference::sub(computed, exact).abs());
    }
    largest
}

struct LargestResidual {
    name: &'static str,
    residual: f64,
    tolerance: f64,
    input: String,
    samples: usize,
}

impl LargestResidual {
    fn record(&mut self, check: Check, input: &str) {
        assert!(
            check.passed(),
            "{} fails at {input}: residual {:e}, tolerance {:e}",
            self.name,
            check.residual(),
            check.tolerance()
        );
        self.samples += 1;
        self.tolerance = check.tolerance();
        if check.residual() >= self.residual {
            self.residual = check.residual();
            self.input = input.to_owned();
        }
    }
}

pub(crate) struct LargestResiduals {
    checks: [LargestResidual; 6],
}

impl LargestResiduals {
    pub(crate) fn new() -> Self {
        Self {
            checks: [
                "isometries",
                "pairings",
                "walls",
                "relators",
                "angles",
                "area",
            ]
            .map(|name| LargestResidual {
                name,
                residual: 0.0,
                tolerance: 0.0,
                input: String::new(),
                samples: 0,
            }),
        }
    }

    pub(crate) fn record(&mut self, built: &FundamentalDomain, input: &str) {
        let verification = built.verify();
        let checks = [
            verification.isometries(),
            verification.pairings(),
            verification.walls(),
            verification.relators(),
            verification.angles(),
            verification.area(),
        ];
        for (largest, check) in self.checks.iter_mut().zip(checks) {
            largest.record(check, input);
        }
        assert!(verification.passed(), "{input}: {verification:?}");
    }

    pub(crate) fn print_and_assert_samples(&self, minimum: usize) {
        for largest in &self.checks {
            println!(
                "{}: largest residual {:.3e}, tolerance {:e}, margin {:.1}, {} samples, at {}",
                largest.name,
                largest.residual,
                largest.tolerance,
                largest.tolerance / largest.residual,
                largest.samples,
                largest.input
            );
            assert!(
                largest.samples >= minimum,
                "{}: checked {} domains, fewer than {minimum}",
                largest.name,
                largest.samples
            );
        }
    }
}

pub(crate) const EUCLIDEAN_TRIANGLES: [[u32; 3]; 3] = [[6, 3, 2], [4, 4, 2], [3, 3, 3]];

const NAMED_ASPECTS: [f64; 5] = [1.0, 1.618, 4.0, 0.125, 8.0];

const SPACED_ASPECTS: u32 = 64;

pub(crate) fn aspects() -> Vec<f64> {
    let mut values = NAMED_ASPECTS.to_vec();
    let last = f64::from(SPACED_ASPECTS - 1);
    values.extend((0..SPACED_ASPECTS).map(|index| (6.0 * f64::from(index) / last - 3.0).exp2()));
    values
}

pub(crate) fn rectangle(aspect: f64) -> FundamentalDomain {
    with_aspect("*2222", aspect)
}

pub(crate) fn with_aspect(text: &str, aspect: f64) -> FundamentalDomain {
    let mut settings = ShapeParameters::new();
    settings.set("aspect", aspect);
    match FundamentalDomain::build_with(&signature(text), &settings) {
        Ok(domain) => domain,
        Err(error) => panic!("{text} at aspect {aspect} has no domain: {error}"),
    }
}

pub(crate) fn assert_same_domain(
    first: &FundamentalDomain,
    second: &FundamentalDomain,
    tolerance: f64,
    input: &str,
) {
    assert_eq!(first.geometry(), second.geometry(), "{input}");
    assert_eq!(first.sides().len(), second.sides().len(), "{input}");
    assert_eq!(first.vertices().len(), second.vertices().len(), "{input}");
    for (one, other) in first.vertices().iter().zip(second.vertices()) {
        let difference = (*one - *other).abs().max_element();
        assert!(difference <= tolerance, "{input}: {difference:e}");
    }
}
