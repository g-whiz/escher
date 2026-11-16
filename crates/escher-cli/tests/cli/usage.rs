use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStringExt as _;

use escher_cli::{Status, run};

use crate::common::{FailingWriter, run_args, run_os, snapshot};

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn help_option_prints_the_help_on_standard_output() {
    insta::assert_snapshot!(snapshot(&["--help"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn short_help_option_prints_the_help_on_standard_output() {
    insta::assert_snapshot!(snapshot(&["-h"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
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
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn no_argument_prints_the_help_on_standard_error() {
    insta::assert_snapshot!(snapshot(&[]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn unknown_subcommand_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["draw"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn help_subcommand_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["help"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn unknown_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["-x"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn missing_signature_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["info"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn repeated_check_option_is_a_usage_error() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "--check", "*732"]));
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
