use std::collections::{BTreeMap, BTreeSet, VecDeque};

use escher_group::{
    Colouring, Colourings, CosetLimit, CosetTable, Letter, Presentation, SearchBudget, Word,
    colourings, count_colourings, enumerate,
};

use crate::common::{
    Expected, SEARCH_FIXTURES, SearchFixture, Table, conway, coxeter, index_limit, presentation,
    word,
};

pub(crate) fn listing(presentation: &Presentation, limit: u32) -> Colourings {
    match colourings(presentation, index_limit(limit), SearchBudget::DEFAULT) {
        Ok(found) => found,
        Err(error) => panic!("the search fails: {error}"),
    }
}

pub(crate) fn rows(table: &CosetTable) -> Vec<Vec<u32>> {
    (0..u32::try_from(table.index()).expect("a small index"))
        .map(|coset| {
            table
                .row(coset)
                .expect("the coset is below the index")
                .to_vec()
        })
        .collect()
}

fn printed(table: Table) -> Vec<Vec<u32>> {
    table.iter().map(|row| row.to_vec()).collect()
}

pub(crate) fn relabelled(rows: &[Vec<u32>], base: u32) -> Vec<Vec<u32>> {
    let count = rows.len();
    let mut label = vec![None; count];
    let mut order = VecDeque::new();
    let mut seen = Vec::new();
    label[usize::try_from(base).expect("small")] = Some(0_u32);
    order.push_back(base);
    while let Some(old) = order.pop_front() {
        seen.push(old);
        for &entry in &rows[usize::try_from(old).expect("small")] {
            let slot = &mut label[usize::try_from(entry).expect("small")];
            if slot.is_none() {
                *slot = Some(u32::try_from(seen.len() + order.len()).expect("small"));
                order.push_back(entry);
            }
        }
    }
    seen.iter()
        .map(|&old| {
            rows[usize::try_from(old).expect("small")]
                .iter()
                .map(|&entry| label[usize::try_from(entry).expect("small")].expect("labelled"))
                .collect()
        })
        .collect()
}

pub(crate) fn class_tables(found: &Colourings) -> Vec<&Colouring> {
    found
        .list()
        .iter()
        .filter(|colouring| colouring.class() == colouring.id())
        .collect()
}

fn fixes_a_coset(table: &CosetTable, words: &[Word]) -> bool {
    (0..u32::try_from(table.index()).expect("small")).any(|coset| {
        words
            .iter()
            .all(|word| table.trace(coset, word) == Some(coset))
    })
}

fn check(fixture: &SearchFixture) {
    let found = listing(&fixture.presentation(), fixture.limit);
    let source = fixture.source;
    assert_eq!(found.complete_to(), fixture.limit, "{source}");
    let classes = class_tables(&found);
    match fixture.expected {
        Expected::ClassTables(_) => {
            let mut wanted: Vec<Vec<Vec<u32>>> = fixture
                .tables()
                .iter()
                .map(|&table| printed(table))
                .collect();
            wanted.sort_by(|left, right| (left.len(), left).cmp(&(right.len(), right)));
            let found: Vec<Vec<Vec<u32>>> = classes
                .iter()
                .map(|colouring| rows(colouring.table()))
                .collect();
            assert_eq!(found, wanted, "{source}");
        }
        Expected::Subgroups { counts, tables } => {
            for (colours, &count) in (1..).zip(counts) {
                let members = found
                    .list()
                    .iter()
                    .filter(|colouring| colouring.colours() == colours);
                assert_eq!(
                    u64::try_from(members.count()).expect("small"),
                    count,
                    "{source} {colours}"
                );
            }
            for colours in 1..=fixture.limit {
                let in_order: Vec<Vec<Vec<u32>>> = found
                    .list()
                    .iter()
                    .filter(|colouring| colouring.colours() == colours)
                    .map(|colouring| rows(colouring.table()))
                    .collect();
                let printed_in_order: Vec<Vec<Vec<u32>>> = tables
                    .iter()
                    .filter(|table| table.len() == usize::try_from(colours).expect("small"))
                    .map(|&table| printed(table))
                    .collect();
                if !printed_in_order.is_empty() {
                    assert_eq!(in_order, printed_in_order, "{source} {colours}");
                }
            }
        }
        Expected::ClassCounts(counts) => {
            for (colours, &count) in (1..).zip(counts) {
                let classes = classes
                    .iter()
                    .filter(|colouring| colouring.colours() == colours);
                assert_eq!(
                    u64::try_from(classes.count()).expect("small"),
                    count,
                    "{source} {colours}"
                );
            }
        }
        Expected::ClassTotal(total) => {
            assert_eq!(
                u64::try_from(classes.len()).expect("small"),
                total,
                "{source}"
            );
        }
        Expected::FixedClasses {
            word: text,
            counts,
            tables,
        } => check_fixed_classes(fixture, &found, &classes, (text, counts, tables)),
    }
}

fn check_fixed_classes(
    fixture: &SearchFixture,
    found: &Colourings,
    classes: &[&Colouring],
    (text, counts, tables): (&str, &[u64], &[Table]),
) {
    let source = fixture.source;
    let element = [word(fixture.generators, text)];
    let fixed: Vec<&&Colouring> = classes
        .iter()
        .filter(|colouring| fixes_a_coset(colouring.table(), &element))
        .collect();
    for (colours, &count) in (1..).zip(counts) {
        let of_count = fixed
            .iter()
            .filter(|colouring| colouring.colours() == colours);
        assert_eq!(
            u64::try_from(of_count.count()).expect("small"),
            count,
            "{source} {colours}"
        );
    }
    let mut classes_of_printed = BTreeSet::new();
    for &table in tables {
        let member = found
            .list()
            .iter()
            .find(|colouring| rows(colouring.table()) == printed(table))
            .unwrap_or_else(|| panic!("{source}: a printed table is no colouring"));
        assert!(
            fixed.iter().any(|class| class.id() == member.class()),
            "{source}: a printed table is in no fixed class"
        );
        classes_of_printed.insert(member.class());
    }
    assert_eq!(classes_of_printed.len(), tables.len(), "{source}");
}

fn fixture(source: &str) -> &'static SearchFixture {
    SEARCH_FIXTURES
        .iter()
        .find(|fixture| fixture.source == source)
        .unwrap_or_else(|| panic!("no search fixture {source}"))
}

#[test]
fn search_examples_give_the_printed_tables_and_counts() {
    for fixture in SEARCH_FIXTURES {
        check(fixture);
    }
}

#[test]
fn the_symmetric_group_of_order_six_has_three_colourings_with_three_colours() {
    let symmetric = presentation("xy", &[("x", 2), ("y", 3), ("xy", 2)]);
    let found = listing(&symmetric, 3);
    let counts: Vec<usize> = (1..=3)
        .map(|colours| {
            found
                .list()
                .iter()
                .filter(|colouring| colouring.colours() == colours)
                .count()
        })
        .collect();
    assert_eq!(counts, [1, 1, 3]);
    assert_eq!(class_tables(&found).len(), 3);
}

#[test]
fn the_classes_of_the_free_product_follow_sims() {
    check(fixture("Sims p. 209"));
}

#[test]
fn the_classes_of_the_symmetric_group_of_order_24_follow_holt() {
    check(fixture("Holt Example 5.10 and the SymPy doctest"));
}

fn classes_fixing(found: &Colourings, words: &[Word]) -> BTreeMap<u32, u64> {
    let mut counts = BTreeMap::new();
    for colouring in class_tables(found) {
        if fixes_a_coset(colouring.table(), words) {
            *counts.entry(colouring.colours()).or_insert(0) += 1;
        }
    }
    counts
}

fn generator_words(indices: &[&[u8]]) -> Vec<Word> {
    indices
        .iter()
        .map(|letters| {
            letters
                .iter()
                .map(|&index| Letter::generator(index))
                .collect()
        })
        .collect()
}

fn assert_row(counts: &BTreeMap<u32, u64>, printed: &[(u32, u64)], up_to: u32, label: &str) {
    let last = printed.last().map_or(0, |&(colours, _)| colours);
    let bound = last.min(up_to);
    let wanted: BTreeMap<u32, u64> = printed
        .iter()
        .copied()
        .filter(|&(colours, _)| colours <= bound)
        .collect();
    let found: BTreeMap<u32, u64> = counts
        .iter()
        .map(|(&colours, &count)| (colours, count))
        .filter(|&(colours, _)| colours <= bound)
        .collect();
    assert_eq!(found, wanted, "{label} up to {bound}");
}

struct FrettlohRow {
    tiling: &'static str,
    sides: &'static [&'static [u8]],
    values: &'static [(u32, u64)],
}

struct FrettlohCase {
    signature: &'static str,
    limit: u32,
    rows: &'static [FrettlohRow],
}

const TABLE_1: &[FrettlohCase] = &[
    FrettlohCase {
        signature: "*732",
        limit: 50,
        rows: &[
            FrettlohRow {
                tiling: "(7^3)",
                sides: &[&[0], &[1]],
                values: &[
                    (1, 1),
                    (8, 1),
                    (15, 1),
                    (22, 1),
                    (24, 1),
                    (30, 1),
                    (36, 2),
                    (44, 1),
                    (50, 5),
                ],
            },
            FrettlohRow {
                tiling: "[3.7.3.7]",
                sides: &[&[2], &[0]],
                values: &[
                    (1, 1),
                    (9, 1),
                    (15, 1),
                    (21, 1),
                    (28, 2),
                    (30, 1),
                    (35, 2),
                    (36, 1),
                    (37, 1),
                    (42, 5),
                    (49, 8),
                    (50, 3),
                ],
            },
            FrettlohRow {
                tiling: "(3^7)",
                sides: &[&[1], &[2]],
                values: &[
                    (1, 1),
                    (22, 1),
                    (28, 5),
                    (37, 1),
                    (42, 4),
                    (44, 1),
                    (49, 7),
                    (50, 3),
                ],
            },
        ],
    },
    FrettlohCase {
        signature: "*832",
        limit: 39,
        rows: &[
            FrettlohRow {
                tiling: "(8^3)",
                sides: &[&[0], &[1]],
                values: &[
                    (1, 1),
                    (3, 1),
                    (6, 1),
                    (12, 1),
                    (17, 1),
                    (21, 4),
                    (24, 1),
                    (25, 5),
                    (27, 3),
                    (29, 4),
                    (31, 4),
                    (33, 6),
                    (37, 6),
                    (39, 8),
                ],
            },
            FrettlohRow {
                tiling: "[3.8.3.8]",
                sides: &[&[2], &[0]],
                values: &[
                    (1, 1),
                    (3, 1),
                    (6, 1),
                    (12, 3),
                    (17, 1),
                    (18, 1),
                    (21, 4),
                    (24, 15),
                    (25, 5),
                    (27, 3),
                    (28, 4),
                    (29, 4),
                    (30, 7),
                ],
            },
            FrettlohRow {
                tiling: "(3^8)",
                sides: &[&[1], &[2]],
                values: &[
                    (1, 1),
                    (2, 1),
                    (4, 1),
                    (8, 1),
                    (10, 2),
                    (12, 1),
                    (14, 1),
                    (16, 2),
                    (18, 1),
                    (20, 4),
                    (24, 3),
                    (25, 5),
                    (26, 1),
                    (28, 12),
                    (29, 1),
                    (30, 2),
                ],
            },
        ],
    },
    FrettlohCase {
        signature: "*542",
        limit: 26,
        rows: &[
            FrettlohRow {
                tiling: "(5^4)",
                sides: &[&[0], &[1]],
                values: &[
                    (1, 1),
                    (2, 1),
                    (6, 1),
                    (11, 1),
                    (12, 1),
                    (16, 2),
                    (21, 3),
                    (22, 5),
                    (24, 1),
                    (26, 9),
                    (28, 1),
                ],
            },
            FrettlohRow {
                tiling: "[4.5.4.5]",
                sides: &[&[2], &[0]],
                values: &[
                    (1, 1),
                    (5, 2),
                    (10, 5),
                    (11, 1),
                    (15, 7),
                    (20, 22),
                    (21, 3),
                    (22, 4),
                    (25, 27),
                    (26, 4),
                    (27, 3),
                    (30, 63),
                ],
            },
            FrettlohRow {
                tiling: "(4^5)",
                sides: &[&[1], &[2]],
                values: &[
                    (1, 1),
                    (5, 2),
                    (10, 4),
                    (11, 1),
                    (15, 7),
                    (16, 1),
                    (20, 9),
                    (21, 3),
                    (22, 1),
                    (25, 27),
                    (26, 1),
                    (27, 3),
                    (30, 38),
                ],
            },
        ],
    },
];

const TABLE_2: &[FrettlohCase] = &[
    FrettlohCase {
        signature: "732",
        limit: 24,
        rows: &[
            FrettlohRow {
                tiling: "(7^3)",
                sides: &[&[1]],
                values: &[(1, 1), (8, 1), (9, 1), (15, 2), (22, 7), (24, 1)],
            },
            FrettlohRow {
                tiling: "[3.7.3.7]",
                sides: &[&[2, 0]],
                values: &[(1, 1), (7, 1), (9, 1), (14, 6), (15, 2), (21, 5), (22, 7)],
            },
            FrettlohRow {
                tiling: "(3^7)",
                sides: &[&[3]],
                values: &[(1, 1), (7, 1), (8, 1), (14, 6), (21, 2), (22, 7)],
            },
        ],
    },
    FrettlohCase {
        signature: "832",
        limit: 19,
        rows: &[
            FrettlohRow {
                tiling: "(8^3)",
                sides: &[&[1]],
                values: &[
                    (1, 1),
                    (3, 1),
                    (6, 1),
                    (9, 1),
                    (10, 1),
                    (12, 1),
                    (13, 2),
                    (15, 1),
                    (17, 5),
                    (18, 5),
                    (19, 5),
                ],
            },
            FrettlohRow {
                tiling: "[3.8.3.8]",
                sides: &[&[2, 0]],
                values: &[
                    (1, 1),
                    (3, 1),
                    (4, 1),
                    (6, 1),
                    (8, 3),
                    (9, 1),
                    (10, 2),
                    (12, 4),
                    (13, 2),
                    (15, 1),
                    (16, 9),
                    (17, 5),
                    (18, 12),
                ],
            },
            FrettlohRow {
                tiling: "(3^8)",
                sides: &[&[3]],
                values: &[
                    (1, 1),
                    (2, 1),
                    (4, 1),
                    (8, 4),
                    (10, 3),
                    (12, 1),
                    (13, 2),
                    (14, 2),
                    (16, 12),
                    (17, 5),
                    (18, 1),
                    (19, 5),
                ],
            },
        ],
    },
    FrettlohCase {
        signature: "542",
        limit: 17,
        rows: &[
            FrettlohRow {
                tiling: "(5^4)",
                sides: &[&[1]],
                values: &[(1, 1), (2, 1), (6, 2), (11, 3), (12, 6), (16, 12), (17, 4)],
            },
            FrettlohRow {
                tiling: "[4.5.4.5]",
                sides: &[&[2, 0]],
                values: &[
                    (1, 1),
                    (5, 2),
                    (6, 1),
                    (10, 9),
                    (11, 3),
                    (12, 4),
                    (15, 15),
                    (16, 10),
                    (17, 4),
                ],
            },
            FrettlohRow {
                tiling: "(4^5)",
                sides: &[&[3]],
                values: &[
                    (1, 1),
                    (5, 2),
                    (6, 1),
                    (10, 6),
                    (11, 3),
                    (15, 15),
                    (16, 2),
                    (17, 4),
                ],
            },
        ],
    },
];

fn triangle(text: &str) -> (u32, u32) {
    let orders: Vec<u32> = text
        .trim_start_matches('*')
        .chars()
        .map(|digit| digit.to_digit(10).expect("a digit"))
        .collect();
    (orders[0], orders[1])
}

fn rotation_sides(p: u32, q: u32) -> Presentation {
    presentation(
        "abcd",
        &[("ab", 1), ("cd", 1), ("b", p), ("ca", 2), ("d", q)],
    )
}

#[test]
fn frettloh_table_1_counts_the_classes_with_a_fixed_coset() {
    for case in TABLE_1 {
        let (p, q) = triangle(case.signature);
        let found = listing(&coxeter(p, q, 2), case.limit);
        assert_eq!(found.complete_to(), case.limit, "{}", case.signature);
        for row in case.rows {
            let counts = classes_fixing(&found, &generator_words(row.sides));
            let label = format!("{} {}", case.signature, row.tiling);
            assert_row(&counts, row.values, case.limit, &label);
        }
    }
}

fn mirror_class(table: &CosetTable) -> Vec<Vec<u32>> {
    let mirrored: Vec<Vec<u32>> = rows(table)
        .into_iter()
        .map(|row| {
            row.chunks(2)
                .flat_map(|pair| pair.iter().rev().copied())
                .collect()
        })
        .collect();
    let bases = u32::try_from(mirrored.len()).expect("small");
    (0..bases)
        .map(|base| relabelled(&mirrored, base))
        .min()
        .expect("a table has a row")
}

#[test]
fn frettloh_table_2_counts_the_classes_with_a_fixed_coset_up_to_the_mirror() {
    for case in TABLE_2 {
        let (p, q) = triangle(case.signature);
        let found = listing(&rotation_sides(p, q), case.limit);
        assert_eq!(found.complete_to(), case.limit, "{}", case.signature);
        for row in case.rows {
            let words = generator_words(row.sides);
            let mut seen: BTreeSet<Vec<Vec<u32>>> = BTreeSet::new();
            let mut counts = BTreeMap::new();
            for colouring in class_tables(&found) {
                if !fixes_a_coset(colouring.table(), &words) {
                    continue;
                }
                let own = rows(colouring.table());
                if seen.contains(&own) {
                    continue;
                }
                seen.insert(own);
                seen.insert(mirror_class(colouring.table()));
                *counts.entry(colouring.colours()).or_insert(0) += 1;
            }
            let label = format!("{} {}", case.signature, row.tiling);
            assert_row(&counts, row.values, case.limit, &label);
        }
    }
}

#[test]
fn the_cube_has_one_perfect_colouring_with_1_3_and_6_colours() {
    let found = listing(&coxeter(4, 3, 2), 48);
    assert_eq!(found.complete_to(), 48);
    let counts = classes_fixing(&found, &generator_words(&[&[0], &[1]]));
    assert_eq!(counts, BTreeMap::from([(1, 1), (3, 1), (6, 1)]));
}

#[test]
fn triangles_with_even_orders_have_seven_two_colourings() {
    for (p, q) in [(6, 4), (8, 4), (6, 6)] {
        let counts = count_colourings(&coxeter(p, q, 2), index_limit(2), SearchBudget::DEFAULT)
            .expect("the search runs");
        assert_eq!(counts.colourings(2), Some(7), "*{p}{q}2");
        assert_eq!(counts.classes(2), Some(7), "*{p}{q}2");
    }
}

#[test]
fn bulatov_colour_counts_exist() {
    let cases: [(&str, Presentation, &[u32]); 4] = [
        ("*642", coxeter(6, 4, 2), &[3, 8, 12]),
        ("642", conway("642"), &[7]),
        ("433", conway("433"), &[3, 4]),
        ("*542", coxeter(5, 4, 2), &[2, 4, 5, 6, 10, 11, 12, 15, 16]),
    ];
    for (label, presentation, shown) in cases {
        let largest = shown.iter().copied().max().expect("a count");
        let counts = count_colourings(&presentation, index_limit(largest), SearchBudget::DEFAULT)
            .expect("the search runs");
        assert_eq!(counts.complete_to(), largest, "{label}");
        for &colours in shown {
            assert!(
                counts.colourings(colours).is_some_and(|count| count > 0),
                "{label} {colours}"
            );
        }
    }
}

fn schreier_generators(table: &CosetTable) -> Vec<Word> {
    let cosets = u32::try_from(table.index()).expect("small");
    let generators = u8::try_from(table.generator_count()).expect("few generators");
    let mut path: Vec<Option<Vec<Letter>>> = vec![None; table.index()];
    path[0] = Some(Vec::new());
    let mut queue = VecDeque::from([0_u32]);
    while let Some(coset) = queue.pop_front() {
        for index in 0..generators {
            for letter in [Letter::generator(index), Letter::inverse_of(index)] {
                let image = table.image(coset, letter).expect("a complete table");
                let slot = usize::try_from(image).expect("small");
                if path[slot].is_none() {
                    let mut longer = path[usize::try_from(coset).expect("small")]
                        .clone()
                        .expect("reached");
                    longer.push(letter);
                    path[slot] = Some(longer);
                    queue.push_back(image);
                }
            }
        }
    }
    let mut words = Vec::new();
    for coset in 0..cosets {
        for index in 0..generators {
            let letter = Letter::generator(index);
            let image = table.image(coset, letter).expect("a complete table");
            let mut letters = path[usize::try_from(coset).expect("small")]
                .clone()
                .expect("reached");
            letters.push(letter);
            let back: Word = path[usize::try_from(image).expect("small")]
                .clone()
                .expect("reached")
                .into_iter()
                .collect();
            letters.extend(back.inverse().letters());
            let word: Word = letters.into_iter().collect::<Word>().freely_reduced();
            if !word.is_empty() {
                words.push(word);
            }
        }
    }
    words
}

#[test]
fn the_search_agrees_with_coset_enumeration() {
    for (label, presentation, limit) in
        [("*642", coxeter(6, 4, 2), 10), ("2222", conway("2222"), 8)]
    {
        let found = listing(&presentation, limit);
        assert_eq!(found.complete_to(), limit, "{label}");
        for colouring in found.list() {
            let subgroup = schreier_generators(colouring.table());
            let table = enumerate(&presentation, &subgroup, CosetLimit::MAX)
                .unwrap_or_else(|error| panic!("{label} {}: {error}", colouring.id()));
            assert_eq!(&table, colouring.table(), "{label} {}", colouring.id());
        }
    }
}
