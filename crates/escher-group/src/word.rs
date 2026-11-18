/// A generator of a presentation or the inverse of a generator.
///
/// A letter names its generator by an index from 0 to 255, so a letter can
/// name any generator of a [`Presentation`](crate::Presentation). Letters
/// order by generator, and a generator precedes its inverse.
///
/// # Examples
///
/// ```
/// use escher_group::Letter;
///
/// let x = Letter::generator(0);
/// assert_eq!(x.inverse(), Letter::inverse_of(0));
/// assert!(x.inverse().is_inverse());
/// assert!(Letter::inverse_of(0) < Letter::generator(1));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Letter {
    column: u16,
}

impl Letter {
    /// Returns the letter of the generator with the given index.
    #[must_use]
    pub fn generator(index: u8) -> Self {
        Self {
            column: u16::from(index) * 2,
        }
    }

    /// Returns the letter of the inverse of the generator with the given
    /// index.
    #[must_use]
    pub fn inverse_of(index: u8) -> Self {
        Self {
            column: u16::from(index) * 2 + 1,
        }
    }

    /// Returns the index of the generator of the letter, whether or not the
    /// letter is an inverse.
    #[must_use]
    pub fn index(self) -> u8 {
        u8::try_from(self.column / 2).unwrap_or(u8::MAX)
    }

    /// Returns whether the letter is the inverse of a generator.
    #[must_use]
    pub fn is_inverse(self) -> bool {
        self.column % 2 == 1
    }

    /// Returns the inverse of the letter.
    ///
    /// The inverse of a generator is its inverse letter, and the inverse of an
    /// inverse letter is the generator.
    #[must_use]
    pub fn inverse(self) -> Self {
        Self {
            column: self.column ^ 1,
        }
    }

    pub(crate) fn column(self) -> u16 {
        self.column
    }
}

/// A finite sequence of letters, read from left to right.
///
/// The empty word is the identity. A word carries no generator names, so it
/// can use a generator that a given presentation lacks. An operation that
/// takes the word together with a presentation or a table reports that case
/// as an error or as `None`.
///
/// # Examples
///
/// ```
/// use escher_group::{Letter, Word};
///
/// let x = Letter::generator(0);
/// let y = Letter::generator(1);
/// let word: Word = [x, y, y.inverse(), x].into_iter().collect();
/// assert_eq!(word.freely_reduced().letters(), [x, x]);
/// assert_eq!(word.inverse().letters(), [x.inverse(), y, y.inverse(), x.inverse()]);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Word {
    letters: Vec<Letter>,
}

impl FromIterator<Letter> for Word {
    fn from_iter<T: IntoIterator<Item = Letter>>(iter: T) -> Self {
        Self {
            letters: iter.into_iter().collect(),
        }
    }
}

impl Word {
    /// Returns the empty word.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the letters of the word, in order.
    #[must_use]
    pub fn letters(&self) -> &[Letter] {
        &self.letters
    }

    /// Returns the number of letters of the word.
    #[must_use]
    pub fn len(&self) -> usize {
        self.letters.len()
    }

    /// Returns whether the word has no letter.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.letters.is_empty()
    }

    /// Returns the inverse word, which reverses the word and inverts each
    /// letter.
    #[must_use]
    pub fn inverse(&self) -> Self {
        self.letters
            .iter()
            .rev()
            .map(|letter| letter.inverse())
            .collect()
    }

    /// Returns the word with each pair of a letter and its inverse that are
    /// next to each other removed, repeating until no such pair remains.
    #[must_use]
    pub fn freely_reduced(&self) -> Self {
        let mut letters: Vec<Letter> = Vec::with_capacity(self.letters.len());
        for &letter in &self.letters {
            if letters.last() == Some(&letter.inverse()) {
                letters.pop();
            } else {
                letters.push(letter);
            }
        }
        Self { letters }
    }

    /// Returns the freely reduced word with its first and last letters
    /// removed while they are inverse to each other.
    ///
    /// The result is freely reduced, and its last letter is not the inverse
    /// of its first, so every cyclic rotation of it is also freely reduced.
    #[must_use]
    pub fn cyclically_reduced(&self) -> Self {
        let reduced = self.freely_reduced();
        let mut start = 0;
        let mut end = reduced.letters.len();
        while end - start >= 2 && reduced.letters[start].inverse() == reduced.letters[end - 1] {
            start += 1;
            end -= 1;
        }
        Self {
            letters: reduced.letters[start..end].to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Letter, Word};

    fn word(text: &str) -> Word {
        crate::common::word("abcd", text)
    }

    #[test]
    fn letter_round_trips_every_index() {
        for index in 0..=u8::MAX {
            let generator = Letter::generator(index);
            let inverse = Letter::inverse_of(index);
            assert_eq!(generator.index(), index);
            assert_eq!(inverse.index(), index);
            assert!(!generator.is_inverse());
            assert!(inverse.is_inverse());
            assert_eq!(generator.inverse(), inverse);
            assert_eq!(inverse.inverse(), generator);
            assert!(generator < inverse);
        }
        assert_eq!(Letter::inverse_of(u8::MAX).column(), 511);
    }

    #[test]
    fn freely_reduced_cancels_nested_pairs() {
        assert_eq!(word("abBAc").freely_reduced(), word("c"));
        assert_eq!(word("aA").freely_reduced(), Word::new());
        assert_eq!(word("abAB").freely_reduced(), word("abAB"));
    }

    #[test]
    fn cyclically_reduced_strips_matching_ends() {
        assert_eq!(word("abcA").cyclically_reduced(), word("bc"));
        assert_eq!(word("bcCB").cyclically_reduced(), Word::new());
        assert_eq!(word("aba").cyclically_reduced(), word("aba"));
        assert_eq!(word("Aa").cyclically_reduced(), Word::new());
        assert_eq!(word("a").cyclically_reduced(), word("a"));
    }

    #[test]
    fn inverse_reverses_and_inverts() {
        assert_eq!(word("bcD").inverse(), word("dCB"));
        assert_eq!(Word::new().inverse(), Word::new());
    }
}
