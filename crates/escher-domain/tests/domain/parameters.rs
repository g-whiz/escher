use escher_domain::{ErrorKind, FundamentalDomain, ShapeParameters, parameters};

use crate::common::{assert_same_domain, signature};
use crate::euclidean::VERTEX_BOUND;

const OUT_OF_RANGE: [f64; 7] = [
    0.1249,
    8.0001,
    0.0,
    -1.0,
    f64::NAN,
    f64::INFINITY,
    f64::NEG_INFINITY,
];

const RANGE_TEXT: &str =
    "cannot build fundamental domain: the parameter aspect is outside its range of 0.125 to 8";

fn settings(name: &str, value: f64) -> ShapeParameters {
    let mut settings = ShapeParameters::new();
    settings.set(name, value);
    settings
}

fn error_of(text: &str, settings: &ShapeParameters) -> Option<(ErrorKind, String)> {
    FundamentalDomain::build_with(&signature(text), settings)
        .err()
        .map(|error| (error.kind(), error.to_string()))
}

#[test]
fn parameters_list_aspect_for_the_rectangle_only() {
    let specs = parameters(&signature("*2222"));
    let listed: Vec<(&str, f64, f64, f64)> = specs
        .iter()
        .map(|spec| {
            (
                spec.name(),
                spec.minimum(),
                spec.maximum(),
                spec.default_value(),
            )
        })
        .collect();
    assert_eq!(listed, [("aspect", 0.125, 8.0, 1.0)]);
    assert_eq!(parameters(&signature("2222")), specs);
    for text in ["*632", "*442", "*333", "*732", "*532", "732", "532"] {
        assert!(parameters(&signature(text)).is_empty(), "{text}");
    }
}

#[test]
fn build_with_rejects_a_value_outside_the_range() {
    for text in ["*2222", "2222"] {
        for value in OUT_OF_RANGE {
            assert_eq!(
                error_of(text, &settings("aspect", value)),
                Some((ErrorKind::ParameterOutOfRange, RANGE_TEXT.to_owned())),
                "{text} {value}"
            );
        }
    }
}

#[test]
fn build_with_accepts_both_ends_of_the_range() {
    for text in ["*2222", "2222"] {
        for value in [0.125, 8.0] {
            let built = FundamentalDomain::build_with(&signature(text), &settings("aspect", value));
            assert!(built.is_ok(), "{text} {value}: {built:?}");
        }
    }
}

#[test]
fn build_with_rejects_an_unknown_name() {
    let rectangle_text = "cannot build fundamental domain: the signature *2222 has no shape parameter with one of the given names";
    for name in ["Aspect", "aspect ", ""] {
        assert_eq!(
            error_of("*2222", &settings(name, 2.0)),
            Some((ErrorKind::UnknownParameter, rectangle_text.to_owned())),
            "{name:?}"
        );
    }
    for text in ["*632", "*732", "732"] {
        assert_eq!(
            error_of(text, &settings("aspect", 2.0)),
            Some((
                ErrorKind::UnknownParameter,
                format!(
                    "cannot build fundamental domain: the signature {text} has no shape parameter with one of the given names"
                )
            )),
            "{text}"
        );
    }
}

#[test]
fn build_with_rejects_an_unsupported_signature_before_its_settings() {
    assert_eq!(
        error_of("3*2", &settings("aspect", 2.0)),
        Some((
            ErrorKind::UnsupportedSignature,
            "cannot build fundamental domain: the signature 3*2 is not supported".to_owned()
        ))
    );
}

#[test]
fn build_is_build_with_no_setting() {
    for text in ["*2222", "*632"] {
        let (built, with) = (
            FundamentalDomain::build(&signature(text)),
            FundamentalDomain::build_with(&signature(text), &ShapeParameters::new()),
        );
        let (Ok(built), Ok(with)) = (built, with) else {
            panic!("{text} has no domain");
        };
        assert_same_domain(&built, &with, 2.0 * VERTEX_BOUND, text);
    }
}

#[test]
fn set_returns_the_value_that_it_replaces() {
    let mut settings = ShapeParameters::new();
    assert_eq!(settings, ShapeParameters::default());
    assert_eq!(settings.set("aspect", 2.0), None);
    assert_eq!(settings.set("aspect", 3.0), Some(2.0));
    assert_eq!(settings.set("other", 4.0), None);
}
