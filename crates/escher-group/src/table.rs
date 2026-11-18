use crate::error::{Detail, Error, Limit, Result};
use crate::word::{Letter, Word};

pub(crate) const MAX_COSETS: u32 = 1 << 20;
pub(crate) const UNDEFINED: u32 = u32::MAX;

/// The largest number of cosets that one enumeration may define.
///
/// The count includes coset 0 and every coset that a coincidence later
/// removes, so an enumeration with a limit below the index of its subgroup
/// always stops with
/// [`ErrorKind::CosetLimitReached`](crate::ErrorKind::CosetLimitReached).
///
/// # Examples
///
/// ```
/// use escher_group::CosetLimit;
///
/// assert_eq!(CosetLimit::default(), CosetLimit::DEFAULT);
/// assert_eq!(CosetLimit::new(60)?.get(), 60);
/// assert!(CosetLimit::new(CosetLimit::MAX.get() + 1).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CosetLimit {
    cosets: u32,
}

impl CosetLimit {
    /// The default limit, 16384 cosets.
    pub const DEFAULT: Self = Self { cosets: 1 << 14 };

    /// The largest limit, 1048576 cosets.
    pub const MAX: Self = Self { cosets: MAX_COSETS };

    /// Returns the limit of `cosets` cosets.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] of kind
    /// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when
    /// `cosets` is 0 or above [`CosetLimit::MAX`].
    pub fn new(cosets: u32) -> Result<Self> {
        if (1..=MAX_COSETS).contains(&cosets) {
            Ok(Self { cosets })
        } else {
            Err(Error::enumerate(Detail::LimitExceeded(Limit::Cosets(
                cosets,
            ))))
        }
    }

    /// Returns the number of cosets of the limit.
    #[must_use]
    pub fn get(self) -> u32 {
        self.cosets
    }
}

impl Default for CosetLimit {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A complete coset table of a subgroup of finite index, in standard form.
///
/// The cosets are the right cosets of the subgroup, numbered from 0, and
/// coset 0 is the subgroup itself. A row holds one entry for each letter,
/// generator `k` then its inverse for `k` from 0, and the entry of coset `c`
/// and letter `x` is the coset that `x` takes `c` to by multiplication from
/// the right. The numbering is standard: reading the rows in order, and each
/// row over all its entries, the first occurrence of each coset number comes
/// before that of every larger number. Two enumerations of the same subgroup
/// therefore give equal tables.
///
/// [`enumerate`](crate::enumerate) returns it.
///
/// # Examples
///
/// ```
/// use escher_group::{CosetLimit, Letter, Presentation, Relator, Word, enumerate};
///
/// let x = Letter::generator(0);
/// let cube = Relator::new([x].into_iter().collect::<Word>(), 3)?;
/// let presentation = Presentation::new(vec!["x".to_owned()], vec![cube])?;
/// let table = enumerate(&presentation, &[], CosetLimit::DEFAULT)?;
/// assert_eq!(table.index(), 3);
/// assert_eq!(table.permutation(x), Some(vec![1, 2, 0]));
/// assert_eq!(table.row(1), Some(&[2, 0][..]));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CosetTable {
    cosets: usize,
    generators: usize,
    entries: Vec<u32>,
}

impl CosetTable {
    /// Returns the number of cosets, which is the index of the subgroup and
    /// at least 1.
    #[must_use]
    pub fn index(&self) -> usize {
        self.cosets
    }

    /// Returns the number of generators of the presentation that was
    /// enumerated.
    #[must_use]
    pub fn generator_count(&self) -> usize {
        self.generators
    }

    /// Returns the coset that `letter` takes `coset` to, or `None` when
    /// `coset` is not below the index or `letter` uses a generator that the
    /// table lacks.
    #[must_use]
    pub fn image(&self, coset: u32, letter: Letter) -> Option<u32> {
        let row = self.row(coset)?;
        row.get(usize::from(letter.column())).copied()
    }

    /// Returns the coset that `word` takes `coset` to, acting letter by
    /// letter from left to right, or `None` when `coset` is not below the
    /// index or `word` uses a generator that the table lacks.
    ///
    /// The empty word leaves every coset below the index where it is.
    #[must_use]
    pub fn trace(&self, coset: u32, word: &Word) -> Option<u32> {
        self.row(coset)?;
        word.letters()
            .iter()
            .try_fold(coset, |current, &letter| self.image(current, letter))
    }

    /// Returns the entries of `coset`, one for each letter in the order of
    /// the columns, or `None` when `coset` is not below the index.
    ///
    /// The row of a table with no generator is empty.
    #[must_use]
    pub fn row(&self, coset: u32) -> Option<&[u32]> {
        let coset = usize::try_from(coset).ok()?;
        if coset >= self.cosets {
            return None;
        }
        let columns = 2 * self.generators;
        self.entries.get(coset * columns..(coset + 1) * columns)
    }

    /// Returns the column of `letter` as a list whose element `c` is the
    /// coset that `letter` takes coset `c` to, or `None` when `letter` uses a
    /// generator that the table lacks.
    ///
    /// The list is a permutation of the cosets.
    #[must_use]
    pub fn permutation(&self, letter: Letter) -> Option<Vec<u32>> {
        let column = usize::from(letter.column());
        let columns = 2 * self.generators;
        if column >= columns {
            return None;
        }
        Some(
            self.entries
                .iter()
                .skip(column)
                .step_by(columns)
                .copied()
                .collect(),
        )
    }

    pub(crate) fn from_entries(cosets: usize, generators: usize, entries: Vec<u32>) -> Self {
        Self {
            cosets,
            generators,
            entries,
        }
    }

    pub(crate) fn entries(&self) -> &[u32] {
        &self.entries
    }

    pub(crate) fn standard(
        columns: usize,
        rows: usize,
        entries: &[u32],
        live: impl Fn(u32) -> u32,
    ) -> (Self, Footprint) {
        let mut label = vec![UNDEFINED; rows];
        let mut order: Vec<u32> = Vec::with_capacity(rows);
        if let Some(first) = label.first_mut() {
            *first = 0;
            order.push(0);
        }
        let mut next = 0;
        while let Some(&old) = order.get(next) {
            next += 1;
            let start = index(old) * columns;
            for &entry in entries.get(start..start + columns).unwrap_or_default() {
                let target = live(entry);
                if let Some(slot) = label.get_mut(index(target))
                    && *slot == UNDEFINED
                {
                    *slot = u32::try_from(order.len()).unwrap_or(UNDEFINED);
                    order.push(target);
                }
            }
        }
        let mut relabelled = Vec::with_capacity(order.len() * columns);
        for &old in &order {
            let start = index(old) * columns;
            for &entry in entries.get(start..start + columns).unwrap_or_default() {
                let target = live(entry);
                relabelled.push(label.get(index(target)).copied().unwrap_or(UNDEFINED));
            }
        }
        let footprint = Footprint {
            label: label.capacity(),
            order: order.capacity(),
            entries: relabelled.capacity(),
        };
        let table = Self {
            cosets: order.len(),
            generators: columns / 2,
            entries: relabelled,
        };
        (table, footprint)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Footprint {
    pub(crate) label: usize,
    pub(crate) order: usize,
    pub(crate) entries: usize,
}

pub(crate) fn index(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::{CosetLimit, CosetTable, Footprint};
    use crate::enumerate::enumerate;
    use crate::error::ErrorKind;
    use crate::presentation::Presentation;
    use crate::word::{Letter, Word};

    #[test]
    fn enumerate_gives_index_one_with_no_column_for_no_generator() {
        let presentation = Presentation::new(Vec::new(), Vec::new()).expect("it is valid");
        let table = enumerate(
            &presentation,
            &[Word::new()],
            CosetLimit::new(1).expect("1 is valid"),
        )
        .expect("the trivial group enumerates");
        assert_eq!(table.index(), 1);
        assert_eq!(table.generator_count(), 0);
        assert_eq!(table.row(0), Some(&[][..]));
        assert_eq!(table.row(1), None);
        assert_eq!(table.trace(0, &Word::new()), Some(0));
        assert_eq!(table.permutation(Letter::generator(0)), None);
    }

    #[test]
    fn standard_numbers_by_first_occurrence_over_all_columns() {
        let entries = [0, 0, 3, 2, 2, 2, 0, 3, 1, 1, 2, 0, 3, 3, 1, 1];
        let (table, footprint) = CosetTable::standard(4, 4, &entries, |coset| coset);
        assert_eq!(
            footprint,
            Footprint {
                label: 4,
                order: 4,
                entries: 16
            }
        );
        assert_eq!(table.index(), 4);
        assert_eq!(table.row(0), Some(&[0, 0, 1, 2][..]));
        assert_eq!(table.row(1), Some(&[1, 1, 3, 3][..]));
    }

    #[test]
    fn reads_return_none_outside_the_table() {
        let entries = [1, 1, 0, 0];
        let (table, _) = CosetTable::standard(2, 2, &entries, |coset| coset);
        let x = Letter::generator(0);
        assert_eq!(table.image(0, x), Some(1));
        assert_eq!(table.image(2, x), None);
        assert_eq!(table.image(0, Letter::generator(1)), None);
        assert_eq!(table.trace(1, &[x, x, x].into_iter().collect()), Some(0));
        assert_eq!(table.trace(2, &Word::new()), None);
        assert_eq!(table.permutation(x.inverse()), Some(vec![1, 0]));
        assert_eq!(table.permutation(Letter::inverse_of(1)), None);
    }

    #[test]
    fn coset_limit_new_checks_its_range() {
        assert_eq!(CosetLimit::new(1).map(CosetLimit::get), Ok(1));
        assert_eq!(CosetLimit::new(1 << 20), Ok(CosetLimit::MAX));
        assert_eq!(CosetLimit::DEFAULT.get(), 16384);
        for cosets in [0, (1 << 20) + 1, u32::MAX] {
            assert_eq!(
                CosetLimit::new(cosets).map_err(|error| error.kind()),
                Err(ErrorKind::LimitExceeded)
            );
        }
    }
}
