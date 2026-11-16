use astro_float_num::BigFloat;
use escher_domain::FundamentalDomain;
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

pub(crate) fn signature(text: &str) -> Signature {
    match text.parse() {
        Ok(signature) => signature,
        Err(error) => panic!("{text:?} is rejected: {error}"),
    }
}

pub(crate) fn domain(orders: [u32; 3]) -> FundamentalDomain {
    let text = text(orders);
    match FundamentalDomain::build(&signature(&text)) {
        Ok(domain) => domain,
        Err(error) => panic!("{text} has no domain: {error}"),
    }
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
