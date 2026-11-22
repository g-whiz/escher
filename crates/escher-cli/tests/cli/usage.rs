use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStringExt as _;

use escher_cli::{Status, run};

use crate::common::{FailingWriter, NEVER_WRITTEN, run_args, run_os, snapshot};

#[test]
fn help_option_prints_the_help_on_standard_output() {
    insta::assert_snapshot!(snapshot(&["--help"]));
}

#[test]
fn short_help_option_prints_the_help_on_standard_output() {
    insta::assert_snapshot!(snapshot(&["-h"]));
}

#[test]
fn info_help_option_prints_the_help_of_info() {
    insta::assert_snapshot!(snapshot(&["info", "--help"]));
}

#[test]
fn version_option_prints_the_package_name_and_version() {
    let outcome = run_args(&["--version"]);
    assert_eq!(outcome.status, Status::Success);
    assert_eq!(
        outcome.stdout,
        format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(outcome.stderr, "");
}

#[test]
fn short_version_option_prints_the_package_name_and_version() {
    let outcome = run_args(&["-V"]);
    assert_eq!(outcome.status, Status::Success);
    assert_eq!(
        outcome.stdout,
        format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(outcome.stderr, "");
}

#[test]
fn no_argument_prints_the_help_on_standard_error() {
    insta::assert_snapshot!(snapshot(&[]));
}

#[test]
fn unknown_subcommand_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["draw"]));
}

#[test]
fn help_subcommand_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["help"]));
}

#[test]
fn unknown_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["-x"]));
}

#[test]
fn missing_signature_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["info"]));
}

#[test]
fn repeated_check_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "--check", "*732"]));
}

#[test]
fn repeated_presentation_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--presentation",
        "--presentation",
        "*732"
    ]));
}

#[test]
fn repeated_colourings_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["info", "--colourings", "--colourings", "*732"]));
}

#[test]
fn repeated_max_colours_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--max-colours",
        "3",
        "--max-colours",
        "4",
        "*732"
    ]));
}

#[test]
fn max_colours_of_zero_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "0", "*732"]));
}

#[test]
fn max_colours_over_the_limit_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "65", "*732"]));
}

#[test]
fn max_colours_that_is_not_a_number_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "x", "*732"]));
}

#[test]
fn unknown_subcommand_returns_usage() {
    let outcome = run_args(&["draw"]);
    assert_eq!(outcome.status, Status::Usage);
    assert_eq!(outcome.stdout, "");
    assert!(
        outcome
            .stderr
            .starts_with("error: unrecognized subcommand 'draw'\n"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn argument_that_is_not_utf8_returns_usage() {
    let outcome = run_os(vec![
        OsString::from("info"),
        OsString::from_vec(vec![b'*', 0xFF, b'2']),
    ]);
    assert_eq!(outcome.status, Status::Usage);
    assert_eq!(outcome.stdout, "");
    assert!(
        outcome
            .stderr
            .starts_with("error: invalid UTF-8 was detected in one or more arguments\n"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn usage_error_replaces_control_characters_of_the_argument() {
    let outcome = run_args(&["info", "--\u{1B}]0;title\u{7}"]);
    assert_eq!(outcome.status, Status::Usage);
    assert!(
        outcome.stderr.contains("'--?]0;title?'"),
        "{}",
        outcome.stderr
    );
    assert!(
        !outcome
            .stderr
            .chars()
            .any(|character| character.is_control() && character != '\n'),
        "{:?}",
        outcome.stderr,
    );
}

#[test]
fn version_gives_failure_and_no_diagnostic_on_a_broken_pipe() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::BrokenPipe);
    let mut stderr = Vec::new();
    let status = run(["escher", "--version"], &mut stdout, &mut stderr);
    assert_eq!(status, Status::Failure);
    assert_eq!(stderr, b"");
}

#[test]
fn usage_error_keeps_usage_when_standard_error_fails() {
    let mut stdout = Vec::new();
    let mut stderr = FailingWriter::failing_write(io::ErrorKind::BrokenPipe);
    let status = run(["escher", "draw"], &mut stdout, &mut stderr);
    assert_eq!(status, Status::Usage);
    assert_eq!(stdout, b"");
}

#[test]
fn render_help_option_prints_the_help_of_render() {
    let text = snapshot(&["render", "--help"]);
    let mut trimmed = String::new();
    for line in text.lines() {
        trimmed.push_str(line.trim_end());
        trimmed.push('\n');
    }
    insta::assert_snapshot!(trimmed);
}

#[test]
fn render_short_help_option_prints_the_summary_of_render() {
    insta::assert_snapshot!(snapshot(&["render", "-h"]));
}

#[test]
fn render_missing_signature_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["render", "-o", NEVER_WRITTEN]));
}

#[test]
fn render_missing_output_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["render", "*732"]));
}

#[test]
fn render_size_without_a_height_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--size",
        "512"
    ]));
}

#[test]
fn render_size_with_an_empty_height_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--size",
        "512x"
    ]));
}

#[test]
fn render_samples_that_are_not_a_number_are_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--samples",
        "x"
    ]));
}

#[test]
fn render_unknown_colouring_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--colouring",
        "three"
    ]));
}

#[test]
fn render_colouring_id_with_upper_case_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--colouring",
        "3A"
    ]));
}

#[test]
fn render_colouring_id_with_leading_zero_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--colouring",
        "03a"
    ]));
}

#[test]
fn render_colouring_id_with_too_many_colours_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--colouring",
        "65a"
    ]));
}

#[test]
fn render_colouring_id_with_too_many_letters_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--colouring",
        "3abcde"
    ]));
}

#[test]
fn render_zero_threads_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--threads",
        "0"
    ]));
}

#[test]
fn render_threads_over_the_limit_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--threads",
        "257"
    ]));
}

#[test]
fn render_repeated_stats_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--stats",
        "--stats"
    ]));
}

#[test]
fn render_repeated_size_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "-o",
        NEVER_WRITTEN,
        "--size",
        "8x8",
        "--size",
        "16x16"
    ]));
}

#[test]
fn render_setting_without_a_value_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*2222",
        "-o",
        NEVER_WRITTEN,
        "--set",
        "aspect"
    ]));
}

#[test]
fn render_setting_without_a_name_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*2222",
        "-o",
        NEVER_WRITTEN,
        "--set",
        "=2"
    ]));
}

#[test]
fn render_rotation_with_two_angles_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*532",
        "-o",
        NEVER_WRITTEN,
        "--rotation",
        "1,2"
    ]));
}

#[test]
fn render_rotation_that_is_not_numbers_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*532",
        "-o",
        NEVER_WRITTEN,
        "--rotation",
        "a,b,c"
    ]));
}

#[test]
fn render_rotation_with_four_angles_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*532",
        "-o",
        NEVER_WRITTEN,
        "--rotation",
        "1,2,3,4"
    ]));
}

#[test]
fn render_light_with_an_empty_component_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*532",
        "-o",
        NEVER_WRITTEN,
        "--light",
        "1,,2"
    ]));
}

#[test]
fn render_unknown_shading_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*532",
        "-o",
        NEVER_WRITTEN,
        "--shading",
        "gouraud"
    ]));
}

#[test]
fn render_repeated_rotation_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*532",
        "-o",
        NEVER_WRITTEN,
        "--rotation",
        "0,0,0",
        "--rotation",
        "1,1,1"
    ]));
}

#[test]
fn render_rotation_followed_by_an_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*532",
        "-o",
        NEVER_WRITTEN,
        "--rotation",
        "--stats"
    ]));
}
