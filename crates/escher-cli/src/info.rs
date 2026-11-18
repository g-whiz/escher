use core::num::NonZeroU64;
use std::io::Write;

use escher_domain::{Check, FundamentalDomain, PairingKind, Verification, parameters};
use escher_group::{CosetLimit, ErrorKind, Presentation, conway_presentation, enumerate};
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
pub(crate) enum Presentations {
    Skip,
    Print,
}

impl Presentations {
    pub(crate) const fn from_flag(presentation: bool) -> Self {
        if presentation {
            Self::Print
        } else {
            Self::Skip
        }
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

pub(crate) fn info(
    text: &str,
    verify: Verify,
    presentations: Presentations,
    stdout: &mut dyn Write,
) -> Vec<anyhow::Error> {
    let signature: Signature = match text.parse() {
        Ok(signature) => signature,
        Err(error) => return vec![anyhow::Error::new(SignatureError::new(text, error))],
    };
    let mut lines = signature_lines(&signature);
    let domain = FundamentalDomain::build(&signature);
    push_domain_summary(&mut lines, domain.as_ref());
    let mut errors = Vec::new();
    if presentations == Presentations::Print {
        errors.extend(push_presentations(
            &mut lines,
            &signature,
            domain.as_ref().ok(),
        ));
    }
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
    errors.extend(verdict.err());
    write_text(stdout, &lines, errors)
}

fn write_text(
    stdout: &mut dyn Write,
    lines: &Lines,
    errors: Vec<anyhow::Error>,
) -> Vec<anyhow::Error> {
    match crate::write_stdout(stdout, &joined(lines)) {
        Ok(()) => errors,
        Err(error) => vec![error],
    }
}

struct Labels {
    size: &'static str,
    generators: &'static str,
    relators: &'static str,
    order: &'static str,
    subject: &'static str,
}

const CONWAY: Labels = Labels {
    size: "Presentation",
    generators: "Generators",
    relators: "Relators",
    order: "Order by coset enumeration",
    subject: "presentation",
};

const SIDE: Labels = Labels {
    size: "Side presentation",
    generators: "Side generators",
    relators: "Side relators",
    order: "Side order by coset enumeration",
    subject: "side presentation",
};

#[derive(Debug)]
enum Fault {
    Mismatch(anyhow::Error),
    Internal(anyhow::Error),
}

impl Fault {
    fn into_error(self) -> anyhow::Error {
        match self {
            Self::Mismatch(error) | Self::Internal(error) => error,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Outcome {
    AboveLimit,
    Index(u64),
    LimitReached,
    Refused(String),
}

fn push_presentations(
    lines: &mut Lines,
    signature: &Signature,
    domain: Option<&FundamentalDomain>,
) -> Vec<anyhow::Error> {
    let order = signature.group_order();
    let conway = push_presentation(
        lines,
        &CONWAY,
        conway_presentation(signature).map_err(|error| error.to_string()),
        order,
    );
    let side = domain.and_then(|domain| {
        push_presentation(
            lines,
            &SIDE,
            domain
                .side_presentation()
                .map_err(|error| error.to_string()),
            order,
        )
    });
    reported(conway, side)
}

fn reported(conway: Option<Fault>, side: Option<Fault>) -> Vec<anyhow::Error> {
    match (conway, side) {
        (Some(Fault::Mismatch(error)), Some(Fault::Mismatch(_))) => vec![error],
        (conway, side) => conway
            .into_iter()
            .chain(side)
            .map(Fault::into_error)
            .collect(),
    }
}

fn push_presentation(
    lines: &mut Lines,
    labels: &Labels,
    presentation: Result<Presentation, String>,
    group_order: Option<NonZeroU64>,
) -> Option<Fault> {
    let presentation = match presentation {
        Ok(presentation) => presentation,
        Err(text) => {
            lines.push((labels.size, format!("none, {text}")));
            return None;
        }
    };
    let relators: Option<Vec<String>> = presentation
        .relators()
        .iter()
        .map(|relator| {
            presentation
                .display_relator(relator)
                .map(|text| text.to_string())
        })
        .collect();
    let Some(relators) = relators else {
        return Some(Fault::Internal(anyhow::anyhow!(
            "internal error: a relator of the {} names a generator that it does not have",
            labels.subject
        )));
    };
    lines.push((
        labels.size,
        size_text(presentation.generator_count(), relators.len()),
    ));
    lines.push((
        labels.generators,
        list_or_none(presentation.generator_names()),
    ));
    lines.push((labels.relators, list_or_none(&relators)));
    let order = group_order?;
    let limit = CosetLimit::DEFAULT;
    let outcome = enumerate_order(&presentation, order, limit);
    push_order(lines, labels, order.get(), limit.get(), &outcome)
}

fn enumerate_order(presentation: &Presentation, order: NonZeroU64, limit: CosetLimit) -> Outcome {
    if order.get() > u64::from(limit.get()) {
        return Outcome::AboveLimit;
    }
    match enumerate(presentation, &[], limit) {
        Ok(table) => Outcome::Index(u64::try_from(table.index()).unwrap_or(u64::MAX)),
        Err(error) => match error.kind() {
            ErrorKind::CosetLimitReached => Outcome::LimitReached,
            _ => Outcome::Refused(error.to_string()),
        },
    }
}

fn push_order(
    lines: &mut Lines,
    labels: &Labels,
    order: u64,
    limit: u32,
    outcome: &Outcome,
) -> Option<Fault> {
    lines.push((labels.order, order_text(order, limit, outcome)));
    match *outcome {
        Outcome::Index(index) if index != order => Some(Fault::Mismatch(order_mismatch(
            labels.subject,
            index,
            order,
        ))),
        Outcome::AboveLimit | Outcome::Index(_) | Outcome::LimitReached | Outcome::Refused(_) => {
            None
        }
    }
}

fn order_text(order: u64, limit: u32, outcome: &Outcome) -> String {
    match outcome {
        Outcome::AboveLimit => {
            format!("not run, the group order {order} is above the limit of {limit} cosets")
        }
        Outcome::Index(index) if *index == order => format!("{order}, equal to 2/chi"),
        Outcome::Index(index) => format!("{index}, different from 2/chi, which is {order}"),
        Outcome::LimitReached => format!("not found within {limit} cosets"),
        Outcome::Refused(text) => format!("not run, {text}"),
    }
}

fn order_mismatch(subject: &str, index: u64, order: u64) -> anyhow::Error {
    anyhow::anyhow!(
        "cannot confirm group order: the {subject} gives {index} cosets, and 2/chi is {order}"
    )
}

fn size_text(generators: usize, relators: usize) -> String {
    format!(
        "{generators} {}, {relators} {}",
        if generators == 1 {
            "generator"
        } else {
            "generators"
        },
        if relators == 1 { "relator" } else { "relators" }
    )
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
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
    for spec in parameters(signature) {
        lines.push((
            "Parameter",
            format!(
                "{}, range {} to {}, default {}",
                spec.name(),
                spec.minimum(),
                spec.maximum(),
                spec.default_value()
            ),
        ));
    }
    lines
}

fn push_domain_summary(
    lines: &mut Lines,
    domain: Result<&FundamentalDomain, &escher_domain::Error>,
) {
    match domain {
        Ok(domain) => {
            let sides = domain.sides();
            if domain.reverses_orientation() {
                lines.push(("Domain", format!("{} mirror sides", sides.len())));
            } else {
                let pairs: Vec<String> = sides
                    .iter()
                    .enumerate()
                    .filter(|&(index, side)| side.partner() > index)
                    .map(|(index, side)| {
                        format!(
                            "{index} and {} by {}",
                            side.partner(),
                            kind_name(side.kind())
                        )
                    })
                    .collect();
                let noun = if pairs.len() == 1 { "pair" } else { "pairs" };
                lines.push((
                    "Domain",
                    format!("{} sides in {} {noun}", sides.len(), pairs.len()),
                ));
                lines.push(("Side pairs", pairs.join(", ")));
                lines.push((
                    "Parity colouring",
                    "none, every side pairing preserves orientation".to_owned(),
                ));
            }
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

fn kind_name(kind: PairingKind) -> &'static str {
    match kind {
        PairingKind::Mirror => "reflection",
        PairingKind::Rotation => "rotation",
        PairingKind::Translation => "translation",
        _ => "another isometry",
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
    use core::num::NonZeroU64;

    use escher_group::{CosetLimit, conway_presentation};
    use escher_signature::Signature;

    use super::{
        CONWAY, CheckRow, Fault, Outcome, SIDE, Verdict, check_text, enumerate_order,
        failed_checks, order_mismatch, order_text, push_order, push_verification, reported,
        size_text, write_text,
    };
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
        let errors = write_text(&mut stdout, &lines, verdict.err().into_iter().collect());
        let [error] = &errors[..] else {
            panic!("a failed check gives one error, not {errors:?}");
        };
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
        assert_eq!(report(error, &mut stderr), Status::Failure);
        assert_eq!(
            String::from_utf8(stderr).expect("standard error is UTF-8"),
            "error: cannot verify fundamental domain: 1 of 6 checks failed\n"
        );
    }

    #[test]
    fn size_text_uses_the_singular_for_one() {
        assert_eq!(size_text(1, 1), "1 generator, 1 relator");
        assert_eq!(size_text(0, 0), "0 generators, 0 relators");
        assert_eq!(size_text(3, 6), "3 generators, 6 relators");
    }

    #[test]
    fn order_text_writes_each_outcome() {
        let cases = [
            (120, Outcome::Index(120), "120, equal to 2/chi"),
            (
                120,
                Outcome::Index(60),
                "60, different from 2/chi, which is 120",
            ),
            (
                60_000,
                Outcome::AboveLimit,
                "not run, the group order 60000 is above the limit of 16384 cosets",
            ),
            (120, Outcome::LimitReached, "not found within 16384 cosets"),
        ];
        for (order, outcome, text) in cases {
            assert_eq!(order_text(order, 16_384, &outcome), text, "{outcome:?}");
        }
    }

    #[test]
    fn order_mismatch_diagnostic_names_the_presentation() {
        for (subject, text) in [
            (
                CONWAY.subject,
                "error: cannot confirm group order: the presentation gives 60 cosets, and 2/chi is 120\n",
            ),
            (
                SIDE.subject,
                "error: cannot confirm group order: the side presentation gives 60 cosets, and 2/chi is 120\n",
            ),
        ] {
            let mut stderr = Vec::new();
            assert_eq!(
                report(&order_mismatch(subject, 60, 120), &mut stderr),
                Status::Failure
            );
            assert_eq!(
                String::from_utf8(stderr).expect("standard error is UTF-8"),
                text
            );
        }
    }

    #[test]
    fn only_a_different_index_gives_an_error() {
        let outcomes = [
            Outcome::Index(120),
            Outcome::AboveLimit,
            Outcome::LimitReached,
            Outcome::Refused("cannot enumerate cosets: a limit".to_owned()),
        ];
        for outcome in outcomes {
            let mut lines = Vec::new();
            assert!(
                push_order(&mut lines, &CONWAY, 120, 16_384, &outcome).is_none(),
                "{outcome:?}"
            );
            assert_eq!(lines.len(), 1, "{outcome:?}");
        }
    }

    #[test]
    fn a_different_index_is_written_before_its_diagnostic() {
        let mut lines = vec![("Signature", "*532".to_owned())];
        let errors: Vec<anyhow::Error> =
            push_order(&mut lines, &SIDE, 120, 16_384, &Outcome::Index(60))
                .into_iter()
                .map(Fault::into_error)
                .collect();
        let mut stdout = Vec::new();
        let errors = write_text(&mut stdout, &lines, errors);
        assert_eq!(
            String::from_utf8(stdout).expect("standard output is UTF-8"),
            "Signature: *532\n\
             Side order by coset enumeration: 60, different from 2/chi, which is 120\n"
        );
        let [error] = &errors[..] else {
            panic!("a different index gives one error, not {errors:?}");
        };
        let mut stderr = Vec::new();
        assert_eq!(report(error, &mut stderr), Status::Failure);
        assert_eq!(
            String::from_utf8(stderr).expect("standard error is UTF-8"),
            "error: cannot confirm group order: the side presentation gives 60 cosets, and 2/chi is 120\n"
        );
    }

    fn conway_mismatch() -> Fault {
        Fault::Mismatch(order_mismatch(CONWAY.subject, 60, 120))
    }

    fn side_mismatch() -> Fault {
        Fault::Mismatch(order_mismatch(SIDE.subject, 30, 120))
    }

    fn internal(subject: &str) -> Fault {
        Fault::Internal(anyhow::anyhow!("internal error: {subject}"))
    }

    #[test]
    fn faults_of_both_presentations_are_reported_in_order_except_two_mismatches() {
        let conway_text =
            "cannot confirm group order: the presentation gives 60 cosets, and 2/chi is 120";
        let side_text =
            "cannot confirm group order: the side presentation gives 30 cosets, and 2/chi is 120";
        let cases: [(Option<Fault>, Option<Fault>, &[&str]); 9] = [
            (None, None, &[]),
            (Some(conway_mismatch()), None, &[conway_text]),
            (None, Some(side_mismatch()), &[side_text]),
            (
                Some(conway_mismatch()),
                Some(side_mismatch()),
                &[conway_text],
            ),
            (
                Some(internal("conway")),
                Some(side_mismatch()),
                &["internal error: conway", side_text],
            ),
            (
                Some(conway_mismatch()),
                Some(internal("side")),
                &[conway_text, "internal error: side"],
            ),
            (
                Some(internal("conway")),
                Some(internal("side")),
                &["internal error: conway", "internal error: side"],
            ),
            (Some(internal("conway")), None, &["internal error: conway"]),
            (None, Some(internal("side")), &["internal error: side"]),
        ];
        for (conway, side, expected) in cases {
            let case = format!("{conway:?} and {side:?}");
            let texts: Vec<String> = reported(conway, side)
                .iter()
                .map(ToString::to_string)
                .collect();
            assert_eq!(texts, expected, "{case}");
        }
    }

    fn conway(text: &str) -> escher_group::Presentation {
        let signature: Signature = text.parse().expect("the signature is valid");
        conway_presentation(&signature).expect("the signature has a presentation")
    }

    #[test]
    fn enumeration_runs_at_the_limit_and_is_skipped_above_it() {
        let presentation = conway("22");
        let limit = CosetLimit::new(2).expect("2 is a valid limit");
        let two = NonZeroU64::new(2).expect("2 is not zero");
        let three = NonZeroU64::new(3).expect("3 is not zero");
        assert_eq!(
            enumerate_order(&presentation, two, limit),
            Outcome::Index(2)
        );
        assert_eq!(
            enumerate_order(&presentation, three, limit),
            Outcome::AboveLimit
        );
    }

    #[test]
    fn enumeration_that_reaches_the_limit_is_not_found() {
        let presentation = conway("*632");
        let limit = CosetLimit::new(10).expect("10 is a valid limit");
        let ten = NonZeroU64::new(10).expect("10 is not zero");
        assert_eq!(
            enumerate_order(&presentation, ten, limit),
            Outcome::LimitReached
        );
    }
}
