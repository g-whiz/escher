use escher_cli::Status;

use crate::common::{run_args, snapshot};

#[test]
fn diagnostic_for_an_empty_signature_has_no_echo() {
    insta::assert_snapshot!(snapshot(&["info", ""]));
}

#[test]
fn diagnostic_marks_an_unexpected_character() {
    insta::assert_snapshot!(snapshot(&["info", "*7q2"]));
}

#[test]
fn diagnostic_marks_an_unclosed_parenthesis() {
    insta::assert_snapshot!(snapshot(&["info", "(12"]));
}

#[test]
fn diagnostic_marks_a_zero_order() {
    insta::assert_snapshot!(snapshot(&["info", "0"]));
}

#[test]
fn diagnostic_marks_a_digit_one_followed_by_a_digit() {
    insta::assert_snapshot!(snapshot(&["info", "2*12"]));
}

#[test]
fn diagnostic_marks_an_ambiguous_finite_order() {
    insta::assert_snapshot!(snapshot(&["info", "*2x3"]));
}

#[test]
fn diagnostic_marks_an_ambiguous_infinite_order() {
    insta::assert_snapshot!(snapshot(&["info", "*o(inf)"]));
}

#[test]
fn diagnostic_gives_the_byte_span_of_a_signature_over_the_length_limit() {
    insta::assert_snapshot!(snapshot(&["info", &"x".repeat(1025)]));
}

#[test]
fn diagnostic_marks_an_order_over_the_limit() {
    insta::assert_snapshot!(snapshot(&["info", "(4294967296)"]));
}

#[test]
fn diagnostic_marks_the_feature_over_the_size_limit() {
    insta::assert_snapshot!(snapshot(&["info", &"x".repeat(65)]));
}

#[test]
fn diagnostic_marks_a_bad_orbifold_as_a_whole() {
    insta::assert_snapshot!(snapshot(&["info", "23"]));
}

#[test]
fn diagnostic_marks_an_infinite_order_with_a_positive_characteristic() {
    insta::assert_snapshot!(snapshot(&["info", "2(inf)"]));
}

#[test]
fn diagnostic_marks_a_characteristic_that_overflows() {
    insta::assert_snapshot!(snapshot(&["info", "(3037000493)(3037000499)x"]));
}

#[test]
fn diagnostic_gives_the_byte_span_for_a_signature_over_120_characters() {
    let text = format!("{}q", "(2)".repeat(41));
    insta::assert_snapshot!(snapshot(&["info", &text]));
}

#[test]
fn diagnostic_echoes_a_control_character_as_a_question_mark() {
    insta::assert_snapshot!(snapshot(&["info", "*7\u{1B}[31m2"]));
}

#[test]
fn diagnostic_counts_the_infinity_sign_as_one_column() {
    insta::assert_snapshot!(snapshot(&["info", "*22\u{221E}q"]));
}

#[test]
fn diagnostic_marks_an_infinity_sign_with_one_caret() {
    insta::assert_snapshot!(snapshot(&["info", "(\u{221E})"]));
}

#[test]
fn diagnostic_echoes_a_signature_of_exactly_120_characters() {
    let text = format!("{}xxq", "(2)".repeat(39));
    assert_eq!(text.chars().count(), 120);
    let outcome = run_args(&["info", &text]);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(
        outcome.stderr,
        format!(
            "error: cannot parse orbifold signature: unexpected character 'q'\n  {text}\n  {}^\n",
            " ".repeat(119)
        ),
    );
}

#[test]
fn diagnostic_gives_the_byte_span_for_a_signature_of_121_characters() {
    let text = format!("{}xxxq", "(2)".repeat(39));
    assert_eq!(text.chars().count(), 121);
    let outcome = run_args(&["info", &text]);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(
        outcome.stderr,
        "error: cannot parse orbifold signature: unexpected character 'q'\n  at bytes 120..121\n",
    );
}

#[test]
fn diagnostic_echoes_a_signature_of_more_than_120_bytes_but_fewer_characters() {
    let text = format!("{}q", "\u{221E}".repeat(41));
    assert_eq!(text.chars().count(), 42);
    assert_eq!(text.len(), 124);
    let outcome = run_args(&["info", &text]);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(
        outcome.stderr,
        format!(
            "error: cannot parse orbifold signature: unexpected character 'q'\n  {text}\n  {}^\n",
            " ".repeat(41)
        ),
    );
}
