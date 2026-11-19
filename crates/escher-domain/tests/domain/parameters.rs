use escher_domain::{ErrorKind, FundamentalDomain, ShapeParameters, parameters};

use crate::common::{assert_same_domain, signature};
use crate::euclidean::VERTEX_BOUND;
use crate::shared::{NAMED_POLYGONS, corner_orders};

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
    assert_eq!(parameters(&signature("2222"))[..1], specs);
    for text in [
        "*632", "*442", "*333", "*732", "*532", "732", "532", "4*2", "3*3",
    ] {
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
    for text in ["*632", "*732", "732", "4*2", "3*3"] {
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
        error_of("4*3", &settings("aspect", 2.0)),
        Some((
            ErrorKind::UnsupportedSignature,
            "cannot build fundamental domain: the signature 4*3 is not supported".to_owned()
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

const POLYGON_RANGE: [f64; 6] = [
    0.4999,
    3.5001,
    0.0,
    f64::NAN,
    f64::INFINITY,
    f64::NEG_INFINITY,
];

fn listed(text: &str) -> Vec<(String, f64, f64, f64)> {
    parameters(&signature(text))
        .iter()
        .map(|spec| {
            (
                spec.name().to_owned(),
                spec.minimum(),
                spec.maximum(),
                spec.default_value(),
            )
        })
        .collect()
}

fn names(text: &str) -> Vec<String> {
    listed(text).into_iter().map(|(name, ..)| name).collect()
}

#[test]
fn parameters_of_polygons_have_their_names_ranges_and_rounded_defaults() {
    let length = |name: &str, default| (name.to_owned(), 0.5, 3.5, default);
    assert_eq!(listed("*3222"), [length("cornerpair1.length", 0.658)]);
    assert_eq!(listed("*2223"), listed("*3222"));
    assert_eq!(listed("*3322"), [length("cornerpair1.length", 0.867)]);
    assert_eq!(listed("*3333"), [length("slice1.length", 1.317)]);
    assert_eq!(
        listed("*22222"),
        [
            length("cornerpair1.length", 1.061),
            length("cornerpair2.length", 1.061)
        ]
    );
    assert_eq!(names("*5432"), ["slice1.length"]);
    assert_eq!(
        names("*222222"),
        [
            "cornerpair1.length",
            "cornerpair2.length",
            "cornerpair3.length"
        ]
    );
    assert_eq!(
        names("*2222222"),
        [
            "cornerpair1.length",
            "cornerpair2.length",
            "cornerpair3.length",
            "slice1.length"
        ]
    );
}

#[test]
fn parameters_of_every_named_polygon_list_the_corner_pairs_then_the_slices() {
    for text in NAMED_POLYGONS {
        let orders = corner_orders(&signature(text));
        let listed = names(text);
        assert_eq!(listed.len(), orders.len() - 3, "{text}");
        let pairs = listed
            .iter()
            .take_while(|name| name.starts_with("cornerpair"))
            .count();
        let expected: Vec<String> = (1..=pairs)
            .map(|number| format!("cornerpair{number}.length"))
            .chain((1..=listed.len() - pairs).map(|number| format!("slice{number}.length")))
            .collect();
        assert_eq!(listed, expected, "{text}");
    }
    for text in ["3333", "3222", "22222", "*222222222", "*(1001)222"] {
        assert!(parameters(&signature(text)).is_empty(), "{text}");
    }
}

#[test]
fn build_with_rejects_a_length_outside_the_range() {
    for (text, name) in [("*3333", "slice1.length"), ("*22222", "cornerpair2.length")] {
        for value in POLYGON_RANGE {
            assert_eq!(
                error_of(text, &settings(name, value)),
                Some((
                    ErrorKind::ParameterOutOfRange,
                    format!(
                        "cannot build fundamental domain: the parameter {name} is outside its range of 0.5 to 3.5"
                    )
                )),
                "{text} {value}"
            );
        }
        for value in [0.5, 3.5] {
            let built = FundamentalDomain::build_with(&signature(text), &settings(name, value));
            match built {
                Ok(domain) => assert!(domain.verify().passed(), "{text} {value}"),
                Err(error) => assert_eq!(error.kind(), ErrorKind::Numerical, "{text} {value}"),
            }
        }
    }
}

#[test]
fn build_with_rejects_a_name_that_the_polygon_does_not_have() {
    for (text, name) in [
        ("*3333", "slice1.Length"),
        ("*3333", "slice2.length"),
        ("*3333", "cornerpair1.length"),
        ("*22222", "slice1.length"),
        ("3333", "slice1.length"),
        ("3333", "aspect"),
    ] {
        assert_eq!(
            error_of(text, &settings(name, 2.0)),
            Some((
                ErrorKind::UnknownParameter,
                format!(
                    "cannot build fundamental domain: the signature {text} has no shape parameter with one of the given names"
                )
            )),
            "{text} {name}"
        );
    }
}

const SKEW_RANGE_TEXT: &str =
    "cannot build fundamental domain: the parameter skew is outside its range of -0.5 to 0.5";

#[test]
fn parameters_of_the_built_wallpaper_domains_have_their_names_ranges_and_defaults() {
    let entry = |name: &str, minimum: f64, maximum: f64, default: f64| {
        (name.to_owned(), minimum, maximum, default)
    };
    for text in ["o", "2222"] {
        assert_eq!(
            listed(text),
            [
                entry("aspect", 0.125, 8.0, 1.0),
                entry("skew", -0.5, 0.5, 0.0)
            ],
            "{text}"
        );
    }
    for text in ["**", "xx", "*x", "22x", "22*", "2*22"] {
        assert_eq!(listed(text), [entry("aspect", 0.125, 8.0, 1.0)], "{text}");
    }
    for text in ["4*2", "3*3"] {
        assert_eq!(listed(text), [], "{text}");
    }
}

#[test]
fn build_with_rejects_a_skew_outside_its_range_and_accepts_both_ends() {
    for value in [
        -0.5001,
        0.5001,
        0.6,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        for text in ["o", "2222"] {
            assert_eq!(
                error_of(text, &settings("skew", value)),
                Some((ErrorKind::ParameterOutOfRange, SKEW_RANGE_TEXT.to_owned())),
                "{text} at {value}"
            );
        }
    }
    for value in [-0.5, 0.5] {
        for text in ["o", "2222"] {
            let built = FundamentalDomain::build_with(&signature(text), &settings("skew", value));
            assert!(built.is_ok(), "{text} at {value}: {built:?}");
        }
    }
}

#[test]
fn build_with_rejects_skew_on_a_signature_without_it() {
    for text in ["**", "xx", "*x", "22x", "22*", "2*22", "*2222"] {
        let canonical = signature(text).to_string();
        assert_eq!(
            error_of(text, &settings("skew", 0.1)),
            Some((
                ErrorKind::UnknownParameter,
                format!(
                    "cannot build fundamental domain: the signature {canonical} has no shape parameter with one of the given names"
                )
            )),
            "{text}"
        );
    }
}
