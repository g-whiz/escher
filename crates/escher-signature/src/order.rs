use core::fmt;

/// The order of a cone point or a corner point: an integer of at least 2,
/// or infinite.
///
/// Orders compare by value, and the infinite order compares above every
/// finite order.
///
/// # Examples
///
/// ```
/// use escher_signature::Signature;
///
/// let signature: Signature = "*22(inf)".parse()?;
/// let corners = signature.boundaries()[0].corner_orders();
/// assert!(corners[0].is_infinite());
/// assert_eq!(corners[1].finite(), Some(2));
/// assert!(corners[0] > corners[1]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Order(Value);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Value {
    Finite(u32),
    Infinite,
}

impl Order {
    pub(crate) const INFINITE: Self = Self(Value::Infinite);

    pub(crate) fn from_value(value: u32) -> Option<Self> {
        (value >= 2).then_some(Self(Value::Finite(value)))
    }

    /// Returns the order as an integer, or `None` when it is infinite.
    ///
    /// A finite order is at least 2.
    #[must_use]
    pub fn finite(self) -> Option<u32> {
        match self.0 {
            Value::Finite(value) => Some(value),
            Value::Infinite => None,
        }
    }

    /// Returns whether the order is infinite.
    #[must_use]
    pub fn is_infinite(self) -> bool {
        self.0 == Value::Infinite
    }

    pub(crate) fn word(self) -> Word {
        Word(self)
    }
}

/// Writes one digit for an order below 10, `(n)` for an order of 10 or more,
/// and `(inf)` for an infinite order.
impl fmt::Display for Order {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Value::Finite(value) if value < 10 => write!(formatter, "{value}"),
            Value::Finite(value) => write!(formatter, "({value})"),
            Value::Infinite => formatter.write_str("(inf)"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Word(Order);

impl fmt::Display for Word {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.0 {
            Value::Finite(value) => write!(formatter, "{value}"),
            Value::Infinite => formatter.write_str("inf"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Order;

    #[test]
    fn display_writes_one_digit_below_ten_and_parentheses_from_ten() {
        let text = [2, 9, 10, 4_294_967_295]
            .into_iter()
            .filter_map(Order::from_value)
            .map(|order| order.to_string())
            .collect::<Vec<_>>();
        assert_eq!(text, ["2", "9", "(10)", "(4294967295)"]);
        assert_eq!(Order::INFINITE.to_string(), "(inf)");
    }

    #[test]
    fn from_value_rejects_orders_below_two() {
        assert_eq!(Order::from_value(0), None);
        assert_eq!(Order::from_value(1), None);
        assert_eq!(Order::from_value(2).and_then(Order::finite), Some(2));
    }

    #[test]
    fn infinite_order_compares_above_every_finite_order() {
        let largest = Order::from_value(u32::MAX);
        assert!(largest.is_some_and(|order| Order::INFINITE > order));
        assert!(Order::INFINITE.is_infinite());
        assert_eq!(Order::INFINITE.finite(), None);
    }

    #[test]
    fn word_writes_the_bare_value() {
        assert_eq!(Order::INFINITE.word().to_string(), "inf");
        let twelve = Order::from_value(12).map(|order| order.word().to_string());
        assert_eq!(twelve.as_deref(), Some("12"));
    }
}
