use std::io;

use escher_cli::{Status, run};
use escher_domain::{Check, FundamentalDomain};
use escher_group::{IndexLimit, SearchBudget, free_classes_of_index};
use escher_render::{EdgeMeeting, WholeFace, Wythoff};
use escher_signature::Signature;

use crate::common::{FailingWriter, run_args, snapshot};

#[test]
fn info_prints_a_rigid_compact_hyperbolic_signature() {
    insta::assert_snapshot!(snapshot(&["info", "*732"]));
}

#[test]
fn info_prints_the_regular_tiling_of_a_triangle_typed_in_another_order() {
    insta::assert_snapshot!(snapshot(&["info", "*237"]));
}

#[test]
fn info_prints_a_spherical_signature_with_its_group_order() {
    insta::assert_snapshot!(snapshot(&["info", "*532"]));
}

#[test]
fn info_prints_a_spherical_signature_without_a_domain() {
    insta::assert_snapshot!(snapshot(&["info", "2*(51)"]));
}

#[test]
fn info_prints_a_hyperbolic_signature_with_a_cone_point_and_a_corner() {
    insta::assert_snapshot!(snapshot(&["info", "4*3"]));
}

#[test]
fn info_prints_a_signature_with_one_deformation_parameter() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "4", "*3333"]));
}

#[test]
fn info_prints_a_reflection_polygon_typed_in_another_order() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "4", "*2223"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_reflection_pentagon() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "4", "*22222"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_reflection_polygon_with_four_orders() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "4", "*5432"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_reflection_octagon() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "3", "*22222222"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_doubled_polygon() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "4", "3333"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_doubled_pentagon() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "4", "22222"]));
}

#[test]
fn info_prints_no_domain_for_a_polygon_with_more_than_eight_corners() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "4", "*222222222"]));
}

#[test]
fn info_check_prints_each_check_of_a_reflection_polygon() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--check",
        "--max-colours",
        "2",
        "*3333"
    ]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_doubled_polygon() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--presentation",
        "--max-colours",
        "2",
        "3333"
    ]));
}

#[test]
fn info_prints_an_order_of_ten_or_more_in_parentheses() {
    insta::assert_snapshot!(snapshot(&["info", "2*(12)"]));
}

#[test]
fn info_prints_a_frieze_with_the_infinite_order_first() {
    insta::assert_snapshot!(snapshot(&["info", "*22(inf)"]));
}

#[test]
fn info_prints_a_euclidean_signature() {
    insta::assert_snapshot!(snapshot(&["info", "*632"]));
}

#[test]
fn info_prints_the_trivial_signature() {
    insta::assert_snapshot!(snapshot(&["info", "1"]));
}

#[test]
fn info_prints_a_general_non_compact_signature_with_its_area() {
    insta::assert_snapshot!(snapshot(&["info", "3*(inf)"]));
}

#[test]
fn info_prints_a_non_compact_triangle_group_as_not_general() {
    insta::assert_snapshot!(snapshot(&["info", "*(inf)32"]));
}

#[test]
fn info_prints_a_handle_and_a_crosscap_as_three_crosscaps() {
    insta::assert_snapshot!(snapshot(&["info", "ox"]));
}

#[test]
fn info_prints_the_canonical_order_of_the_corners() {
    insta::assert_snapshot!(snapshot(&["info", "*236"]));
}

#[test]
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
    let status = run(["escher", "info", "4*3"], &mut stdout, &mut stderr);
    assert_eq!(status, Status::Failure);
    assert_eq!(stderr, b"");
}

#[test]
fn info_gives_failure_and_a_diagnostic_when_standard_output_fails() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::StorageFull);
    let mut stderr = Vec::new();
    let status = run(["escher", "info", "4*3"], &mut stdout, &mut stderr);
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
    let status = run(["escher", "info", "4*3"], &mut stdout, &mut stderr);
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
fn info_prints_the_domain_summary_of_a_triangle_with_a_right_angle() {
    insta::assert_snapshot!(snapshot(&["info", "*542"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_triangle_with_two_equal_corners() {
    insta::assert_snapshot!(snapshot(&["info", "*433"]));
}

#[test]
fn info_prints_no_domain_for_a_corner_order_over_the_limit() {
    insta::assert_snapshot!(snapshot(&["info", "*(101)32"]));
}

#[test]
fn info_check_prints_each_check_of_a_verified_domain() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*732"]));
}

#[test]
fn info_check_prints_a_diagnostic_for_a_signature_without_a_domain() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "4*3"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_hyperbolic_kite() {
    insta::assert_snapshot!(snapshot(&["info", "732"]));
}

#[test]
fn info_check_prints_each_check_of_a_hyperbolic_kite() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "732"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_euclidean_kite() {
    insta::assert_snapshot!(snapshot(&["info", "442"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_doubled_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "2222"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_spherical_kite() {
    insta::assert_snapshot!(snapshot(&["info", "532"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_spherical_kite_with_a_flat_vertex() {
    insta::assert_snapshot!(snapshot(&["info", "(12)22"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_doubled_lune() {
    insta::assert_snapshot!(snapshot(&["info", "55"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_hemisphere() {
    insta::assert_snapshot!(snapshot(&["info", "22"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_euclidean_triangle_with_two_equal_corners() {
    insta::assert_snapshot!(snapshot(&["info", "*442"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_equilateral_euclidean_triangle() {
    insta::assert_snapshot!(snapshot(&["info", "*333"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "*2222"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_torus() {
    insta::assert_snapshot!(snapshot(&["info", "o"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_annulus_with_its_mirror_sides() {
    insta::assert_snapshot!(snapshot(&["info", "**"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_klein_bottle_with_its_glide_reflections() {
    insta::assert_snapshot!(snapshot(&["info", "xx"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_mobius_strip_with_its_mirror_sides() {
    insta::assert_snapshot!(snapshot(&["info", "*x"]));
}

#[test]
fn info_prints_the_mobius_strip_typed_with_the_cross_cap_first_in_canonical_form() {
    insta::assert_snapshot!(snapshot(&["info", "x*"]));
}

#[test]
fn info_prints_the_domain_summary_of_two_half_turns_and_a_cross_cap() {
    insta::assert_snapshot!(snapshot(&["info", "22x"]));
}

#[test]
fn info_prints_the_domain_summary_of_two_half_turns_and_a_mirror() {
    insta::assert_snapshot!(snapshot(&["info", "22*"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_half_turn_and_two_mirror_corners() {
    insta::assert_snapshot!(snapshot(&["info", "2*22"]));
}

#[test]
fn info_check_prints_each_check_of_two_half_turns_and_a_mirror() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "22*"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_kite_with_two_mirrors_and_a_quarter_turn() {
    insta::assert_snapshot!(snapshot(&["info", "4*2"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_kite_with_two_mirrors_and_a_third_of_a_turn() {
    insta::assert_snapshot!(snapshot(&["info", "3*3"]));
}

#[test]
fn info_presentation_prints_the_side_relators_of_the_kite_with_a_third_of_a_turn() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "3*3"]));
}

#[test]
fn info_presentation_prints_the_side_relators_of_the_mobius_strip() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*x"]));
}

#[test]
fn info_check_prints_each_check_of_the_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*2222"]));
}

#[test]
fn info_check_verifies_a_domain_at_the_largest_corner_order() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*(100)32"]));
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
fn info_prints_the_domain_summary_of_the_spherical_triangle_of_the_tetrahedron() {
    insta::assert_snapshot!(snapshot(&["info", "*332"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_spherical_triangle_of_a_dihedral_group() {
    insta::assert_snapshot!(snapshot(&["info", "*722"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_lune() {
    insta::assert_snapshot!(snapshot(&["info", "*55"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_lune_of_two_right_angles() {
    insta::assert_snapshot!(snapshot(&["info", "*22"]));
}

#[test]
fn info_check_prints_each_check_of_a_spherical_triangle() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*532"]));
}

#[test]
fn info_check_prints_each_check_of_a_lune() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*55"]));
}

#[test]
fn info_check_verifies_a_spherical_triangle_at_the_largest_corner_order() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*(100)22"]));
}

#[test]
fn info_check_prints_a_diagnostic_for_a_lune_over_the_limit() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "*(101)(101)"]));
}

#[test]
fn info_check_returns_failure_for_a_signature_without_a_domain() {
    let outcome = run_args(&["info", "--check", "4*3"]);
    assert_eq!(outcome.status, Status::Failure);
    assert!(
        outcome.stdout.ends_with(
            "\nDomain: none, cannot build fundamental domain: the signature 4*3 is not supported\n"
        ),
        "{}",
        outcome.stdout
    );
    assert_eq!(
        outcome.stderr,
        "error: cannot verify fundamental domain\n  caused by: cannot build fundamental domain: the signature 4*3 is not supported\n"
    );
}

#[test]
fn info_returns_success_for_a_signature_without_a_domain() {
    let outcome = run_args(&["info", "4*3"]);
    assert_eq!(outcome.status, Status::Success);
    assert!(
        outcome.stdout.ends_with(
            "\nDomain: none, cannot build fundamental domain: the signature 4*3 is not supported\n"
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
        ["escher", "info", "--check", "4*3"],
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
fn info_presentation_prints_the_presentations_of_a_reflection_triangle_on_the_sphere() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*532"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_spherical_kite() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "532"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_the_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*2222"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_the_doubled_rectangle() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "2222"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_hyperbolic_triangle() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*732"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_spherical_signature_without_a_domain() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "2*(51)"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_spherical_kite_with_a_cone_of_order_three_and_a_corner() {
    insta::assert_snapshot!(snapshot(&["info", "3*2"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_spherical_kite_with_a_cone_and_a_mirror_circle() {
    insta::assert_snapshot!(snapshot(&["info", "3*"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_spherical_kite_with_a_half_turn_and_a_corner() {
    insta::assert_snapshot!(snapshot(&["info", "2*3"]));
}

#[test]
fn info_prints_the_domain_summary_of_a_lune_paired_by_rotary_reflections() {
    insta::assert_snapshot!(snapshot(&["info", "5x"]));
}

#[test]
fn info_prints_the_domain_summary_of_the_projective_plane() {
    insta::assert_snapshot!(snapshot(&["info", "x"]));
}

#[test]
fn info_prints_the_domain_summary_of_one_mirror() {
    insta::assert_snapshot!(snapshot(&["info", "*"]));
}

#[test]
fn info_prints_a_cross_cap_typed_before_its_cone_in_canonical_order() {
    insta::assert_snapshot!(snapshot(&["info", "x3"]));
}

#[test]
fn info_check_prints_each_check_of_a_lune_paired_by_rotary_reflections() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "5x"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_lune_paired_by_rotary_reflections() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "5x"]));
}

#[test]
fn info_check_prints_each_check_of_a_spherical_kite_with_mirrors() {
    insta::assert_snapshot!(snapshot(&["info", "--check", "3*2"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_spherical_kite_with_mirrors() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "3*2"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_the_projective_plane() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "x"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_the_doubled_hemisphere() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "22"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_boundary_without_corners() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_the_trivial_signature() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "1"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_frieze() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*(inf)22"]));
}

#[test]
fn info_presentation_prints_the_presentations_of_a_group_order_above_the_coset_limit() {
    insta::assert_snapshot!(snapshot(&["info", "--presentation", "*(9000)(9000)"]));
}

#[test]
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
    let outcome = run_args(&["info", "--presentation", "2*(51)"]);
    assert_eq!(outcome.status, Status::Success);
    assert_eq!(outcome.stderr, "");
    assert!(
        outcome
            .stdout
            .ends_with("\nOrder by coset enumeration: 204, equal to 2/chi\n"),
        "{}",
        outcome.stdout
    );
}

#[test]
fn info_presentation_confirms_both_orders_of_the_hemispheres() {
    for signature in ["x", "*"] {
        let outcome = run_args(&["info", "--presentation", signature]);
        assert_eq!(outcome.status, Status::Success, "{signature}");
        assert_eq!(outcome.stderr, "", "{signature}");
        for line in [
            "\nOrder by coset enumeration: 2, equal to 2/chi\n",
            "\nSide order by coset enumeration: 2, equal to 2/chi\n",
        ] {
            assert!(
                outcome.stdout.contains(line),
                "{signature}: {}",
                outcome.stdout
            );
        }
    }
}

#[test]
fn info_colourings_lists_a_hyperbolic_triangle_up_to_eight_colours() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--colourings",
        "--max-colours",
        "8",
        "*732"
    ]));
}

#[test]
fn info_colourings_lists_a_hyperbolic_triangle_up_to_three_colours() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--colourings",
        "--max-colours",
        "3",
        "*642"
    ]));
}

#[test]
fn info_colourings_lists_the_doubled_hemisphere() {
    insta::assert_snapshot!(snapshot(&["info", "--colourings", "22"]));
}

#[test]
fn info_colourings_lists_the_rectangle_up_to_four_colours() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--colourings",
        "--max-colours",
        "4",
        "*2222"
    ]));
}

#[test]
fn info_colourings_lists_a_hyperbolic_kite_up_to_nine_colours() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--colourings",
        "--max-colours",
        "9",
        "732"
    ]));
}

#[test]
fn info_colourings_lists_a_spherical_kite_up_to_six_colours() {
    insta::assert_snapshot!(snapshot(&[
        "info",
        "--colourings",
        "--max-colours",
        "6",
        "532"
    ]));
}

#[test]
fn info_counts_the_colourings_of_one_colour() {
    insta::assert_snapshot!(snapshot(&["info", "--max-colours", "1", "*732"]));
}

#[test]
fn info_colourings_lists_the_colourings_of_the_doubled_hemisphere() {
    for args in [
        ["info", "--colourings", "22"],
        ["info", "22", "--colourings"],
    ] {
        let outcome = run_args(&args);
        assert_eq!(outcome.status, Status::Success, "{args:?}");
        assert_eq!(outcome.stderr, "", "{args:?}");
        assert!(
            outcome
                .stdout
                .ends_with("\nColouring: 2a, 2 colours, alias parity\n"),
            "{args:?}: {}",
            outcome.stdout
        );
    }
}

#[test]
fn info_counts_the_colourings_of_the_doubled_hemisphere() {
    let outcome = run_args(&["info", "22"]);
    assert_eq!(outcome.status, Status::Success);
    assert_eq!(outcome.stderr, "");
    assert_eq!(
        outcome.stdout.lines().last(),
        Some("Colourings with 2 colours: 1")
    );
}

#[test]
fn info_colourings_up_to_one_colour_ends_with_the_alias_one() {
    let outcome = run_args(&["info", "--colourings", "--max-colours", "1", "1"]);
    assert_eq!(outcome.status, Status::Success);
    assert_eq!(outcome.stderr, "");
    assert!(
        outcome
            .stdout
            .ends_with("\nColouring: 1a, 1 colour, alias one\n"),
        "{}",
        outcome.stdout
    );
}

#[test]
fn info_check_returns_success_for_the_doubled_hemisphere() {
    for args in [["info", "--check", "22"], ["info", "22", "--check"]] {
        let outcome = run_args(&args);
        assert_eq!(outcome.status, Status::Success, "{args:?}");
        assert!(
            outcome.stdout.ends_with("\nVerification: passed\n"),
            "{args:?}: {}",
            outcome.stdout
        );
        assert_eq!(outcome.stderr, "", "{args:?}");
    }
}

#[test]
fn info_check_gives_failure_and_no_diagnostic_on_a_broken_pipe_for_the_doubled_hemisphere() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::BrokenPipe);
    let mut stderr = Vec::new();
    let status = run(
        ["escher", "info", "--check", "22"],
        &mut stdout,
        &mut stderr,
    );
    assert_eq!(status, Status::Failure);
    assert_eq!(stderr, b"");
}

#[test]
fn info_lists_the_arrangements_of_the_hexagons() {
    insta::assert_snapshot!(snapshot(&["info", "*632", "--wythoff", "010"]));
}

#[test]
fn info_lists_the_arrangements_of_the_triangles_of_the_dual_of_the_hexagons() {
    insta::assert_snapshot!(snapshot(&["info", "*632", "--wythoff", "100"]));
}

#[test]
fn info_prints_no_arrangement_for_the_squares_of_542() {
    insta::assert_snapshot!(snapshot(&["info", "*542", "--wythoff", "100"]));
}

#[test]
fn info_prints_the_refusal_of_arrangements_on_a_tiling_that_is_not_regular() {
    insta::assert_snapshot!(snapshot(&["info", "*732", "--wythoff", "111"]));
}

#[test]
fn info_prints_the_index_limit_for_the_arrangements_of_a_polygon_of_33_sides() {
    insta::assert_snapshot!(snapshot(&["info", "*(33)32", "--wythoff", "010"]));
}

const REGULAR: [bool; 3] = [false, true, false];

const DUAL: [bool; 3] = [true, false, false];

const WHOLE_FACE_SET: [(&str, [bool; 3]); 16] = [
    ("*632", REGULAR),
    ("*632", DUAL),
    ("*442", REGULAR),
    ("*332", REGULAR),
    ("*432", REGULAR),
    ("*432", DUAL),
    ("*532", REGULAR),
    ("*532", DUAL),
    ("*732", REGULAR),
    ("*832", REGULAR),
    ("*542", REGULAR),
    ("*552", REGULAR),
    ("*642", REGULAR),
    ("*732", DUAL),
    ("*542", DUAL),
    ("*642", DUAL),
];

fn marks_text(marks: [bool; 3]) -> String {
    marks.iter().map(|&on| if on { '1' } else { '0' }).collect()
}

fn arrangement_lines(signature: &str, marks: [bool; 3]) -> Vec<String> {
    let outcome = run_args(&[
        "info",
        signature,
        "--max-colours",
        "1",
        "--wythoff",
        &marks_text(marks),
    ]);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    let lines: Vec<String> = outcome
        .stdout
        .lines()
        .filter(|line| line.starts_with("Arrangement"))
        .map(str::to_owned)
        .collect();
    let count = lines.len().saturating_sub(1);
    assert_eq!(
        lines.first(),
        Some(&format!("Arrangements: {count}")),
        "{signature}"
    );
    lines
}

fn pairing_text(edges: &[EdgeMeeting]) -> String {
    let mut pairs = Vec::new();
    for (edge, meeting) in (0_u32..).zip(edges) {
        let kind = if meeting.mirrored() {
            "mirrored"
        } else {
            "turned"
        };
        if meeting.partner() == edge {
            pairs.push(format!("{edge} {kind}"));
        } else if meeting.partner() > edge {
            pairs.push(format!("{edge} and {} {kind}", meeting.partner()));
        }
    }
    pairs.join(", ")
}

#[test]
fn info_lists_exactly_the_arrangements_of_escher_group_with_the_edges_of_escher_render() {
    let mut listed = 0;
    for (text, marks) in WHOLE_FACE_SET {
        let signature: Signature = text.parse().expect("the signature is valid");
        let domain = FundamentalDomain::build(&signature).expect("the domain is built");
        let wythoff = Wythoff::new(&domain, marks).expect("the construction is built");
        let face = WholeFace::new(&domain, wythoff).expect("the tiling is regular");
        let found = free_classes_of_index(
            face.presentation(),
            face.stabilizer(),
            IndexLimit::new(face.index()).expect("the index is within the limit"),
            SearchBudget::DEFAULT,
        )
        .expect("the search runs");
        assert!(found.is_complete(), "{text}");
        let mut expected = vec![format!("Arrangements: {}", found.list().len())];
        for arrangement in found.list() {
            let edges = face.edges(arrangement).expect("the edges are read");
            let neighbours = if edges.iter().any(|meeting| meeting.mirrored()) {
                "with mirrored neighbours"
            } else {
                "turned only"
            };
            expected.push(format!(
                "Arrangement: {}, {neighbours}, edges {}",
                arrangement.id(),
                pairing_text(&edges)
            ));
        }
        assert_eq!(arrangement_lines(text, marks), expected, "{text}");
        listed += found.list().len();
    }
    println!("{listed} arrangements");
    assert_eq!(listed, 114);
}

fn turned_only(lines: &[String]) -> usize {
    lines
        .iter()
        .filter(|line| line.contains(", turned only, "))
        .count()
}

#[test]
fn info_lists_the_numbers_of_arrangements_of_the_euclidean_catalogue() {
    for (text, marks, count, turned) in [
        ("*632", REGULAR, 7, Some(3)),
        ("*632", DUAL, 6, Some(2)),
        ("*442", REGULAR, 16, None),
    ] {
        let lines = arrangement_lines(text, marks);
        assert_eq!(lines.len() - 1, count, "{text}");
        if let Some(turned) = turned {
            assert_eq!(turned_only(&lines), turned, "{text}");
        }
    }
}

#[test]
fn info_lists_turned_only_arrangements_of_hyperbolic_tilings_only_where_the_rotations_allow() {
    for text in ["*732", "*832", "*542"] {
        assert!(
            turned_only(&arrangement_lines(text, REGULAR)) >= 1,
            "{text}"
        );
    }
    for text in ["*542", "*732"] {
        assert_no_arrangement_listed(text, DUAL);
    }
}

fn assert_no_arrangement_listed(signature: &str, marks: [bool; 3]) {
    let outcome = run_args(&[
        "info",
        signature,
        "--max-colours",
        "1",
        "--wythoff",
        &marks_text(marks),
    ]);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    let lines: Vec<&str> = outcome.stdout.lines().collect();
    assert_eq!(
        lines
            .iter()
            .filter(|line| **line == "Arrangements: 0")
            .count(),
        1,
        "{signature}"
    );
    assert!(
        lines.iter().all(|line| !line.starts_with("Arrangement: ")),
        "{signature}"
    );
    assert_eq!(lines.last(), Some(&"Arrangements: 0"), "{signature}");
}

#[test]
fn info_prints_arrangements_0_and_no_arrangement_line_for_the_dual_of_a_triangle_without_arrangements()
 {
    for text in ["*732", "*542", "*532"] {
        assert_no_arrangement_listed(text, DUAL);
    }
}
