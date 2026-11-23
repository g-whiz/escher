use crate::colouring::{
    Colouring, ColouringCounts, ColouringId, Colourings, ColouringsOfIndex, fixed_generator,
};
use crate::enumerate::MAX_SUBGROUP_GENERATORS;
use crate::enumerate::{Prepared, build, check_expanded, grow_within};
use crate::error::{Detail, Error, Limit, Result};
use crate::presentation::{MAX_BASE, Presentation, uses_only};
use crate::table::{CosetTable, UNDEFINED, index};
use crate::word::Word;

pub(crate) const MAX_INDEX: u32 = 64;
pub(crate) const MAX_STEPS: u64 = 1 << 36;
pub(crate) const MAX_STORED_COLOURINGS: usize = 1 << 16;
pub(crate) const MAX_STORED_ENTRIES: usize = 1 << 22;

/// The largest colour count that a search lists, which is the largest index
/// of the subgroups that it finds.
///
/// # Examples
///
/// ```
/// use escher_group::IndexLimit;
///
/// assert_eq!(IndexLimit::default(), IndexLimit::DEFAULT);
/// assert_eq!(IndexLimit::DEFAULT.get(), 12);
/// assert_eq!(IndexLimit::new(24)?.get(), 24);
/// assert!(IndexLimit::new(IndexLimit::MAX.get() + 1).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IndexLimit {
    index: u32,
}

impl IndexLimit {
    /// The default limit, 12 colours.
    pub const DEFAULT: Self = Self { index: 12 };

    /// The largest limit, 64 colours.
    pub const MAX: Self = Self { index: MAX_INDEX };

    /// Returns the limit of `index` colours.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] of kind
    /// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when
    /// `index` is 0 or above [`IndexLimit::MAX`].
    pub fn new(index: u32) -> Result<Self> {
        if (1..=MAX_INDEX).contains(&index) {
            Ok(Self { index })
        } else {
            Err(Error::search(Detail::LimitExceeded(Limit::Index(index))))
        }
    }

    /// Returns the number of colours of the limit.
    #[must_use]
    pub fn get(self) -> u32 {
        self.index
    }
}

impl Default for IndexLimit {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The largest number of steps that a search may take, where a step is one
/// read of an entry of a coset table.
///
/// Counting reads bounds the time of a search and gives the same result on
/// every machine. A call of [`count_colourings`] or [`colourings`] takes at
/// most twice its budget, and a call of [`colourings_of_index`] at most its
/// budget.
///
/// # Examples
///
/// ```
/// use escher_group::SearchBudget;
///
/// assert_eq!(SearchBudget::default(), SearchBudget::DEFAULT);
/// assert_eq!(SearchBudget::DEFAULT.get(), 1 << 28);
/// assert_eq!(SearchBudget::new(1000)?.get(), 1000);
/// assert!(SearchBudget::new(0).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SearchBudget {
    steps: u64,
}

impl SearchBudget {
    /// The default budget, 268435456 steps.
    pub const DEFAULT: Self = Self { steps: 1 << 28 };

    /// The largest budget, 68719476736 steps.
    pub const MAX: Self = Self { steps: MAX_STEPS };

    /// Returns the budget of `steps` steps.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] of kind
    /// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when
    /// `steps` is 0 or above [`SearchBudget::MAX`].
    pub fn new(steps: u64) -> Result<Self> {
        if (1..=MAX_STEPS).contains(&steps) {
            Ok(Self { steps })
        } else {
            Err(Error::search(Detail::LimitExceeded(Limit::Budget(steps))))
        }
    }

    /// Returns the number of steps of the budget.
    #[must_use]
    pub fn get(self) -> u64 {
        self.steps
    }
}

impl Default for SearchBudget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The colourings that a search returns: every subgroup of finite index, only
/// the valid ones, or one valid colouring for each conjugacy class.
///
/// A colouring is valid when no generator of the presentation lies in its
/// subgroup, which [`Colouring::contained_generator`] tests. `Valid` returns
/// exactly the colourings that `All` returns with `contained_generator()`
/// equal to `None`, in the same order. `ValidClasses` returns exactly the
/// colourings that `Valid` returns with [`Colouring::class`] equal to their
/// [`Colouring::id`], in the same order, so a class without a valid subgroup
/// gives none. The ids of each colour count are ranked within the set, so
/// with `Valid` and `ValidClasses` they run from `a` with no gap, and the
/// same id can name another colouring, or none, in another set.
///
/// # Examples
///
/// ```
/// use escher_group::{
///     ColouringSet, IndexLimit, Letter, Presentation, Relator, SearchBudget, Word, colourings,
/// };
///
/// let [x, y] = [Letter::generator(0), Letter::generator(1)];
/// let relators = vec![
///     Relator::new([x].into_iter().collect::<Word>(), 2)?,
///     Relator::new([y].into_iter().collect::<Word>(), 2)?,
///     Relator::new([x, y].into_iter().collect::<Word>(), 3)?,
/// ];
/// let presentation = Presentation::new(vec!["x".to_owned(), "y".to_owned()], relators)?;
/// let limit = IndexLimit::new(6)?;
/// let all = colourings(&presentation, ColouringSet::All, limit, SearchBudget::DEFAULT)?;
/// let valid = colourings(&presentation, ColouringSet::Valid, limit, SearchBudget::DEFAULT)?;
/// let filtered: Vec<_> = all
///     .list()
///     .iter()
///     .filter(|colouring| colouring.contained_generator().is_none())
///     .map(|colouring| colouring.table())
///     .collect();
/// let tables: Vec<_> = valid.list().iter().map(|colouring| colouring.table()).collect();
/// assert_eq!(tables, filtered);
/// let ids: Vec<String> = valid.list().iter().map(|colouring| colouring.id().to_string()).collect();
/// assert_eq!(ids, ["2a", "3a", "6a"]);
/// let classes = colourings(&presentation, ColouringSet::ValidClasses, limit, SearchBudget::DEFAULT)?;
/// assert!(classes.list().iter().all(|colouring| colouring.class() == colouring.id()));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "a search returns every colouring, the valid ones, or one valid colouring of each class, and callers match on each of these sets"
)]
pub enum ColouringSet {
    /// Every subgroup of finite index, each its own colouring.
    All,
    /// The colourings in which no generator lies in the subgroup.
    Valid,
    /// One valid colouring for each conjugacy class that has a valid subgroup:
    /// the first valid table that relabelling the table of the class from
    /// coset 0 upward gives, which is the colouring that names the class
    /// under `Valid`.
    ValidClasses,
}

/// Counts the colourings of `set`, as [`ColouringSet`] describes them, and
/// their conjugacy classes for each colour count from 1 to `limit` in the
/// group of `presentation`.
///
/// The search is the low-index subgroup search over partial coset tables. It
/// keeps one table for each conjugacy class and counts the distinct tables
/// that relabelling it from each of its cosets gives, which are the
/// subgroups of the class. With [`ColouringSet::Valid`] it counts only the
/// valid ones, and a class counts only when one of its subgroups is valid.
/// With [`ColouringSet::ValidClasses`] it stops relabelling a class at its
/// first valid table and counts that one, so the counts of colourings and of
/// classes are equal. It stores none of them, so only `budget` can stop it.
///
/// A first run looks for every subgroup of index up to `limit`. When it
/// reaches the budget, the call discards what it found and runs the bounds
/// 1, 2, and so on to `limit` in turn, with a fresh budget of the same size,
/// until one of them reaches the budget. The result is then complete to the
/// last bound that finished and says nothing about the colour counts above
/// it. The result depends only on the arguments.
///
/// # Examples
///
/// ```
/// use escher_group::{
///     ColouringSet, IndexLimit, Letter, Presentation, Relator, SearchBudget, Word, count_colourings,
/// };
///
/// let x = Letter::generator(0);
/// let square = Relator::new([x].into_iter().collect::<Word>(), 2)?;
/// let presentation = Presentation::new(vec!["x".to_owned(), "y".to_owned()], vec![square])?;
/// let limit = IndexLimit::new(5)?;
/// let counts = count_colourings(&presentation, ColouringSet::All, limit, SearchBudget::DEFAULT)?;
/// let numbers: Vec<Option<u64>> = (1..=5).map(|colours| counts.colourings(colours)).collect();
/// assert_eq!(numbers, [Some(1), Some(3), Some(7), Some(23), Some(71)]);
/// assert_eq!(counts.classes(3), Some(3));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an [`Error`] of kind
/// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when the
/// relators written out with their exponents have more than 65536 letters in
/// total. A search that reaches its budget returns its result and not an
/// error.
pub fn count_colourings(
    presentation: &Presentation,
    set: ColouringSet,
    limit: IndexLimit,
    budget: SearchBudget,
) -> Result<ColouringCounts> {
    let settings = Settings::new(budget).within(set);
    let (found, _) = search(presentation, limit.get(), Request::Count, settings)?;
    Ok(ColouringCounts::new(
        limit.get(),
        found.complete_to,
        found.colourings,
        found.classes,
    ))
}

/// Lists the colourings of `set`, as [`ColouringSet`] describes them, for
/// each colour count from 1 to `limit` in the group of `presentation`,
/// ordered by id.
///
/// The search is that of [`count_colourings`], and it stores the standard
/// table of every subgroup of `set` of every class that it finds. The id of
/// a colouring ranks its table among the tables of its colour count in
/// `set`, so it does not depend on `limit` or `budget`. A call stores at
/// most 65536 colourings with at most 4194304 table entries in total, and a
/// colouring that `set` leaves out counts toward neither. Reaching either
/// limit counts as reaching the budget, so the result is then complete to a
/// lower colour count and says nothing about the colour counts above it.
///
/// # Examples
///
/// ```
/// use escher_group::{ColouringSet, IndexLimit, SearchBudget, colourings, conway_presentation};
/// use escher_signature::Signature;
///
/// let signature: Signature = "*642".parse()?;
/// let presentation = conway_presentation(&signature)?;
/// let found = colourings(&presentation, ColouringSet::All, IndexLimit::new(3)?, SearchBudget::DEFAULT)?;
/// assert_eq!(found.complete_to(), 3);
/// let two = found.list().iter().filter(|colouring| colouring.colours() == 2).count();
/// assert_eq!(two, 7);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an [`Error`] of kind
/// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when the
/// relators written out with their exponents have more than 65536 letters in
/// total. A search that reaches a limit returns its result and not an error.
pub fn colourings(
    presentation: &Presentation,
    set: ColouringSet,
    limit: IndexLimit,
    budget: SearchBudget,
) -> Result<Colourings> {
    let settings = Settings::new(budget).within(set);
    let (found, _) = search(presentation, limit.get(), Request::List, settings)?;
    Ok(found.into_colourings(limit.get()))
}

/// Lists the colourings of `set` with exactly `colours` colours of the group
/// of `presentation`, ordered by id.
///
/// The call searches only the subgroups of index `colours`, which is all that
/// a caller resolving one [`ColouringId`] needs, and gives each the same id as
/// [`colourings`] does with the same `set`. When that search reaches `budget` or the limits on
/// what one call stores, the list is empty and
/// [`ColouringsOfIndex::is_complete`] is false, which says nothing about the
/// colourings. The result says nothing about any other colour count.
///
/// # Examples
///
/// ```
/// use escher_group::{ColouringSet, IndexLimit, SearchBudget, colourings_of_index, conway_presentation};
/// use escher_signature::Signature;
///
/// let signature: Signature = "*642".parse()?;
/// let presentation = conway_presentation(&signature)?;
/// let colours = IndexLimit::new(3)?;
/// let found = colourings_of_index(&presentation, ColouringSet::All, colours, SearchBudget::DEFAULT)?;
/// assert!(found.is_complete());
/// assert_eq!(found.list().len(), 3);
/// assert!(found.find("3d".parse()?).is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an [`Error`] of kind
/// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when the
/// relators written out with their exponents have more than 65536 letters in
/// total. A search that reaches a limit returns its result and not an error.
pub fn colourings_of_index(
    presentation: &Presentation,
    set: ColouringSet,
    colours: IndexLimit,
    budget: SearchBudget,
) -> Result<ColouringsOfIndex> {
    let settings = Settings::new(budget).within(set);
    let (found, _) = search(presentation, colours.get(), Request::Index, settings)?;
    let complete = found.complete_to == colours.get();
    Ok(ColouringsOfIndex::new(
        colours.get(),
        complete,
        found.into_list(),
    ))
}

/// Lists the class table of each conjugacy class of the subgroups of index
/// exactly `colours` in the group of `presentation` in whose coset table no
/// word of `stabilizer` fixes a coset, ordered by id.
///
/// The search is that of [`colourings_of_index`] with [`ColouringSet::All`],
/// and of each class it considers only the class table, the table that names
/// the class there. It keeps that table when tracing each word from each
/// coset, letter by letter from the left and without reducing the word,
/// leads to another coset, and drops the class otherwise. The other
/// subgroups of a class have the same tables with the cosets renamed, so the
/// one table decides for the whole class. The ids rank the kept tables among
/// themselves, from `a` with no gap, and the class of each kept colouring is
/// its own id. Each table entry that the test reads counts toward `budget`,
/// and only the kept tables count toward the limits on what one call
/// stores. When the search reaches `budget` or those limits, the list is
/// empty and [`ColouringsOfIndex::is_complete`] is false, which says nothing
/// about the classes.
///
/// An empty `stabilizer` keeps every class, and a `stabilizer` that holds
/// the empty word keeps none, because the empty word fixes every coset.
///
/// # Examples
///
/// ```
/// use escher_group::{
///     ColouringSet, IndexLimit, Letter, SearchBudget, Word, colourings_of_index,
///     conway_presentation, free_classes_of_index,
/// };
/// use escher_signature::Signature;
///
/// let signature: Signature = "*632".parse()?;
/// let presentation = conway_presentation(&signature)?;
/// let [s0, s1] = [Letter::generator(0), Letter::generator(1)];
/// let words: Vec<Word> = (1..12)
///     .map(|length| (0..length).map(|position| if position % 2 == 0 { s0 } else { s1 }).collect())
///     .collect();
/// let colours = IndexLimit::new(12)?;
/// let free = free_classes_of_index(&presentation, &words, colours, SearchBudget::DEFAULT)?;
/// assert!(free.is_complete());
/// assert!(free.list().iter().all(|colouring| colouring.class() == colouring.id()));
/// let all = colourings_of_index(&presentation, ColouringSet::All, colours, SearchBudget::DEFAULT)?;
/// assert!(free.list().len() < all.list().len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an [`Error`] for the first of these causes, in this order:
///
/// - [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when
///   `stabilizer` has more than 256 words,
/// - for the first word in order that has one,
///   [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when it
///   has more than 4096 letters, and
///   [`ErrorKind::InvalidWord`](crate::ErrorKind::InvalidWord) when it uses
///   a generator that the presentation does not have,
/// - [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when the
///   relators written out with their exponents have more than 65536 letters
///   in total.
///
/// A search that reaches a limit returns its result and not an error.
pub fn free_classes_of_index(
    presentation: &Presentation,
    stabilizer: &[Word],
    colours: IndexLimit,
    budget: SearchBudget,
) -> Result<ColouringsOfIndex> {
    check_stabilizer(presentation, stabilizer)?;
    let settings = Settings::new(budget);
    let (found, _) = search_with(
        presentation,
        stabilizer,
        colours.get(),
        Request::Free,
        settings,
    )?;
    let complete = found.complete_to == colours.get();
    Ok(ColouringsOfIndex::new(
        colours.get(),
        complete,
        found.into_list(),
    ))
}

fn check_stabilizer(presentation: &Presentation, stabilizer: &[Word]) -> Result<()> {
    if stabilizer.len() > MAX_SUBGROUP_GENERATORS {
        return Err(Error::search(Detail::LimitExceeded(
            Limit::StabilizerElements,
        )));
    }
    let generators = presentation.generator_count();
    for (position, word) in (1..).zip(stabilizer) {
        if word.len() > MAX_BASE {
            return Err(Error::search(Detail::LimitExceeded(
                Limit::StabilizerLetters(position),
            )));
        }
        if !uses_only(word, generators) {
            return Err(Error::search(Detail::StabilizerLetter(position)));
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Request {
    Count,
    List,
    Index,
    Free,
}

impl Request {
    fn keeping(self) -> Keeping {
        match self {
            Self::Count => Keeping::Counts,
            Self::List | Self::Index => Keeping::Tables,
            Self::Free => Keeping::FreeClasses,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Keeping {
    Counts,
    Tables,
    FreeClasses,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rows {
    AtMost,
    Exact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pruning {
    Partial,
    Complete,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) budget: u64,
    pub(crate) colourings: usize,
    pub(crate) entries: usize,
    pub(crate) pruning: Pruning,
    pub(crate) set: ColouringSet,
}

impl Settings {
    pub(crate) fn new(budget: SearchBudget) -> Self {
        Self {
            budget: budget.get(),
            colourings: MAX_STORED_COLOURINGS,
            entries: MAX_STORED_ENTRIES,
            pruning: Pruning::Partial,
            set: ColouringSet::All,
        }
    }

    pub(crate) fn within(self, set: ColouringSet) -> Self {
        Self { set, ..self }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Capacities {
    pub(crate) table: usize,
    pub(crate) trail: usize,
    pub(crate) stack: usize,
    pub(crate) frames: usize,
    pub(crate) label: usize,
    pub(crate) order: usize,
    pub(crate) scratch: usize,
    pub(crate) bases: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Stats {
    pub(crate) steps: u64,
    pub(crate) runs: u32,
    pub(crate) first_run_stopped: bool,
    pub(crate) nodes: u64,
    pub(crate) contradictions: u64,
    pub(crate) leaves: u64,
    pub(crate) leaf_discards: u64,
    pub(crate) relabel_changes: u64,
    pub(crate) run_steps: Vec<u64>,
    pub(crate) capacities: Capacities,
}

struct Pending {
    table: CosetTable,
    class: u32,
    representative: bool,
}

pub(crate) struct Found {
    pub(crate) complete_to: u32,
    pub(crate) colourings: Vec<u64>,
    pub(crate) classes: Vec<u64>,
    pending: Vec<Pending>,
    class_count: u32,
    held: usize,
}

impl Found {
    fn new(limit: u32) -> Self {
        Self {
            complete_to: 0,
            colourings: vec![0; index(limit)],
            classes: vec![0; index(limit)],
            pending: Vec::new(),
            class_count: 0,
            held: 0,
        }
    }

    fn mark(&self) -> Mark {
        Mark {
            pending: self.pending.len(),
            class_count: self.class_count,
            held: self.held,
        }
    }

    fn restore(&mut self, mark: Mark, colours: u32) {
        self.pending.truncate(mark.pending);
        self.class_count = mark.class_count;
        self.held = mark.held;
        if let Some(count) = self.colourings.get_mut(index(colours) - 1) {
            *count = 0;
        }
        if let Some(count) = self.classes.get_mut(index(colours) - 1) {
            *count = 0;
        }
    }

    fn clear(&mut self) {
        self.pending = Vec::new();
        self.class_count = 0;
        self.held = 0;
        self.colourings.fill(0);
        self.classes.fill(0);
    }

    pub(crate) fn into_list(mut self) -> Vec<Colouring> {
        self.pending.sort_unstable_by(|left, right| {
            (left.table.index(), left.table.entries())
                .cmp(&(right.table.index(), right.table.entries()))
        });
        let mut class_ids = vec![ColouringId::new(0, 0); index(self.class_count)];
        let mut ranks = Ranks::default();
        for pending in &self.pending {
            let id = ranks.next(&pending.table);
            if pending.representative
                && let Some(slot) = class_ids.get_mut(index(pending.class))
            {
                *slot = id;
            }
        }
        let mut ranks = Ranks::default();
        let mut list = Vec::with_capacity(self.pending.len());
        for pending in self.pending {
            let id = ranks.next(&pending.table);
            let class = class_ids.get(index(pending.class)).copied().unwrap_or(id);
            list.push(Colouring::new(id, class, pending.table));
        }
        list
    }

    pub(crate) fn into_colourings(mut self, requested: u32) -> Colourings {
        let complete_to = self.complete_to;
        let mut counts = core::mem::take(&mut self.colourings);
        counts.truncate(index(complete_to));
        Colourings::new(requested, complete_to, self.into_list(), counts)
    }
}

#[derive(Default)]
struct Ranks {
    colours: u32,
    ordinal: u32,
}

impl Ranks {
    fn next(&mut self, table: &CosetTable) -> ColouringId {
        let colours = u32::try_from(table.index()).unwrap_or(u32::MAX);
        self.ordinal = if colours == self.colours {
            self.ordinal + 1
        } else {
            1
        };
        self.colours = colours;
        ColouringId::new(colours, self.ordinal)
    }
}

#[derive(Clone, Copy, Debug)]
struct Mark {
    pending: usize,
    class_count: u32,
    held: usize,
}

pub(crate) fn search(
    presentation: &Presentation,
    limit: u32,
    request: Request,
    settings: Settings,
) -> Result<(Found, Stats)> {
    search_with(presentation, &[], limit, request, settings)
}

pub(crate) fn search_with(
    presentation: &Presentation,
    stabilizer: &[Word],
    limit: u32,
    request: Request,
    settings: Settings,
) -> Result<(Found, Stats)> {
    check_expanded(presentation).map_err(Error::search)?;
    let prepared = build(presentation, stabilizer);
    let mut found = Found::new(limit);
    let mut stats = Stats::default();
    let keeping = request.keeping();
    if matches!(request, Request::Index | Request::Free) {
        let mut steps = 0;
        let outcome = Run::new(&prepared, limit, Rows::Exact, settings, &mut stats)
            .execute(&mut found, keeping, &mut steps, &mut stats);
        if outcome.is_ok() {
            found.complete_to = limit;
        } else {
            found.clear();
        }
        return Ok((found, stats));
    }
    let mut steps = 0;
    let first = Run::new(&prepared, limit, Rows::AtMost, settings, &mut stats)
        .execute(&mut found, keeping, &mut steps, &mut stats);
    if first.is_ok() {
        found.complete_to = limit;
        return Ok((found, stats));
    }
    stats.first_run_stopped = true;
    found.clear();
    let mut steps = 0;
    for bound in 1..=limit {
        let mark = found.mark();
        let outcome = Run::new(&prepared, bound, Rows::Exact, settings, &mut stats)
            .execute(&mut found, keeping, &mut steps, &mut stats);
        if outcome.is_err() {
            found.restore(mark, bound);
            break;
        }
        found.complete_to = bound;
    }
    Ok((found, stats))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stop;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Correct,
    Contradiction,
    Deduced,
    Open,
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    slot: u32,
    next: u32,
    trail: u32,
    rows: u32,
}

struct Run<'a> {
    prepared: &'a Prepared,
    columns: usize,
    bound: u32,
    rows_kept: Rows,
    settings: Settings,
    steps: u64,
    rows: u32,
    table: Vec<u32>,
    trail: Vec<u32>,
    stack: Vec<(u32, u16)>,
    frames: Vec<Frame>,
    label: Vec<u32>,
    order: Vec<u32>,
    scratch: Vec<u32>,
    bases: Vec<u32>,
}

impl<'a> Run<'a> {
    fn new(
        prepared: &'a Prepared,
        bound: u32,
        rows_kept: Rows,
        settings: Settings,
        stats: &mut Stats,
    ) -> Self {
        stats.runs += 1;
        let columns = prepared.columns;
        let size = index(bound) * columns;
        let pairs = (size / 2).max(1);
        Self {
            prepared,
            columns,
            bound,
            rows_kept,
            settings,
            steps: 0,
            rows: 1,
            table: vec![UNDEFINED; size],
            trail: Vec::with_capacity(size),
            stack: Vec::with_capacity(pairs),
            frames: Vec::with_capacity(pairs),
            label: vec![UNDEFINED; index(bound)],
            order: Vec::with_capacity(index(bound)),
            scratch: Vec::new(),
            bases: Vec::with_capacity(index(bound)),
        }
    }

    fn execute(
        mut self,
        found: &mut Found,
        keeping: Keeping,
        steps: &mut u64,
        stats: &mut Stats,
    ) -> core::result::Result<(), Stop> {
        self.steps = *steps;
        let before = self.steps;
        let outcome = self.traverse(found, keeping, stats);
        *steps = self.steps;
        stats.steps += self.steps - before;
        stats.run_steps.push(self.steps - before);
        stats.capacities = Capacities {
            table: self.table.capacity(),
            trail: self.trail.capacity(),
            stack: self.stack.capacity(),
            frames: self.frames.capacity(),
            label: self.label.capacity(),
            order: self.order.capacity(),
            scratch: self.scratch.capacity(),
            bases: self.bases.capacity(),
        };
        outcome
    }

    fn traverse(
        &mut self,
        found: &mut Found,
        keeping: Keeping,
        stats: &mut Stats,
    ) -> core::result::Result<(), Stop> {
        stats.nodes += 1;
        match self.next_gap(0)? {
            None => return self.leaf(found, keeping, stats),
            Some(slot) => self.frames.push(Frame {
                slot: u32::try_from(slot).unwrap_or(UNDEFINED),
                next: 0,
                trail: 0,
                rows: 1,
            }),
        }
        while let Some(&frame) = self.frames.last() {
            self.undo(frame.trail, frame.rows);
            let slot = index(frame.slot);
            let coset = u32::try_from(slot / self.columns).unwrap_or(UNDEFINED);
            let column = u16::try_from(slot % self.columns).unwrap_or(u16::MAX);
            let Some(candidate) = self.candidate(column, frame.next)? else {
                self.frames.pop();
                continue;
            };
            if let Some(top) = self.frames.last_mut() {
                top.next = candidate + 1;
            }
            if candidate == self.rows {
                self.rows += 1;
            }
            self.write(coset, column, candidate);
            if !self.process()? {
                stats.contradictions += 1;
                continue;
            }
            if self.settings.pruning == Pruning::Partial && !self.first_in_class()? {
                continue;
            }
            stats.nodes += 1;
            match self.next_gap(slot + 1)? {
                None => self.leaf(found, keeping, stats)?,
                Some(gap) => self.frames.push(Frame {
                    slot: u32::try_from(gap).unwrap_or(UNDEFINED),
                    next: 0,
                    trail: u32::try_from(self.trail.len()).unwrap_or(UNDEFINED),
                    rows: self.rows,
                }),
            }
        }
        Ok(())
    }

    fn read(&mut self, slot: usize) -> core::result::Result<u32, Stop> {
        self.steps += 1;
        if self.steps > self.settings.budget {
            return Err(Stop);
        }
        Ok(self.table.get(slot).copied().unwrap_or(UNDEFINED))
    }

    fn slot(&self, coset: u32, column: u16) -> usize {
        index(coset)
            .saturating_mul(self.columns)
            .saturating_add(usize::from(column))
    }

    fn set(&mut self, slot: usize, value: u32) {
        if let Some(entry) = self.table.get_mut(slot) {
            *entry = value;
            self.trail.push(u32::try_from(slot).unwrap_or(UNDEFINED));
        }
    }

    fn write(&mut self, coset: u32, column: u16, target: u32) {
        self.set(self.slot(coset, column), target);
        self.set(self.slot(target, column ^ 1), coset);
        self.stack.push((coset, column));
    }

    fn undo(&mut self, trail: u32, rows: u32) {
        while self.trail.len() > index(trail) {
            if let Some(slot) = self.trail.pop()
                && let Some(entry) = self.table.get_mut(index(slot))
            {
                *entry = UNDEFINED;
            }
        }
        self.rows = rows;
        self.stack.clear();
    }

    fn next_gap(&mut self, from: usize) -> core::result::Result<Option<usize>, Stop> {
        let end = index(self.rows) * self.columns;
        for slot in from..end {
            if self.read(slot)? == UNDEFINED {
                return Ok(Some(slot));
            }
        }
        Ok(None)
    }

    fn candidate(&mut self, column: u16, next: u32) -> core::result::Result<Option<u32>, Stop> {
        for target in next..self.rows {
            if self.read(self.slot(target, column ^ 1))? == UNDEFINED {
                return Ok(Some(target));
            }
        }
        if next <= self.rows && self.rows < self.bound {
            return Ok(Some(self.rows));
        }
        Ok(None)
    }

    fn scan(&mut self, coset: u32, word: &[u16]) -> core::result::Result<Outcome, Stop> {
        let mut forward = coset;
        let mut first = 0;
        while let Some(&column) = word.get(first) {
            let next = self.read(self.slot(forward, column))?;
            if next == UNDEFINED {
                break;
            }
            forward = next;
            first += 1;
        }
        if first == word.len() {
            return Ok(if forward == coset {
                Outcome::Correct
            } else {
                Outcome::Contradiction
            });
        }
        let mut backward = coset;
        let mut end = word.len();
        while end > first {
            let Some(&column) = word.get(end - 1) else {
                break;
            };
            let next = self.read(self.slot(backward, column ^ 1))?;
            if next == UNDEFINED {
                break;
            }
            backward = next;
            end -= 1;
        }
        if end == first {
            return Ok(if forward == backward {
                Outcome::Correct
            } else {
                Outcome::Contradiction
            });
        }
        if end == first + 1
            && let Some(&column) = word.get(first)
        {
            self.write(forward, column, backward);
            return Ok(Outcome::Deduced);
        }
        Ok(Outcome::Open)
    }

    fn scan_list(&mut self, coset: u32, column: u16) -> core::result::Result<bool, Stop> {
        let prepared = self.prepared;
        let Some(list) = prepared.lists.get(usize::from(column)) else {
            return Ok(true);
        };
        for span in list {
            if self.scan(coset, span.letters(&prepared.data))? == Outcome::Contradiction {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn process(&mut self) -> core::result::Result<bool, Stop> {
        while let Some((coset, column)) = self.stack.pop() {
            if !self.scan_list(coset, column)? {
                self.stack.clear();
                return Ok(false);
            }
            let image = self.read(self.slot(coset, column))?;
            if image != UNDEFINED && !self.scan_list(image, column ^ 1)? {
                self.stack.clear();
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn first_in_class(&mut self) -> core::result::Result<bool, Stop> {
        let rows = self.rows;
        'base: for base in 1..rows {
            self.start_labels(base);
            for row in 0..rows {
                let Some(&old) = self.order.get(index(row)) else {
                    continue 'base;
                };
                for column in 0..self.columns {
                    let column = u16::try_from(column).unwrap_or(u16::MAX);
                    let relabelled = self.read(self.slot(old, column))?;
                    let original = self.read(self.slot(row, column))?;
                    if relabelled == UNDEFINED || original == UNDEFINED {
                        continue 'base;
                    }
                    let label = self.label_of(relabelled);
                    if label < original {
                        return Ok(false);
                    }
                    if label > original {
                        continue 'base;
                    }
                }
            }
        }
        Ok(true)
    }

    fn start_labels(&mut self, base: u32) {
        self.label.fill(UNDEFINED);
        self.order.clear();
        if let Some(label) = self.label.get_mut(index(base)) {
            *label = 0;
            self.order.push(base);
        }
    }

    fn label_of(&mut self, coset: u32) -> u32 {
        let next = u32::try_from(self.order.len()).unwrap_or(UNDEFINED);
        match self.label.get_mut(index(coset)) {
            Some(label) if *label == UNDEFINED => {
                *label = next;
                self.order.push(coset);
                next
            }
            Some(label) => *label,
            None => UNDEFINED,
        }
    }

    fn closed(&mut self) -> core::result::Result<bool, Stop> {
        let prepared = self.prepared;
        for coset in 0..self.rows {
            for span in &prepared.relators {
                let mut current = coset;
                for &column in span.letters(&prepared.data) {
                    current = self.read(self.slot(current, column))?;
                }
                if current != coset {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn leaf(
        &mut self,
        found: &mut Found,
        keeping: Keeping,
        stats: &mut Stats,
    ) -> core::result::Result<(), Stop> {
        stats.leaves += 1;
        if self.rows_kept == Rows::Exact && self.rows != self.bound {
            return Ok(());
        }
        if self.settings.pruning == Pruning::Complete && !self.first_in_class()? {
            return Ok(());
        }
        if !self.closed()? {
            stats.leaf_discards += 1;
            return Ok(());
        }
        let bases = if self.settings.pruning == Pruning::Off || keeping == Keeping::FreeClasses {
            1
        } else {
            self.rows
        };
        self.expand(bases, stats)?;
        if keeping == Keeping::FreeClasses && !self.free()? {
            return Ok(());
        }
        self.keep(found, keeping)
    }

    fn free(&mut self) -> core::result::Result<bool, Stop> {
        let prepared = self.prepared;
        for coset in 0..self.rows {
            for word in &prepared.subgroup {
                let mut current = coset;
                for &column in word {
                    self.tally(1)?;
                    current = self
                        .scratch
                        .get(index(current) * self.columns + usize::from(column))
                        .copied()
                        .unwrap_or(UNDEFINED);
                }
                if current == coset {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn expand(&mut self, bases: u32, stats: &mut Stats) -> core::result::Result<(), Stop> {
        let size = index(self.rows) * self.columns;
        self.scratch.clear();
        let needed = index(bases) * size;
        if needed > self.scratch.capacity() {
            self.scratch.reserve_exact(needed);
        }
        self.bases.clear();
        for base in 0..bases {
            let start = self.scratch.len();
            self.relabel(base)?;
            if cfg!(test)
                && base == 0
                && self.scratch.get(start..start + size) != self.table.get(..size)
            {
                stats.relabel_changes += 1;
            }
            if !self.valid(start)? || self.repeats(start, size)? {
                self.scratch.truncate(start);
            } else {
                self.bases.push(base);
                if self.settings.set == ColouringSet::ValidClasses {
                    break;
                }
            }
        }
        Ok(())
    }

    fn valid(&mut self, start: usize) -> core::result::Result<bool, Stop> {
        if self.settings.set == ColouringSet::All {
            return Ok(true);
        }
        let generators = self.columns / 2;
        let row = self
            .scratch
            .get(start..start + self.columns)
            .unwrap_or_default();
        let fixed = fixed_generator(row);
        let reads = fixed.map_or(generators, |generator| usize::from(generator) + 1);
        self.tally(u64::try_from(reads).unwrap_or(u64::MAX))?;
        Ok(fixed.is_none())
    }

    fn repeats(&mut self, start: usize, size: usize) -> core::result::Result<bool, Stop> {
        if size == 0 {
            return Ok(start > 0);
        }
        let mut earlier = 0;
        while earlier + size <= start {
            let latest = self.scratch.get(start..start + size).unwrap_or_default();
            let common = self
                .scratch
                .get(earlier..earlier + size)
                .unwrap_or_default()
                .iter()
                .zip(latest)
                .take_while(|(left, right)| left == right)
                .count();
            self.tally(u64::try_from(2 * (common + 1).min(size)).unwrap_or(u64::MAX))?;
            if common == size {
                return Ok(true);
            }
            earlier += size;
        }
        Ok(false)
    }

    fn tally(&mut self, reads: u64) -> core::result::Result<(), Stop> {
        self.steps = self.steps.saturating_add(reads);
        if self.steps > self.settings.budget {
            return Err(Stop);
        }
        Ok(())
    }

    fn relabel(&mut self, base: u32) -> core::result::Result<(), Stop> {
        self.start_labels(base);
        let mut next = 0;
        while let Some(&old) = self.order.get(next) {
            next += 1;
            for column in 0..self.columns {
                let column = u16::try_from(column).unwrap_or(u16::MAX);
                let entry = self.read(self.slot(old, column))?;
                self.label_of(entry);
            }
        }
        for position in 0..self.order.len() {
            let old = self.order.get(position).copied().unwrap_or(UNDEFINED);
            for column in 0..self.columns {
                let column = u16::try_from(column).unwrap_or(u16::MAX);
                let entry = self.read(self.slot(old, column))?;
                let label = self.label.get(index(entry)).copied().unwrap_or(UNDEFINED);
                self.scratch.push(label);
            }
        }
        Ok(())
    }

    fn keep(&mut self, found: &mut Found, keeping: Keeping) -> core::result::Result<(), Stop> {
        if self.bases.is_empty() {
            return Ok(());
        }
        let colours = self.rows;
        let position = index(colours) - 1;
        if let Some(count) = found.colourings.get_mut(position) {
            *count += u64::try_from(self.bases.len()).unwrap_or(u64::MAX);
        }
        if let Some(count) = found.classes.get_mut(position) {
            *count += 1;
        }
        if keeping == Keeping::Counts {
            return Ok(());
        }
        let size = index(self.rows) * self.columns;
        let class = found.class_count;
        found.class_count += 1;
        for position in 0..self.bases.len() {
            if found.pending.len() >= self.settings.colourings
                || found.held + size > self.settings.entries
            {
                return Err(Stop);
            }
            let start = position * size;
            let entries = self
                .scratch
                .get(start..start + size)
                .unwrap_or_default()
                .to_vec();
            let table = CosetTable::from_entries(index(self.rows), self.columns / 2, entries);
            grow_within(&mut found.pending, 1, self.settings.colourings);
            found.pending.push(Pending {
                table,
                class,
                representative: position == 0,
            });
            found.held += size;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{prop_assert, prop_assert_eq};
    use proptest::proptest;

    use super::{
        Capacities, ColouringSet, IndexLimit, Pruning, Request, Rows, Run, SearchBudget, Settings,
        Stats, free_classes_of_index, search, search_with,
    };
    use crate::colouring::{ColouringId, fixed_generator};
    use crate::common::{
        Expected, SEARCH_BUDGET, SEARCH_FIXTURES, budget, corner_stabilizer, coxeter, letters,
        names, presentation, small_presentation, word,
    };
    use crate::enumerate::build;
    use crate::error::ErrorKind;
    use crate::presentation::Presentation;
    use crate::presentation::Relator;
    use crate::table::CosetTable;
    use crate::word::{Letter, Word};

    const NONE: u32 = u32::MAX;
    const S4: &[(&str, u32)] = &[("x", 2), ("y", 3), ("xy", 4)];

    struct PartialTable {
        source: &'static str,
        relators: &'static [(&'static str, u32)],
        rows: &'static [&'static [u32]],
        first_in_class: bool,
    }

    const PARTIAL_TABLES: &[PartialTable] = &[
        PartialTable {
            source: "Sims Example 9.3",
            relators: &[("x", 2)],
            rows: &[
                &[1, 1, 2, 3],
                &[0, 0, 1, 1],
                &[NONE, NONE, NONE, 0],
                &[NONE, NONE, 0, NONE],
            ],
            first_in_class: false,
        },
        PartialTable {
            source: "SymPy first_in_class doctest 1",
            relators: S4,
            rows: &[&[0, 0, NONE, NONE]],
            first_in_class: true,
        },
        PartialTable {
            source: "SymPy first_in_class doctest 2",
            relators: S4,
            rows: &[&[1, 1, 1, NONE], &[0, 0, NONE, 1]],
            first_in_class: true,
        },
        PartialTable {
            source: "SymPy first_in_class doctest 3",
            relators: S4,
            rows: &[&[1, 1, 2, 1], &[0, 0, 0, NONE], &[NONE, NONE, NONE, 0]],
            first_in_class: false,
        },
        PartialTable {
            source: "SymPy first_in_class doctest 4",
            relators: S4,
            rows: &[&[1, 1, 1, 2], &[0, 0, 2, 0], &[2, NONE, 0, 1]],
            first_in_class: false,
        },
    ];

    fn settings(steps: u64, pruning: Pruning) -> Settings {
        Settings::new(budget(steps)).with(pruning)
    }

    impl Settings {
        fn with(mut self, pruning: Pruning) -> Self {
            self.pruning = pruning;
            self
        }
    }

    type Listing = (u32, Vec<(ColouringId, ColouringId, CosetTable)>);

    fn listed(presentation: &Presentation, limit: u32, settings: Settings) -> (Listing, Stats) {
        let (found, stats) =
            search(presentation, limit, Request::List, settings).expect("the search runs");
        let colourings = found.into_colourings(limit);
        let list = colourings
            .list()
            .iter()
            .map(|colouring| (colouring.id(), colouring.class(), colouring.table().clone()))
            .collect();
        ((colourings.complete_to(), list), stats)
    }

    #[test]
    fn first_in_class_follows_the_printed_partial_tables() {
        for case in PARTIAL_TABLES {
            let presentation = presentation("xy", case.relators);
            let prepared = build(&presentation, &[]);
            let rows = u32::try_from(case.rows.len()).expect("few rows");
            let mut stats = Stats::default();
            let settings = Settings::new(SearchBudget::MAX);
            let mut run = Run::new(&prepared, rows, Rows::AtMost, settings, &mut stats);
            run.table = case
                .rows
                .iter()
                .flat_map(|row| row.iter().copied())
                .collect();
            run.rows = rows;
            assert_eq!(
                run.first_in_class(),
                Ok(case.first_in_class),
                "{}",
                case.source
            );
        }
    }

    #[test]
    fn search_fixtures_fit_their_presentation_and_limit() {
        for fixture in SEARCH_FIXTURES {
            let presentation = fixture.presentation();
            assert_eq!(
                presentation.generator_count(),
                fixture.generators.len(),
                "{}",
                fixture.source
            );
            let limit = usize::try_from(fixture.limit).expect("small");
            match fixture.expected {
                Expected::ClassTables(_) => {}
                Expected::Subgroups { counts, .. } => assert!(counts.len() <= limit),
                Expected::ClassCounts(counts) => assert_eq!(counts.len(), limit),
                Expected::ClassTotal(total) => assert!(total >= 1),
                Expected::FixedClasses {
                    word: text, counts, ..
                } => {
                    assert_eq!(counts.len(), limit);
                    assert!(!word(fixture.generators, text).is_empty());
                }
            }
            for table in fixture.tables() {
                assert!(
                    u32::try_from(table.len()).is_ok_and(|rows| rows <= fixture.limit),
                    "{}",
                    fixture.source
                );
                for row in *table {
                    assert_eq!(
                        row.len(),
                        2 * fixture.generators.len(),
                        "{}",
                        fixture.source
                    );
                }
            }
        }
    }

    #[test]
    fn a_pending_colouring_has_a_fixed_size() {
        assert_eq!(size_of::<super::Pending>(), 48);
    }

    #[test]
    fn index_limit_and_search_budget_check_their_range() {
        assert_eq!(IndexLimit::new(1).map(IndexLimit::get), Ok(1));
        assert_eq!(IndexLimit::new(64), Ok(IndexLimit::MAX));
        assert_eq!(SearchBudget::new(1).map(SearchBudget::get), Ok(1));
        assert_eq!(SearchBudget::new(1 << 36), Ok(SearchBudget::MAX));
        for (index, text) in [
            (
                0,
                "cannot search subgroups: the index limit 0 is not between 1 and 64",
            ),
            (
                65,
                "cannot search subgroups: the index limit 65 is not between 1 and 64",
            ),
        ] {
            let error = IndexLimit::new(index).expect_err("out of range");
            assert_eq!(
                (error.kind(), error.to_string()),
                (ErrorKind::LimitExceeded, text.to_owned())
            );
        }
        for (steps, text) in [
            (
                0,
                "cannot search subgroups: the search budget 0 is not between 1 and 68719476736",
            ),
            (
                (1 << 36) + 1,
                "cannot search subgroups: the search budget 68719476737 is not between 1 and 68719476736",
            ),
        ] {
            let error = SearchBudget::new(steps).expect_err("out of range");
            assert_eq!(
                (error.kind(), error.to_string()),
                (ErrorKind::LimitExceeded, text.to_owned())
            );
        }
    }

    #[test]
    fn the_search_refuses_more_expanded_letters_than_the_limit() {
        use crate::common::{letters, names};
        use crate::presentation::Relator;
        use crate::word::Letter;
        let relator = Relator::new(letters(&[Letter::generator(0)]), 65537).expect("valid");
        let presentation = Presentation::new(names(1), vec![relator]).expect("valid");
        let error = super::count_colourings(
            &presentation,
            ColouringSet::All,
            IndexLimit::DEFAULT,
            SearchBudget::DEFAULT,
        )
        .expect_err("over the limit");
        assert_eq!(
            (error.kind(), error.to_string()),
            (
                ErrorKind::LimitExceeded,
                "cannot search subgroups: the relators expand to 65537 letters, more than 65536"
                    .to_owned()
            )
        );
    }

    #[test]
    fn the_trivial_group_has_one_colouring() {
        let trivial = Presentation::new(Vec::new(), Vec::new()).expect("it is valid");
        let found = super::colourings(
            &trivial,
            ColouringSet::All,
            IndexLimit::new(3).expect("valid"),
            SearchBudget::DEFAULT,
        )
        .expect("the search runs");
        assert_eq!(found.complete_to(), 3);
        let ids: Vec<String> = found
            .list()
            .iter()
            .map(|colouring| colouring.id().to_string())
            .collect();
        assert_eq!(ids, ["1a"]);
        let first = found.list().first().expect("one colouring");
        assert_eq!(first.class(), first.id());
        assert_eq!(first.table().index(), 1);
        assert_eq!(first.colour_of(&[]), Some(0));
        assert_eq!(first.colour_of(&[0]), None);
        let single = super::colourings_of_index(
            &trivial,
            ColouringSet::All,
            IndexLimit::new(2).expect("valid"),
            SearchBudget::DEFAULT,
        )
        .expect("the search runs");
        assert_eq!((single.is_complete(), single.list().len()), (true, 0));
        let counts = super::count_colourings(
            &trivial,
            ColouringSet::All,
            IndexLimit::new(2).expect("valid"),
            SearchBudget::DEFAULT,
        )
        .expect("the search runs");
        assert_eq!(
            [
                counts.colourings(1),
                counts.colourings(2),
                counts.classes(1)
            ],
            [Some(1), Some(0), Some(1)]
        );
    }

    fn run_steps(presentation: &Presentation, bound: u32) -> u64 {
        let (_, stats) = search(
            presentation,
            bound,
            Request::Index,
            Settings::new(SearchBudget::MAX),
        )
        .expect("the search runs");
        stats.steps
    }

    #[test]
    fn runs_by_index_end_at_the_last_bound_that_fits_the_budget() {
        let presentation = coxeter(6, 4, 2);
        let full = listed(&presentation, 12, Settings::new(SearchBudget::MAX)).0;
        assert_eq!(full.0, 12);
        let one_run = search(
            &presentation,
            12,
            Request::Count,
            Settings::new(SearchBudget::MAX),
        )
        .expect("the search runs")
        .1
        .steps;
        let budget = one_run - 1;
        let mut spent = 0;
        let mut expected = 0;
        for bound in 1..=12 {
            spent += run_steps(&presentation, bound);
            if spent > budget {
                break;
            }
            expected = bound;
        }
        assert!(expected < 12, "{expected}");
        for request in [Request::Count, Request::List] {
            let (found, stats) = search(
                &presentation,
                12,
                request,
                settings(budget, Pruning::Partial),
            )
            .expect("the search runs");
            assert!(stats.first_run_stopped);
            assert_eq!(found.complete_to, expected);
            assert_eq!(found.colourings.len(), 12);
            for (position, &count) in found.colourings.iter().enumerate() {
                let colours = u32::try_from(position + 1).expect("small");
                let wanted = full
                    .1
                    .iter()
                    .filter(|(id, _, _)| id.colours() == colours)
                    .count();
                let wanted = if colours <= expected { wanted } else { 0 };
                assert_eq!(count, u64::try_from(wanted).expect("small"), "{colours}");
            }
        }
        let (truncated, _) = listed(&presentation, 12, settings(budget, Pruning::Partial));
        let kept: Vec<_> = full
            .1
            .iter()
            .filter(|(id, _, _)| id.colours() <= expected)
            .cloned()
            .collect();
        assert_eq!(truncated, (expected, kept));
    }

    #[test]
    fn stored_limits_stop_a_run_as_the_budget_does() {
        let presentation = coxeter(6, 4, 2);
        let full = listed(&presentation, 12, Settings::new(SearchBudget::MAX)).0;
        let columns = 6;
        for (colourings, entries) in [(100, usize::MAX), (usize::MAX, 3000), (1, 1)] {
            let mut held = (0, 0);
            let mut expected = 0;
            for colours in 1..=12 {
                for (id, _, table) in &full.1 {
                    if id.colours() == colours {
                        held.0 += 1;
                        held.1 += table.index() * columns;
                    }
                }
                if held.0 > colourings || held.1 > entries {
                    break;
                }
                expected = colours;
            }
            let limited = Settings {
                colourings,
                entries,
                ..Settings::new(SearchBudget::MAX)
            };
            let (listing, stats) = listed(&presentation, 12, limited);
            assert!(stats.first_run_stopped);
            let kept: Vec<_> = full
                .1
                .iter()
                .filter(|(id, _, _)| id.colours() <= expected)
                .cloned()
                .collect();
            assert_eq!(listing, (expected, kept), "{colourings} {entries}");
            let (counted, stats) =
                search(&presentation, 12, Request::Count, limited).expect("the search runs");
            assert!(!stats.first_run_stopped);
            assert_eq!(counted.complete_to, 12);
        }
        let limited = Settings {
            colourings: 660,
            ..Settings::new(SearchBudget::MAX)
        };
        let (found, _) =
            search(&presentation, 12, Request::Index, limited).expect("the search runs");
        assert_eq!((found.complete_to, found.into_list().len()), (0, 0));
        let limited = Settings {
            colourings: 661,
            ..Settings::new(SearchBudget::MAX)
        };
        let (found, _) =
            search(&presentation, 12, Request::Index, limited).expect("the search runs");
        assert_eq!((found.complete_to, found.into_list().len()), (12, 661));
    }

    fn dihedral() -> Presentation {
        presentation("xy", &[("x", 2), ("y", 2), ("xy", 3)])
    }

    fn is_valid(table: &CosetTable) -> bool {
        table.row(0).and_then(fixed_generator).is_none()
    }

    fn valid_listing(presentation: &Presentation, limit: u32) -> Listing {
        listed(
            presentation,
            limit,
            Settings::new(SearchBudget::DEFAULT).within(ColouringSet::Valid),
        )
        .0
    }

    #[test]
    fn a_class_whose_class_table_is_not_valid_keeps_its_valid_conjugates() {
        let mut found = 0;
        for presentation in [dihedral(), coxeter(6, 3, 2), coxeter(6, 4, 2)] {
            let all = listed(&presentation, 6, Settings::new(SearchBudget::DEFAULT)).0;
            let valid = valid_listing(&presentation, 6);
            assert_eq!((all.0, valid.0), (6, 6));
            let valid_ids: Vec<(ColouringId, ColouringId)> =
                valid.1.iter().map(|(id, class, _)| (*id, *class)).collect();
            for (id, class, table) in &all.1 {
                if id != class || is_valid(table) {
                    continue;
                }
                let members: Vec<&CosetTable> = all
                    .1
                    .iter()
                    .filter(|(_, member, member_table)| member == class && is_valid(member_table))
                    .map(|(_, _, member_table)| member_table)
                    .collect();
                if members.is_empty() {
                    continue;
                }
                found += 1;
                assert!(!valid.1.iter().any(|(_, _, kept)| kept == table));
                let classes: Vec<ColouringId> = valid
                    .1
                    .iter()
                    .filter(|(_, _, kept)| members.contains(&kept))
                    .map(|(_, kept_class, _)| *kept_class)
                    .collect();
                assert_eq!(classes.len(), members.len());
                let named = classes.first().copied().expect("a valid member");
                assert!(classes.iter().all(|&kept_class| kept_class == named));
                assert!(valid_ids.contains(&(named, named)));
            }
        }
        assert!(found > 0);
    }

    #[test]
    fn stored_limits_count_only_the_valid_colourings() {
        let presentation = coxeter(6, 4, 2);
        let mut stored = Vec::new();
        for set in [ColouringSet::Valid, ColouringSet::ValidClasses] {
            let unlimited = Settings::new(SearchBudget::MAX).within(set);
            let (found, _) =
                search(&presentation, 12, Request::Index, unlimited).expect("the search runs");
            let kept = found.into_list().len();
            assert!((2..661).contains(&kept), "{set:?} {kept}");
            stored.push(kept);
            for (colourings, expected) in [(kept, (12, kept)), (kept - 1, (0, 0))] {
                let limited = Settings {
                    colourings,
                    ..unlimited
                };
                let (found, _) =
                    search(&presentation, 12, Request::Index, limited).expect("the search runs");
                assert_eq!(
                    (found.complete_to, found.into_list().len()),
                    expected,
                    "{set:?} {colourings}"
                );
            }
        }
        assert!(stored[1] < stored[0], "{stored:?}");
    }

    fn expanded(set: ColouringSet) -> (u64, Vec<u32>, Vec<u32>) {
        let prepared = build(&dihedral(), &[]);
        let mut stats = Stats::default();
        let settings = Settings::new(SearchBudget::MAX).within(set);
        let mut run = Run::new(&prepared, 3, Rows::AtMost, settings, &mut stats);
        run.table = vec![0, 0, 1, 1, 2, 2, 0, 0, 1, 1, 2, 2];
        run.rows = 3;
        assert_eq!(run.expand(3, &mut stats), Ok(()));
        (run.steps, run.bases.clone(), run.scratch.clone())
    }

    #[test]
    fn valid_classes_stop_the_expansion_at_the_first_valid_relabelling() {
        let relabelling = 2 * 3 * 4;
        let from_1 = vec![1, 1, 2, 2, 0, 0, 1, 1, 2, 2, 0, 0];
        assert_eq!(
            expanded(ColouringSet::ValidClasses),
            (2 * relabelling + 1 + 2, vec![1], from_1.clone())
        );
        assert_eq!(
            expanded(ColouringSet::Valid),
            (3 * relabelling + 1 + 2 + 2, vec![1], from_1)
        );
    }

    #[test]
    fn capacities_stay_at_the_size_that_the_bound_sets() {
        let (_, stats) = search(
            &coxeter(6, 4, 2),
            12,
            Request::List,
            Settings::new(SearchBudget::MAX),
        )
        .expect("the search runs");
        assert_eq!(stats.runs, 1);
        assert_eq!(
            stats.capacities,
            Capacities {
                table: 72,
                trail: 72,
                stack: 36,
                frames: 36,
                label: 12,
                order: 12,
                scratch: 864,
                bases: 12,
            }
        );
    }

    proptest! {
        #[test]
        fn pruning_partial_tables_loses_no_class((presentation, limit) in small_presentation()) {
            for set in [ColouringSet::All, ColouringSet::Valid, ColouringSet::ValidClasses] {
                let partial = listed(&presentation, limit, settings(SEARCH_BUDGET, Pruning::Partial).within(set)).0;
                let complete = listed(&presentation, limit, settings(SEARCH_BUDGET, Pruning::Complete).within(set)).0;
                let unpruned = listed(&presentation, limit, settings(SEARCH_BUDGET, Pruning::Off).within(set)).0;
                if partial.0 == limit && complete.0 == limit {
                    prop_assert_eq!(&partial, &complete);
                }
                if set != ColouringSet::ValidClasses && partial.0 == limit && unpruned.0 == limit {
                    let ids = |listing: &Listing| -> Vec<(ColouringId, CosetTable)> {
                        listing.1.iter().map(|(id, _, table)| (*id, table.clone())).collect()
                    };
                    prop_assert_eq!(ids(&partial), ids(&unpruned));
                }
            }
        }

        #[test]
        fn leaf_checks_discard_nothing_and_leaves_are_standard((presentation, limit) in small_presentation()) {
            for set in [ColouringSet::All, ColouringSet::Valid, ColouringSet::ValidClasses] {
                for pruning in [Pruning::Partial, Pruning::Complete] {
                    let (_, stats) = search(&presentation, limit, Request::Count, settings(SEARCH_BUDGET, pruning).within(set))
                        .expect("the search runs");
                    prop_assert_eq!(stats.leaf_discards, 0);
                    prop_assert_eq!(stats.relabel_changes, 0);
                    prop_assert!(stats.leaves >= 1 || stats.first_run_stopped);
                }
            }
        }
    }

    fn free_listing(stabilizer: &[Word], settings: Settings) -> (u32, usize) {
        let (found, _) = search_with(&coxeter(6, 4, 2), stabilizer, 12, Request::Free, settings)
            .expect("the search runs");
        (found.complete_to, found.into_list().len())
    }

    #[test]
    fn an_empty_stabilizer_keeps_every_class_and_the_empty_word_keeps_none() {
        let settings = Settings::new(SearchBudget::MAX);
        let (all, _) =
            search(&coxeter(6, 4, 2), 12, Request::Index, settings).expect("the search runs");
        let classes = all
            .into_list()
            .iter()
            .filter(|colouring| colouring.class() == colouring.id())
            .count();
        assert_eq!(classes, 115);
        assert_eq!(free_listing(&[], settings), (12, classes));
        assert_eq!(free_listing(&[Word::new()], settings), (12, 0));
        let mut words = corner_stabilizer(6);
        words.push(Word::new());
        assert_eq!(free_listing(&words, settings), (12, 0));
    }

    #[test]
    fn stored_limits_count_only_the_kept_class_tables() {
        let words = corner_stabilizer(6);
        let unlimited = Settings::new(SearchBudget::MAX);
        let (_, kept) = free_listing(&words, unlimited);
        assert_eq!(kept, 39);
        let entries = kept * 12 * 6;
        for (colourings, entries, expected) in [
            (kept, entries, (12, kept)),
            (kept - 1, entries, (0, 0)),
            (kept, entries - 1, (0, 0)),
        ] {
            let limited = Settings {
                colourings,
                entries,
                ..unlimited
            };
            assert_eq!(
                free_listing(&words, limited),
                expected,
                "{colourings} {entries}"
            );
        }
    }

    #[test]
    fn the_free_search_checks_its_words_in_order() {
        let fine = letters(&[Letter::generator(0)]);
        let foreign = letters(&[Letter::generator(3)]);
        let long: Word = core::iter::repeat_n(Letter::generator(0), 4097).collect();
        let long_and_foreign: Word = long
            .letters()
            .iter()
            .copied()
            .chain([Letter::generator(3)])
            .collect();
        let over = Relator::new(letters(&[Letter::generator(0)]), 65537).expect("valid");
        let expanded = Presentation::new(names(3), vec![over]).expect("valid");
        let cases = [
            (
                coxeter(6, 4, 2),
                vec![foreign.clone(); 257],
                ErrorKind::LimitExceeded,
                "the stabilizer has more than 256 elements",
            ),
            (
                coxeter(6, 4, 2),
                vec![fine.clone(), long_and_foreign, foreign.clone()],
                ErrorKind::LimitExceeded,
                "stabilizer element 2 has more than 4096 letters",
            ),
            (
                coxeter(6, 4, 2),
                vec![fine.clone(), foreign.clone(), long],
                ErrorKind::InvalidWord,
                "stabilizer element 2 uses a generator that the presentation does not have",
            ),
            (
                expanded.clone(),
                vec![foreign],
                ErrorKind::InvalidWord,
                "stabilizer element 1 uses a generator that the presentation does not have",
            ),
            (
                expanded,
                vec![fine.clone()],
                ErrorKind::LimitExceeded,
                "the relators expand to 65537 letters, more than 65536",
            ),
        ];
        for (presentation, words, kind, text) in cases {
            let error = free_classes_of_index(
                &presentation,
                &words,
                IndexLimit::new(1).expect("valid"),
                SearchBudget::DEFAULT,
            )
            .expect_err("a word or the presentation is over a limit");
            assert_eq!(
                (error.kind(), error.to_string()),
                (kind, format!("cannot search subgroups: {text}"))
            );
        }
        let widest: Word = core::iter::repeat_n(Letter::generator(1), 4096).collect();
        let found = free_classes_of_index(
            &coxeter(6, 4, 2),
            &vec![widest; 256],
            IndexLimit::new(2).expect("valid"),
            SearchBudget::DEFAULT,
        )
        .expect("256 words of 4096 letters are within the limits");
        assert!(found.is_complete());
    }
}
