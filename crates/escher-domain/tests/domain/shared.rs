use escher_domain::FundamentalDomain;
use escher_group::{Presentation, Relator};
use escher_signature::Signature;

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
