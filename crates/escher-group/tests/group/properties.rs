use std::collections::VecDeque;

use escher_group::{
    CosetLimit, CosetTable, ErrorKind, Letter, Presentation, Relator, Word, conway_presentation,
    enumerate,
};
use proptest::collection::vec;
use proptest::prelude::{any, prop_assert, prop_assert_eq};
use proptest::proptest;

use crate::common::{FIXTURES, letter, names, proptest_config, random_presentation};
use crate::spherical::spherical_signatures;

fn expanded(relator: &Relator) -> Word {
    (0..relator.exponent())
        .flat_map(|_| relator.base().letters().iter().copied())
        .collect()
}

fn letters_of(table: &CosetTable) -> Vec<Letter> {
    (0..table.generator_count())
        .filter_map(|index| u8::try_from(index).ok())
        .flat_map(|index| [Letter::generator(index), Letter::inverse_of(index)])
        .collect()
}

fn check_laws(
    table: &CosetTable,
    presentation: &Presentation,
    subgroup: &[Word],
) -> Result<(), String> {
    let cosets = u32::try_from(table.index()).map_err(|error| error.to_string())?;
    for letter in letters_of(table) {
        let column = table.permutation(letter).ok_or("a column is missing")?;
        let inverse = table
            .permutation(letter.inverse())
            .ok_or("a column is missing")?;
        let mut sorted = column.clone();
        sorted.sort_unstable();
        if sorted != (0..cosets).collect::<Vec<_>>() {
            return Err(format!("the column of {letter:?} is not a permutation"));
        }
        for coset in 0..cosets {
            let image = column.get(usize::try_from(coset).map_err(|error| error.to_string())?);
            let back = image.and_then(|&image| inverse.get(usize::try_from(image).ok()?));
            if back != Some(&coset) {
                return Err(format!(
                    "the columns of {letter:?} are not inverse at {coset}"
                ));
            }
        }
    }
    for relator in presentation.relators() {
        let word = expanded(relator);
        for coset in 0..cosets {
            if table.trace(coset, &word) != Some(coset) {
                return Err(format!("a relator moves coset {coset}"));
            }
        }
    }
    for word in subgroup {
        if table.trace(0, word) != Some(0) {
            return Err("a subgroup generator moves coset 0".to_owned());
        }
    }
    let mut reached = vec![false; table.index()];
    let mut queue = VecDeque::from([0_u32]);
    if let Some(first) = reached.first_mut() {
        *first = true;
    }
    while let Some(coset) = queue.pop_front() {
        for &entry in table.row(coset).ok_or("a row is missing")? {
            let slot = reached
                .get_mut(usize::try_from(entry).map_err(|error| error.to_string())?)
                .ok_or("an entry is out of range")?;
            if !*slot {
                *slot = true;
                queue.push_back(entry);
            }
        }
    }
    if reached.contains(&false) {
        return Err("the action is not transitive".to_owned());
    }
    let mut next = 1;
    for coset in 0..cosets {
        for &entry in table.row(coset).ok_or("a row is missing")? {
            if entry == next {
                next += 1;
            } else if entry > next {
                return Err(format!("coset {entry} first occurs before coset {next}"));
            }
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    #[cfg_attr(miri, ignore = "the enumeration runs a loop too long for the interpreter")]
    fn returned_tables_obey_the_table_laws((presentation, subgroup) in random_presentation()) {
        let limit = CosetLimit::new(2000);
        prop_assert!(limit.is_ok());
        if let Ok(limit) = limit {
            match enumerate(&presentation, &subgroup, limit) {
                Ok(table) => {
                    prop_assert_eq!(table.generator_count(), presentation.generator_count());
                    let laws = check_laws(&table, &presentation, &subgroup);
                    prop_assert_eq!(laws, Ok(()));
                }
                Err(error) => prop_assert_eq!(error.kind(), ErrorKind::CosetLimitReached),
            }
        }
    }

    #[test]
    fn word_laws_hold(letters in vec(letter(4), 0..=12)) {
        let word: Word = letters.into_iter().collect();
        let reduced = word.freely_reduced();
        prop_assert_eq!(reduced.freely_reduced(), reduced.clone());
        prop_assert!(reduced.letters().windows(2).all(|pair| pair[0].inverse() != pair[1]));
        let product: Word = word.letters().iter().chain(word.inverse().letters()).copied().collect();
        prop_assert!(product.freely_reduced().is_empty());
        let cyclic = word.cyclically_reduced();
        prop_assert_eq!(cyclic.freely_reduced(), cyclic.clone());
        if let (Some(first), Some(last)) = (cyclic.letters().first(), cyclic.letters().last()) {
            prop_assert!(cyclic.len() < 2 || first.inverse() != *last);
        }
        prop_assert_eq!(word.inverse().inverse(), word);
    }

    #[test]
    #[cfg_attr(miri, ignore = "the enumeration runs a loop too long for the interpreter")]
    fn enumerate_returns_without_a_panic(
        generators in 0_u8..=3,
        bases in vec((vec(letter(4), 1..=6), 1_u32..=3), 0..=4),
        subgroup in vec(vec(letter(4), 0..=5), 0..=3),
        cosets in 1_u32..=300,
    ) {
        let names = names(usize::from(generators));
        let relators = bases
            .into_iter()
            .filter_map(|(letters, exponent)| {
                let letters: Vec<Letter> = letters
                    .into_iter()
                    .filter(|letter| letter.index() < generators)
                    .collect();
                Relator::new(letters.into_iter().collect(), exponent).ok()
            })
            .collect();
        let presentation = Presentation::new(names, relators);
        prop_assert!(presentation.is_ok());
        let subgroup: Vec<Word> = subgroup.into_iter().map(|letters| letters.into_iter().collect()).collect();
        let limit = CosetLimit::new(cosets);
        if let (Ok(presentation), Ok(limit)) = (presentation, limit) {
            match enumerate(&presentation, &subgroup, limit) {
                Ok(table) => {
                    prop_assert!(table.index() >= 1);
                    prop_assert_eq!(check_laws(&table, &presentation, &subgroup), Ok(()));
                }
                Err(error) => prop_assert!(matches!(
                    error.kind(),
                    ErrorKind::CosetLimitReached | ErrorKind::InvalidWord
                )),
            }
        }
    }
}

fn invariance_corpus() -> Vec<(String, Presentation, Vec<Word>)> {
    let mut corpus: Vec<(String, Presentation, Vec<Word>)> = FIXTURES
        .iter()
        .map(|fixture| {
            (
                fixture.source.to_owned(),
                fixture.presentation(),
                fixture.subgroup(),
            )
        })
        .collect();
    for signature in spherical_signatures() {
        if signature
            .group_order()
            .is_some_and(|order| order.get() <= 24)
            && let Ok(presentation) = conway_presentation(&signature)
        {
            corpus.push((signature.to_string(), presentation, Vec::new()));
        }
    }
    corpus
}

fn rotated(relator: &Relator, by: usize, invert: bool) -> Relator {
    let letters = relator.base().letters();
    let shift = by % letters.len().max(1);
    let mut base: Word = letters[shift..]
        .iter()
        .chain(&letters[..shift])
        .copied()
        .collect();
    if invert {
        base = base.inverse();
    }
    match Relator::new(base, relator.exponent()) {
        Ok(relator) => relator,
        Err(error) => panic!("{error}"),
    }
}

fn shuffled<T: Clone>(items: &[T], keys: &[u32]) -> Vec<T> {
    let mut keyed: Vec<(u32, usize, T)> = items
        .iter()
        .enumerate()
        .map(|(position, item)| {
            (
                keys.get(position).copied().unwrap_or(0),
                position,
                item.clone(),
            )
        })
        .collect();
    keyed.sort_by_key(|&(key, position, _)| (key, position));
    keyed.into_iter().map(|(_, _, item)| item).collect()
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    #[cfg_attr(miri, ignore = "the enumeration runs a loop too long for the interpreter")]
    fn equal_subgroups_give_equal_tables(
        choice in any::<usize>(),
        relator_keys in vec(any::<u32>(), 16),
        target in any::<usize>(),
        by in any::<usize>(),
        invert in any::<bool>(),
        subgroup_keys in vec(any::<u32>(), 4),
    ) {
        let corpus = invariance_corpus();
        let (label, presentation, subgroup) = &corpus[choice % corpus.len()];
        let mut relators = shuffled(presentation.relators(), &relator_keys);
        if !relators.is_empty() {
            let position = target % relators.len();
            relators[position] = rotated(&relators[position], by, invert);
        }
        let variant = Presentation::new(presentation.generator_names().to_vec(), relators);
        prop_assert!(variant.is_ok());
        if let Ok(variant) = variant {
            let subgroup_variant = shuffled(subgroup, &subgroup_keys);
            let original = enumerate(presentation, subgroup, CosetLimit::DEFAULT);
            let changed = enumerate(&variant, &subgroup_variant, CosetLimit::DEFAULT);
            prop_assert!(original.is_ok(), "{}", label);
            prop_assert_eq!(original.ok(), changed.ok(), "{}", label);
        }
    }
}
