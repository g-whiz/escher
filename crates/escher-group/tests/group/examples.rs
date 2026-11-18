use escher_group::{CosetLimit, CosetTable, Letter, enumerate};

use crate::common::{FIXTURES, Fixture};

fn table(fixture: &Fixture) -> CosetTable {
    match enumerate(
        &fixture.presentation(),
        &fixture.subgroup(),
        CosetLimit::DEFAULT,
    ) {
        Ok(table) => table,
        Err(error) => panic!("{} does not finish: {error}", fixture.source),
    }
}

fn fixture(source: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.source == source)
        .unwrap_or_else(|| panic!("no fixture {source}"))
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn worked_examples_give_the_printed_index_and_rows() {
    for fixture in FIXTURES {
        let table = table(fixture);
        assert_eq!(table.index(), fixture.index, "{}", fixture.source);
        for (coset, row) in (0..).zip(fixture.rows) {
            assert_eq!(
                table.row(coset),
                Some(*row),
                "{} row {coset}",
                fixture.source
            );
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn worked_examples_give_the_printed_permutations() {
    let [first, second] = [0, 1].map(Letter::generator);
    let hulpke = table(fixture("Hulpke pp. 72-75"));
    assert_eq!(hulpke.permutation(first), Some(vec![0, 3, 2, 1, 5, 4]));
    assert_eq!(hulpke.permutation(second), Some(vec![1, 2, 0, 4, 5, 3]));
    let holt = table(fixture("Holt p. 15, subgroup yx"));
    assert_eq!(holt.permutation(first), Some(vec![1, 0, 2]));
    assert_eq!(holt.permutation(second), Some(vec![1, 2, 0]));
}
