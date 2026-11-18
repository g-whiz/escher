use escher_signature::Signature;
use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, any};
use proptest::test_runner::Config;

use crate::{CosetLimit, Letter, Presentation, Relator, Word, conway_presentation};

pub(crate) fn proptest_config() -> Config {
    if cfg!(miri) {
        let mut config = Config::with_cases(8);
        config.failure_persistence = None;
        config
    } else {
        Config::default()
    }
}

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

impl Fixture {
    pub(crate) fn presentation(&self) -> Presentation {
        let names = self.generators.chars().map(String::from).collect();
        let relators = self
            .relators
            .iter()
            .map(|&(base, exponent)| {
                Relator::new(word(self.generators, base), exponent)
                    .expect("a fixture relator is valid")
            })
            .collect();
        Presentation::new(names, relators).expect("a fixture presentation is valid")
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
