use core::fmt;
use core::str::FromStr;

use crate::error::{Detail, Error, Result};
use crate::search::MAX_INDEX;
use crate::table::{CosetTable, index};
use crate::word::Letter;

const LETTERS: u32 = 26;
const MAX_LETTERS: usize = 4;

/// The name of one colouring: its colour count in decimal, followed by its
/// rank among the colourings of that colour count, written in lower-case
/// letters.
///
/// The colourings of one colour count are ranked from 1 in the
/// lexicographic order of their standard coset tables, read row by row over
/// every column, and the rank is written in bijective base 26: `a` to `z`
/// for 1 to 26, `aa` for 27, and `zzzz` for 475254, the largest rank that
/// four letters write. An id therefore depends on the presentation and on
/// the order of its generators, and not on the limits of the search that
/// found it, so `3a` names the same colouring in every call that is complete
/// to 3 colours.
///
/// Ids are ordered by colour count, then by rank. Parsing an id checks only
/// its form. [`Colourings::find`] tells whether a colouring with the id
/// exists.
///
/// # Examples
///
/// ```
/// use escher_group::ColouringId;
///
/// let id: ColouringId = "12dk".parse()?;
/// assert_eq!((id.colours(), id.ordinal()), (12, 115));
/// assert_eq!(id.to_string(), "12dk");
/// assert!("03a".parse::<ColouringId>().is_err());
/// assert!("65a".parse::<ColouringId>().is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColouringId {
    colours: u32,
    ordinal: u32,
}

impl ColouringId {
    pub(crate) fn new(colours: u32, ordinal: u32) -> Self {
        Self { colours, ordinal }
    }

    /// Returns the colour count of the id, from 1 to 64.
    #[must_use]
    pub fn colours(self) -> u32 {
        self.colours
    }

    /// Returns the rank of the colouring among the colourings of its colour
    /// count, from 1 to 475254.
    #[must_use]
    pub fn ordinal(self) -> u32 {
        self.ordinal
    }

    fn parse(text: &[u8]) -> Option<Self> {
        let digits = text.iter().take_while(|byte| byte.is_ascii_digit()).count();
        let (count, letters) = text.split_at(digits);
        let colours = match *count {
            [first @ b'1'..=b'9'] => u32::from(first - b'0'),
            [first @ b'1'..=b'9', second] => {
                10 * u32::from(first - b'0') + u32::from(second - b'0')
            }
            _ => return None,
        };
        if colours > MAX_INDEX
            || !(1..=MAX_LETTERS).contains(&letters.len())
            || !letters.iter().all(u8::is_ascii_lowercase)
        {
            return None;
        }
        let ordinal = letters.iter().fold(0, |ordinal, &letter| {
            ordinal * LETTERS + u32::from(letter - b'a') + 1
        });
        Some(Self { colours, ordinal })
    }
}

impl fmt::Display for ColouringId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut letters = [0_u8; MAX_LETTERS];
        let mut written = 0;
        let mut rest = self.ordinal;
        while rest > 0 && written < MAX_LETTERS {
            rest -= 1;
            let digit = u8::try_from(rest % LETTERS).unwrap_or(0);
            if let Some(slot) = letters.get_mut(written) {
                *slot = b'a' + digit;
            }
            written += 1;
            rest /= LETTERS;
        }
        write!(formatter, "{}", self.colours)?;
        for &letter in letters.get(..written).unwrap_or_default().iter().rev() {
            write!(formatter, "{}", char::from(letter))?;
        }
        Ok(())
    }
}

impl FromStr for ColouringId {
    type Err = Error;

    /// Reads a colouring id: a colour count from 1 to 64 in decimal without
    /// a leading zero, followed by 1 to 4 lower-case letters and nothing
    /// else.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] of kind
    /// [`ErrorKind::InvalidColouringId`](crate::ErrorKind::InvalidColouringId)
    /// for every other text, such as `0a`, `03a`, `65a`, `3A`, `3`, `a`,
    /// `3aaaaa`, or `3a` followed by a space. The error text holds none of
    /// the text that it rejects.
    fn from_str(text: &str) -> Result<Self> {
        Self::parse(text.as_bytes()).ok_or_else(|| Error::read_id(Detail::InvalidColouringId))
    }
}

/// One colouring of a tiling: a subgroup of finite index of the group of a
/// presentation, with its standard coset table, read as the colouring of the
/// tiles by its cosets.
///
/// Its colour count is the index of the subgroup, and coset 0 of its table is
/// the subgroup itself. [`colourings`](crate::colourings) returns it in a
/// [`Colourings`], and [`colourings_of_index`](crate::colourings_of_index) in
/// a [`ColouringsOfIndex`].
///
/// # Examples
///
/// ```
/// use escher_group::{
///     ColouringId, IndexLimit, SearchBudget, colourings_of_index, conway_presentation,
/// };
/// use escher_signature::Signature;
///
/// let signature: Signature = "532".parse()?;
/// let presentation = conway_presentation(&signature)?;
/// let found = colourings_of_index(&presentation, IndexLimit::new(5)?, SearchBudget::DEFAULT)?;
/// assert!(found.is_complete());
/// let colouring = found.find("5a".parse()?).ok_or("5a is missing")?;
/// assert_eq!(colouring.colours(), 5);
/// assert_eq!(colouring.class(), "5a".parse::<ColouringId>()?);
/// assert_eq!(colouring.colour_of(&[]), Some(0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Colouring {
    id: ColouringId,
    class: ColouringId,
    table: CosetTable,
}

impl Colouring {
    pub(crate) fn new(id: ColouringId, class: ColouringId, table: CosetTable) -> Self {
        Self { id, class, table }
    }

    /// Returns the id of the colouring.
    #[must_use]
    pub fn id(&self) -> ColouringId {
        self.id
    }

    /// Returns the least id in the conjugacy class of the subgroup.
    ///
    /// All conjugates of one subgroup return the same id. The search keeps
    /// one table for each class, the least of the tables that relabelling it
    /// from each of its cosets gives, and ids rank tables in that same order.
    /// The kept table therefore has the least id, and it is the one member of
    /// the class whose `class()` equals its `id()`.
    #[must_use]
    pub fn class(&self) -> ColouringId {
        self.class
    }

    /// Returns the colour count, which is the index of the subgroup.
    #[must_use]
    pub fn colours(&self) -> u32 {
        self.id.colours
    }

    /// Returns the standard coset table of the subgroup.
    #[must_use]
    pub fn table(&self) -> &CosetTable {
        &self.table
    }

    /// Returns the colour of a tile, from 0 to the colour count minus 1, or
    /// `None` when a side has no generator in the table.
    ///
    /// `sides` lists the sides that the fold of a point crossed on its way to
    /// the fundamental domain, in the order of crossing. Side `i` is generator
    /// `i` of the presentation. The colour is the coset that the table reaches
    /// from coset 0 by applying, for each side from the last to the first, the
    /// inverse of its generator. The first side of the fold therefore acts
    /// last, and the empty slice gives colour 0, the colour of the
    /// fundamental domain itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_group::{IndexLimit, Letter, SearchBudget, colourings, conway_presentation};
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "532".parse()?;
    /// let presentation = conway_presentation(&signature)?;
    /// let found = colourings(&presentation, IndexLimit::new(6)?, SearchBudget::DEFAULT)?;
    /// for colouring in found.list() {
    ///     let inner = colouring.colour_of(&[1, 2]).ok_or("side 1 or 2 is missing")?;
    ///     let outer = colouring.colour_of(&[0, 1, 2]).ok_or("side 0 is missing")?;
    ///     assert_eq!(colouring.table().image(inner, Letter::inverse_of(0)), Some(outer));
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn colour_of(&self, sides: &[u8]) -> Option<u32> {
        sides.iter().rev().try_fold(0, |coset, &side| {
            self.table.image(coset, Letter::inverse_of(side))
        })
    }
}

/// The colourings that one search found, ordered by id, with how far the
/// search is complete.
///
/// [`colourings`](crate::colourings) returns it. The list holds every
/// colouring of every colour count from 1 up to [`Colourings::complete_to`],
/// and nothing above. A search that stops at one of its limits says nothing
/// about the colour counts above `complete_to()`.
///
/// # Examples
///
/// ```
/// use escher_group::{IndexLimit, SearchBudget, colourings, conway_presentation};
/// use escher_signature::Signature;
///
/// let signature: Signature = "532".parse()?;
/// let presentation = conway_presentation(&signature)?;
/// let found = colourings(&presentation, IndexLimit::new(6)?, SearchBudget::DEFAULT)?;
/// assert_eq!((found.requested(), found.complete_to()), (6, 6));
/// let ids: Vec<String> = found.list().iter().map(|colouring| colouring.id().to_string()).collect();
/// assert_eq!(ids.first().map(String::as_str), Some("1a"));
/// assert_eq!(ids.len(), 12);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Colourings {
    requested: u32,
    complete_to: u32,
    list: Vec<Colouring>,
    counts: Vec<u64>,
}

impl Colourings {
    pub(crate) fn new(
        requested: u32,
        complete_to: u32,
        list: Vec<Colouring>,
        counts: Vec<u64>,
    ) -> Self {
        Self {
            requested,
            complete_to,
            list,
            counts,
        }
    }

    /// Returns the index limit that the call asked for.
    #[must_use]
    pub fn requested(&self) -> u32 {
        self.requested
    }

    /// Returns the largest colour count up to which the call found every
    /// colouring, from 0 to [`Colourings::requested`].
    ///
    /// It is below `requested()` only when the search stopped at its step
    /// budget or at its limits on what one call stores.
    #[must_use]
    pub fn complete_to(&self) -> u32 {
        self.complete_to
    }

    /// Returns the colourings, ordered by id.
    #[must_use]
    pub fn list(&self) -> &[Colouring] {
        &self.list
    }

    /// Returns the number of colourings with `colours` colours in the list,
    /// or `None` when `colours` is 0 or above [`Colourings::complete_to`].
    ///
    /// It is `Some(0)` for a colour count up to `complete_to()` that has no
    /// colouring.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_group::{IndexLimit, SearchBudget, colourings, conway_presentation};
    /// use escher_signature::Signature;
    ///
    /// let signature: Signature = "*642".parse()?;
    /// let presentation = conway_presentation(&signature)?;
    /// let found = colourings(&presentation, IndexLimit::new(3)?, SearchBudget::DEFAULT)?;
    /// assert_eq!(found.count(2), Some(7));
    /// assert_eq!(found.count(4), None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn count(&self, colours: u32) -> Option<u64> {
        let position = index(colours.checked_sub(1)?);
        self.counts.get(position).copied()
    }

    /// Returns the colouring with the id `id`, or `None` when the list holds
    /// none.
    #[must_use]
    pub fn find(&self, id: ColouringId) -> Option<&Colouring> {
        self.list
            .binary_search_by_key(&id, Colouring::id)
            .ok()
            .and_then(|position| self.list.get(position))
    }
}

/// The colourings with one colour count that one search found, ordered by
/// id, or nothing when the search stopped at a limit.
///
/// [`colourings_of_index`](crate::colourings_of_index) returns it. It says
/// nothing about any other colour count, so [`ColouringsOfIndex::find`]
/// returns `None` for an id of another colour count whatever colourings that
/// count has. When [`ColouringsOfIndex::is_complete`] is false, the list is
/// empty and the result says nothing about the colourings.
///
/// # Examples
///
/// ```
/// use escher_group::{ColouringId, IndexLimit, SearchBudget, colourings_of_index, conway_presentation};
/// use escher_signature::Signature;
///
/// let signature: Signature = "*642".parse()?;
/// let presentation = conway_presentation(&signature)?;
/// let found = colourings_of_index(&presentation, IndexLimit::new(3)?, SearchBudget::DEFAULT)?;
/// assert_eq!(found.colours(), 3);
/// assert!(found.is_complete());
/// assert_eq!(found.list().len(), 3);
/// let id: ColouringId = "3d".parse()?;
/// assert!(found.is_complete() && id.colours() == found.colours() && found.find(id).is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColouringsOfIndex {
    colours: u32,
    complete: bool,
    list: Vec<Colouring>,
}

impl ColouringsOfIndex {
    pub(crate) fn new(colours: u32, complete: bool, list: Vec<Colouring>) -> Self {
        Self {
            colours,
            complete,
            list,
        }
    }

    /// Returns the colour count that the call asked for.
    #[must_use]
    pub fn colours(&self) -> u32 {
        self.colours
    }

    /// Returns whether the search found every colouring with
    /// [`ColouringsOfIndex::colours`] colours.
    ///
    /// It is false only when the search stopped at its step budget or at its
    /// limits on what one call stores. The list is then empty, and the result
    /// says nothing about whether any colouring exists.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Returns the colourings, ordered by id, or an empty slice when the
    /// search is not complete.
    #[must_use]
    pub fn list(&self) -> &[Colouring] {
        &self.list
    }

    /// Returns the colouring with the id `id`, or `None` when the list holds
    /// none.
    ///
    /// `None` shows that no colouring has the id only when the call is
    /// complete and `id` has [`ColouringsOfIndex::colours`] colours.
    #[must_use]
    pub fn find(&self, id: ColouringId) -> Option<&Colouring> {
        self.list
            .binary_search_by_key(&id, Colouring::id)
            .ok()
            .and_then(|position| self.list.get(position))
    }
}

/// The numbers of colourings and of conjugacy classes of each colour count
/// that one search found, with how far the search is complete.
///
/// [`count_colourings`](crate::count_colourings) returns it.
///
/// # Examples
///
/// ```
/// use escher_group::{IndexLimit, SearchBudget, conway_presentation, count_colourings};
/// use escher_signature::Signature;
///
/// let signature: Signature = "532".parse()?;
/// let presentation = conway_presentation(&signature)?;
/// let counts = count_colourings(&presentation, IndexLimit::new(6)?, SearchBudget::DEFAULT)?;
/// assert_eq!(counts.complete_to(), 6);
/// assert_eq!(counts.colourings(5), Some(5));
/// assert_eq!(counts.classes(5), Some(1));
/// assert_eq!(counts.colourings(2), Some(0));
/// assert_eq!(counts.colourings(7), None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColouringCounts {
    requested: u32,
    complete_to: u32,
    colourings: Vec<u64>,
    classes: Vec<u64>,
}

impl ColouringCounts {
    pub(crate) fn new(
        requested: u32,
        complete_to: u32,
        mut colourings: Vec<u64>,
        mut classes: Vec<u64>,
    ) -> Self {
        colourings.truncate(index(complete_to));
        classes.truncate(index(complete_to));
        Self {
            requested,
            complete_to,
            colourings,
            classes,
        }
    }

    /// Returns the index limit that the call asked for.
    #[must_use]
    pub fn requested(&self) -> u32 {
        self.requested
    }

    /// Returns the largest colour count up to which the call counted every
    /// colouring, from 0 to [`ColouringCounts::requested`].
    ///
    /// It is below `requested()` only when the search stopped at its step
    /// budget.
    #[must_use]
    pub fn complete_to(&self) -> u32 {
        self.complete_to
    }

    /// Returns the number of colourings with `colours` colours, or `None`
    /// when `colours` is 0 or above [`ColouringCounts::complete_to`].
    #[must_use]
    pub fn colourings(&self, colours: u32) -> Option<u64> {
        Self::read(&self.colourings, colours)
    }

    /// Returns the number of conjugacy classes of the colourings with
    /// `colours` colours, or `None` when `colours` is 0 or above
    /// [`ColouringCounts::complete_to`].
    #[must_use]
    pub fn classes(&self, colours: u32) -> Option<u64> {
        Self::read(&self.classes, colours)
    }

    fn read(counts: &[u64], colours: u32) -> Option<u64> {
        let position = index(colours.checked_sub(1)?);
        counts.get(position).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{Colouring, ColouringId};
    use crate::common::{conway, index_limit};
    use crate::error::ErrorKind;
    use crate::search::{SearchBudget, colourings};
    use crate::word::Letter;

    #[test]
    fn ids_write_their_rank_in_bijective_base_26() {
        for (ordinal, letters) in [
            (1, "a"),
            (26, "z"),
            (27, "aa"),
            (702, "zz"),
            (703, "aaa"),
            (475_254, "zzzz"),
        ] {
            let id = ColouringId::new(12, ordinal);
            assert_eq!(id.to_string(), format!("12{letters}"));
            assert_eq!(id.to_string().parse::<ColouringId>(), Ok(id));
        }
    }

    #[test]
    fn ids_read_back_as_they_were_written() {
        for text in ["1a", "2g", "3a", "12dk", "64zzzz", "10a", "9z"] {
            let id: ColouringId = text.parse().expect("the id is well formed");
            assert_eq!(id.to_string(), text);
        }
        let id: ColouringId = "64zzzz".parse().expect("the id is well formed");
        assert_eq!((id.colours(), id.ordinal()), (64, 475_254));
    }

    #[test]
    fn malformed_ids_are_refused() {
        for text in [
            "", "0a", "03a", "65a", "99a", "100a", "3A", "3", "a", "3aaaaa", "3a ", " 3a", "3-a",
            "3a1", "+3a", "3\u{e9}",
        ] {
            let error = text.parse::<ColouringId>().expect_err(text);
            assert_eq!(error.kind(), ErrorKind::InvalidColouringId, "{text:?}");
            assert_eq!(
                error.to_string(),
                "cannot read colouring id: the colouring id is not a colour count from 1 to 64 followed by 1 to 4 lower-case letters"
            );
        }
    }

    #[test]
    fn ids_order_by_colour_count_then_rank() {
        let ids: Vec<ColouringId> = ["1a", "2b", "2aa", "10a", "3z"]
            .iter()
            .map(|text| text.parse().expect("well formed"))
            .collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        let texts: Vec<String> = sorted.iter().map(ToString::to_string).collect();
        assert_eq!(texts, ["1a", "2b", "2aa", "3z", "10a"]);
    }

    fn sequences(generators: u8, longest: usize) -> Vec<Vec<u8>> {
        let mut all = vec![Vec::new()];
        let mut start = 0;
        for _ in 0..longest {
            let end = all.len();
            for position in start..end {
                for side in 0..generators {
                    let mut longer = all[position].clone();
                    longer.push(side);
                    all.push(longer);
                }
            }
            start = end;
        }
        all
    }

    #[test]
    fn colour_of_applies_the_inverse_of_the_first_side_last() {
        let found = colourings(&conway("532"), index_limit(6), SearchBudget::DEFAULT)
            .expect("the search runs");
        assert_eq!(found.complete_to(), 6);
        assert_eq!(found.list().len(), 12);
        for colouring in found.list() {
            assert_eq!(colouring.colour_of(&[]), Some(0));
            for sequence in sequences(3, 3) {
                let inner = colouring
                    .colour_of(&sequence)
                    .expect("every side is a generator");
                for side in 0..3 {
                    let mut longer = vec![side];
                    longer.extend_from_slice(&sequence);
                    assert_eq!(
                        colouring.colour_of(&longer),
                        colouring.table().image(inner, Letter::inverse_of(side)),
                        "{} {longer:?}",
                        colouring.id()
                    );
                }
            }
            assert_eq!(colouring.colour_of(&[3]), None);
            assert_eq!(colouring.colour_of(&[0, 255, 1]), None);
        }
    }

    #[test]
    fn a_stored_colouring_has_a_fixed_size() {
        assert_eq!(size_of::<Colouring>(), 56);
    }
}
