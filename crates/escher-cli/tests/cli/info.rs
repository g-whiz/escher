use std::io;

use escher_cli::{Status, run};

use crate::common::{FailingWriter, run_args, snapshot};

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_rigid_compact_hyperbolic_signature() {
    insta::assert_snapshot!(snapshot(&["info", "*732"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_spherical_signature_with_its_group_order() {
    insta::assert_snapshot!(snapshot(&["info", "*532"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_hyperbolic_signature_with_a_cone_point_and_a_corner() {
    insta::assert_snapshot!(snapshot(&["info", "4*3"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_signature_with_one_deformation_parameter() {
    insta::assert_snapshot!(snapshot(&["info", "*3333"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_an_order_of_ten_or_more_in_parentheses() {
    insta::assert_snapshot!(snapshot(&["info", "2*(12)"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_frieze_with_the_infinite_order_first() {
    insta::assert_snapshot!(snapshot(&["info", "*22(inf)"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_euclidean_signature() {
    insta::assert_snapshot!(snapshot(&["info", "*632"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_trivial_signature() {
    insta::assert_snapshot!(snapshot(&["info", "1"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_general_non_compact_signature_with_its_area() {
    insta::assert_snapshot!(snapshot(&["info", "3*(inf)"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_non_compact_triangle_group_as_not_general() {
    insta::assert_snapshot!(snapshot(&["info", "*(inf)32"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_a_handle_and_a_crosscap_as_three_crosscaps() {
    insta::assert_snapshot!(snapshot(&["info", "ox"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_canonical_order_of_the_corners() {
    insta::assert_snapshot!(snapshot(&["info", "*236"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_an_area_of_exactly_pi_without_a_coefficient() {
    insta::assert_snapshot!(snapshot(&["info", "2*"]));
}

#[test]
fn info_returns_success_for_a_valid_signature() {
    let outcome = run_args(&["info", "*732"]);
    assert_eq!(outcome.status, Status::Success);
    assert!(
        outcome
            .stdout
            .starts_with("Signature: *732\nGeometry: hyperbolic\n"),
        "{}",
        outcome.stdout
    );
    assert_eq!(outcome.stderr, "");
}

#[test]
fn info_returns_failure_for_a_bad_orbifold() {
    let outcome = run_args(&["info", "23"]);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert!(
        outcome
            .stderr
            .starts_with("error: cannot parse orbifold signature: bad orbifold"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn info_prints_the_same_text_for_both_spellings_of_infinity() {
    let ascii = run_args(&["info", "*22(inf)"]);
    let sign = run_args(&["info", "*22\u{221E}"]);
    assert_eq!(ascii.status, Status::Success);
    assert_eq!(sign.status, Status::Success);
    assert_eq!(ascii.stdout, sign.stdout);
    assert_eq!(sign.stderr, "");
}

#[test]
fn info_gives_failure_and_no_diagnostic_on_a_broken_pipe() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::BrokenPipe);
    let mut stderr = Vec::new();
    let status = run(["escher", "info", "*632"], &mut stdout, &mut stderr);
    assert_eq!(status, Status::Failure);
    assert_eq!(stderr, b"");
}

#[test]
fn info_gives_failure_and_a_diagnostic_when_standard_output_fails() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::StorageFull);
    let mut stderr = Vec::new();
    let status = run(["escher", "info", "*632"], &mut stdout, &mut stderr);
    assert_eq!(status, Status::Failure);
    assert_eq!(
        String::from_utf8(stderr).expect("standard error is UTF-8"),
        format!(
            "error: cannot write to standard output\n  caused by: {}\n",
            io::Error::from(io::ErrorKind::StorageFull)
        ),
    );
}

#[test]
fn info_gives_failure_when_flushing_standard_output_fails() {
    let mut stdout = FailingWriter::failing_flush(io::ErrorKind::StorageFull);
    let mut stderr = Vec::new();
    let status = run(["escher", "info", "*632"], &mut stdout, &mut stderr);
    assert_eq!(status, Status::Failure);
    assert!(
        String::from_utf8(stderr)
            .expect("standard error is UTF-8")
            .starts_with("error: cannot write to standard output\n"),
    );
}

#[test]
fn info_keeps_failure_when_standard_error_fails_too() {
    let mut stdout = Vec::new();
    let mut stderr = FailingWriter::failing_write(io::ErrorKind::BrokenPipe);
    let status = run(["escher", "info", "23"], &mut stdout, &mut stderr);
    assert_eq!(status, Status::Failure);
    assert_eq!(stdout, b"");
}
