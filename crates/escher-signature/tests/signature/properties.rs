use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq, prop_oneof, proptest};

use escher_signature::{ErrorKind, Geometry};

use crate::common::{Model, Value, assert_same_result, parse, proptest_config};

fn value() -> impl Strategy<Value = Value> {
    prop_oneof![
        6 => (2_u32..=9).prop_map(Value::Finite),
        2 => (10_u32..=40).prop_map(Value::Finite),
        2 => Just(Value::Infinite),
        1 => (2_000_000_000_u32..=u32::MAX).prop_map(Value::Finite),
    ]
}

fn model() -> impl Strategy<Value = Model> {
    (
        0_u32..=2,
        0_u32..=3,
        vec(value(), 0..=4),
        vec(vec(value(), 0..=4), 0..=3),
    )
        .prop_map(|(handles, crosscaps, cones, boundaries)| Model {
            handles,
            crosscaps,
            cones,
            boundaries,
        })
}

fn apply_moves(model: &Model, picks: &[u16]) -> Model {
    let mut changed = model.clone();
    for &pick in picks {
        let moves = changed.moves();
        if let Some(next) = moves.get(usize::from(pick) % moves.len().max(1)) {
            changed = next.clone();
        }
    }
    changed
}

fn insert_ones(mut tokens: Vec<String>, insertions: &[(u16, bool)]) -> String {
    for &(place, parenthesized) in insertions {
        let place = usize::from(place) % (tokens.len() + 1);
        let follows_one = place > 0 && tokens.get(place - 1).is_some_and(|token| token == "1");
        let precedes_digit = tokens
            .get(place)
            .is_some_and(|token| token.starts_with(|character: char| character.is_ascii_digit()));
        if parenthesized {
            tokens.insert(place, "(1)".to_owned());
        } else if !follows_one && !precedes_digit {
            tokens.insert(place, "1".to_owned());
        }
    }
    let text = tokens.concat();
    if text.is_empty() {
        "1".to_owned()
    } else {
        text
    }
}

fn vary_layout(model: &Model, choices: &[(u16, u8)]) -> String {
    let mut tokens = model.tokens();
    for &(place, choice) in choices {
        let Some(index) = usize::from(place).checked_rem(tokens.len()) else {
            break;
        };
        let token = tokens[index].clone();
        match token.as_str() {
            "o" | "x" if choice % 2 == 0 => {
                tokens[index] = token.to_uppercase();
            }
            "o" | "x" | "O" | "X" => {
                tokens.remove(index);
                let first_boundary = tokens.iter().position(|token| token == "*");
                let gaps = (0..=tokens.len())
                    .filter(|&gap| {
                        first_boundary.is_none_or(|first| gap <= first)
                            || gap == tokens.len()
                            || tokens[gap] == "*"
                    })
                    .collect::<Vec<_>>();
                let gap = gaps[usize::from(choice) % gaps.len()];
                tokens.insert(gap, token);
            }
            "(inf)" => "\u{221e}".clone_into(&mut tokens[index]),
            digit
                if digit.len() == 1
                    && digit.starts_with(|character: char| character.is_ascii_digit()) =>
            {
                let zeros = "0".repeat(usize::from(choice % 3));
                tokens[index] = format!("({zeros}{digit})");
            }
            _ => {}
        }
    }
    let text = tokens.concat();
    if text.is_empty() {
        "1".to_owned()
    } else {
        text
    }
}

fn compact_model() -> impl Strategy<Value = Model> {
    let finite = || (2_u32..=12).prop_map(Value::Finite);
    (
        0_u32..=4,
        0_u32..=6,
        vec(finite(), 0..=10),
        vec(vec(finite(), 0..=6), 0..=4),
    )
        .prop_map(|(handles, crosscaps, cones, boundaries)| Model {
            handles,
            crosscaps,
            cones,
            boundaries,
        })
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn equivalent_spellings_give_equal_signatures_or_the_same_error_kind(
        model in model(),
        picks in vec(any::<u16>(), 0..24),
        insertions in vec((any::<u16>(), any::<bool>()), 0..6),
    ) {
        let changed = apply_moves(&model, &picks);
        let text = insert_ones(changed.tokens(), &insertions);
        prop_assert!(text.len() <= 1024);
        assert_same_result(&model.text(), &text);
    }

    #[test]
    fn layout_variants_give_equal_signatures_or_the_same_error_kind(
        model in model(),
        choices in vec((any::<u16>(), any::<u8>()), 0..8),
    ) {
        assert_same_result(&model.text(), &vary_layout(&model, &choices));
    }

    #[test]
    fn canonical_string_is_idempotent_and_round_trips(model in model()) {
        if let Ok(signature) = parse(&model.text()) {
            let text = signature.to_string();
            let again = parse(&text);
            prop_assert_eq!(again.as_ref(), Ok(&signature));
            prop_assert_eq!(again.map(|signature| signature.to_string()), Ok(text));
        }
    }

    #[test]
    fn parse_of_any_text_returns_a_value_or_an_error_with_a_valid_span(
        text in prop_oneof![
            any::<String>(),
            "[oOxX*0-9()inf\u{221e} q]{0,24}",
        ],
    ) {
        match parse(&text) {
            Ok(signature) => {
                if signature.geometry() == Geometry::Spherical {
                    prop_assert!(signature.group_order().is_some());
                }
                prop_assert_eq!(parse(&signature.to_string()), Ok(signature));
            }
            Err(error) => {
                let span = error.span();
                prop_assert!(span.start <= span.end && span.end <= text.len());
                prop_assert!(text.is_char_boundary(span.start));
                prop_assert!(text.is_char_boundary(span.end));
            }
        }
    }

    #[test]
    fn compact_hyperbolic_signatures_parse_with_a_deformation_dimension(model in compact_model()) {
        let text = model.text();
        match parse(&text) {
            Ok(signature) => {
                prop_assert!(signature.is_compact());
                let dimension = signature.deformation_dimension();
                if signature.geometry() == Geometry::Hyperbolic {
                    let count = |length: usize| i64::try_from(length).expect("small count");
                    let corners = signature
                        .boundaries()
                        .iter()
                        .map(|boundary| count(boundary.corner_orders().len()))
                        .sum::<i64>();
                    let expected = 6 * i64::from(signature.handles())
                        + 3 * i64::from(signature.crosscaps())
                        + 3 * count(signature.boundaries().len())
                        + 2 * count(signature.cone_orders().len())
                        + corners
                        - 6;
                    prop_assert_eq!(dimension.map(i64::from), Some(expected), "{}", text);
                } else {
                    prop_assert_eq!(dimension, None);
                }
            }
            Err(error) => prop_assert_eq!(error.kind(), ErrorKind::BadOrbifold, "{}", text),
        }
    }
}
