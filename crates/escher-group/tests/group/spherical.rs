use std::collections::BTreeSet;

use escher_group::{CosetLimit, conway_presentation, enumerate};
use escher_signature::Signature;

use crate::common::signature;

fn order_text(order: u32) -> String {
    if (2..10).contains(&order) {
        order.to_string()
    } else {
        format!("({order})")
    }
}

pub(crate) fn spherical_texts() -> Vec<String> {
    let mut texts = Vec::new();
    for n in 1..=12 {
        let order = order_text(n);
        texts.push(format!("{order}{order}"));
        texts.push(format!("*{order}{order}"));
        texts.push(format!("{order}*"));
        texts.push(format!("{order}x"));
        texts.push(format!("{order}22"));
        texts.push(format!("*{order}22"));
        texts.push(format!("2*{order}"));
    }
    for text in ["332", "*332", "3*2", "432", "*432", "532", "*532"] {
        texts.push(text.to_owned());
    }
    texts
}

pub(crate) fn spherical_signatures() -> Vec<Signature> {
    let mut seen = BTreeSet::new();
    let mut signatures = Vec::new();
    for text in spherical_texts() {
        let signature = signature(&text);
        if seen.insert(signature.to_string()) {
            signatures.push(signature);
        }
    }
    signatures
}

fn assert_index_is_group_order(signature: &Signature) {
    let presentation = match conway_presentation(signature) {
        Ok(presentation) => presentation,
        Err(error) => panic!("{signature} has no presentation: {error}"),
    };
    let order = signature.group_order().map(core::num::NonZeroU64::get);
    let index = enumerate(&presentation, &[], CosetLimit::DEFAULT)
        .map(|table| u64::try_from(table.index()).unwrap_or(u64::MAX));
    assert_eq!(index.ok(), order, "{signature}");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn spherical_orders_equal_the_group_order() {
    assert_eq!(spherical_texts().len(), 91);
    let signatures = spherical_signatures();
    assert_eq!(signatures.len(), 87);
    assert!(
        signatures
            .iter()
            .any(|signature| signature.to_string() == "1")
    );
    for signature in &signatures {
        assert_index_is_group_order(signature);
    }
}

#[test]
fn spherical_order_of_2_star_3() {
    assert_index_is_group_order(&signature("2*3"));
}

#[test]
fn spherical_order_of_3_star_2() {
    assert_index_is_group_order(&signature("3*2"));
}

#[test]
fn order_two_signatures_enumerate_to_index_two() {
    for text in ["x", "22", "*"] {
        let signature = signature(text);
        assert_eq!(
            signature.group_order().map(core::num::NonZeroU64::get),
            Some(2)
        );
        assert_index_is_group_order(&signature);
    }
}
