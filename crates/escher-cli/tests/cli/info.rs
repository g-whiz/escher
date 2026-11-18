use std::io;

use escher_cli::{Status, run};
use escher_domain::{Check, FundamentalDomain};
use escher_signature::Signature;

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
fn info_prints_a_spherical_signature_without_a_domain() {
    insta::assert_snapshot!(snapshot(&["info", "3*2"]));
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
    let status = run(["escher", "info", "3*2"], &mut stdout, &mut stderr);
    assert_eq!(status, Status::Failure);
    assert_eq!(stderr, b"");
}

#[test]
fn info_gives_failure_and_a_diagnostic_when_standard_output_fails() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::StorageFull);
    let mut stderr = Vec::new();
    let status = run(["escher", "info", "3*2"], &mut stdout, &mut stderr);
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
    let status = run(["escher", "info", "3*2"], &mut stdout, &mut stderr);
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

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_triangle_with_a_right_angle() {
    insta::assert_snapshot!(snapshot(&["info", "*542"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_triangle_with_two_equal_corners() {
    insta::assert_snapshot!(snapshot(&["info", "*433"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_no_domain_for_a_corner_order_over_the_limit() {
    insta::assert_snapshot!(snapshot(&["info", "*(1001)32"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_prints_each_check_of_a_verified_domain() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*732"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_prints_a_diagnostic_for_a_signature_without_a_domain() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "3*2"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_hyperbolic_kite() {
    insta::assert_snapshot!(snapshot(&["info", "732"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_prints_each_check_of_a_hyperbolic_kite() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "732"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_euclidean_kite() {
    insta::assert_snapshot!(snapshot(&["info", "442"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_the_doubled_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "2222"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_spherical_kite() {
    insta::assert_snapshot!(snapshot(&["info", "532"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_spherical_kite_with_a_flat_vertex() {
    insta::assert_snapshot!(snapshot(&["info", "(12)22"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_doubled_lune() {
    insta::assert_snapshot!(snapshot(&["info", "55"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_the_hemisphere() {
    insta::assert_snapshot!(snapshot(&["info", "22"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_euclidean_triangle_with_two_equal_corners() {
    insta::assert_snapshot!(snapshot(&["info", "*442"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_the_equilateral_euclidean_triangle() {
    insta::assert_snapshot!(snapshot(&["info", "*333"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_the_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "*2222"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_prints_each_check_of_the_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*2222"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_verifies_a_domain_at_the_largest_corner_order() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*(1000)32"]));
}

#[test]
fn info_check_returns_success_when_every_check_passes() {
    let outcome = run_args(&["info", "--check", "*732"]);
    assert_eq!(outcome.status, Status::Success);
    assert!(
        outcome.stdout.ends_with("\nVerification: passed\n"),
        "{}",
        outcome.stdout
    );
    assert_eq!(outcome.stderr, "");
}

#[test]
fn info_check_after_the_signature_returns_success_when_every_check_passes() {
    let outcome = run_args(&["info", "*732", "--check"]);
    assert_eq!(outcome.status, Status::Success);
    assert!(
        outcome.stdout.ends_with("\nVerification: passed\n"),
        "{}",
        outcome.stdout
    );
    assert_eq!(outcome.stderr, "");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_the_spherical_triangle_of_the_tetrahedron() {
    insta::assert_snapshot!(snapshot(&["info", "*332"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_spherical_triangle_of_a_dihedral_group() {
    insta::assert_snapshot!(snapshot(&["info", "*722"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_a_lune() {
    insta::assert_snapshot!(snapshot(&["info", "*55"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_prints_the_domain_summary_of_the_lune_of_two_right_angles() {
    insta::assert_snapshot!(snapshot(&["info", "*22"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_prints_each_check_of_a_spherical_triangle() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*532"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_prints_each_check_of_a_lune() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*55"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_verifies_a_spherical_triangle_at_the_largest_corner_order() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*(1000)22"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_prints_a_diagnostic_for_a_lune_over_the_limit() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*(1001)(1001)"]));
}

#[test]
fn info_check_returns_failure_for_a_signature_without_a_domain() {
    let outcome = run_args(&["info", "--check", "3*2"]);
    assert_eq!(outcome.status, Status::Failure);
    assert!(
        outcome.stdout.ends_with(
            "\nDomain: none, cannot build fundamental domain: the signature 3*2 is not supported\n"
        ),
        "{}",
        outcome.stdout
    );
    assert_eq!(
        outcome.stderr,
        "error: cannot verify fundamental domain\n  caused by: cannot build fundamental domain: the signature 3*2 is not supported\n"
    );
}

#[test]
fn info_returns_success_for_a_signature_without_a_domain() {
    let outcome = run_args(&["info", "3*2"]);
    assert_eq!(outcome.status, Status::Success);
    assert!(
        outcome.stdout.ends_with(
            "\nDomain: none, cannot build fundamental domain: the signature 3*2 is not supported\n"
        ),
        "{}",
        outcome.stdout
    );
    assert_eq!(outcome.stderr, "");
}

#[test]
fn info_check_gives_failure_and_no_diagnostic_on_a_broken_pipe() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::BrokenPipe);
    let mut stderr = Vec::new();
    let status = run(
        ["escher", "info", "--check", "*732"],
        &mut stdout,
        &mut stderr,
    );
    assert_eq!(status, Status::Failure);
    assert_eq!(stderr, b"");
}

#[test]
fn info_check_writes_no_verification_diagnostic_when_standard_output_fails() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::StorageFull);
    let mut stderr = Vec::new();
    let status = run(
        ["escher", "info", "--check", "3*2"],
        &mut stdout,
        &mut stderr,
    );
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
#[cfg_attr(
    miri,
    ignore = "the interpreter rounds the transcendental functions differently on each call, so two builds of one domain differ in the last digits of a residual"
)]
fn info_check_prints_the_residual_of_each_check_beside_its_label() {
    let text = "*(11)(10)5";
    let signature: Signature = text.parse().expect("the signature is valid");
    let verification = FundamentalDomain::build(&signature)
        .expect("the signature has a domain")
        .verify();
    let expected: [(&str, Check); 6] = [
        ("isometries", verification.isometries()),
        ("pairings", verification.pairings()),
        ("walls", verification.walls()),
        ("relators", verification.relators()),
        ("angles", verification.angles()),
        ("area", verification.area()),
    ];
    let outcome = run_args(&["info", "--check", text]);
    assert_eq!(outcome.status, Status::Success);
    let check_lines: Vec<&str> = outcome
        .stdout
        .lines()
        .filter(|line| line.starts_with("Check "))
        .collect();
    let expected_lines: Vec<String> = expected
        .iter()
        .map(|(name, check)| {
            format!(
                "Check {name}: passed, residual {:.1e}, tolerance {:.1e}",
                check.residual(),
                check.tolerance()
            )
        })
        .collect();
    assert_eq!(check_lines, expected_lines);
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_a_reflection_triangle_on_the_sphere() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*532"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_a_spherical_kite() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "532"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_the_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*2222"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_the_doubled_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "2222"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_a_hyperbolic_triangle() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*732"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_a_spherical_signature_without_a_domain() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "3*2"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_the_projective_plane() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "x"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_the_doubled_hemisphere() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "22"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_a_boundary_without_corners() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_the_trivial_signature() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "1"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_a_frieze() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*(inf)22"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_presentation_prints_the_presentations_of_a_group_order_above_the_coset_limit() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*(9000)(9000)"]));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the snapshot library reads snapshot files and starts a process"
)]
fn info_check_presentation_prints_the_presentations_before_the_checks() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "--presentation", "*532"]));
}

#[test]
fn info_presentation_confirms_the_order_of_the_doubled_hemisphere() {
    for args in [
        ["info", "--presentation", "22"],
        ["info", "22", "--presentation"],
    ] {
        let outcome = run_args(&args);
        assert_eq!(outcome.status, Status::Success, "{args:?}");
        assert_eq!(outcome.stderr, "", "{args:?}");
        assert!(
            outcome
                .stdout
                .ends_with("\nSide order by coset enumeration: 2, equal to 2/chi\n"),
            "{args:?}: {}",
            outcome.stdout
        );
    }
}

#[test]
fn info_presentation_confirms_the_order_of_signatures_without_a_domain() {
    for signature in ["x", "*"] {
        let outcome = run_args(&["info", "--presentation", signature]);
        assert_eq!(outcome.status, Status::Success, "{signature}");
        assert_eq!(outcome.stderr, "", "{signature}");
        assert!(
            outcome
                .stdout
                .ends_with("\nOrder by coset enumeration: 2, equal to 2/chi\n"),
            "{signature}: {}",
            outcome.stdout
        );
    }
}
