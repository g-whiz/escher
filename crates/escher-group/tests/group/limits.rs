use escher_group::{CosetLimit, ErrorKind, Letter, Presentation, Relator, Word, enumerate};

use crate::common::{conway, letters, limit, names};

fn outcome(presentation: &Presentation, cosets: u32) -> Result<usize, ErrorKind> {
    enumerate(presentation, &[], limit(cosets))
        .map(|table| table.index())
        .map_err(|error| error.kind())
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn infinite_groups_reach_the_limit() {
    for signature in ["*632", "2222", "o", "*732"] {
        assert_eq!(
            outcome(&conway(signature), 1000),
            Err(ErrorKind::CosetLimitReached),
            "{signature}"
        );
    }
}

#[test]
fn an_infinite_group_reaches_a_small_limit() {
    assert_eq!(
        outcome(&conway("2222"), 50),
        Err(ErrorKind::CosetLimitReached)
    );
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn limit_counts_coset_zero() {
    let presentation = conway("532");
    assert_eq!(outcome(&presentation, 60), Ok(60));
    assert_eq!(
        outcome(&presentation, 59),
        Err(ErrorKind::CosetLimitReached)
    );
}

fn sims(n: u32) -> Presentation {
    let [x, y] = [0, 1].map(Letter::generator);
    let relator = |power: Letter, other: Letter| {
        let base: Word = core::iter::repeat_n(power, usize::try_from(n).unwrap_or(0))
            .chain([other.inverse()])
            .collect();
        match Relator::new(base, 1) {
            Ok(relator) => relator,
            Err(error) => panic!("{error}"),
        }
    };
    match Presentation::new(
        vec!["x".to_owned(), "y".to_owned()],
        vec![relator(x, y), relator(y, x)],
    ) {
        Ok(presentation) => presentation,
        Err(error) => panic!("{error}"),
    }
}

const SIMS_DEFINED: [(u32, u32); 6] = [(3, 8), (4, 15), (5, 28), (6, 47), (7, 83), (8, 138)];

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn sims_presentation_finishes_at_its_count_and_not_below() {
    for (n, defined) in SIMS_DEFINED {
        let presentation = sims(n);
        let order = usize::try_from(n * n - 1).unwrap_or(0);
        assert_eq!(
            outcome(&presentation, CosetLimit::DEFAULT.get()),
            Ok(order),
            "{n}"
        );
        assert_eq!(outcome(&presentation, defined), Ok(order), "{n}");
        assert_eq!(
            outcome(&presentation, defined - 1),
            Err(ErrorKind::CosetLimitReached),
            "{n}"
        );
        if n >= 5 {
            assert!(u64::from(defined) > u64::from(n * n - 1), "{n}");
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn the_maximum_limit_fills_the_largest_table() {
    let free = match Presentation::new(names(8), Vec::new()) {
        Ok(presentation) => presentation,
        Err(error) => panic!("{error}"),
    };
    assert_eq!(
        outcome(&free, CosetLimit::MAX.get()),
        Err(ErrorKind::CosetLimitReached)
    );
}

fn presentation(generators: usize, relators: &[(&[Letter], u32)]) -> Presentation {
    let relators = relators
        .iter()
        .map(
            |&(base, exponent)| match Relator::new(letters(base), exponent) {
                Ok(relator) => relator,
                Err(error) => panic!("{error}"),
            },
        )
        .collect();
    match Presentation::new(names(generators), relators) {
        Ok(presentation) => presentation,
        Err(error) => panic!("{error}"),
    }
}

#[test]
fn a_long_subgroup_generator_finishes_within_the_index() {
    let x = Letter::generator(0);
    let order_two = presentation(1, &[(&[x], 2)]);
    let result = enumerate(&order_two, &[letters(&[x; 4096])], limit(2));
    assert_eq!(result.map(|table| table.index()), Ok(2));
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn many_long_subgroup_generators_finish_within_the_index() {
    let [x, y] = [0, 1].map(Letter::generator);
    let klein = presentation(2, &[(&[x], 2), (&[y], 2), (&[x, y], 2)]);
    let words: Vec<Word> = [
        [x, y],
        [x, y.inverse()],
        [x.inverse(), y],
        [x.inverse(), y.inverse()],
        [y, x],
    ]
    .iter()
    .map(|pair| letters(&pair.repeat(2048)))
    .collect();
    let result = enumerate(&klein, &words, limit(4));
    assert_eq!(result.map(|table| table.index()), Ok(4));
    let reversed: Vec<Word> = words.iter().rev().cloned().collect();
    let result = enumerate(&klein, &reversed, limit(4));
    assert_eq!(result.map(|table| table.index()), Ok(4));
}
