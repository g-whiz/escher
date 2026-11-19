use escher_domain::{ErrorKind, FundamentalDomain};

use crate::common::signature;

const UNSUPPORTED: [&str; 19] = [
    "3*2",
    "3*3",
    "4*3",
    "2*22",
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
    ] {
        let built = FundamentalDomain::build(&signature(text));
        assert!(built.is_ok(), "{text}: {built:?}");
    }
}
