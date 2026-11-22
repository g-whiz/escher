use escher_signature::Signature;
use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, any};

use crate::{
    CosetLimit, IndexLimit, Letter, Presentation, Relator, SearchBudget, Word, conway_presentation,
};

pub(crate) fn names(count: usize) -> Vec<String> {
    (0..count).map(|index| format!("g{index}")).collect()
}

pub(crate) fn letters(letters: &[Letter]) -> Word {
    letters.iter().copied().collect()
}

pub(crate) fn signature(text: &str) -> Signature {
    match text.parse() {
        Ok(signature) => signature,
        Err(error) => panic!("{text} is rejected: {error}"),
    }
}

pub(crate) fn conway(text: &str) -> Presentation {
    match conway_presentation(&signature(text)) {
        Ok(presentation) => presentation,
        Err(error) => panic!("{text} has no presentation: {error}"),
    }
}

pub(crate) fn limit(cosets: u32) -> CosetLimit {
    match CosetLimit::new(cosets) {
        Ok(limit) => limit,
        Err(error) => panic!("{cosets} is not a coset limit: {error}"),
    }
}

pub(crate) fn letter(generators: u8) -> impl Strategy<Value = Letter> + Clone {
    (0..generators, any::<bool>()).prop_map(|(index, inverse)| {
        if inverse {
            Letter::inverse_of(index)
        } else {
            Letter::generator(index)
        }
    })
}

pub(crate) fn word(generators: &str, text: &str) -> Word {
    text.chars()
        .map(|character| {
            let position = generators
                .find(character.to_ascii_lowercase())
                .unwrap_or_else(|| panic!("{character:?} is not a generator of {generators:?}"));
            let index = u8::try_from(position).expect("a test presentation has few generators");
            if character.is_ascii_uppercase() {
                Letter::inverse_of(index)
            } else {
                Letter::generator(index)
            }
        })
        .collect()
}

pub(crate) struct Fixture {
    pub(crate) source: &'static str,
    pub(crate) generators: &'static str,
    pub(crate) relators: &'static [(&'static str, u32)],
    pub(crate) subgroup: &'static [&'static str],
    pub(crate) index: usize,
    pub(crate) rows: &'static [&'static [u32]],
}

pub(crate) fn presentation(generators: &str, relators: &[(&str, u32)]) -> Presentation {
    let names = generators.chars().map(String::from).collect();
    let relators = relators
        .iter()
        .map(|&(base, exponent)| {
            Relator::new(word(generators, base), exponent).expect("a fixture relator is valid")
        })
        .collect();
    Presentation::new(names, relators).expect("a fixture presentation is valid")
}

pub(crate) fn coxeter(p: u32, q: u32, r: u32) -> Presentation {
    let [s0, s1, s2] = [0, 1, 2].map(Letter::generator);
    let relators = [
        (vec![s0], 2),
        (vec![s1], 2),
        (vec![s2], 2),
        (vec![s0, s1], p),
        (vec![s1, s2], q),
        (vec![s2, s0], r),
    ]
    .into_iter()
    .map(|(base, exponent)| {
        Relator::new(letters(&base), exponent).expect("a Coxeter relator is valid")
    })
    .collect();
    let names = ["s0", "s1", "s2"].map(String::from).to_vec();
    Presentation::new(names, relators).expect("a Coxeter presentation is valid")
}

pub(crate) fn index_limit(colours: u32) -> IndexLimit {
    match IndexLimit::new(colours) {
        Ok(limit) => limit,
        Err(error) => panic!("{colours} is not an index limit: {error}"),
    }
}

pub(crate) fn budget(steps: u64) -> SearchBudget {
    match SearchBudget::new(steps) {
        Ok(budget) => budget,
        Err(error) => panic!("{steps} is not a search budget: {error}"),
    }
}

impl Fixture {
    pub(crate) fn presentation(&self) -> Presentation {
        presentation(self.generators, self.relators)
    }

    pub(crate) fn subgroup(&self) -> Vec<Word> {
        self.subgroup
            .iter()
            .map(|text| word(self.generators, text))
            .collect()
    }
}

const COXETER_532: &[(&str, u32)] = &[("a", 2), ("b", 3), ("ab", 5)];
const HOLT_QUATERNION: &[(&str, u32)] = &[("AbaBBB", 1), ("BabAAA", 1)];
const HOLT_A5: &[(&str, u32)] = &[("a", 3), ("b", 5), ("ab", 2)];

pub(crate) const FIXTURES: &[Fixture] = &[
    Fixture {
        source: "Brown p. 5",
        generators: "ab",
        relators: &[("a", 3), ("b", 3), ("ab", 2)],
        subgroup: &["a"],
        index: 4,
        rows: &[&[0, 0, 1, 2], &[2, 3, 2, 0], &[3, 1, 0, 1], &[1, 2, 3, 3]],
    },
    Fixture {
        source: "Hulpke pp. 72-75",
        generators: "ab",
        relators: COXETER_532,
        subgroup: &["a", "Bab"],
        index: 6,
        rows: &[
            &[0, 0, 1, 2],
            &[3, 3, 2, 0],
            &[2, 2, 0, 1],
            &[1, 1, 4, 5],
            &[5, 5, 5, 3],
            &[4, 4, 3, 4],
        ],
    },
    Fixture {
        source: "Hulpke Example III.35, trivial subgroup",
        generators: "ab",
        relators: COXETER_532,
        subgroup: &[],
        index: 60,
        rows: &[],
    },
    Fixture {
        source: "GAP subgroup a, babaB",
        generators: "ab",
        relators: COXETER_532,
        subgroup: &["a", "babaB"],
        index: 5,
        rows: &[
            &[0, 0, 1, 2],
            &[3, 3, 2, 0],
            &[4, 4, 0, 1],
            &[1, 1, 3, 3],
            &[2, 2, 4, 4],
        ],
    },
    Fixture {
        source: "Holt p. 15, subgroup xy",
        generators: "xy",
        relators: &[("x", 2), ("y", 3), ("xyxy", 1)],
        subgroup: &["xy"],
        index: 3,
        rows: &[],
    },
    Fixture {
        source: "Holt p. 15, subgroup yx",
        generators: "xy",
        relators: &[("x", 2), ("y", 3), ("xyxy", 1)],
        subgroup: &["yx"],
        index: 3,
        rows: &[&[1, 1, 1, 2], &[0, 0, 2, 0], &[2, 2, 0, 1]],
    },
    Fixture {
        source: "Holt pp. 16-17",
        generators: "xy",
        relators: &[("x", 3), ("y", 4), ("xy", 2)],
        subgroup: &["y"],
        index: 6,
        rows: &[],
    },
    Fixture {
        source: "Holt p. 17",
        generators: "ab",
        relators: HOLT_QUATERNION,
        subgroup: &["a"],
        index: 2,
        rows: &[],
    },
    Fixture {
        source: "Holt p. 17, trivial subgroup",
        generators: "ab",
        relators: HOLT_QUATERNION,
        subgroup: &[],
        index: 8,
        rows: &[],
    },
    Fixture {
        source: "Holt pp. 21-22",
        generators: "ab",
        relators: HOLT_A5,
        subgroup: &["b", "aBabbA"],
        index: 6,
        rows: &[],
    },
    Fixture {
        source: "Holt pp. 21-22, trivial subgroup",
        generators: "ab",
        relators: HOLT_A5,
        subgroup: &[],
        index: 60,
        rows: &[],
    },
    Fixture {
        source: "Holt pp. 22-23",
        generators: "xy",
        relators: &[("x", 3), ("y", 5), ("xy", 2)],
        subgroup: &["xy", "XYxyx"],
        index: 5,
        rows: &[],
    },
    Fixture {
        source: "SymPy Example 5.1",
        generators: "xy",
        relators: &[("x", 3), ("y", 3), ("XYxy", 1)],
        subgroup: &["x"],
        index: 3,
        rows: &[&[0, 0, 1, 2], &[1, 1, 2, 0], &[2, 2, 0, 1]],
    },
    Fixture {
        source: "SymPy Example 5.2",
        generators: "xy",
        relators: &[("x", 2), ("y", 3), ("xy", 3)],
        subgroup: &["xy"],
        index: 4,
        rows: &[&[1, 1, 2, 1], &[0, 0, 0, 2], &[3, 3, 1, 0], &[2, 2, 3, 3]],
    },
    Fixture {
        source: "SymPy Example 5.3",
        generators: "xy",
        relators: &[("xxyy", 1), ("xxxyyyyy", 1)],
        subgroup: &[],
        index: 4,
        rows: &[&[1, 2, 1, 2], &[3, 0, 3, 0], &[0, 3, 0, 3], &[2, 1, 2, 1]],
    },
    Fixture {
        source: "SymPy Example 5.4",
        generators: "abcde",
        relators: &[("abC", 1), ("bcD", 1), ("cdE", 1), ("deA", 1), ("eaB", 1)],
        subgroup: &["a"],
        index: 1,
        rows: &[&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]],
    },
    Fixture {
        source: "SymPy Exercises Pg. 161, Q2",
        generators: "xy",
        relators: &[("xxyy", 1), ("YxyXXX", 1)],
        subgroup: &[],
        index: 8,
        rows: &[
            &[1, 2, 3, 4],
            &[5, 0, 6, 7],
            &[0, 5, 7, 6],
            &[7, 6, 5, 0],
            &[6, 7, 0, 5],
            &[2, 1, 4, 3],
            &[3, 4, 2, 1],
            &[4, 3, 1, 2],
        ],
    },
    Fixture {
        source: "SymPy Table 1. Ex. 1 of CDHW1973",
        generators: "rst",
        relators: &[("TrtRR", 1), ("RsrSS", 1), ("StsTT", 1)],
        subgroup: &["r"],
        index: 1,
        rows: &[&[0, 0, 0, 0, 0, 0]],
    },
    Fixture {
        source: "SymPy Cox",
        generators: "ab",
        relators: &[("a", 6), ("b", 6), ("ab", 2), ("aabb", 2), ("aaabbb", 5)],
        subgroup: &["a"],
        index: 500,
        rows: &[],
    },
    Fixture {
        source: "SymPy B_2_4",
        generators: "ab",
        relators: &[
            ("a", 4),
            ("b", 4),
            ("ab", 4),
            ("Ab", 4),
            ("aab", 4),
            ("abb", 4),
            ("aabb", 4),
            ("Abab", 4),
            ("aBab", 4),
        ],
        subgroup: &["a"],
        index: 1024,
        rows: &[],
    },
    Fixture {
        source: "ACE Appendix B.3",
        generators: "ab",
        relators: COXETER_532,
        subgroup: &["ab"],
        index: 12,
        rows: &[],
    },
];

pub(crate) const SEARCH_BUDGET: u64 = 1 << 20;

pub(crate) fn small_presentation() -> impl Strategy<Value = (Presentation, u32)> {
    (random_presentation(), 1_u32..=5).prop_map(|((presentation, _), limit)| (presentation, limit))
}

pub(crate) fn random_presentation() -> impl Strategy<Value = (Presentation, Vec<Word>)> {
    (2_u8..=3).prop_flat_map(|generators| {
        let relators = vec(vec(letter(generators), 1..=6), 1..=4);
        let subgroup = vec(vec(letter(generators), 0..=4), 0..=2);
        (Just(generators), relators, subgroup).prop_map(|(generators, relators, subgroup)| {
            let names = names(usize::from(generators));
            let relators = relators
                .into_iter()
                .map(|letters| {
                    Relator::new(letters.into_iter().collect(), 1)
                        .expect("a random relator is valid")
                })
                .collect();
            let presentation =
                Presentation::new(names, relators).expect("a random presentation is valid");
            let subgroup = subgroup
                .into_iter()
                .map(|letters| letters.into_iter().collect())
                .collect();
            (presentation, subgroup)
        })
    })
}

pub(crate) type Table = &'static [&'static [u32]];

pub(crate) enum Expected {
    ClassTables(&'static [Table]),
    Subgroups {
        counts: &'static [u64],
        tables: &'static [Table],
    },
    ClassCounts(&'static [u64]),
    ClassTotal(u64),
    FixedClasses {
        word: &'static str,
        counts: &'static [u64],
        tables: &'static [Table],
    },
}

pub(crate) struct SearchFixture {
    pub(crate) source: &'static str,
    pub(crate) generators: &'static str,
    pub(crate) relators: &'static [(&'static str, u32)],
    pub(crate) limit: u32,
    pub(crate) expected: Expected,
}

impl SearchFixture {
    pub(crate) fn presentation(&self) -> Presentation {
        presentation(self.generators, self.relators)
    }

    pub(crate) fn tables(&self) -> &'static [Table] {
        match self.expected {
            Expected::ClassTables(tables)
            | Expected::Subgroups { tables, .. }
            | Expected::FixedClasses { tables, .. } => tables,
            Expected::ClassCounts(_) | Expected::ClassTotal(_) => &[],
        }
    }
}

const S4: &[(&str, u32)] = &[("x", 2), ("y", 3), ("xy", 4)];
const FREE_PRODUCT: &[(&str, u32)] = &[("x", 2)];
const HIGMAN: &[(&str, u32)] = &[("TrtRR", 1), ("RsrSS", 1), ("StsTT", 1)];

const SIMS_SUBGROUPS: &[Table] = &[
    &[&[0, 0, 0, 0]],
    &[&[0, 0, 1, 1], &[1, 1, 0, 0]],
    &[&[0, 0, 1, 1], &[2, 2, 0, 0], &[1, 1, 2, 2]],
    &[&[0, 0, 1, 2], &[1, 1, 2, 0], &[2, 2, 0, 1]],
    &[&[0, 0, 1, 2], &[2, 2, 2, 0], &[1, 1, 0, 1]],
    &[&[1, 1, 0, 0], &[0, 0, 1, 1]],
    &[&[1, 1, 0, 0], &[0, 0, 2, 2], &[2, 2, 1, 1]],
    &[&[1, 1, 1, 1], &[0, 0, 0, 0]],
    &[&[1, 1, 1, 2], &[0, 0, 2, 0], &[2, 2, 0, 1]],
    &[&[1, 1, 2, 1], &[0, 0, 0, 2], &[2, 2, 1, 0]],
    &[&[1, 1, 2, 2], &[0, 0, 1, 1], &[2, 2, 0, 0]],
];

pub(crate) const SEARCH_FIXTURES: &[SearchFixture] = &[
    SearchFixture {
        source: "Holt Example 5.10 and the SymPy doctest",
        generators: "xy",
        relators: S4,
        limit: 4,
        expected: Expected::ClassTables(&[
            &[&[0, 0, 0, 0]],
            &[&[0, 0, 1, 2], &[1, 1, 2, 0], &[3, 3, 0, 1], &[2, 2, 3, 3]],
            &[&[0, 0, 1, 2], &[2, 2, 2, 0], &[1, 1, 0, 1]],
            &[&[1, 1, 0, 0], &[0, 0, 1, 1]],
        ]),
    },
    SearchFixture {
        source: "Sims Example 6.1",
        generators: "xy",
        relators: &[("x", 3), ("y", 3), ("xy", 2)],
        limit: 4,
        expected: Expected::ClassTables(&[
            &[&[0, 0, 0, 0]],
            &[&[0, 0, 1, 2], &[2, 3, 2, 0], &[3, 1, 0, 1], &[1, 2, 3, 3]],
            &[&[1, 2, 2, 1], &[2, 0, 0, 2], &[0, 1, 1, 0]],
        ]),
    },
    SearchFixture {
        source: "Sims Example 9.2 and p. 207",
        generators: "xy",
        relators: FREE_PRODUCT,
        limit: 5,
        expected: Expected::Subgroups {
            counts: &[1, 3, 7, 23, 71],
            tables: SIMS_SUBGROUPS,
        },
    },
    SearchFixture {
        source: "Sims p. 209",
        generators: "xy",
        relators: FREE_PRODUCT,
        limit: 3,
        expected: Expected::ClassTables(&[
            SIMS_SUBGROUPS[0],
            SIMS_SUBGROUPS[1],
            SIMS_SUBGROUPS[2],
            SIMS_SUBGROUPS[3],
            SIMS_SUBGROUPS[4],
            SIMS_SUBGROUPS[5],
            SIMS_SUBGROUPS[7],
        ]),
    },
    SearchFixture {
        source: "Sims Example 6.2",
        generators: "ab",
        relators: &[("a", 2), ("b", 3), ("ab", 7)],
        limit: 10,
        expected: Expected::FixedClasses {
            word: "abaB",
            counts: &[1, 0, 0, 0, 0, 0, 2, 0, 0, 0],
            tables: &[
                &[
                    &[0, 0, 1, 2],
                    &[1, 1, 2, 0],
                    &[3, 3, 0, 1],
                    &[2, 2, 4, 5],
                    &[4, 4, 5, 3],
                    &[6, 6, 3, 4],
                    &[5, 5, 6, 6],
                ],
                &[
                    &[0, 0, 1, 2],
                    &[1, 1, 2, 0],
                    &[3, 3, 0, 1],
                    &[2, 2, 4, 5],
                    &[6, 6, 5, 3],
                    &[5, 5, 3, 4],
                    &[4, 4, 6, 6],
                ],
            ],
        },
    },
    SearchFixture {
        source: "Sims Example 6.3 at 8",
        generators: "rst",
        relators: HIGMAN,
        limit: 8,
        expected: Expected::ClassCounts(&[1, 0, 0, 0, 0, 0, 0, 0]),
    },
    SearchFixture {
        source: "Sims Example 6.3 at 12",
        generators: "rst",
        relators: HIGMAN,
        limit: 12,
        expected: Expected::ClassCounts(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    },
    SearchFixture {
        source: "Sims Example 6.4",
        generators: "ab",
        relators: &[("a", 11), ("b", 5), ("baBAAA", 1)],
        limit: 11,
        expected: Expected::ClassCounts(&[1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1]),
    },
    SearchFixture {
        source: "Sims Example 6.5",
        generators: "abc",
        relators: &[
            ("a", 11),
            ("b", 5),
            ("c", 4),
            ("ac", 3),
            ("CbcBB", 1),
            ("baBAAA", 1),
        ],
        limit: 12,
        expected: Expected::ClassCounts(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1]),
    },
    SearchFixture {
        source: "GAP manual 47.10-1",
        generators: "ab",
        relators: &[("a", 2), ("b", 3), ("ab", 5)],
        limit: 10,
        expected: Expected::ClassTotal(4),
    },
    SearchFixture {
        source: "GAP Chapter 47, six relators",
        generators: "xyz",
        relators: &[
            ("x", 3),
            ("y", 3),
            ("z", 5),
            ("xyxxyy", 2),
            ("xz", 2),
            ("yzzz", 2),
        ],
        limit: 6,
        expected: Expected::ClassCounts(&[1, 0, 0, 0, 4, 6]),
    },
];
