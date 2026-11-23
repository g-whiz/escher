use std::collections::VecDeque;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use escher_group::{
    Colouring, ColouringSet, Colourings, CosetLimit, CosetTable, ErrorKind, Letter, Presentation,
    Relator, SearchBudget, Word, colourings, colourings_of_index, conway_presentation,
    count_colourings, enumerate,
};
use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, any, prop_assert, prop_assert_eq};
use proptest::proptest;

use crate::common::{
    FIXTURES, SEARCH_BUDGET, budget, conway, coxeter, index_limit, letter, names,
    random_presentation, small_presentation,
};
use crate::search::{class_tables, free_filter_laws, published_presentations, relabelled, rows};
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
    #[test]
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
    #[test]
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

fn search(presentation: &Presentation, limit: u32, steps: u64) -> Colourings {
    search_set(presentation, ColouringSet::All, limit, steps)
}

fn search_set(
    presentation: &Presentation,
    set: ColouringSet,
    limit: u32,
    steps: u64,
) -> Colourings {
    match colourings(presentation, set, index_limit(limit), budget(steps)) {
        Ok(found) => found,
        Err(error) => panic!("the search fails: {error}"),
    }
}

fn of_count(found: &Colourings, colours: u32) -> Vec<Colouring> {
    found
        .list()
        .iter()
        .filter(|colouring| colouring.colours() == colours)
        .cloned()
        .collect()
}

fn class_laws(found: &Colourings, presentation: &Presentation, limit: u32) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for colouring in found.list() {
        check_laws(colouring.table(), presentation, &[])?;
        if colouring.colours() > limit
            || colouring.table().index()
                != usize::try_from(colouring.colours()).map_err(|error| error.to_string())?
        {
            return Err(format!("{} has the wrong number of rows", colouring.id()));
        }
        if !seen.insert(rows(colouring.table())) {
            return Err(format!("{} repeats a table", colouring.id()));
        }
    }
    for class in class_tables(found) {
        let own = rows(class.table());
        let bases = u32::try_from(own.len()).map_err(|error| error.to_string())?;
        let expansion: BTreeSet<Vec<Vec<u32>>> =
            (0..bases).map(|base| relabelled(&own, base)).collect();
        let members: BTreeSet<Vec<Vec<u32>>> = found
            .list()
            .iter()
            .filter(|colouring| colouring.class() == class.id())
            .map(|colouring| rows(colouring.table()))
            .collect();
        if expansion != members {
            return Err(format!(
                "the members of {} are not its relabellings",
                class.id()
            ));
        }
        if expansion.first() != Some(&own) {
            return Err(format!(
                "the class table {} is not the least of its class",
                class.id()
            ));
        }
    }
    Ok(())
}

fn appended_conjugate(presentation: &Presentation, target: usize, by: usize) -> Presentation {
    let mut relators = presentation.relators().to_vec();
    if !relators.is_empty() {
        let source = &relators[target % relators.len()];
        relators.push(rotated(source, by, false));
    }
    match Presentation::new(presentation.generator_names().to_vec(), relators) {
        Ok(presentation) => presentation,
        Err(error) => panic!("{error}"),
    }
}

proptest! {
    #[test]
    fn the_free_filter_keeps_exactly_the_class_tables_that_no_word_fixes(
        ((presentation, colours), words) in small_presentation().prop_flat_map(|(presentation, colours)| {
            let generators = u8::try_from(presentation.generator_count()).expect("a random presentation has few generators");
            (Just((presentation, colours)), vec(vec(letter(generators), 0..=4), 0..=3))
        }),
    ) {
        let words: Vec<Word> = words.into_iter().map(|letters| letters.into_iter().collect()).collect();
        let found = free_filter_laws(&presentation, &words, colours, SEARCH_BUDGET);
        prop_assert!(found.is_ok(), "{found:?}");
    }

    #[test]
    fn classes_are_least_and_hold_their_relabellings((presentation, limit) in small_presentation()) {
        let found = search(&presentation, limit, SEARCH_BUDGET);
        prop_assert_eq!(class_laws(&found, &presentation, limit), Ok(()));
    }

    #[test]
    fn equal_presentations_give_equal_colourings(
        (presentation, limit) in small_presentation(),
        relator_keys in vec(any::<u32>(), 4),
        target in any::<usize>(),
        by in any::<usize>(),
        invert in any::<bool>(),
        append in any::<bool>(),
    ) {
        let variant = if append {
            appended_conjugate(&presentation, target, by)
        } else {
            let mut relators = shuffled(presentation.relators(), &relator_keys);
            let position = target % relators.len();
            relators[position] = rotated(&relators[position], by, invert);
            match Presentation::new(presentation.generator_names().to_vec(), relators) {
                Ok(variant) => variant,
                Err(error) => panic!("{error}"),
            }
        };
        let original = search(&presentation, limit, SEARCH_BUDGET);
        let changed = search(&variant, limit, SEARCH_BUDGET);
        if original.complete_to() == limit && changed.complete_to() == limit {
            prop_assert_eq!(original.list(), changed.list());
        }
    }

    #[test]
    fn budgets_and_limits_change_only_how_far_a_call_is_complete(
        (presentation, limit) in small_presentation(),
        other_limit in 1_u32..=5,
        smaller in 1_u64..=20_000,
        extra in 0_u64..=200_000,
    ) {
        let low = search(&presentation, limit, smaller);
        let high = search(&presentation, limit, smaller + extra);
        prop_assert!(high.complete_to() >= low.complete_to());
        let other = search(&presentation, other_limit, smaller + extra);
        for (left, right) in [(&low, &high), (&low, &other), (&high, &other)] {
            for colours in 1..=left.complete_to().min(right.complete_to()) {
                prop_assert_eq!(of_count(left, colours), of_count(right, colours));
                prop_assert_eq!(left.count(colours), u64::try_from(of_count(left, colours).len()).ok());
            }
            let above = |found: &Colourings| found.list().iter().all(|colouring| colouring.colours() <= found.complete_to());
            prop_assert!(above(left) && above(right));
        }
    }

    #[test]
    fn counts_and_single_colour_counts_agree_with_the_list((presentation, limit) in small_presentation()) {
        let found = search(&presentation, limit, SEARCH_BUDGET);
        let counts = count_colourings(&presentation, ColouringSet::All, index_limit(limit), budget(SEARCH_BUDGET));
        prop_assert!(counts.is_ok());
        if let Ok(counts) = counts
            && found.complete_to() == limit
            && counts.complete_to() == limit
        {
            for colours in 1..=limit {
                let listed = of_count(&found, colours);
                let classes = listed.iter().filter(|colouring| colouring.class() == colouring.id()).count();
                prop_assert_eq!(counts.colourings(colours), u64::try_from(listed.len()).ok());
                prop_assert_eq!(found.count(colours), u64::try_from(listed.len()).ok());
                prop_assert_eq!(counts.classes(colours), u64::try_from(classes).ok());
                let single = colourings_of_index(&presentation, ColouringSet::All, index_limit(colours), budget(SEARCH_BUDGET));
                prop_assert!(single.is_ok());
                if let Ok(single) = single
                    && single.is_complete()
                {
                    prop_assert_eq!(single.list(), listed.as_slice());
                }
            }
            prop_assert_eq!(counts.colourings(limit + 1), None);
            prop_assert_eq!(counts.colourings(0), None);
            prop_assert_eq!(found.count(limit + 1), None);
            prop_assert_eq!(found.count(0), None);
        }
    }
}

fn relator_sides(presentation: &Presentation) -> Vec<Vec<u8>> {
    presentation
        .relators()
        .iter()
        .filter(|relator| {
            relator
                .base()
                .letters()
                .iter()
                .all(|letter| !letter.is_inverse())
        })
        .map(|relator| {
            (0..relator.exponent())
                .flat_map(|_| relator.base().letters().iter().map(|letter| letter.index()))
                .collect()
        })
        .collect()
}

static WORD_CASES: LazyLock<Vec<(Presentation, Colourings)>> = LazyLock::new(word_cases);

fn word_cases() -> Vec<(Presentation, Colourings)> {
    let conway_2222 = conway("2222");
    let reflections = coxeter(6, 4, 2);
    let reflection_colourings = search(&reflections, 8, u64::from(u32::MAX));
    let rotation_colourings = search(&conway_2222, 6, u64::from(u32::MAX));
    vec![
        (reflections, reflection_colourings),
        (conway_2222, rotation_colourings),
    ]
}

proptest! {
    #[test]
    fn a_relator_inside_a_side_sequence_keeps_the_colour(
        choice in any::<usize>(),
        first in vec(0_u8..4, 0..=6),
        last in vec(0_u8..4, 0..=6),
    ) {
        for (presentation, found) in WORD_CASES.iter() {
            let generators = u8::try_from(presentation.generator_count()).unwrap_or(u8::MAX);
            let first: Vec<u8> = first.iter().map(|side| side % generators).collect();
            let last: Vec<u8> = last.iter().map(|side| side % generators).collect();
            prop_assert!(!found.list().is_empty());
            let colouring = &found.list()[choice % found.list().len()];
            let plain: Vec<u8> = first.iter().chain(&last).copied().collect();
            for relator in relator_sides(presentation) {
                let padded: Vec<u8> = first.iter().chain(&relator).chain(&last).copied().collect();
                prop_assert_eq!(colouring.colour_of(&padded), colouring.colour_of(&plain), "{}", colouring.id());
            }
        }
    }
}

#[test]
fn search_budgets_are_built_from_steps() {
    assert_eq!(budget(5).get(), 5);
    assert_eq!(SearchBudget::default().get(), 1 << 28);
}

fn is_valid(rows: &[Vec<u32>]) -> bool {
    rows.first()
        .is_some_and(|row| row.iter().step_by(2).all(|&entry| entry != 0))
}

fn valid_classes(
    all: &Colourings,
    colours: u32,
    by_rows: &BTreeMap<Vec<Vec<u32>>, &Colouring>,
) -> Result<u64, String> {
    let mut classes = 0_u64;
    for class in class_tables(all)
        .into_iter()
        .filter(|class| class.colours() == colours)
    {
        let own = rows(class.table());
        let bases = u32::try_from(own.len()).map_err(|error| error.to_string())?;
        let Some(first) = (0..bases)
            .map(|base| relabelled(&own, base))
            .find(|table| is_valid(table))
        else {
            continue;
        };
        classes += 1;
        let named = by_rows
            .get(&first)
            .ok_or_else(|| format!("the first valid conjugate of {} is missing", class.id()))?
            .id();
        for member in all
            .list()
            .iter()
            .filter(|member| member.class() == class.id() && member.contained_generator().is_none())
        {
            let kept = by_rows
                .get(&rows(member.table()))
                .ok_or_else(|| format!("the valid colouring {} is missing", member.id()))?;
            if kept.class() != named {
                return Err(format!(
                    "{} has the class {} and not {named}",
                    kept.id(),
                    kept.class()
                ));
            }
        }
    }
    Ok(classes)
}

fn valid_laws(presentation: &Presentation, limit: u32, steps: u64) -> Result<u32, String> {
    let all = search_set(presentation, ColouringSet::All, limit, steps);
    let valid = search_set(presentation, ColouringSet::Valid, limit, steps);
    let counts = count_colourings(
        presentation,
        ColouringSet::Valid,
        index_limit(limit),
        budget(steps),
    )
    .map_err(|error| error.to_string())?;
    if valid
        .list()
        .iter()
        .any(|colouring| colouring.colours() > valid.complete_to())
    {
        return Err("a valid colouring lies above complete_to()".to_owned());
    }
    let by_rows: BTreeMap<Vec<Vec<u32>>, &Colouring> = valid
        .list()
        .iter()
        .map(|colouring| (rows(colouring.table()), colouring))
        .collect();
    let checked = all.complete_to().min(valid.complete_to());
    for colours in 1..=checked {
        let listed = of_count(&valid, colours);
        let kept: Vec<CosetTable> = of_count(&all, colours)
            .into_iter()
            .filter(|colouring| colouring.contained_generator().is_none())
            .map(|colouring| colouring.table().clone())
            .collect();
        let tables: Vec<CosetTable> = listed
            .iter()
            .map(|colouring| colouring.table().clone())
            .collect();
        if tables != kept {
            return Err(format!(
                "the valid tables of {colours} colours are not the filtered ones"
            ));
        }
        ranked_from_a(&listed, colours)?;
        if valid.count(colours) != u64::try_from(listed.len()).ok() {
            return Err(format!(
                "the count of {colours} colours differs from the list"
            ));
        }
        let classes = valid_classes(&all, colours, &by_rows)?;
        let named = listed
            .iter()
            .filter(|colouring| colouring.class() == colouring.id())
            .count();
        if u64::try_from(named).ok() != Some(classes) {
            return Err(format!(
                "{colours} colours name {named} classes, not {classes}"
            ));
        }
        if colours <= counts.complete_to()
            && (counts.colourings(colours) != u64::try_from(listed.len()).ok()
                || counts.classes(colours) != Some(classes))
        {
            return Err(format!(
                "the counts of {colours} colours differ from the list"
            ));
        }
        let single = colourings_of_index(
            presentation,
            ColouringSet::Valid,
            index_limit(colours),
            budget(steps),
        )
        .map_err(|error| error.to_string())?;
        if single.is_complete() && single.list() != listed.as_slice() {
            return Err(format!(
                "the single search of {colours} colours differs from the list"
            ));
        }
    }
    Ok(checked)
}

fn ranked_from_a(listed: &[Colouring], colours: u32) -> Result<(), String> {
    for (rank, colouring) in listed.iter().enumerate() {
        let ordinal = u32::try_from(rank + 1).map_err(|error| error.to_string())?;
        if colouring.id().colours() != colours || colouring.id().ordinal() != ordinal {
            return Err(format!("{} is not ranked {ordinal}", colouring.id()));
        }
    }
    Ok(())
}

fn valid_class_laws(presentation: &Presentation, limit: u32, steps: u64) -> Result<u32, String> {
    let valid = search_set(presentation, ColouringSet::Valid, limit, steps);
    let classes = search_set(presentation, ColouringSet::ValidClasses, limit, steps);
    let counts = count_colourings(
        presentation,
        ColouringSet::ValidClasses,
        index_limit(limit),
        budget(steps),
    )
    .map_err(|error| error.to_string())?;
    if classes
        .list()
        .iter()
        .any(|colouring| colouring.colours() > classes.complete_to())
    {
        return Err("a class colouring lies above complete_to()".to_owned());
    }
    let checked = valid.complete_to().min(classes.complete_to());
    for colours in 1..=checked {
        let listed = of_count(&classes, colours);
        let named: Vec<CosetTable> = of_count(&valid, colours)
            .into_iter()
            .filter(|colouring| colouring.class() == colouring.id())
            .map(|colouring| colouring.table().clone())
            .collect();
        let tables: Vec<CosetTable> = listed
            .iter()
            .map(|colouring| colouring.table().clone())
            .collect();
        if tables != named {
            return Err(format!(
                "the class tables of {colours} colours are not those that name a valid class"
            ));
        }
        ranked_from_a(&listed, colours)?;
        if let Some(colouring) = listed
            .iter()
            .find(|colouring| colouring.class() != colouring.id())
        {
            return Err(format!(
                "{} has the class {}",
                colouring.id(),
                colouring.class()
            ));
        }
        let total = u64::try_from(listed.len()).ok();
        if classes.count(colours) != total {
            return Err(format!(
                "the count of {colours} colours differs from the list"
            ));
        }
        if colours <= counts.complete_to()
            && (counts.colourings(colours) != total || counts.classes(colours) != total)
        {
            return Err(format!(
                "the counts of {colours} colours differ from the list"
            ));
        }
        let single = colourings_of_index(
            presentation,
            ColouringSet::ValidClasses,
            index_limit(colours),
            budget(steps),
        )
        .map_err(|error| error.to_string())?;
        if single.is_complete() && single.list() != listed.as_slice() {
            return Err(format!(
                "the single search of {colours} colours differs from the list"
            ));
        }
    }
    Ok(checked)
}

fn valid_two_colourings(presentation: &Presentation) -> Result<usize, String> {
    let found = colourings_of_index(
        presentation,
        ColouringSet::Valid,
        index_limit(2),
        SearchBudget::DEFAULT,
    )
    .map_err(|error| error.to_string())?;
    if !found.is_complete() {
        return Err("the search of 2 colours stopped at a limit".to_owned());
    }
    Ok(found.list().len())
}

proptest! {
    #[test]
    fn valid_colourings_are_the_filtered_colourings_of_all((presentation, limit) in small_presentation()) {
        let checked = valid_laws(&presentation, limit, SEARCH_BUDGET);
        prop_assert!(checked.is_ok(), "{:?}", checked);
    }

    #[test]
    fn valid_classes_keep_the_colouring_that_names_each_valid_class((presentation, limit) in small_presentation()) {
        let checked = valid_class_laws(&presentation, limit, SEARCH_BUDGET);
        prop_assert!(checked.is_ok(), "{:?}", checked);
    }

    #[test]
    fn a_presentation_has_at_most_one_valid_two_colouring((presentation, _) in small_presentation()) {
        let count = valid_two_colourings(&presentation);
        prop_assert!(count.as_ref().is_ok_and(|&count| count <= 1), "{:?}", count);
    }
}

#[test]
fn the_published_presentations_filter_their_valid_colourings() {
    for (label, presentation) in published_presentations() {
        assert_eq!(
            valid_laws(&presentation, 10, SearchBudget::DEFAULT.get()),
            Ok(10),
            "{label}"
        );
    }
}

#[test]
fn the_published_presentations_keep_one_colouring_for_each_valid_class() {
    for (label, presentation) in published_presentations() {
        assert_eq!(
            valid_class_laws(&presentation, 10, SearchBudget::DEFAULT.get()),
            Ok(10),
            "{label}"
        );
    }
}

#[test]
fn the_published_presentations_have_at_most_one_valid_two_colouring() {
    for (label, presentation) in published_presentations() {
        let count = valid_two_colourings(&presentation);
        assert!(
            count.as_ref().is_ok_and(|&count| count <= 1),
            "{label} {count:?}"
        );
    }
}
