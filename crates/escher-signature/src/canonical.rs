use core::cmp::Reverse;

use crate::order::Order;
use crate::parse::Features;
use crate::signature::Boundary;

pub(crate) fn canonicalize(features: Features) -> Option<Features> {
    let Features {
        mut handles,
        mut crosscaps,
        mut cone_orders,
        boundaries,
    } = features;
    let boundaries = boundaries
        .into_iter()
        .map(|boundary| boundary.corners)
        .collect::<Vec<_>>();
    if crosscaps > 0 {
        crosscaps = handles
            .checked_mul(2)
            .and_then(|traded| traded.checked_add(crosscaps))?;
        handles = 0;
    }
    cone_orders.sort_unstable_by_key(|&order| Reverse(order));
    let boundaries = if crosscaps > 0 {
        let mut each = boundaries
            .iter()
            .map(|corners| {
                let reversed = corners.iter().rev().copied().collect::<Vec<_>>();
                largest_rotation(corners).max(largest_rotation(&reversed))
            })
            .collect::<Vec<_>>();
        sort_descending(&mut each);
        each
    } else {
        let mut forward = boundaries
            .iter()
            .map(|corners| largest_rotation(corners))
            .collect::<Vec<_>>();
        let mut backward = boundaries
            .iter()
            .map(|corners| {
                let reversed = corners.iter().rev().copied().collect::<Vec<_>>();
                largest_rotation(&reversed)
            })
            .collect::<Vec<_>>();
        sort_descending(&mut forward);
        sort_descending(&mut backward);
        forward.max(backward)
    };
    Some(Features {
        handles,
        crosscaps,
        cone_orders,
        boundaries: boundaries
            .into_iter()
            .map(|corners| Boundary { corners })
            .collect(),
    })
}

fn largest_rotation(corners: &[Order]) -> Vec<Order> {
    (0..corners.len())
        .map(|shift| {
            corners
                .iter()
                .skip(shift)
                .chain(corners.iter().take(shift))
                .copied()
                .collect::<Vec<_>>()
        })
        .max()
        .unwrap_or_default()
}

fn sort_descending(boundaries: &mut [Vec<Order>]) {
    boundaries.sort_unstable_by(|first, second| second.cmp(first));
}

#[cfg(test)]
mod tests {
    use super::{canonicalize, largest_rotation};
    use crate::order::Order;
    use crate::parse::Features;

    fn orders(values: &[u32]) -> Vec<Order> {
        values
            .iter()
            .map(|&value| Order::from_value(value).unwrap_or(Order::INFINITE))
            .collect()
    }

    #[test]
    fn largest_rotation_is_the_greatest_cyclic_shift() {
        assert_eq!(largest_rotation(&orders(&[2, 3, 6])), orders(&[6, 2, 3]));
        assert_eq!(
            largest_rotation(&orders(&[2, 6, 2, 6])),
            orders(&[6, 2, 6, 2])
        );
        assert_eq!(largest_rotation(&[]), Vec::new());
    }

    #[test]
    fn canonicalize_trades_every_handle_for_two_crosscaps_with_a_crosscap() {
        let features = Features {
            handles: 2,
            crosscaps: 1,
            ..Features::default()
        };
        let canonical = canonicalize(features).expect("small counts fit");
        assert_eq!((canonical.handles, canonical.crosscaps), (0, 5));
    }

    #[test]
    fn canonicalize_fails_when_the_crosscap_count_does_not_fit() {
        let features = Features {
            handles: u32::MAX,
            crosscaps: 1,
            ..Features::default()
        };
        assert_eq!(canonicalize(features), None);
    }
}
