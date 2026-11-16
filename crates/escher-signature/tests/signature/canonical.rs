use crate::common::{canonical, parse};

#[test]
fn canonical_string_of_each_example_text() {
    let cases = [
        ("2o3", "o32"),
        ("x*2", "*2x"),
        ("*2x*3", "*3*2x"),
        ("*2x", "*2x"),
        ("*2x1", "*2x"),
        ("XO", "xxx"),
        ("2*(12)", "2*(12)"),
        ("*22(inf)", "*(inf)22"),
        ("*22\u{221e}", "*(inf)22"),
        ("1", "1"),
        ("(1)", "1"),
        ("(1)(1)", "1"),
        ("*", "*"),
        ("x", "x"),
    ];
    for (text, expected) in cases {
        assert_eq!(canonical(text), expected, "{text:?}");
    }
}

#[test]
fn canonical_string_applies_each_equivalence_rule() {
    let cases = [
        ("*236", "*632"),
        ("*2345", "*5432"),
        ("ox", "xxx"),
        ("oox", "xxxxx"),
        ("o", "o"),
        ("*2345*6432x", "*6432*5432x"),
        ("*2345*2346x", "*6432*5432x"),
        ("*2345*6432", "*6432*5234"),
        ("*2345*2346", "*6432*5432"),
        ("*1", "*"),
        ("1*", "*"),
        ("**2", "*2*"),
        ("*23(inf)", "*(inf)32"),
    ];
    for (text, expected) in cases {
        assert_eq!(canonical(text), expected, "{text:?}");
    }
}

#[test]
fn canonical_string_reverses_one_boundary_only_with_a_crosscap() {
    assert_ne!(parse("*2345*6432"), parse("*2345*2346"));
    assert_eq!(parse("*2345*6432x"), parse("*2345*2346x"));
}

#[test]
fn canonical_string_is_a_fixed_point_of_parsing() {
    for text in [
        "o32",
        "*3*2x",
        "xxx",
        "*(inf)22",
        "1",
        "*6432*5234",
        "2*(12)",
    ] {
        assert_eq!(canonical(text), text, "{text:?}");
    }
}
