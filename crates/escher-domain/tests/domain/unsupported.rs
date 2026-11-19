use escher_domain::{ErrorKind, FundamentalDomain};
use escher_signature::Geometry;

use crate::common::signature;

const UNSUPPORTED: [&str; 20] = [
    "3*2",
    "2*",
    "x",
    "4*3",
    "3*22",
    "*(inf)32",
    "(inf)32",
    "*(inf)(inf)",
    "(inf)(inf)",
    "*22(inf)",
    "*(inf)222",
    "(inf)222",
    "1",
    "*",
    "oo",
    "xxx",
    "*732*",
    "7*32",
    "x732",
    "(1001)*32",
];

fn error_of(text: &str) -> Option<(ErrorKind, String)> {
    FundamentalDomain::build(&signature(text))
        .err()
        .map(|error| (error.kind(), error.to_string()))
}

#[test]
fn build_rejects_every_unsupported_signature() {
    for text in UNSUPPORTED {
        let canonical = signature(text).to_string();
        assert_eq!(
            error_of(text),
            Some((
                ErrorKind::UnsupportedSignature,
                format!(
                    "cannot build fundamental domain: the signature {canonical} is not supported"
                )
            )),
            "{text}"
        );
    }
}

#[test]
fn build_gives_a_domain_of_every_euclidean_signature_of_its_list() {
    let euclidean = [
        "*632", "*442", "*333", "*2222", "632", "442", "333", "2222", "o", "**", "xx", "*x", "22*",
        "22x", "2*22", "4*2", "3*3",
    ];
    let canonical: std::collections::BTreeSet<String> = euclidean
        .iter()
        .map(|text| {
            let parsed = signature(text);
            assert_eq!(parsed.geometry(), Geometry::Euclidean, "{text}");
            parsed.to_string()
        })
        .collect();
    assert_eq!(canonical.len(), 17);
    for text in euclidean {
        let built = FundamentalDomain::build(&signature(text));
        assert!(
            built.as_ref().is_ok_and(|domain| domain.verify().passed()),
            "{text}: {built:?}"
        );
    }
}

#[test]
fn build_rejects_an_order_above_the_limit() {
    for text in [
        "*(1001)32",
        "*(1001)22",
        "*(1001)(1001)",
        "*(1001)222",
        "*(1001)(1001)(1001)(1001)",
        "(1001)32",
        "(1001)22",
        "(1001)(1001)",
        "(1001)222",
        "(1001)(1001)(1001)(1001)",
    ] {
        assert_eq!(
            error_of(text),
            Some((
                ErrorKind::LimitExceeded,
                "cannot build fundamental domain: order 1001 is larger than 1000".to_owned()
            )),
            "{text}"
        );
    }
}

#[test]
fn build_rejects_more_than_eight_orders_before_the_order_limit() {
    for text in [
        "*222222222",
        "*(1001)22222222",
        "222222222",
        "(1001)22222222",
    ] {
        assert_eq!(
            error_of(text),
            Some((
                ErrorKind::LimitExceeded,
                "cannot build fundamental domain: the signature has 9 orders, more than 8"
                    .to_owned()
            )),
            "{text}"
        );
    }
}

#[test]
fn build_accepts_the_order_at_the_limit() {
    for text in [
        "*(1000)32",
        "*(1000)22",
        "*(1000)(1000)",
        "*(1000)222",
        "*22222222",
        "(1000)32",
        "(1000)22",
        "(1000)(1000)",
        "(1000)222",
        "22222222",
        "3222",
        "22222",
        "3333",
        "o",
        "**",
        "xx",
        "*x",
        "x*",
        "22x",
        "x22",
        "2x2",
        "22*",
        "2*22",
        "4*2",
        "3*3",
    ] {
        let built = FundamentalDomain::build(&signature(text));
        assert!(built.is_ok(), "{text}: {built:?}");
    }
}
