use std::io::Write;

use escher_domain::{Check, FundamentalDomain, Verification};
use escher_signature::{Geometry, Rational, Signature};

use crate::diagnostic::SignatureError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verify {
    Skip,
    Run,
}

impl Verify {
    pub(crate) const fn from_flag(check: bool) -> Self {
        if check { Self::Run } else { Self::Skip }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Passed,
    Failed,
}

impl Verdict {
    const fn from_passed(passed: bool) -> Self {
        if passed { Self::Passed } else { Self::Failed }
    }

    const fn text(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

type Lines = Vec<(&'static str, String)>;

const CANNOT_VERIFY: &str = "cannot verify fundamental domain";

#[derive(Clone, Copy, Debug)]
struct CheckRow {
    label: &'static str,
    verdict: Verdict,
    residual: f64,
    tolerance: f64,
}

impl CheckRow {
    fn new(label: &'static str, check: Check) -> Self {
        Self {
            label,
            verdict: Verdict::from_passed(check.passed()),
            residual: check.residual(),
            tolerance: check.tolerance(),
        }
    }
}

pub(crate) fn info(text: &str, verify: Verify, stdout: &mut dyn Write) -> anyhow::Result<()> {
    let signature: Signature = text
        .parse()
        .map_err(|error| SignatureError::new(text, error))?;
    let mut lines = signature_lines(&signature);
    let domain = FundamentalDomain::build(&signature);
    push_domain_summary(&mut lines, domain.as_ref());
    let verdict = match (verify, domain) {
        (Verify::Skip, _) => Ok(()),
        (Verify::Run, Err(error)) => Err(anyhow::Error::new(error).context(CANNOT_VERIFY)),
        (Verify::Run, Ok(domain)) => {
            let verification = domain.verify();
            push_verification(
                &mut lines,
                &check_rows(&verification),
                Verdict::from_passed(verification.passed()),
            )
        }
    };
    write_text(stdout, &lines, verdict)
}

fn write_text(
    stdout: &mut dyn Write,
    lines: &Lines,
    verdict: anyhow::Result<()>,
) -> anyhow::Result<()> {
    crate::write_stdout(stdout, &joined(lines))?;
    verdict
}

fn signature_lines(signature: &Signature) -> Lines {
    let mut lines = vec![
        ("Signature", signature.to_string()),
        ("Geometry", geometry_name(signature.geometry()).to_owned()),
        (
            "Euler characteristic",
            signature.euler_characteristic().to_string(),
        ),
        ("Compact", yes_or_no(signature.is_compact()).to_owned()),
    ];
    if !signature.is_compact() {
        lines.push(("Frieze", yes_or_no(signature.is_frieze()).to_owned()));
        lines.push((
            "General non-compact",
            yes_or_no(signature.is_general_non_compact()).to_owned(),
        ));
    }
    if let Some(order) = signature.group_order() {
        lines.push(("Group order", order.to_string()));
    }
    if let Some(area) = signature.area_over_pi() {
        lines.push(("Area", area_text(area)));
    }
    if let Some(dimension) = signature.deformation_dimension() {
        lines.push(("Deformation dimension", dimension.to_string()));
    }
    lines
}

fn push_domain_summary(
    lines: &mut Lines,
    domain: Result<&FundamentalDomain, &escher_domain::Error>,
) {
    match domain {
        Ok(domain) => {
            lines.push(("Domain", format!("{} mirror sides", domain.sides().len())));
            lines.push((
                "Corner angles in degrees",
                fixed_list(domain.corner_angles().into_iter().map(f64::to_degrees)),
            ));
            lines.push((
                "Side lengths",
                fixed_list(domain.side_lengths().into_iter()),
            ));
            lines.push(("Domain area", format!("{:.6}", domain.area())));
        }
        Err(error) => lines.push(("Domain", format!("none, {error}"))),
    }
}

fn check_rows(verification: &Verification) -> [CheckRow; 6] {
    [
        CheckRow::new("Check isometries", verification.isometries()),
        CheckRow::new("Check pairings", verification.pairings()),
        CheckRow::new("Check walls", verification.walls()),
        CheckRow::new("Check relators", verification.relators()),
        CheckRow::new("Check angles", verification.angles()),
        CheckRow::new("Check area", verification.area()),
    ]
}

fn push_verification(lines: &mut Lines, rows: &[CheckRow], overall: Verdict) -> anyhow::Result<()> {
    for row in rows {
        lines.push((
            row.label,
            check_text(row.verdict, row.residual, row.tolerance),
        ));
    }
    lines.push(("Verification", overall.text().to_owned()));
    match overall {
        Verdict::Passed => Ok(()),
        Verdict::Failed => {
            let failed = rows
                .iter()
                .filter(|row| row.verdict == Verdict::Failed)
                .count();
            Err(failed_checks(failed, rows.len()))
        }
    }
}

fn check_text(verdict: Verdict, residual: f64, tolerance: f64) -> String {
    format!(
        "{}, residual {residual:.1e}, tolerance {tolerance:.1e}",
        verdict.text()
    )
}

fn failed_checks(failed: usize, total: usize) -> anyhow::Error {
    anyhow::anyhow!("{CANNOT_VERIFY}: {failed} of {total} checks failed")
}

fn fixed_list(values: impl Iterator<Item = f64>) -> String {
    values
        .map(|value| format!("{value:.6}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn joined(lines: &Lines) -> String {
    let mut text = String::new();
    for (label, value) in lines {
        text.push_str(label);
        text.push_str(": ");
        text.push_str(value);
        text.push('\n');
    }
    text
}

fn geometry_name(geometry: Geometry) -> &'static str {
    match geometry {
        Geometry::Spherical => "spherical",
        Geometry::Euclidean => "Euclidean",
        Geometry::Hyperbolic => "hyperbolic",
    }
}

fn yes_or_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn area_text(area_over_pi: Rational) -> String {
    let numerator = area_over_pi.numerator();
    let denominator = area_over_pi.denominator().get();
    match (numerator == 1, denominator == 1) {
        (true, true) => "pi".to_owned(),
        (true, false) => format!("pi/{denominator}"),
        (false, true) => format!("{numerator}pi"),
        (false, false) => format!("{numerator}pi/{denominator}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckRow, Verdict, check_text, failed_checks, push_verification, write_text};
    use crate::Status;
    use crate::diagnostic::report;

    #[test]
    fn check_text_writes_a_failed_check() {
        assert_eq!(
            check_text(Verdict::Failed, 2.5e-3, 1.0e-6),
            "failed, residual 2.5e-3, tolerance 1.0e-6"
        );
    }

    #[test]
    fn check_text_writes_an_infinite_residual_as_inf() {
        assert_eq!(
            check_text(Verdict::Failed, f64::INFINITY, 1.0e-9),
            "failed, residual inf, tolerance 1.0e-9"
        );
    }

    #[test]
    fn check_text_writes_a_nan_residual_as_nan() {
        assert_eq!(
            check_text(Verdict::Failed, f64::NAN, 2.0e-6),
            "failed, residual NaN, tolerance 2.0e-6"
        );
    }

    #[test]
    fn failed_checks_diagnostic_counts_two_of_six_checks() {
        let mut stderr = Vec::new();
        let status = report(&failed_checks(2, 6), &mut stderr);
        assert_eq!(status, Status::Failure);
        assert_eq!(
            String::from_utf8(stderr).expect("standard error is UTF-8"),
            "error: cannot verify fundamental domain: 2 of 6 checks failed\n"
        );
    }

    fn passed_row(label: &'static str) -> CheckRow {
        CheckRow {
            label,
            verdict: Verdict::Passed,
            residual: 1.0e-15,
            tolerance: 1.0e-9,
        }
    }

    #[test]
    fn one_failed_check_of_six_is_written_before_its_diagnostic() {
        let rows = [
            passed_row("Check isometries"),
            passed_row("Check pairings"),
            passed_row("Check walls"),
            CheckRow {
                label: "Check relators",
                verdict: Verdict::Failed,
                residual: 2.5e-3,
                tolerance: 2.0e-6,
            },
            passed_row("Check angles"),
            passed_row("Check area"),
        ];
        let mut lines = vec![("Signature", "*732".to_owned())];
        let verdict = push_verification(&mut lines, &rows, Verdict::Failed);
        let mut stdout = Vec::new();
        let error = write_text(&mut stdout, &lines, verdict)
            .expect_err("a failed check fails the verification");
        assert_eq!(
            String::from_utf8(stdout).expect("standard output is UTF-8"),
            "Signature: *732\n\
             Check isometries: passed, residual 1.0e-15, tolerance 1.0e-9\n\
             Check pairings: passed, residual 1.0e-15, tolerance 1.0e-9\n\
             Check walls: passed, residual 1.0e-15, tolerance 1.0e-9\n\
             Check relators: failed, residual 2.5e-3, tolerance 2.0e-6\n\
             Check angles: passed, residual 1.0e-15, tolerance 1.0e-9\n\
             Check area: passed, residual 1.0e-15, tolerance 1.0e-9\n\
             Verification: failed\n"
        );
        let mut stderr = Vec::new();
        assert_eq!(report(&error, &mut stderr), Status::Failure);
        assert_eq!(
            String::from_utf8(stderr).expect("standard error is UTF-8"),
            "error: cannot verify fundamental domain: 1 of 6 checks failed\n"
        );
    }
}
