use escher_domain::{FundamentalDomain, ShapeParameters, parameters};
use escher_group::{Presentation, Relator};
use escher_signature::Signature;

pub(crate) const GRID_ASPECTS: [f64; 6] = [0.125, 0.3, 1.0, 1.618, 4.0, 8.0];

pub(crate) const GRID_SKEWS: [f64; 5] = [-0.5, -0.3, 0.0, 0.25, 0.5];

pub(crate) fn signature(text: &str) -> Signature {
    match text.parse() {
        Ok(signature) => signature,
        Err(error) => panic!("{text:?} is rejected: {error}"),
    }
}

pub(crate) fn built(text: &str) -> FundamentalDomain {
    match FundamentalDomain::build(&signature(text)) {
        Ok(domain) => domain,
        Err(error) => panic!("{text} has no domain: {error}"),
    }
}

pub(crate) fn build_with_aspect(
    text: &str,
    aspect: f64,
    skew: Option<f64>,
) -> escher_domain::Result<FundamentalDomain> {
    let parsed = signature(text);
    let mut settings = ShapeParameters::new();
    for spec in parameters(&parsed) {
        match (spec.name(), skew) {
            ("aspect", _) => settings.set("aspect", aspect),
            ("skew", Some(skew)) => settings.set("skew", skew),
            _ => None,
        };
    }
    FundamentalDomain::build_with(&parsed, &settings)
}

pub(crate) fn built_with_aspect(text: &str, aspect: f64, skew: Option<f64>) -> FundamentalDomain {
    match build_with_aspect(text, aspect, skew) {
        Ok(domain) => domain,
        Err(error) => panic!("{text} at aspect {aspect} and skew {skew:?} has no domain: {error}"),
    }
}

pub(crate) fn relator_text(presentation: &Presentation, relator: &Relator) -> String {
    match presentation.display_relator(relator) {
        Some(text) => text.to_string(),
        None => panic!("relator {relator:?} names a missing generator"),
    }
}

pub(crate) fn relator_texts(presentation: &Presentation) -> Vec<String> {
    presentation
        .relators()
        .iter()
        .map(|relator| relator_text(presentation, relator))
        .collect()
}

pub(crate) const NAMED_POLYGONS: [&str; 22] = [
    "*3222",
    "*3322",
    "*3333",
    "*4444",
    "*5432",
    "*99994",
    "*22222",
    "*33333",
    "*222222",
    "*444444",
    "*2222222",
    "*8765432",
    "*22222222",
    "*33333333",
    "*44444444",
    "*(12)(12)(12)(12)(12)(12)(12)(12)",
    "*(12)(12)333333",
    "*(1000)222",
    "*(1000)2222",
    "*(1000)(1000)(1000)(1000)",
    "*(1000)2222222",
    "*(1000)(1000)(1000)(1000)(1000)(1000)(1000)(1000)",
];

const SMALL_POLYGON_ORDERS: [(usize, u32); 5] = [(4, 7), (5, 7), (6, 6), (7, 5), (8, 4)];

pub(crate) fn corner_orders(signature: &Signature) -> Vec<u32> {
    match signature.boundaries() {
        [boundary] => boundary
            .corner_orders()
            .iter()
            .map(|order| match order.finite() {
                Some(order) => order,
                None => panic!("{signature} has an infinite corner order"),
            })
            .collect(),
        _ => panic!("{signature} has no single boundary"),
    }
}

pub(crate) fn small_polygons() -> Vec<Signature> {
    let mut canonical = std::collections::BTreeMap::new();
    for (count, largest) in SMALL_POLYGON_ORDERS {
        let mut orders = vec![2; count];
        loop {
            let text: String = core::iter::once('*'.to_string())
                .chain(orders.iter().map(ToString::to_string))
                .collect();
            let parsed = signature(&text);
            if parsed.geometry() == escher_signature::Geometry::Hyperbolic {
                canonical.entry(parsed.to_string()).or_insert(parsed);
            }
            let Some(position) = orders.iter().position(|&order| order < largest) else {
                break;
            };
            orders[position] += 1;
            for order in &mut orders[..position] {
                *order = 2;
            }
        }
    }
    canonical.into_values().collect()
}
