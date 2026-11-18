use core::fmt;

/// A specialized [`Result`](core::result::Result) whose error is [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

const BUILD: &str = "build presentation";
const ENUMERATE: &str = "enumerate cosets";
const SEARCH: &str = "search subgroups";
const READ_ID: &str = "read colouring id";

/// The reason why a presentation could not be built, its cosets could not be
/// enumerated, its subgroups could not be searched, or a colouring id could
/// not be read.
///
/// Match on [`Error::kind`] to tell the causes apart. The
/// [`Display`](core::fmt::Display) text starts with
/// `cannot build presentation: `, `cannot enumerate cosets: `,
/// `cannot search subgroups: `, or `cannot read colouring id: ` and is meant
/// for a person, not for matching. It shows positions, counted from 1,
/// numbers, and the text of an unsupported signature, and never a generator
/// name or a word.
///
/// # Examples
///
/// ```
/// use escher_group::{ErrorKind, Relator, Word};
///
/// let error = Relator::new(Word::new(), 2).err();
/// assert_eq!(error.as_ref().map(|error| error.kind()), Some(ErrorKind::InvalidPresentation));
/// assert_eq!(
///     error.map(|error| error.to_string()).as_deref(),
///     Some("cannot build presentation: the relator base is empty"),
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("cannot {operation}: {detail}")]
pub struct Error {
    operation: &'static str,
    detail: Detail,
}

impl Error {
    pub(crate) fn build(detail: Detail) -> Self {
        Self {
            operation: BUILD,
            detail,
        }
    }

    pub(crate) fn enumerate(detail: Detail) -> Self {
        Self {
            operation: ENUMERATE,
            detail,
        }
    }

    pub(crate) fn search(detail: Detail) -> Self {
        Self {
            operation: SEARCH,
            detail,
        }
    }

    pub(crate) fn read_id(detail: Detail) -> Self {
        Self {
            operation: READ_ID,
            detail,
        }
    }

    /// Returns the cause of the error as an [`ErrorKind`].
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self.detail {
            Detail::EmptyBase
            | Detail::ZeroExponent
            | Detail::BadName(_)
            | Detail::RepeatedName { .. }
            | Detail::RelatorLetter(_) => ErrorKind::InvalidPresentation,
            Detail::SubgroupLetter(_) => ErrorKind::InvalidWord,
            Detail::LimitExceeded(_) => ErrorKind::LimitExceeded,
            Detail::CosetLimitReached(_) => ErrorKind::CosetLimitReached,
            Detail::UnsupportedSignature(_) => ErrorKind::UnsupportedSignature,
            Detail::InvalidColouringId => ErrorKind::InvalidColouringId,
        }
    }
}

/// The cause of an [`Error`], for a caller to match on.
///
/// The enum is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
///
/// # Examples
///
/// ```
/// use escher_group::{CosetLimit, ErrorKind};
///
/// let kind = CosetLimit::new(0).map_err(|error| error.kind());
/// assert_eq!(kind, Err(ErrorKind::LimitExceeded));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A relator has an empty base or an exponent of 0, a generator name is
    /// not a letter followed by up to 31 letters and digits or repeats an
    /// earlier name, or a relator uses a generator that the presentation does
    /// not have.
    InvalidPresentation,
    /// A subgroup generator uses a generator that the presentation does not
    /// have.
    InvalidWord,
    /// An input is over one of the limits of the crate, or a coset limit, an
    /// index limit, or a search budget is out of its range.
    ///
    /// The limits are 256 generators and 1024 relators in a presentation,
    /// 4096 letters in a relator base, 256 subgroup generators of at most
    /// 4096 letters each, 65536 letters in the relators written out with
    /// their exponents, and 16777216 table entries, which is the coset limit
    /// times twice the number of generators. A coset limit is between 1 and
    /// [`CosetLimit::MAX`](crate::CosetLimit::MAX), an index limit between 1
    /// and [`IndexLimit::MAX`](crate::IndexLimit::MAX), and a search budget
    /// between 1 and [`SearchBudget::MAX`](crate::SearchBudget::MAX).
    LimitExceeded,
    /// The enumeration defined as many cosets as its limit allows and needed
    /// another.
    ///
    /// A larger limit may or may not let the enumeration finish.
    CosetLimitReached,
    /// The signature has a cone point or a corner of infinite order, for
    /// which the crate builds no presentation.
    UnsupportedSignature,
    /// A text is not a colouring id: a colour count from 1 to 64 in decimal
    /// without a leading zero, followed by 1 to 4 lower-case letters.
    InvalidColouringId,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum Detail {
    #[error("the relator base is empty")]
    EmptyBase,
    #[error("the relator exponent is 0")]
    ZeroExponent,
    #[error("generator name {0} is not a letter followed by up to 31 letters and digits")]
    BadName(usize),
    #[error("generator name {name} repeats generator name {earlier}")]
    RepeatedName { name: usize, earlier: usize },
    #[error("relator {0} uses a generator that the presentation does not have")]
    RelatorLetter(usize),
    #[error("subgroup generator {0} uses a generator that the presentation does not have")]
    SubgroupLetter(usize),
    #[error("{0}")]
    LimitExceeded(Limit),
    #[error("the enumeration defined {0} cosets without finishing")]
    CosetLimitReached(u32),
    #[error("the signature {0} has an infinite order")]
    UnsupportedSignature(String),
    #[error(
        "the colouring id is not a colour count from 1 to 64 followed by 1 to 4 lower-case letters"
    )]
    InvalidColouringId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Limit {
    BaseLetters,
    Generators,
    Relators,
    SubgroupGenerators,
    SubgroupLetters(usize),
    Expanded(u64),
    Cosets(u32),
    TableEntries { limit: u32, columns: usize },
    Index(u32),
    Budget(u64),
}

impl fmt::Display for Limit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::enumerate::{MAX_EXPANDED, MAX_SUBGROUP_GENERATORS, MAX_TABLE_ENTRIES};
        use crate::presentation::{MAX_BASE, MAX_GENERATORS, MAX_RELATORS};
        use crate::search::{MAX_INDEX, MAX_STEPS};
        use crate::table::MAX_COSETS;
        match *self {
            Self::BaseLetters => write!(
                formatter,
                "the relator base has more than {MAX_BASE} letters"
            ),
            Self::Generators => {
                write!(
                    formatter,
                    "the presentation has more than {MAX_GENERATORS} generators"
                )
            }
            Self::Relators => {
                write!(
                    formatter,
                    "the presentation has more than {MAX_RELATORS} relators"
                )
            }
            Self::SubgroupGenerators => write!(
                formatter,
                "the subgroup has more than {MAX_SUBGROUP_GENERATORS} generators"
            ),
            Self::SubgroupLetters(position) => write!(
                formatter,
                "subgroup generator {position} has more than {MAX_BASE} letters"
            ),
            Self::Expanded(letters) => write!(
                formatter,
                "the relators expand to {letters} letters, more than {MAX_EXPANDED}"
            ),
            Self::Cosets(cosets) => write!(
                formatter,
                "the coset limit {cosets} is not between 1 and {MAX_COSETS}"
            ),
            Self::TableEntries { limit, columns } => write!(
                formatter,
                "the coset limit {limit} times {columns} columns is more than {MAX_TABLE_ENTRIES} table entries"
            ),
            Self::Index(index) => write!(
                formatter,
                "the index limit {index} is not between 1 and {MAX_INDEX}"
            ),
            Self::Budget(steps) => write!(
                formatter,
                "the search budget {steps} is not between 1 and {MAX_STEPS}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Detail, Error, ErrorKind, Limit};

    fn every_error() -> Vec<(Error, ErrorKind, &'static str)> {
        let mut errors = vec![
            (
                Error::build(Detail::EmptyBase),
                ErrorKind::InvalidPresentation,
                "cannot build presentation: the relator base is empty",
            ),
            (
                Error::build(Detail::ZeroExponent),
                ErrorKind::InvalidPresentation,
                "cannot build presentation: the relator exponent is 0",
            ),
            (
                Error::build(Detail::BadName(3)),
                ErrorKind::InvalidPresentation,
                "cannot build presentation: generator name 3 is not a letter followed by up to 31 letters and digits",
            ),
            (
                Error::build(Detail::RepeatedName {
                    name: 4,
                    earlier: 2,
                }),
                ErrorKind::InvalidPresentation,
                "cannot build presentation: generator name 4 repeats generator name 2",
            ),
            (
                Error::build(Detail::RelatorLetter(3)),
                ErrorKind::InvalidPresentation,
                "cannot build presentation: relator 3 uses a generator that the presentation does not have",
            ),
            (
                Error::enumerate(Detail::SubgroupLetter(2)),
                ErrorKind::InvalidWord,
                "cannot enumerate cosets: subgroup generator 2 uses a generator that the presentation does not have",
            ),
            (
                Error::build(Detail::LimitExceeded(Limit::BaseLetters)),
                ErrorKind::LimitExceeded,
                "cannot build presentation: the relator base has more than 4096 letters",
            ),
            (
                Error::build(Detail::LimitExceeded(Limit::Generators)),
                ErrorKind::LimitExceeded,
                "cannot build presentation: the presentation has more than 256 generators",
            ),
            (
                Error::build(Detail::LimitExceeded(Limit::Relators)),
                ErrorKind::LimitExceeded,
                "cannot build presentation: the presentation has more than 1024 relators",
            ),
            (
                Error::enumerate(Detail::LimitExceeded(Limit::SubgroupGenerators)),
                ErrorKind::LimitExceeded,
                "cannot enumerate cosets: the subgroup has more than 256 generators",
            ),
            (
                Error::enumerate(Detail::LimitExceeded(Limit::SubgroupLetters(2))),
                ErrorKind::LimitExceeded,
                "cannot enumerate cosets: subgroup generator 2 has more than 4096 letters",
            ),
            (
                Error::enumerate(Detail::LimitExceeded(Limit::Expanded(120_011))),
                ErrorKind::LimitExceeded,
                "cannot enumerate cosets: the relators expand to 120011 letters, more than 65536",
            ),
            (
                Error::enumerate(Detail::LimitExceeded(Limit::Cosets(0))),
                ErrorKind::LimitExceeded,
                "cannot enumerate cosets: the coset limit 0 is not between 1 and 1048576",
            ),
            (
                Error::enumerate(Detail::LimitExceeded(Limit::TableEntries {
                    limit: 1_048_576,
                    columns: 32,
                })),
                ErrorKind::LimitExceeded,
                "cannot enumerate cosets: the coset limit 1048576 times 32 columns is more than 16777216 table entries",
            ),
            (
                Error::enumerate(Detail::CosetLimitReached(16384)),
                ErrorKind::CosetLimitReached,
                "cannot enumerate cosets: the enumeration defined 16384 cosets without finishing",
            ),
            (
                Error::build(Detail::UnsupportedSignature("*(inf)22".to_owned())),
                ErrorKind::UnsupportedSignature,
                "cannot build presentation: the signature *(inf)22 has an infinite order",
            ),
        ];
        errors.extend(search_errors());
        errors
    }

    fn search_errors() -> Vec<(Error, ErrorKind, &'static str)> {
        vec![
            (
                Error::search(Detail::LimitExceeded(Limit::Index(0))),
                ErrorKind::LimitExceeded,
                "cannot search subgroups: the index limit 0 is not between 1 and 64",
            ),
            (
                Error::search(Detail::LimitExceeded(Limit::Budget(0))),
                ErrorKind::LimitExceeded,
                "cannot search subgroups: the search budget 0 is not between 1 and 68719476736",
            ),
            (
                Error::search(Detail::LimitExceeded(Limit::Expanded(120_011))),
                ErrorKind::LimitExceeded,
                "cannot search subgroups: the relators expand to 120011 letters, more than 65536",
            ),
            (
                Error::read_id(Detail::InvalidColouringId),
                ErrorKind::InvalidColouringId,
                "cannot read colouring id: the colouring id is not a colour count from 1 to 64 followed by 1 to 4 lower-case letters",
            ),
        ]
    }

    #[test]
    fn display_writes_the_operation_and_the_text_of_each_cause() {
        for (error, _, text) in every_error() {
            assert_eq!(error.to_string(), text);
        }
    }

    #[test]
    fn display_starts_with_an_operation_and_is_lower_case() {
        for (error, _, _) in every_error() {
            let shown = error.to_string();
            assert!(
                [
                    "cannot build presentation: ",
                    "cannot enumerate cosets: ",
                    "cannot search subgroups: ",
                    "cannot read colouring id: ",
                ]
                .iter()
                .any(|prefix| shown.starts_with(prefix)),
                "{shown}"
            );
            assert_eq!(shown, shown.to_lowercase(), "{shown}");
            assert!(!shown.ends_with('.'), "{shown}");
        }
    }

    #[test]
    fn kind_matches_each_cause() {
        for (error, kind, _) in every_error() {
            assert_eq!(error.kind(), kind);
        }
    }

    #[test]
    fn source_is_none() {
        let error = Error::build(Detail::EmptyBase);
        assert!(core::error::Error::source(&error).is_none());
    }
}
