use std::io::Write;

use escher_signature::{Geometry, Rational, Signature};

use crate::diagnostic::SignatureError;

pub(crate) fn info(text: &str, stdout: &mut dyn Write) -> anyhow::Result<()> {
    let signature: Signature = text
        .parse()
        .map_err(|error| SignatureError::new(text, error))?;
    crate::write_stdout(stdout, &information_text(&signature))
}

fn information_text(signature: &Signature) -> String {
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
    let mut text = String::new();
    for (label, value) in lines {
        text.push_str(label);
        text.push_str(": ");
        text.push_str(&value);
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
