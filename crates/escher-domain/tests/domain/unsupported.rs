use escher_domain::{ErrorKind, FundamentalDomain};

use crate::common::signature;

const UNSUPPORTED: [&str; 15] = [
    "632",
    "*22222",
    "*532",
    "*3333",
    "732",
    "4*3",
    "*(inf)32",
    "*(inf)(inf)",
    "(inf)(inf)",
    "*22(inf)",
    "1",
    "*",
    "o",
    "*732*",
    "x732",
];

#[test]
fn build_rejects_every_unsupported_signature() {
    for text in UNSUPPORTED {
        let canonical = signature(text).to_string();
        let error = FundamentalDomain::build(&signature(text)).err();
        assert_eq!(
            error.as_ref().map(escher_domain::Error::kind),
            Some(ErrorKind::UnsupportedSignature),
            "{text}"
        );
        assert_eq!(
            error.map(|error| error.to_string()),
            Some(format!(
                "cannot build fundamental domain: the signature {canonical} is not supported"
            )),
            "{text}"
        );
    }
}

#[test]
fn build_rejects_an_order_above_the_limit() {
    let error = FundamentalDomain::build(&signature("*(1001)32")).err();
    assert_eq!(
        error.as_ref().map(escher_domain::Error::kind),
        Some(ErrorKind::LimitExceeded)
    );
    assert_eq!(
        error.map(|error| error.to_string()).as_deref(),
        Some("cannot build fundamental domain: order 1001 is larger than 1000")
    );
}

#[test]
fn build_accepts_the_order_at_the_limit() {
    let built = FundamentalDomain::build(&signature("*(1000)32"));
    assert!(built.is_ok(), "{built:?}");
}

#[test]
fn build_rejects_an_unsupported_signature_over_the_limit_as_unsupported() {
    let error = FundamentalDomain::build(&signature("*(1001)(1001)(1001)(1001)")).err();
    assert_eq!(
        error.map(|error| error.kind()),
        Some(ErrorKind::UnsupportedSignature)
    );
}
