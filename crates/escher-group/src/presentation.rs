use core::fmt;
use std::collections::BTreeMap;

use crate::error::{Detail, Error, Limit, Result};
use crate::word::{Letter, Word};

pub(crate) const MAX_GENERATORS: usize = 256;
pub(crate) const MAX_RELATORS: usize = 1024;
pub(crate) const MAX_BASE: usize = 4096;
const MAX_NAME: usize = 32;

/// A word that equals the identity in a group, stored as a base word and an
/// exponent.
///
/// A relator stands for its base written out exponent times. It keeps the
/// base as given, without reducing it, and never writes it out, so a relator
/// such as `g1^4000000000` takes the space of its base.
///
/// # Examples
///
/// ```
/// use escher_group::{Letter, Relator, Word};
///
/// let base: Word = [Letter::generator(0), Letter::generator(1)].into_iter().collect();
/// let relator = Relator::new(base, 5)?;
/// assert_eq!(relator.expanded_len(), 10);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Relator {
    base: Word,
    exponent: u32,
}

impl Relator {
    /// Returns the relator whose base is `base` and whose exponent is
    /// `exponent`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] of kind
    /// [`ErrorKind::InvalidPresentation`](crate::ErrorKind::InvalidPresentation)
    /// when the base is empty or the exponent is 0, and of kind
    /// [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when the
    /// base has more than 4096 letters, checked in that order.
    pub fn new(base: Word, exponent: u32) -> Result<Self> {
        if base.is_empty() {
            return Err(Error::build(Detail::EmptyBase));
        }
        if exponent == 0 {
            return Err(Error::build(Detail::ZeroExponent));
        }
        if base.len() > MAX_BASE {
            return Err(Error::build(Detail::LimitExceeded(Limit::BaseLetters)));
        }
        Ok(Self { base, exponent })
    }

    /// Returns the base word, as given to [`Relator::new`].
    #[must_use]
    pub fn base(&self) -> &Word {
        &self.base
    }

    /// Returns the exponent, which is at least 1.
    #[must_use]
    pub fn exponent(&self) -> u32 {
        self.exponent
    }

    /// Returns the number of letters of the base written out exponent times.
    #[must_use]
    pub fn expanded_len(&self) -> u64 {
        u64::try_from(self.base.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::from(self.exponent))
    }
}

/// A list of named generators and a list of relators, which present a group.
///
/// A presentation is valid by construction: every name is a letter followed
/// by up to 31 letters and digits and differs from every other name, and
/// every letter of every relator names one of its generators. It keeps the
/// relators as given, in the order given. The presentation with no generator
/// and no relator presents the trivial group.
///
/// # Examples
///
/// ```
/// use escher_group::{Letter, Presentation, Relator, Word};
///
/// let x = Letter::generator(0);
/// let relator = Relator::new([x].into_iter().collect::<Word>(), 3)?;
/// let presentation = Presentation::new(vec!["x".to_owned()], vec![relator])?;
/// assert_eq!(presentation.generator_count(), 1);
/// let text = presentation.display_relator(&presentation.relators()[0]).map(|text| text.to_string());
/// assert_eq!(text.as_deref(), Some("x^3"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Presentation {
    names: Vec<String>,
    relators: Vec<Relator>,
}

impl Presentation {
    /// Returns the presentation with the generators named by `names`, in
    /// order, and the relators `relators`.
    ///
    /// The generator at position `k` of `names`, counting from 0, is the
    /// generator of [`Letter::generator(k)`](Letter::generator).
    ///
    /// # Errors
    ///
    /// Returns the first of these errors, checked in this order:
    ///
    /// - [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when
    ///   there are more than 256 names.
    /// - [`ErrorKind::InvalidPresentation`](crate::ErrorKind::InvalidPresentation)
    ///   for the first name, in order, that is not 1 to 32 ASCII letters and
    ///   digits starting with a letter, or that repeats an earlier name.
    /// - [`ErrorKind::LimitExceeded`](crate::ErrorKind::LimitExceeded) when
    ///   there are more than 1024 relators.
    /// - [`ErrorKind::InvalidPresentation`](crate::ErrorKind::InvalidPresentation)
    ///   for the first relator, in order, that uses a generator that the
    ///   presentation does not have.
    pub fn new(names: Vec<String>, relators: Vec<Relator>) -> Result<Self> {
        if names.len() > MAX_GENERATORS {
            return Err(Error::build(Detail::LimitExceeded(Limit::Generators)));
        }
        let mut positions: BTreeMap<&str, usize> = BTreeMap::new();
        for (position, name) in (1..).zip(&names) {
            if !is_valid_name(name) {
                return Err(Error::build(Detail::BadName(position)));
            }
            if let Some(&earlier) = positions.get(name.as_str()) {
                return Err(Error::build(Detail::RepeatedName {
                    name: position,
                    earlier,
                }));
            }
            positions.insert(name, position);
        }
        if relators.len() > MAX_RELATORS {
            return Err(Error::build(Detail::LimitExceeded(Limit::Relators)));
        }
        for (index, relator) in relators.iter().enumerate() {
            if !uses_only(relator.base(), names.len()) {
                return Err(Error::build(Detail::RelatorLetter(index + 1)));
            }
        }
        Ok(Self { names, relators })
    }

    /// Returns the number of generators.
    #[must_use]
    pub fn generator_count(&self) -> usize {
        self.names.len()
    }

    /// Returns the generator names, in the order of their indexes.
    #[must_use]
    pub fn generator_names(&self) -> &[String] {
        &self.names
    }

    /// Returns the relators, as given and in the order given.
    #[must_use]
    pub fn relators(&self) -> &[Relator] {
        &self.relators
    }

    /// Returns an adapter that writes `word` with the generator names of the
    /// presentation, or `None` when `word` uses a generator that the
    /// presentation does not have.
    ///
    /// The text is the letters separated by one space, each generator
    /// written as its name and each inverse as the name followed by `^-1`.
    /// The empty word is written `1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use escher_group::{Letter, Presentation, Word};
    ///
    /// let presentation = Presentation::new(vec!["z1".to_owned(), "w1".to_owned()], Vec::new())?;
    /// let word: Word = [Letter::generator(0), Letter::inverse_of(1)].into_iter().collect();
    /// let text = presentation.display_word(&word).map(|text| text.to_string());
    /// assert_eq!(text.as_deref(), Some("z1 w1^-1"));
    /// let text = presentation.display_word(&Word::new()).map(|text| text.to_string());
    /// assert_eq!(text.as_deref(), Some("1"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn display_word<'a>(&'a self, word: &'a Word) -> Option<WordDisplay<'a>> {
        uses_only(word, self.names.len()).then_some(WordDisplay {
            names: &self.names,
            word,
        })
    }

    /// Returns an adapter that writes `relator` with the generator names of
    /// the presentation, or `None` when `relator` uses a generator that the
    /// presentation does not have.
    ///
    /// A relator with exponent 1 is written as its base. A base of one
    /// generator with a larger exponent is written as the name, `^`, and the
    /// exponent, such as `g1^5`. Any other base with a larger exponent is
    /// written in parentheses followed by `^` and the exponent, such as
    /// `(r1 r2)^5` or `(g1^-1)^3`.
    #[must_use]
    pub fn display_relator<'a>(&'a self, relator: &'a Relator) -> Option<RelatorDisplay<'a>> {
        uses_only(relator.base(), self.names.len()).then_some(RelatorDisplay {
            names: &self.names,
            relator,
        })
    }
}

/// Writes a word with the generator names of a presentation.
///
/// [`Presentation::display_word`] returns it.
#[derive(Clone, Copy, Debug)]
pub struct WordDisplay<'a> {
    names: &'a [String],
    word: &'a Word,
}

impl fmt::Display for WordDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.word.is_empty() {
            return formatter.write_str("1");
        }
        write_letters(formatter, self.names, self.word.letters())
    }
}

/// Writes a relator with the generator names of a presentation.
///
/// [`Presentation::display_relator`] returns it.
#[derive(Clone, Copy, Debug)]
pub struct RelatorDisplay<'a> {
    names: &'a [String],
    relator: &'a Relator,
}

impl fmt::Display for RelatorDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letters = self.relator.base().letters();
        let exponent = self.relator.exponent();
        match letters {
            _ if exponent == 1 => write_letters(formatter, self.names, letters),
            [letter] if !letter.is_inverse() => {
                write_letters(formatter, self.names, letters)?;
                write!(formatter, "^{exponent}")
            }
            _ => {
                formatter.write_str("(")?;
                write_letters(formatter, self.names, letters)?;
                write!(formatter, ")^{exponent}")
            }
        }
    }
}

fn write_letters(
    formatter: &mut fmt::Formatter<'_>,
    names: &[String],
    letters: &[Letter],
) -> fmt::Result {
    for (position, letter) in letters.iter().enumerate() {
        if position > 0 {
            formatter.write_str(" ")?;
        }
        let name = names
            .get(usize::from(letter.index()))
            .map_or("?", String::as_str);
        formatter.write_str(name)?;
        if letter.is_inverse() {
            formatter.write_str("^-1")?;
        }
    }
    Ok(())
}

pub(crate) fn uses_only(word: &Word, generators: usize) -> bool {
    word.letters()
        .iter()
        .all(|letter| usize::from(letter.index()) < generators)
}

fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    match bytes {
        [first, rest @ ..] if bytes.len() <= MAX_NAME => {
            first.is_ascii_alphabetic() && rest.iter().all(u8::is_ascii_alphanumeric)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{Presentation, Relator};
    use crate::common::{letters, names};
    use crate::error::ErrorKind;
    use crate::word::{Letter, Word};

    fn relator(base: &[Letter], exponent: u32) -> Relator {
        Relator::new(letters(base), exponent).expect("the test relator is valid")
    }

    fn relator_text(presentation: &Presentation, relator: &Relator) -> String {
        presentation
            .display_relator(relator)
            .expect("the relator uses the generators of the presentation")
            .to_string()
    }

    #[test]
    fn display_relator_follows_each_rule() {
        let presentation = Presentation::new(
            ["x1", "y1", "a1", "g1", "r1", "r2", "l1"]
                .map(str::to_owned)
                .to_vec(),
            Vec::new(),
        )
        .expect("the names are valid");
        let [x1, y1, a1, g1, r1, r2, l1] = [0, 1, 2, 3, 4, 5, 6].map(Letter::generator);
        let cases = [
            (
                relator(&[x1.inverse(), y1.inverse(), x1, y1, a1.inverse()], 1),
                "x1^-1 y1^-1 x1 y1 a1^-1",
            ),
            (relator(&[l1], 1), "l1"),
            (relator(&[g1.inverse()], 1), "g1^-1"),
            (relator(&[g1], 5), "g1^5"),
            (relator(&[r1, r2], 5), "(r1 r2)^5"),
            (relator(&[g1.inverse()], 3), "(g1^-1)^3"),
            (relator(&[r1, r1, l1.inverse()], 1), "r1 r1 l1^-1"),
        ];
        for (relator, text) in cases {
            assert_eq!(relator_text(&presentation, &relator), text);
        }
    }

    #[test]
    fn display_word_writes_one_for_the_empty_word_and_spaces_between_letters() {
        let presentation = Presentation::new(names(2), Vec::new()).expect("the names are valid");
        let empty = Word::new();
        assert_eq!(
            presentation
                .display_word(&empty)
                .map(|text| text.to_string()),
            Some("1".to_owned())
        );
        let pair = letters(&[Letter::generator(1), Letter::inverse_of(0)]);
        assert_eq!(
            presentation
                .display_word(&pair)
                .map(|text| text.to_string()),
            Some("g1 g0^-1".to_owned())
        );
    }

    #[test]
    fn display_returns_none_for_a_letter_outside_the_generators() {
        let presentation = Presentation::new(names(2), Vec::new()).expect("the names are valid");
        let outside = letters(&[Letter::generator(0), Letter::inverse_of(2)]);
        assert!(presentation.display_word(&outside).is_none());
        let outside = relator(&[Letter::generator(2)], 2);
        assert!(presentation.display_relator(&outside).is_none());
    }

    #[test]
    fn relator_new_checks_the_base_and_exponent() {
        let x = Letter::generator(0);
        let kind =
            |base: Word, exponent| Relator::new(base, exponent).map_err(|error| error.kind());
        assert_eq!(kind(Word::new(), 1), Err(ErrorKind::InvalidPresentation));
        assert_eq!(kind(letters(&[x]), 0), Err(ErrorKind::InvalidPresentation));
        assert_eq!(kind(Word::new(), 0), Err(ErrorKind::InvalidPresentation));
        assert!(kind(letters(&[x; 4096]), u32::MAX).is_ok());
        assert_eq!(kind(letters(&[x; 4097]), 1), Err(ErrorKind::LimitExceeded));
        let relator = relator(&[x; 4096], u32::MAX);
        assert_eq!(relator.expanded_len(), 4096 * u64::from(u32::MAX));
    }

    #[test]
    fn relator_keeps_an_unreduced_base() {
        let x = Letter::generator(0);
        let relator = relator(&[x, x.inverse()], 3);
        assert_eq!(relator.base().letters(), [x, x.inverse()]);
        assert_eq!(relator.exponent(), 3);
    }

    #[test]
    fn presentation_new_checks_the_number_of_generators() {
        assert!(Presentation::new(names(256), Vec::new()).is_ok());
        assert_eq!(
            Presentation::new(names(257), Vec::new()).map_err(|error| error.kind()),
            Err(ErrorKind::LimitExceeded)
        );
    }

    #[test]
    fn presentation_new_checks_the_number_of_relators() {
        let x = Letter::generator(0);
        let relators = vec![relator(&[x], 2); 1024];
        assert!(Presentation::new(names(1), relators.clone()).is_ok());
        let mut over = relators;
        over.push(relator(&[x], 2));
        assert_eq!(
            Presentation::new(names(1), over).map_err(|error| error.kind()),
            Err(ErrorKind::LimitExceeded)
        );
    }

    #[test]
    fn presentation_new_checks_each_name_rule() {
        let check = |name: &str| {
            Presentation::new(vec!["a".to_owned(), name.to_owned()], Vec::new())
                .map(|_| ())
                .map_err(|error| (error.kind(), error.to_string()))
        };
        assert!(check("Z9").is_ok());
        assert!(check(&format!("b{}", "7".repeat(31))).is_ok());
        let bad = "cannot build presentation: generator name 2 is not a letter followed by up to 31 letters and digits";
        for name in [
            "",
            "9a",
            "a_b",
            "a b",
            "\u{e9}",
            "a-1",
            &format!("b{}", "7".repeat(32)),
        ] {
            assert_eq!(
                check(name),
                Err((ErrorKind::InvalidPresentation, bad.to_owned())),
                "{name:?}"
            );
        }
        assert_eq!(
            check("a"),
            Err((
                ErrorKind::InvalidPresentation,
                "cannot build presentation: generator name 2 repeats generator name 1".to_owned()
            ))
        );
    }

    #[test]
    fn presentation_new_rejects_a_relator_letter_outside_the_generators() {
        let relators = vec![
            relator(&[Letter::generator(0)], 2),
            relator(&[Letter::inverse_of(1)], 2),
        ];
        let error = Presentation::new(names(1), relators).err();
        assert_eq!(
            error.map(|error| (error.kind(), error.to_string())),
            Some((
                ErrorKind::InvalidPresentation,
                "cannot build presentation: relator 2 uses a generator that the presentation does not have"
                    .to_owned()
            ))
        );
    }

    #[test]
    fn presentation_new_checks_the_generators_before_the_relators() {
        let relators = vec![relator(&[Letter::generator(5)], 2); 1025];
        let kind = Presentation::new(vec!["1".to_owned()], relators).map_err(|error| error.kind());
        assert_eq!(kind, Err(ErrorKind::InvalidPresentation));
    }

    #[test]
    fn presentation_with_nothing_is_valid() {
        let presentation = Presentation::new(Vec::new(), Vec::new()).expect("it is valid");
        assert_eq!(presentation.generator_count(), 0);
        assert_eq!(presentation.relators(), []);
    }
}
