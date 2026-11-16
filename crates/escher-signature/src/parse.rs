use core::iter::Peekable;
use core::ops::Range;
use core::str::CharIndices;

use crate::error::{Detail, Error, Limit};
use crate::order::Order;
use crate::signature::Boundary;

pub(crate) const MAX_LENGTH: usize = 1024;
pub(crate) const MAX_SIZE: u32 = 64;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Features {
    pub(crate) handles: u32,
    pub(crate) crosscaps: u32,
    pub(crate) cone_orders: Vec<Order>,
    pub(crate) boundaries: Vec<Boundary>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(
    clippy::enum_variant_names,
    reason = "each name states the position relative to the mirror boundaries, which a shorter name would drop"
)]
enum Position {
    BeforeBoundary,
    InBoundary,
    AfterBoundary,
}

#[derive(Clone, Copy)]
enum Token {
    Value { value: u32, overflowed: bool },
    Infinite,
}

pub(crate) fn scan(text: &str) -> Result<Features, Error> {
    if text.len() > MAX_LENGTH {
        return Err(Error::new(
            Detail::LimitExceeded(Limit::Length),
            0..text.len(),
        ));
    }
    if text.is_empty() {
        return Err(Error::new(Detail::Empty, 0..0));
    }
    let mut scanner = Scanner {
        text,
        chars: text.char_indices().peekable(),
        features: Features::default(),
        size: 0,
        position: Position::BeforeBoundary,
    };
    while let Some((start, character)) = scanner.chars.next() {
        scanner.item(start, character)?;
    }
    Ok(scanner.features)
}

struct Scanner<'text> {
    text: &'text str,
    chars: Peekable<CharIndices<'text>>,
    features: Features,
    size: u32,
    position: Position,
}

impl Scanner<'_> {
    fn item(&mut self, start: usize, character: char) -> Result<(), Error> {
        match character {
            'o' | 'O' => {
                self.features.handles = self.count(self.features.handles, start)?;
                self.grow(2, start)?;
                self.leave_boundary();
            }
            'x' | 'X' => {
                self.features.crosscaps = self.count(self.features.crosscaps, start)?;
                self.grow(1, start)?;
                self.leave_boundary();
            }
            '*' => {
                self.features.boundaries.push(Boundary {
                    corners: Vec::new(),
                });
                self.grow(1, start)?;
                self.position = Position::InBoundary;
            }
            '1' if self
                .chars
                .peek()
                .is_some_and(|&(_, next)| next.is_ascii_digit()) =>
            {
                self.chars.next();
                return Err(Error::new(Detail::MultiDigitOrder, start..self.end()));
            }
            '0'..='9' => {
                let value = character.to_digit(10).unwrap_or_default();
                let token = Token::Value {
                    value,
                    overflowed: false,
                };
                let end = self.end();
                self.order(token, start..end)?;
            }
            '\u{221e}' => {
                let end = self.end();
                self.order(Token::Infinite, start..end)?;
            }
            '(' => {
                let token = self.parenthesized(start)?;
                let end = self.end();
                self.order(token, start..end)?;
            }
            other => {
                return Err(Error::new(
                    Detail::UnexpectedCharacter(other),
                    start..self.end(),
                ));
            }
        }
        Ok(())
    }

    fn end(&mut self) -> usize {
        self.chars
            .peek()
            .map_or(self.text.len(), |&(index, _)| index)
    }

    fn count(&mut self, count: u32, start: usize) -> Result<u32, Error> {
        count.checked_add(1).ok_or_else(|| self.size_error(start))
    }

    fn grow(&mut self, weight: u32, start: usize) -> Result<(), Error> {
        match self.size.checked_add(weight) {
            Some(size) if size <= MAX_SIZE => {
                self.size = size;
                Ok(())
            }
            _ => Err(self.size_error(start)),
        }
    }

    fn size_error(&mut self, start: usize) -> Error {
        Error::new(Detail::LimitExceeded(Limit::Size), start..self.end())
    }

    fn leave_boundary(&mut self) {
        if self.position == Position::InBoundary {
            self.position = Position::AfterBoundary;
        }
    }

    fn require(&mut self, start: usize, wanted: char) -> Result<(), Error> {
        match self.chars.next() {
            Some((_, found)) if found == wanted => Ok(()),
            Some((index, found)) => Err(Error::new(
                Detail::UnexpectedCharacter(found),
                index..self.end(),
            )),
            None => Err(Error::new(
                Detail::UnclosedParenthesis,
                start..self.text.len(),
            )),
        }
    }

    fn parenthesized(&mut self, start: usize) -> Result<Token, Error> {
        let token = match self.chars.peek() {
            Some(&(_, digit)) if digit.is_ascii_digit() => self.digits(),
            Some(&(_, 'i')) => {
                self.chars.next();
                self.require(start, 'n')?;
                self.require(start, 'f')?;
                Token::Infinite
            }
            Some(&(index, other)) => {
                self.chars.next();
                return Err(Error::new(
                    Detail::UnexpectedCharacter(other),
                    index..self.end(),
                ));
            }
            None => {
                return Err(Error::new(
                    Detail::UnclosedParenthesis,
                    start..self.text.len(),
                ));
            }
        };
        self.require(start, ')')?;
        Ok(token)
    }

    fn digits(&mut self) -> Token {
        let mut value: u32 = 0;
        let mut overflowed = false;
        while let Some(digit) = self
            .chars
            .next_if(|&(_, next)| next.is_ascii_digit())
            .and_then(|(_, next)| next.to_digit(10))
        {
            match value
                .checked_mul(10)
                .and_then(|shifted| shifted.checked_add(digit))
            {
                Some(next) => value = next,
                None => overflowed = true,
            }
        }
        Token::Value { value, overflowed }
    }

    fn order(&mut self, token: Token, span: Range<usize>) -> Result<(), Error> {
        let order = match token {
            Token::Value {
                overflowed: true, ..
            } => return Err(Error::new(Detail::LimitExceeded(Limit::Order), span)),
            Token::Value { value: 0, .. } => return Err(Error::new(Detail::ZeroOrder, span)),
            Token::Value { value, .. } => match Order::from_value(value) {
                Some(order) => order,
                None => return Ok(()),
            },
            Token::Infinite => Order::INFINITE,
        };
        if self.position == Position::AfterBoundary {
            return Err(Error::new(Detail::AmbiguousOrder(order), span));
        }
        self.grow(1, span.start)?;
        match self.features.boundaries.last_mut() {
            Some(boundary) if self.position == Position::InBoundary => boundary.corners.push(order),
            Some(_) | None => self.features.cone_orders.push(order),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_LENGTH, MAX_SIZE, scan};
    use crate::ErrorKind;
    use crate::order::Order;

    fn failure(text: &str) -> (ErrorKind, core::ops::Range<usize>) {
        let error = scan(text).expect_err("the text is rejected");
        (error.kind(), error.span())
    }

    #[test]
    fn scan_reports_the_kind_and_span_of_the_first_error() {
        let cases = [
            ("", ErrorKind::Empty, 0..0),
            ("*7q2", ErrorKind::UnexpectedCharacter, 2..3),
            ("*7 32", ErrorKind::UnexpectedCharacter, 2..3),
            ("()", ErrorKind::UnexpectedCharacter, 1..2),
            ("(INF)", ErrorKind::UnexpectedCharacter, 1..2),
            ("(in)", ErrorKind::UnexpectedCharacter, 3..4),
            ("(\u{221e})", ErrorKind::UnexpectedCharacter, 1..4),
            ("(12a)", ErrorKind::UnexpectedCharacter, 3..4),
            ("(12", ErrorKind::UnclosedParenthesis, 0..3),
            ("2(in", ErrorKind::UnclosedParenthesis, 1..4),
            ("(", ErrorKind::UnclosedParenthesis, 0..1),
            ("0", ErrorKind::ZeroOrder, 0..1),
            ("(00)", ErrorKind::ZeroOrder, 0..4),
            ("11", ErrorKind::MultiDigitOrder, 0..2),
            ("*11", ErrorKind::MultiDigitOrder, 1..3),
            ("2*12", ErrorKind::MultiDigitOrder, 2..4),
            ("*2x3", ErrorKind::AmbiguousOrder, 3..4),
            ("*o2", ErrorKind::AmbiguousOrder, 2..3),
            ("*x(inf)", ErrorKind::AmbiguousOrder, 2..7),
            ("(4294967296)", ErrorKind::LimitExceeded, 0..12),
        ];
        for (text, kind, span) in cases {
            assert_eq!(failure(text), (kind, span), "{text:?}");
        }
    }

    #[test]
    fn scan_accepts_the_largest_order() {
        let features = scan("(4294967295)").expect("the largest order is accepted");
        assert_eq!(
            features.cone_orders,
            [Order::from_value(u32::MAX).expect("valid")]
        );
    }

    #[test]
    fn scan_rejects_text_longer_than_the_length_limit() {
        let longest = "x".repeat(MAX_LENGTH);
        assert_eq!(failure(&longest).0, ErrorKind::LimitExceeded);
        assert_eq!(failure(&longest).1, 64..65);
        let too_long = "x".repeat(MAX_LENGTH + 1);
        assert_eq!(
            failure(&too_long),
            (ErrorKind::LimitExceeded, 0..MAX_LENGTH + 1)
        );
    }

    #[test]
    fn scan_rejects_the_feature_after_the_size_limit() {
        let size = usize::try_from(MAX_SIZE).expect("the limit fits usize");
        assert!(scan(&"2".repeat(size)).is_ok());
        assert_eq!(
            failure(&"2".repeat(size + 1)),
            (ErrorKind::LimitExceeded, size..size + 1)
        );
        assert!(scan(&"o".repeat(size / 2)).is_ok());
        assert_eq!(
            failure(&"o".repeat(size / 2 + 1)),
            (ErrorKind::LimitExceeded, size / 2..size / 2 + 1)
        );
        let parenthesized = format!("{}(12)", "*".repeat(size));
        assert_eq!(
            failure(&parenthesized),
            (ErrorKind::LimitExceeded, size..size + 4)
        );
    }

    #[test]
    fn scan_drops_orders_of_one_without_counting_them() {
        let size = usize::try_from(MAX_SIZE).expect("the limit fits usize");
        let text = format!("{}(1)1*1", "2".repeat(size - 1));
        let features = scan(&text).expect("orders of 1 do not count");
        assert_eq!(features.cone_orders.len(), size - 1);
        assert_eq!(features.boundaries.len(), 1);
        assert!(
            features
                .boundaries
                .iter()
                .all(|boundary| boundary.corners.is_empty())
        );
    }

    #[test]
    fn scan_drops_an_order_of_one_after_a_boundary_and_a_crosscap() {
        let features = scan("*2x1").expect("an order of 1 is never ambiguous");
        assert_eq!(features.crosscaps, 1);
        assert_eq!(features.boundaries.len(), 1);
    }

    #[test]
    fn scan_reads_orders_before_the_first_boundary_as_cone_points() {
        let features = scan("2o3*4x*5").expect("valid");
        let values = |orders: &[Order]| {
            orders
                .iter()
                .filter_map(|order| order.finite())
                .collect::<Vec<_>>()
        };
        assert_eq!(values(&features.cone_orders), [2, 3]);
        let corners = features
            .boundaries
            .iter()
            .map(|boundary| values(&boundary.corners))
            .collect::<Vec<_>>();
        assert_eq!(corners, [vec![4], vec![5]]);
        assert_eq!((features.handles, features.crosscaps), (1, 1));
    }
}
