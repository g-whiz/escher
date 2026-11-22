use astro_float_num::BigFloat;
use escher_domain::{Check, FundamentalDomain, Verification};
use glam::DVec3;

use crate::reference::{Vector, vector};
use crate::shared::{NAMED_POLYGONS, built_with_aspect};
pub(crate) use crate::shared::{built, order_text, signature};

pub(crate) const EXAMPLE_TRIPLES: [[u32; 3]; 6] = [
    [7, 3, 2],
    [5, 4, 2],
    [4, 4, 4],
    [4, 3, 3],
    [12, 3, 2],
    [100, 3, 2],
];

pub(crate) const WORST_TRIPLES: [[u32; 3]; 24] = [
    [98, 95, 88],
    [100, 89, 87],
    [100, 98, 95],
    [99, 98, 87],
    [94, 94, 91],
    [100, 99, 95],
    [99, 97, 93],
    [100, 96, 95],
    [90, 85, 82],
    [95, 93, 84],
    [94, 6, 4],
    [98, 7, 3],
    [99, 4, 4],
    [100, 4, 3],
    [100, 4, 4],
    [71, 4, 4],
    [100, 6, 2],
    [87, 6, 4],
    [91, 7, 4],
    [99, 6, 4],
    [100, 100, 2],
    [100, 100, 100],
    [100, 7, 6],
    [92, 6, 5],
];

pub(crate) const SMALL_ORDER: u32 = 12;

pub(crate) const LIMIT_ORDER: u32 = 100;

pub(crate) const TWO_MIRROR_LIMIT_ORDER: u32 = LIMIT_ORDER / 2;

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

pub(crate) fn text(orders: [u32; 3]) -> String {
    let corners: String = orders.into_iter().map(order_text).collect();
    format!("*{corners}")
}

pub(crate) fn cone_text(orders: [u32; 3]) -> String {
    orders.into_iter().map(order_text).collect()
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
        self.record_verification(&built.verify(), input);
    }

    pub(crate) fn record_verification(&mut self, verification: &Verification, input: &str) {
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
    built_with_aspect(text, aspect, None)
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

pub(crate) fn starred(rotation: &str) -> String {
    format!("*{rotation}")
}

pub(crate) fn named_rotation_polygons() -> Vec<String> {
    NAMED_POLYGONS
        .iter()
        .map(|text| text.trim_start_matches('*').to_owned())
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kite {
    ThreeMirrorTwo,
    ConeMirror(u32),
    TwoMirror(u32),
}

impl Kite {
    pub(crate) fn text(self) -> String {
        match self {
            Self::ThreeMirrorTwo => "3*2".to_owned(),
            Self::ConeMirror(order) => format!("{}*", order_text(order)),
            Self::TwoMirror(order) => format!("2*{}", order_text(order)),
        }
    }

    pub(crate) const fn orders(self) -> [u32; 3] {
        match self {
            Self::ThreeMirrorTwo => [4, 3, 2],
            Self::ConeMirror(order) => [order, 2, 2],
            Self::TwoMirror(order) => [2 * order, 2, 2],
        }
    }

    pub(crate) fn double(self) -> String {
        cone_text(self.orders())
    }

    pub(crate) fn starred(self) -> String {
        format!("*{}", self.double())
    }

    pub(crate) const fn mirrors(self) -> [usize; 2] {
        match self {
            Self::ConeMirror(_) => [2, 3],
            Self::ThreeMirrorTwo | Self::TwoMirror(_) => [0, 1],
        }
    }

    pub(crate) const fn cycles(self) -> [(u32, &'static [usize]); 3] {
        match self {
            Self::ThreeMirrorTwo => [(2, &[0, 0]), (1, &[1, 3, 3, 1]), (3, &[2])],
            Self::TwoMirror(order) => [(order, &[0, 0]), (1, &[1, 3, 3, 1]), (2, &[2])],
            Self::ConeMirror(order) => [(order, &[0]), (1, &[1, 1, 3, 3]), (1, &[2, 2])],
        }
    }

    pub(crate) const fn flat_vertices(self) -> &'static [usize] {
        match self {
            Self::ThreeMirrorTwo => &[],
            Self::ConeMirror(2) => &[0, 2],
            Self::ConeMirror(_) | Self::TwoMirror(_) => &[2],
        }
    }
}

pub(crate) fn small_kites() -> Vec<Kite> {
    let mut kites = vec![Kite::ThreeMirrorTwo];
    kites.extend((2..=SMALL_ORDER).map(Kite::ConeMirror));
    kites.extend((2..=SMALL_ORDER).map(Kite::TwoMirror));
    kites
}

pub(crate) fn limit_kites() -> [Kite; 2] {
    [
        Kite::ConeMirror(LIMIT_ORDER),
        Kite::TwoMirror(TWO_MIRROR_LIMIT_ORDER),
    ]
}

pub(crate) fn test_set() -> Vec<Kite> {
    let mut kites = small_kites();
    kites.extend(limit_kites());
    kites
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LuneGroup {
    ConeCrossCap(u32),
    CrossCap,
    Mirror,
    Trivial,
}

impl LuneGroup {
    pub(crate) fn text(self) -> String {
        match self {
            Self::ConeCrossCap(order) => format!("{}x", order_text(order)),
            Self::CrossCap => "x".to_owned(),
            Self::Mirror => "*".to_owned(),
            Self::Trivial => "1".to_owned(),
        }
    }

    pub(crate) const fn order(self) -> Option<u32> {
        match self {
            Self::ConeCrossCap(order) => Some(order),
            Self::CrossCap | Self::Mirror => Some(1),
            Self::Trivial => None,
        }
    }
}

pub(crate) fn small_lune_groups() -> Vec<LuneGroup> {
    let mut groups: Vec<LuneGroup> = (2..=SMALL_ORDER).map(LuneGroup::ConeCrossCap).collect();
    groups.extend([LuneGroup::CrossCap, LuneGroup::Mirror, LuneGroup::Trivial]);
    groups
}

pub(crate) fn lune_test_set() -> Vec<LuneGroup> {
    let mut groups = small_lune_groups();
    groups.push(LuneGroup::ConeCrossCap(LIMIT_ORDER));
    groups
}
