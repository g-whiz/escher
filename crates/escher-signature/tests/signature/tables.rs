use escher_signature::{ErrorKind, Geometry};

use crate::common::{canonical, chi, kind_and_span, parse, signature};

const EUCLIDEAN: [&str; 17] = [
    "*632", "632", "*442", "4*2", "442", "*333", "3*3", "333", "*2222", "2*22", "22*", "2222",
    "**", "*x", "xx", "22x", "o",
];

fn order_text(order: u32) -> String {
    match order {
        1 => "(1)".to_owned(),
        2..=9 => order.to_string(),
        _ => format!("({order})"),
    }
}

fn assert_spherical(text: &str, group_order: u32) {
    let expected = match group_order {
        1 => "2".to_owned(),
        2 => "1".to_owned(),
        even if even % 2 == 0 => format!("1/{}", even / 2),
        odd => format!("2/{odd}"),
    };
    assert_eq!(chi(text), expected, "{text:?}");
    let signature = signature(text);
    assert_eq!(signature.geometry(), Geometry::Spherical, "{text:?}");
    assert!(signature.is_compact(), "{text:?}");
    assert!(!signature.is_frieze(), "{text:?}");
    assert!(!signature.is_general_non_compact(), "{text:?}");
    let order = signature.group_order().map(core::num::NonZeroU64::get);
    assert_eq!(order, Some(u64::from(group_order)), "{text:?}");
}

#[test]
fn euclidean_signatures_have_characteristic_zero_and_display_as_written() {
    for text in EUCLIDEAN {
        assert_eq!(chi(text), "0", "{text:?}");
        assert_eq!(canonical(text), text);
        let signature = signature(text);
        assert_eq!(signature.geometry(), Geometry::Euclidean, "{text:?}");
        assert!(signature.is_compact(), "{text:?}");
        assert!(!signature.is_frieze(), "{text:?}");
        assert!(!signature.is_general_non_compact(), "{text:?}");
    }
}

#[test]
fn spherical_series_have_the_listed_group_order_and_characteristic() {
    for n in 1..=12 {
        let order = order_text(n);
        let series = [
            (format!("{order}{order}"), n),
            (format!("*{order}{order}"), 2 * n),
            (format!("{order}*"), 2 * n),
            (format!("{order}x"), 2 * n),
            (format!("22{order}"), 2 * n),
            (format!("*22{order}"), 4 * n),
            (format!("2*{order}"), 4 * n),
        ];
        for (text, group_order) in series {
            assert_spherical(&text, group_order);
        }
    }
}

#[test]
fn spherical_particular_groups_have_the_listed_group_order_and_characteristic() {
    let groups = [
        ("332", 12),
        ("*332", 24),
        ("3*2", 24),
        ("432", 24),
        ("*432", 48),
        ("532", 60),
        ("*532", 120),
    ];
    for (text, group_order) in groups {
        assert_spherical(text, group_order);
    }
}

#[test]
fn frieze_signatures_are_euclidean_non_compact_friezes_in_both_spellings() {
    let friezes = [
        ("(inf)(inf)", "\u{221e}\u{221e}"),
        ("(inf)x", "\u{221e}x"),
        ("(inf)*", "\u{221e}*"),
        ("*(inf)(inf)", "*\u{221e}\u{221e}"),
        ("22(inf)", "22\u{221e}"),
        ("2*(inf)", "2*\u{221e}"),
        ("*22(inf)", "*22\u{221e}"),
    ];
    for (ascii, sign) in friezes {
        assert_eq!(chi(ascii), "0", "{ascii:?}");
        assert_eq!(parse(ascii), parse(sign));
        let signature = signature(ascii);
        assert_eq!(signature.geometry(), Geometry::Euclidean, "{ascii:?}");
        assert!(signature.is_frieze(), "{ascii:?}");
        assert!(!signature.is_compact(), "{ascii:?}");
        assert!(!signature.is_general_non_compact(), "{ascii:?}");
    }
}

#[test]
fn characteristic_of_the_reference_examples() {
    let cases = [
        ("4*3", "-1/12"),
        ("*3333", "-1/3"),
        ("*532", "1/60"),
        ("*22(inf)", "0"),
    ];
    for (text, expected) in cases {
        assert_eq!(chi(text), expected, "{text:?}");
    }
}

#[test]
fn bad_orbifold_families_are_rejected_and_equal_orders_accepted() {
    for m in 2..=9 {
        for prefix in ["", "*"] {
            let one = format!("{prefix}{m}");
            assert_eq!(kind_and_span(&one), (ErrorKind::BadOrbifold, 0..one.len()));
            for n in 2..=9 {
                let two = format!("{prefix}{m}{n}");
                if m == n {
                    assert!(parse(&two).is_ok(), "{two:?}");
                } else {
                    assert_eq!(kind_and_span(&two), (ErrorKind::BadOrbifold, 0..two.len()));
                }
            }
        }
    }
    assert_eq!(kind_and_span("(12)(13)").0, ErrorKind::BadOrbifold);
    assert!(parse("2x").is_ok());
    assert!(parse("o2").is_ok());
    assert!(parse("2*").is_ok());
    assert!(parse("*2*").is_ok());
}

#[test]
fn signatures_of_order_two_are_three_different_groups() {
    let [crosscap, rotation, mirror] = ["x", "22", "*"].map(parse);
    assert_ne!(crosscap, rotation);
    assert_ne!(rotation, mirror);
    assert_ne!(crosscap, mirror);
    assert_ne!(parse("*632"), parse("632"));
    for text in ["x", "22", "*"] {
        let order = signature(text)
            .group_order()
            .map(core::num::NonZeroU64::get);
        assert_eq!(order, Some(2), "{text:?}");
    }
}

#[test]
fn area_and_deformation_dimension_of_the_reference_examples() {
    let area = |text: &str| signature(text).area_over_pi().map(|area| area.to_string());
    assert_eq!(area("*3333").as_deref(), Some("2/3"));
    assert_eq!(area("*732").as_deref(), Some("1/42"));
    assert_eq!(area("*532").as_deref(), Some("1/30"));
    assert_eq!(area("3*(inf)").as_deref(), Some("1/3"));
    assert_eq!(area("*632"), None);
    let dimension = |text: &str| signature(text).deformation_dimension();
    assert_eq!(dimension("*732"), Some(0));
    assert_eq!(dimension("4*3"), Some(0));
    assert_eq!(dimension("*3333"), Some(1));
    assert_eq!(dimension("4*235*74xx"), Some(13));
    for text in ["*632", "o", "*532", "1", "*22(inf)", "3*(inf)", "*23(inf)"] {
        assert_eq!(dimension(text), None, "{text:?}");
    }
}

#[test]
fn general_non_compact_signatures_exclude_friezes_and_ideal_triangles() {
    let general = [
        ("3*(inf)", "-1/6"),
        ("(inf)(inf)(inf)(inf)", "-2"),
        ("o*23(inf)", "-25/12"),
        ("x*23(inf)", "-13/12"),
        ("2*23(inf)", "-7/12"),
        ("(inf)32*", "-7/6"),
        ("*732*(inf)", "-127/84"),
    ];
    for (text, characteristic) in general {
        assert_eq!(chi(text), characteristic, "{text:?}");
        let signature = signature(text);
        assert!(signature.is_general_non_compact(), "{text:?}");
        assert_eq!(signature.geometry(), Geometry::Hyperbolic, "{text:?}");
    }
    for text in [
        "*23(inf)",
        "(inf)32",
        "*22(inf)",
        "*732",
        "*(inf)(inf)(inf)",
    ] {
        assert!(!signature(text).is_general_non_compact(), "{text:?}");
    }
}
