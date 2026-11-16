use escher_signature::ErrorKind;

use crate::common::{canonical, chi, kind_and_span, parse};

#[test]
fn parse_reports_the_kind_and_span_of_each_rejected_text() {
    let too_long = "x".repeat(1025);
    let cases = [
        ("", ErrorKind::Empty, 0..0),
        ("*7q2", ErrorKind::UnexpectedCharacter, 2..3),
        ("*7 32", ErrorKind::UnexpectedCharacter, 2..3),
        ("()", ErrorKind::UnexpectedCharacter, 1..2),
        ("(INF)", ErrorKind::UnexpectedCharacter, 1..2),
        ("(in)", ErrorKind::UnexpectedCharacter, 3..4),
        ("(\u{221e})", ErrorKind::UnexpectedCharacter, 1..4),
        ("(12", ErrorKind::UnclosedParenthesis, 0..3),
        ("0", ErrorKind::ZeroOrder, 0..1),
        ("(00)", ErrorKind::ZeroOrder, 0..4),
        ("11", ErrorKind::MultiDigitOrder, 0..2),
        ("*11", ErrorKind::MultiDigitOrder, 1..3),
        ("2*12", ErrorKind::MultiDigitOrder, 2..4),
        ("*2x3", ErrorKind::AmbiguousOrder, 3..4),
        ("*o2", ErrorKind::AmbiguousOrder, 2..3),
        ("(4294967296)", ErrorKind::LimitExceeded, 0..12),
        (too_long.as_str(), ErrorKind::LimitExceeded, 0..1025),
        ("23", ErrorKind::BadOrbifold, 0..2),
        ("2(inf)", ErrorKind::PositiveNonCompact, 0..6),
        ("\u{221e}", ErrorKind::PositiveNonCompact, 0..3),
    ];
    for (text, kind, span) in cases {
        assert_eq!(kind_and_span(text), (kind, span), "{text:?}");
    }
}

#[test]
fn parse_rejects_whitespace_everywhere() {
    for text in [" 632", "632 ", "6\t32", "*632\n", "\u{a0}632"] {
        assert_eq!(
            kind_and_span(text).0,
            ErrorKind::UnexpectedCharacter,
            "{text:?}"
        );
    }
}

#[test]
fn parse_rejects_characters_outside_the_grammar() {
    let cases = [
        ("\u{2217}632", ErrorKind::UnexpectedCharacter, 0..3),
        ("6\u{0663}2", ErrorKind::UnexpectedCharacter, 1..3),
        ("o\u{25cb}", ErrorKind::UnexpectedCharacter, 1..4),
        ("*6(3", ErrorKind::UnclosedParenthesis, 2..4),
        ("2(inf", ErrorKind::UnclosedParenthesis, 1..5),
    ];
    for (text, kind, span) in cases {
        assert_eq!(kind_and_span(text), (kind, span), "{text:?}");
    }
}

#[test]
fn parse_returns_the_first_error_in_the_text() {
    assert_eq!(kind_and_span("0q"), (ErrorKind::ZeroOrder, 0..1));
    assert_eq!(kind_and_span("q0"), (ErrorKind::UnexpectedCharacter, 0..1));
    assert_eq!(kind_and_span("*2x3 "), (ErrorKind::AmbiguousOrder, 3..4));
    assert_eq!(
        kind_and_span("(0)(4294967296)"),
        (ErrorKind::ZeroOrder, 0..3)
    );
}

#[test]
fn parse_accepts_orders_of_one_wherever_they_stand() {
    let cases = [
        ("1", "1"),
        ("*1", "*"),
        ("1*", "*"),
        ("221", "22"),
        ("2*1", "2*"),
        ("(1)", "1"),
        ("(01)", "1"),
        ("*2x1", "*2x"),
        ("*2x(1)", "*2x"),
        ("o1", "o"),
    ];
    for (text, expected) in cases {
        assert_eq!(canonical(text), expected, "{text:?}");
    }
}

#[test]
fn parse_reads_parenthesized_orders_by_value() {
    assert_eq!(canonical("(07)(3)(2)"), "732");
    assert_eq!(canonical("*(000012)(12)"), "*(12)(12)");
    assert_eq!(
        canonical("(4294967295)(4294967295)"),
        "(4294967295)(4294967295)"
    );
}

#[test]
fn parse_accepts_the_infinity_sign_and_lower_case_inf_only() {
    assert_eq!(canonical("\u{221e}\u{221e}"), "(inf)(inf)");
    assert_eq!(canonical("*22\u{221e}"), "*(inf)22");
    assert_eq!(
        kind_and_span("(Inf)(inf)"),
        (ErrorKind::UnexpectedCharacter, 1..2)
    );
}

#[test]
fn parse_accepts_upper_case_handles_and_crosscaps() {
    assert_eq!(canonical("O"), "o");
    assert_eq!(canonical("XO"), "xxx");
    assert_eq!(canonical("22X"), "22x");
}

#[test]
fn parse_rejects_more_than_sixty_four_features() {
    assert!(parse(&"2".repeat(64)).is_ok());
    assert_eq!(
        kind_and_span(&"2".repeat(65)),
        (ErrorKind::LimitExceeded, 64..65)
    );
    assert_eq!(
        kind_and_span(&"*".repeat(65)),
        (ErrorKind::LimitExceeded, 64..65)
    );
}

#[test]
fn parse_counts_a_handle_as_two_features() {
    assert!(parse(&"o".repeat(32)).is_ok());
    assert_eq!(
        kind_and_span(&"o".repeat(33)),
        (ErrorKind::LimitExceeded, 32..33)
    );
    let mixed = format!("{}{}", "o".repeat(31), "xxx");
    assert_eq!(kind_and_span(&mixed), (ErrorKind::LimitExceeded, 33..34));
}

#[test]
fn parse_accepts_the_longest_canonical_string() {
    let longest = "(4294967295)".repeat(64);
    assert_eq!(longest.len(), 768);
    assert_eq!(canonical(&longest), longest);
}

#[test]
fn parse_computes_the_characteristic_when_the_common_denominator_exceeds_i64() {
    let signature = parse("(2147483647)(2147483649)2").expect("the reduced values fit");
    let chi = signature.euler_characteristic();
    assert!(chi.numerator() < 0);
    assert!(chi.denominator().get() > 1);
}

#[test]
fn parse_computes_the_largest_characteristic_of_the_prime_power_table() {
    assert_eq!(
        chi("(41)(37)(32)(31)(29)(27)(25)(23)(19)(17)(13)(11)7"),
        "-2258057581140506927/219060189739591200"
    );
}

#[test]
fn parse_rejects_a_characteristic_that_does_not_fit_with_overflow() {
    for text in [
        "(43)(41)(37)(32)(31)(29)(27)(25)(23)(19)(17)(13)(11)7",
        "(3037000493)(3037000499)x",
    ] {
        assert_eq!(
            kind_and_span(text),
            (ErrorKind::Overflow, 0..text.len()),
            "{text:?}"
        );
    }
}

#[test]
fn parse_reports_overflow_before_the_bad_orbifold_check() {
    let text = "*(4294967295)(4294967294)";
    assert_eq!(kind_and_span(text), (ErrorKind::Overflow, 0..text.len()));
    let fits = "(4294967295)(4294967294)";
    assert_eq!(kind_and_span(fits), (ErrorKind::BadOrbifold, 0..fits.len()));
}

#[test]
fn parse_rejects_infinite_orders_with_a_positive_characteristic() {
    for text in ["(inf)", "2(inf)", "*(inf)", "*2(inf)", "*(inf)3"] {
        assert_eq!(
            kind_and_span(text),
            (ErrorKind::PositiveNonCompact, 0..text.len()),
            "{text:?}"
        );
    }
}

#[test]
fn parse_accepts_infinite_orders_with_a_characteristic_of_zero_or_less() {
    assert_eq!(chi("(inf)*"), "0");
    assert_eq!(chi("(inf)x"), "0");
    assert_eq!(chi("3*(inf)"), "-1/6");
}
