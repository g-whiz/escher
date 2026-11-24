use core::cmp::Ordering;
use core::num::NonZeroU64;
use std::borrow::Cow;
use std::io::Write;

use escher_domain::{Check, FundamentalDomain, PairingKind, Verification, parameters};
use escher_group::{
    Colouring, ColouringCounts, ColouringId, Colourings, CosetLimit, ErrorKind, IndexLimit,
    Presentation, SearchBudget, colourings, conway_presentation, count_colourings, enumerate,
};
use escher_render::{EdgeMeeting, WholeFace};
use escher_signature::{Geometry, Rational, Signature};

use crate::count_of;
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
pub(crate) enum Listing {
    Count,
    List,
}

impl Listing {
    pub(crate) const fn from_flag(colourings: bool) -> Self {
        if colourings { Self::List } else { Self::Count }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Options {
    pub(crate) verify: Verify,
    pub(crate) presentations: Presentations,
    pub(crate) listing: Listing,
    pub(crate) max_colours: IndexLimit,
    pub(crate) wythoff: Option<[bool; 3]>,
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

type Lines = Vec<(Cow<'static, str>, String)>;

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

pub(crate) fn info(text: &str, options: Options, stdout: &mut dyn Write) -> Vec<anyhow::Error> {
    let signature: Signature = match text.parse() {
        Ok(signature) => signature,
        Err(error) => return vec![anyhow::Error::new(SignatureError::new(text, error))],
    };
    let mut lines = signature_lines(&signature);
    let domain = FundamentalDomain::build(&signature);
    push_domain_summary(&mut lines, domain.as_ref());
    let side = match &domain {
        Ok(domain) => {
            let presentation = domain.side_presentation();
            let found = match &presentation {
                Ok(presentation) => search_colourings(
                    presentation,
                    options.max_colours,
                    SearchBudget::DEFAULT,
                    options.listing,
                )
                .map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            push_colourings(&mut lines, found);
            if let Some(marks) = options.wythoff {
                push_arrangements(
                    &mut lines,
                    &find_arrangements(domain, marks, options.max_colours, SearchBudget::DEFAULT),
                );
            }
            Some(presentation.map_err(|error| error.to_string()))
        }
        Err(_) => None,
    };
    let mut errors = Vec::new();
    if options.presentations == Presentations::Print {
        errors.extend(push_presentations(&mut lines, &signature, side));
    }
    let verdict = match (options.verify, domain) {
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
    side: Option<Result<Presentation, String>>,
) -> Vec<anyhow::Error> {
    let order = signature.group_order();
    let conway = push_presentation(
        lines,
        &CONWAY,
        conway_presentation(signature).map_err(|error| error.to_string()),
        order,
    );
    let side = side.and_then(|side| push_presentation(lines, &SIDE, side, order));
    reported(conway, side)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Alias {
    One,
    Parity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Found {
    Counted(ColouringCounts),
    Listed(Colourings),
}

impl Found {
    fn requested(&self) -> u32 {
        match self {
            Self::Counted(counts) => counts.requested(),
            Self::Listed(found) => found.requested(),
        }
    }

    fn complete_to(&self) -> u32 {
        match self {
            Self::Counted(counts) => counts.complete_to(),
            Self::Listed(found) => found.complete_to(),
        }
    }

    fn count(&self, colours: u32) -> Option<u64> {
        match self {
            Self::Counted(counts) => counts.colourings(colours),
            Self::Listed(found) => found.count(colours),
        }
    }

    fn list(&self) -> &[Colouring] {
        match self {
            Self::Counted(_) => &[],
            Self::Listed(found) => found.list(),
        }
    }
}

fn search_colourings(
    presentation: &Presentation,
    limit: IndexLimit,
    budget: SearchBudget,
    listing: Listing,
) -> escher_group::Result<Found> {
    Ok(match listing {
        Listing::Count => Found::Counted(count_colourings(
            presentation,
            crate::COLOURING_SET,
            limit,
            budget,
        )?),
        Listing::List => Found::Listed(colourings(
            presentation,
            crate::COLOURING_SET,
            limit,
            budget,
        )?),
    })
}

fn alias_of(id: ColouringId) -> Option<Alias> {
    match (id.colours(), id.ordinal()) {
        (1, 1) => Some(Alias::One),
        (2, 1) => Some(Alias::Parity),
        _ => None,
    }
}

fn push_colourings(lines: &mut Lines, found: Result<Found, String>) {
    let found = match found {
        Ok(found) => found,
        Err(text) => {
            lines.push(("Colourings".into(), format!("none, {text}")));
            return;
        }
    };
    let counts: Vec<(u32, u64)> = (1..=found.complete_to())
        .filter_map(|colours| Some((colours, found.count(colours)?)))
        .collect();
    let total = counts
        .iter()
        .fold(0_u64, |total, &(_, count)| total.saturating_add(count));
    lines.push((
        "Colourings".into(),
        summary_text(total, found.complete_to(), found.requested()),
    ));
    for &(colours, count) in &counts {
        if count > 0 {
            lines.push((count_label(colours).into(), count.to_string()));
        }
    }
    for colouring in found.list() {
        lines.push((
            "Colouring".into(),
            listed_text(colouring.id(), alias_of(colouring.id())),
        ));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Arrangements {
    Stopped,
    Listed {
        sides: u32,
        list: Vec<(ColouringId, Vec<EdgeMeeting>)>,
    },
}

fn find_arrangements(
    domain: &FundamentalDomain,
    marks: [bool; 3],
    max_pictures: IndexLimit,
    budget: SearchBudget,
) -> Vec<Result<Arrangements, String>> {
    let face = match crate::render::whole_face(domain, marks) {
        Ok(face) => face,
        Err(error) => return vec![Err(error.to_string())],
    };
    let mut found = Vec::new();
    for pictures in 1..=largest_pictures(max_pictures, face.index()) {
        let listed = arrangements_of(&face, pictures, budget);
        let complete = matches!(listed, Ok(Arrangements::Listed { .. }));
        found.push(listed);
        if !complete {
            break;
        }
    }
    found
}

fn largest_pictures(max_pictures: IndexLimit, index: u32) -> u32 {
    max_pictures.get().min(IndexLimit::MAX.get() / index).max(1)
}

fn arrangements_of(
    face: &WholeFace,
    pictures: u32,
    budget: SearchBudget,
) -> Result<Arrangements, String> {
    let found =
        crate::render::arrangements(face, pictures, budget).map_err(|error| error.to_string())?;
    if !found.is_complete() {
        return Ok(Arrangements::Stopped);
    }
    let list = found
        .list()
        .iter()
        .map(|arrangement| {
            let edges = face.edges(arrangement).map_err(|error| error.to_string())?;
            Ok((arrangement.id(), edges))
        })
        .collect::<Result<_, String>>()?;
    Ok(Arrangements::Listed {
        sides: face.index() / 2,
        list,
    })
}

fn push_arrangements(lines: &mut Lines, found: &[Result<Arrangements, String>]) {
    for (pictures, found) in (1..).zip(found) {
        let label: Cow<'static, str> = if pictures == 1 {
            "Arrangements".into()
        } else {
            format!("Arrangements with {pictures} pictures").into()
        };
        match found {
            Err(text) => lines.push((label, format!("none, {text}"))),
            Ok(Arrangements::Stopped) => {
                lines.push((label, "none, the search stopped at a limit".to_owned()));
            }
            Ok(Arrangements::Listed { sides, list }) => {
                lines.push((label, list.len().to_string()));
                for (id, meetings) in list {
                    lines.push((
                        "Arrangement".into(),
                        arrangement_text(*id, pictures, *sides, meetings),
                    ));
                }
            }
        }
    }
}

fn arrangement_text(
    id: ColouringId,
    pictures: u32,
    sides: u32,
    meetings: &[EdgeMeeting],
) -> String {
    let name = |picture: u32, edge: u32| {
        if pictures == 1 {
            edge.to_string()
        } else {
            format!("{}{edge}", crate::picture_letter(picture))
        }
    };
    let kind = |meeting: &EdgeMeeting| {
        if meeting.mirrored() {
            "mirrored"
        } else {
            "turned"
        }
    };
    let neighbours = if meetings.iter().any(|meeting| meeting.mirrored()) {
        "with mirrored neighbours"
    } else {
        "turned only"
    };
    let pairs: Vec<String> = (0..)
        .zip(meetings)
        .filter_map(|(position, meeting)| {
            let own = name(position / sides, position % sides);
            match (meeting.picture() * sides + meeting.partner()).cmp(&position) {
                Ordering::Less => None,
                Ordering::Equal => Some(format!("{own} {}", kind(meeting))),
                Ordering::Greater => Some(format!(
                    "{own} and {} {}",
                    name(meeting.picture(), meeting.partner()),
                    kind(meeting)
                )),
            }
        })
        .collect();
    let count = if pictures == 1 {
        String::new()
    } else {
        format!("{pictures} pictures, ")
    };
    format!("{id}, {count}{neighbours}, edges {}", pairs.join(", "))
}

fn summary_text(total: u64, complete_to: u32, requested: u32) -> String {
    let complete = count_of(u64::from(complete_to), "colour");
    if complete_to >= requested {
        format!("{total}, complete to {complete}")
    } else {
        format!(
            "{total}, complete to {complete}, the search stopped at a limit before {}",
            count_of(u64::from(complete_to) + 1, "colour")
        )
    }
}

fn count_label(colours: u32) -> String {
    format!("Colourings with {}", count_of(u64::from(colours), "colour"))
}

fn listed_text(id: ColouringId, alias: Option<Alias>) -> String {
    let alias = match alias {
        Some(Alias::One) => ", alias one",
        Some(Alias::Parity) => ", alias parity",
        None => "",
    };
    format!(
        "{id}, {}{alias}",
        count_of(u64::from(id.colours()), "colour")
    )
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
            lines.push((labels.size.into(), format!("none, {text}")));
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
        labels.size.into(),
        size_text(presentation.generator_count(), relators.len()),
    ));
    lines.push((
        labels.generators.into(),
        list_or_none(presentation.generator_names()),
    ));
    lines.push((labels.relators.into(), list_or_none(&relators)));
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
    lines.push((labels.order.into(), order_text(order, limit, outcome)));
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
        ("Signature".into(), signature.to_string()),
        (
            "Geometry".into(),
            geometry_name(signature.geometry()).to_owned(),
        ),
        (
            "Euler characteristic".into(),
            signature.euler_characteristic().to_string(),
        ),
        (
            "Compact".into(),
            yes_or_no(signature.is_compact()).to_owned(),
        ),
    ];
    if !signature.is_compact() {
        lines.push(("Frieze".into(), yes_or_no(signature.is_frieze()).to_owned()));
        lines.push((
            "General non-compact".into(),
            yes_or_no(signature.is_general_non_compact()).to_owned(),
        ));
    }
    if let Some(order) = signature.group_order() {
        lines.push(("Group order".into(), order.to_string()));
    }
    if let Some(area) = signature.area_over_pi() {
        lines.push(("Area".into(), area_text(area)));
    }
    if let Some(dimension) = signature.deformation_dimension() {
        lines.push(("Deformation dimension".into(), dimension.to_string()));
    }
    if let Some((larger, smaller)) = regular_tiling(signature) {
        let value = if larger == smaller {
            format!("{{{larger},{larger}}}, self-dual")
        } else {
            format!("{{{larger},{smaller}}}, dual {{{smaller},{larger}}}")
        };
        lines.push(("Regular tiling".into(), value));
    }
    for spec in parameters(signature) {
        lines.push((
            "Parameter".into(),
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

fn regular_tiling(signature: &Signature) -> Option<(u32, u32)> {
    if signature.handles() != 0 || signature.crosscaps() != 0 {
        return None;
    }
    let orders = match (signature.cone_orders(), signature.boundaries()) {
        ([], [boundary]) => boundary.corner_orders(),
        (cones, []) => cones,
        _ => return None,
    };
    let [larger, smaller, last] = orders else {
        return None;
    };
    match (larger.finite()?, smaller.finite()?, last.finite()?) {
        (larger, smaller, 2) if smaller >= 3 => Some((larger, smaller)),
        _ => None,
    }
}

fn push_domain_summary(
    lines: &mut Lines,
    domain: Result<&FundamentalDomain, &escher_domain::Error>,
) {
    match domain {
        Ok(domain) => {
            let sides = domain.sides();
            let mirrors: Vec<usize> = sides
                .iter()
                .enumerate()
                .filter(|(_, side)| side.kind() == PairingKind::Mirror)
                .map(|(index, _)| index)
                .collect();
            if sides.is_empty() {
                lines.push(("Domain".into(), "no sides".to_owned()));
            } else if mirrors.len() == sides.len() {
                lines.push(("Domain".into(), format!("{} mirror sides", sides.len())));
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
                let pair_noun = if pairs.len() == 1 { "pair" } else { "pairs" };
                let summary = if mirrors.is_empty() {
                    format!("{} sides in {} {pair_noun}", sides.len(), pairs.len())
                } else {
                    let mirror_noun = if mirrors.len() == 1 {
                        "mirror side"
                    } else {
                        "mirror sides"
                    };
                    format!(
                        "{} sides, {} {mirror_noun} and {} {pair_noun}",
                        sides.len(),
                        mirrors.len(),
                        pairs.len()
                    )
                };
                lines.push(("Domain".into(), summary));
                if !mirrors.is_empty() {
                    lines.push(("Mirror sides".into(), index_list(&mirrors)));
                }
                lines.push(("Side pairs".into(), pairs.join(", ")));
            }
            if !sides.is_empty() {
                lines.push((
                    "Corner angles in degrees".into(),
                    fixed_list(domain.corner_angles().into_iter().map(f64::to_degrees)),
                ));
                lines.push((
                    "Side lengths".into(),
                    fixed_list(domain.side_lengths().into_iter()),
                ));
            }
            lines.push(("Domain area".into(), format!("{:.6}", domain.area())));
        }
        Err(error) => lines.push(("Domain".into(), format!("none, {error}"))),
    }
}

fn index_list(indices: &[usize]) -> String {
    match indices {
        [] => String::new(),
        [only] => only.to_string(),
        [first, second] => format!("{first} and {second}"),
        [init @ .., last] => {
            let leading: Vec<String> = init.iter().map(ToString::to_string).collect();
            format!("{}, and {last}", leading.join(", "))
        }
    }
}

fn kind_name(kind: PairingKind) -> &'static str {
    match kind {
        PairingKind::Mirror => "reflection",
        PairingKind::Rotation => "rotation",
        PairingKind::Translation => "translation",
        PairingKind::GlideReflection => "glide reflection",
        PairingKind::RotaryReflection => "rotary reflection",
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
            row.label.into(),
            check_text(row.verdict, row.residual, row.tolerance),
        ));
    }
    lines.push(("Verification".into(), overall.text().to_owned()));
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

    use escher_domain::{FundamentalDomain, PairingKind};
    use escher_group::{CosetLimit, IndexLimit, SearchBudget, conway_presentation};
    use escher_signature::Signature;

    use super::{
        Alias, CONWAY, CheckRow, Fault, Listing, Outcome, SIDE, Verdict, alias_of,
        arrangement_text, check_text, count_label, enumerate_order, failed_checks,
        find_arrangements, index_list, joined, kind_name, largest_pictures, listed_text,
        order_mismatch, order_text, push_arrangements, push_colourings, push_order,
        push_verification, reported, search_colourings, signature_lines, size_text, summary_text,
        write_text,
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
        let mut lines = vec![("Signature".into(), "*732".to_owned())];
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
        let mut lines = vec![("Signature".into(), "*532".to_owned())];
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

    #[test]
    fn index_list_joins_one_two_and_more_indices() {
        assert_eq!(index_list(&[2]), "2");
        assert_eq!(index_list(&[2, 5]), "2 and 5");
        assert_eq!(index_list(&[2, 3, 4]), "2, 3, and 4");
        assert_eq!(index_list(&[0, 1, 2, 3]), "0, 1, 2, and 3");
    }

    #[test]
    fn kind_name_names_each_kind_of_pairing() {
        assert_eq!(kind_name(PairingKind::Mirror), "reflection");
        assert_eq!(kind_name(PairingKind::Rotation), "rotation");
        assert_eq!(kind_name(PairingKind::Translation), "translation");
        assert_eq!(kind_name(PairingKind::GlideReflection), "glide reflection");
        assert_eq!(
            kind_name(PairingKind::RotaryReflection),
            "rotary reflection"
        );
    }

    #[test]
    fn summary_text_writes_each_form() {
        assert_eq!(summary_text(11, 3, 3), "11, complete to 3 colours");
        assert_eq!(summary_text(1, 1, 1), "1, complete to 1 colour");
        assert_eq!(
            summary_text(23, 9, 12),
            "23, complete to 9 colours, the search stopped at a limit before 10 colours"
        );
        assert_eq!(
            summary_text(0, 0, 12),
            "0, complete to 0 colours, the search stopped at a limit before 1 colour"
        );
    }

    #[test]
    fn count_label_uses_the_singular_for_one() {
        assert_eq!(count_label(1), "Colourings with 1 colour");
        assert_eq!(count_label(8), "Colourings with 8 colours");
    }

    #[test]
    fn listed_text_marks_the_aliases() {
        assert_eq!(
            listed_text(crate::colouring_id("1a"), Some(Alias::One)),
            "1a, 1 colour, alias one"
        );
        assert_eq!(
            listed_text(crate::colouring_id("2a"), Some(Alias::Parity)),
            "2a, 2 colours, alias parity"
        );
        assert_eq!(
            listed_text(crate::colouring_id("3a"), None),
            "3a, 3 colours"
        );
    }

    #[test]
    fn alias_of_marks_exactly_1a_and_2a() {
        let aliases: Vec<Option<Alias>> = ["1a", "1b", "2a", "2b", "3a", "12a"]
            .into_iter()
            .map(|text| alias_of(crate::colouring_id(text)))
            .collect();
        assert_eq!(
            aliases,
            [
                Some(Alias::One),
                None,
                Some(Alias::Parity),
                None,
                None,
                None
            ]
        );
    }

    fn colouring_text(signature: &str, steps: u64, listing: Listing) -> String {
        let signature: Signature = signature.parse().expect("the signature is valid");
        let domain = FundamentalDomain::build(&signature).expect("the signature has a domain");
        let presentation = domain
            .side_presentation()
            .expect("the domain has a side presentation");
        let budget = SearchBudget::new(steps).expect("the budget is valid");
        let found = search_colourings(&presentation, IndexLimit::DEFAULT, budget, listing)
            .map_err(|error| error.to_string());
        let mut lines = Vec::new();
        push_colourings(&mut lines, found);
        joined(&lines)
    }

    #[test]
    fn a_search_stopped_at_a_limit_prints_only_the_complete_colour_counts() {
        let head = "Colourings: 1, complete to 2 colours, the search stopped at a limit before 3 colours\n\
                    Colourings with 2 colours: 1\n";
        assert_eq!(colouring_text("*632", 3_000, Listing::Count), head);
        assert_eq!(
            colouring_text("*632", 3_000, Listing::List),
            format!("{head}Colouring: 2a, 2 colours, alias parity\n")
        );
    }

    #[test]
    fn a_search_stopped_before_one_colour_prints_no_count() {
        assert_eq!(
            colouring_text("*632", 100, Listing::List),
            "Colourings: 0, complete to 0 colours, the search stopped at a limit before 1 colour\n"
        );
    }

    #[test]
    fn a_failed_search_prints_none_and_its_text() {
        let mut lines = Vec::new();
        push_colourings(
            &mut lines,
            Err("cannot search colourings: a limit".to_owned()),
        );
        assert_eq!(
            joined(&lines),
            "Colourings: none, cannot search colourings: a limit\n"
        );
    }

    fn regular_tiling_line(text: &str) -> Option<String> {
        let signature: Signature = text.parse().expect("the signature is valid");
        signature_lines(&signature)
            .into_iter()
            .find(|(label, _)| label == "Regular tiling")
            .map(|(_, value)| value)
    }

    #[test]
    fn regular_tiling_names_the_larger_order_first_and_then_the_dual() {
        for text in ["*732", "732"] {
            assert_eq!(
                regular_tiling_line(text).as_deref(),
                Some("{7,3}, dual {3,7}"),
                "{text}"
            );
        }
    }

    #[test]
    fn regular_tiling_with_two_equal_orders_is_self_dual() {
        for text in ["*442", "442"] {
            assert_eq!(
                regular_tiling_line(text).as_deref(),
                Some("{4,4}, self-dual"),
                "{text}"
            );
        }
    }

    #[test]
    fn regular_tiling_writes_the_largest_order_in_decimal() {
        assert_eq!(
            regular_tiling_line("*(4294967295)32").as_deref(),
            Some("{4294967295,3}, dual {3,4294967295}")
        );
    }

    #[test]
    fn regular_tiling_is_absent_for_every_other_signature() {
        for text in [
            "*722", "722", "*222", "222", "*333", "333", "*3332", "2222", "o732", "732x", "2*732",
            "732*", "*732*", "*(inf)32", "(inf)32",
        ] {
            assert_eq!(regular_tiling_line(text), None, "{text}");
        }
    }

    fn arrangement_lines(
        signature: &str,
        marks: [bool; 3],
        max_pictures: u32,
        steps: u64,
    ) -> String {
        let signature: Signature = signature.parse().expect("the signature is valid");
        let domain = FundamentalDomain::build(&signature).expect("the signature has a domain");
        let limit = IndexLimit::new(max_pictures).expect("the limit is valid");
        let budget = SearchBudget::new(steps).expect("the budget is valid");
        let mut lines = Vec::new();
        push_arrangements(
            &mut lines,
            &find_arrangements(&domain, marks, limit, budget),
        );
        joined(&lines)
    }

    const HEXAGONS: [bool; 3] = [false, true, false];

    #[test]
    fn an_arrangement_search_stopped_at_a_limit_prints_no_number() {
        assert_eq!(
            arrangement_lines("*632", HEXAGONS, 12, 1),
            "Arrangements: none, the search stopped at a limit\n"
        );
    }

    #[test]
    fn a_search_stopped_at_two_pictures_ends_the_listing_after_the_lines_of_one_picture() {
        let complete = arrangement_lines("*632", HEXAGONS, 1, SearchBudget::DEFAULT.get());
        assert_eq!(complete.lines().count(), 8);
        assert_eq!(
            arrangement_lines("*632", HEXAGONS, 12, 100_000),
            format!(
                "{complete}Arrangements with 2 pictures: none, the search stopped at a limit\n"
            )
        );
    }

    #[test]
    fn a_colour_limit_of_one_lists_the_arrangements_of_one_picture_only() {
        let lines = arrangement_lines("*632", HEXAGONS, 1, SearchBudget::DEFAULT.get());
        assert!(lines.starts_with("Arrangements: 7\n"), "{lines}");
        assert!(!lines.contains("pictures"), "{lines}");
        let two = arrangement_lines("*632", HEXAGONS, 2, SearchBudget::DEFAULT.get());
        assert!(two.starts_with(&lines), "{two}");
        assert_eq!(
            two[lines.len()..].lines().next(),
            Some("Arrangements with 2 pictures: 11")
        );
        assert!(!two.contains("Arrangements with 3 pictures"), "{two}");
    }

    #[test]
    fn the_largest_number_of_pictures_is_the_colour_limit_or_what_the_index_limit_allows() {
        let limit = |value| IndexLimit::new(value).expect("the limit is valid");
        for (colours, index, largest) in [
            (12, 6, 10),
            (12, 8, 8),
            (12, 10, 6),
            (12, 12, 5),
            (12, 14, 4),
            (12, 16, 4),
            (12, 18, 3),
            (12, 32, 2),
            (12, 34, 1),
            (12, 64, 1),
            (3, 6, 3),
            (1, 6, 1),
            (64, 6, 10),
        ] {
            assert_eq!(
                largest_pictures(limit(colours), index),
                largest,
                "{colours} {index}"
            );
        }
    }

    fn listed_arrangement(signature: &str, marks: [bool; 3], id: &str) -> String {
        let signature: Signature = signature.parse().expect("the signature is valid");
        let domain = FundamentalDomain::build(&signature).expect("the signature has a domain");
        let face = crate::render::whole_face(&domain, marks).expect("the face exists");
        let id = crate::colouring_id(id);
        let pictures = crate::render::pictures(&face, id);
        let found = crate::render::arrangements(&face, pictures, SearchBudget::DEFAULT)
            .expect("the search runs");
        assert!(found.is_complete(), "{id}");
        let arrangement = found.find(id).expect("the arrangement exists");
        let edges = face.edges(arrangement).expect("the arrangement has edges");
        arrangement_text(id, pictures, face.index() / 2, &edges)
    }

    const SQUARES: [bool; 3] = [false, true, false];
    const TRIANGLES: [bool; 3] = [true, false, false];

    #[test]
    fn an_arrangement_of_one_picture_is_written_with_edge_numbers_only() {
        for (id, text) in [
            (
                "8e",
                "8e, with mirrored neighbours, edges 0 mirrored, 1 and 3 turned, 2 turned",
            ),
            (
                "8o",
                "8o, with mirrored neighbours, edges 0 and 2 mirrored, 1 and 3 turned",
            ),
            (
                "8n",
                "8n, turned only, edges 0 and 2 turned, 1 and 3 turned",
            ),
        ] {
            assert_eq!(listed_arrangement("*442", SQUARES, id), text);
        }
    }

    #[test]
    fn an_arrangement_of_two_pictures_writes_each_edge_after_the_letter_of_its_picture() {
        for (id, text) in [
            (
                "16ab",
                "16ab, 2 pictures, turned only, edges A0 and B0 turned, A1 and B1 turned, A2 and B2 turned, A3 and B3 turned",
            ),
            (
                "16h",
                "16h, 2 pictures, with mirrored neighbours, edges A0 mirrored, A1 and B2 mirrored, A2 mirrored, A3 and B0 turned, B1 mirrored, B3 mirrored",
            ),
        ] {
            assert_eq!(listed_arrangement("*442", SQUARES, id), text);
        }
    }

    #[test]
    fn an_arrangement_of_three_pictures_writes_each_edge_after_the_letter_of_its_picture() {
        assert_eq!(
            listed_arrangement("*732", TRIANGLES, "18a"),
            "18a, 3 pictures, turned only, edges A0 and A1 turned, A2 and B0 turned, B1 and C0 turned, B2 turned, C1 and C2 turned"
        );
    }
}
