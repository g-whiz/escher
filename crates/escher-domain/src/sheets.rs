use core::f64::consts::PI;
use std::collections::VecDeque;

use escher_geometry::hyperbolic::{ORIGIN, boost, distance, form, rotation, wall_through};
use glam::DVec3;

use crate::parameters::Parameter;

pub(crate) const SHORTEST: f64 = 0.5;

pub(crate) const LONGEST: f64 = 3.5;

const BISECTION_STEPS: u32 = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Corner,
    CornerPair(usize),
    Slice(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Feature {
    pub(crate) kind: Kind,
    pub(crate) first: usize,
    pub(crate) last: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Cuts {
    pub(crate) count: usize,
    pub(crate) pairs: Vec<Feature>,
    pub(crate) slices: Vec<Feature>,
    pub(crate) sheets: Vec<[Feature; 3]>,
}

impl Cuts {
    pub(crate) fn of(orders: &[u32]) -> Self {
        let count = orders.len();
        let mut list = VecDeque::with_capacity(count);
        let mut pairs = Vec::new();
        let mut corner = 0;
        while let Some(&order) = orders.get(corner) {
            if order == 2 && orders.get(corner + 1) == Some(&2) {
                let pair = Feature {
                    kind: Kind::CornerPair(pairs.len()),
                    first: corner,
                    last: corner + 1,
                };
                pairs.push(pair);
                list.push_back(pair);
                corner += 2;
            } else {
                list.push_back(Feature {
                    kind: Kind::Corner,
                    first: corner,
                    last: corner,
                });
                corner += 1;
            }
        }
        let mut slices = Vec::new();
        let mut sheets = Vec::new();
        while list.len() > 3 {
            let (Some(x), Some(y)) = (list.pop_back(), list.pop_back()) else {
                break;
            };
            let kind = Kind::Slice(slices.len());
            let slice = Feature {
                kind,
                first: y.first,
                last: x.last,
            };
            let complement = Feature {
                kind,
                first: (x.last + 1) % count,
                last: (y.first + count - 1) % count,
            };
            sheets.push([y, x, complement]);
            slices.push(slice);
            list.push_front(slice);
        }
        if let (Some(&first), Some(&second), Some(&third)) =
            (list.front(), list.get(1), list.get(2))
        {
            sheets.push([first, second, third]);
        }
        Self {
            count,
            pairs,
            slices,
            sheets,
        }
    }

    pub(crate) fn parameters(&self) -> Vec<(Feature, Parameter)> {
        let pairs = (1..)
            .zip(&self.pairs)
            .map(|(number, &pair)| (pair, Parameter::CornerPair(number)));
        let slices = (1..)
            .zip(&self.slices)
            .map(|(number, &slice)| (slice, Parameter::Slice(number)));
        pairs.chain(slices).collect()
    }

    fn value_index(&self, kind: Kind) -> Option<usize> {
        self.parameters()
            .iter()
            .position(|(feature, _)| feature.kind == kind)
    }

    fn pair(&self, side: usize) -> bool {
        self.pairs.iter().any(|pair| pair.last == side)
    }
}

pub(crate) fn side_lengths(orders: &[u32], cuts: &Cuts, values: &[f64]) -> Option<Vec<f64>> {
    let count = cuts.count;
    if orders.len() != count {
        return None;
    }
    let value = |kind: Kind| values.get(cuts.value_index(kind)?).copied();
    let trig = |feature: Feature| -> Option<(bool, f64, f64)> {
        if feature.kind == Kind::Corner {
            let angle = PI / f64::from(orders[feature.first]);
            return Some((true, angle.cos(), angle.sin()));
        }
        let length = value(feature.kind)?;
        Some((false, length.cosh(), length.sinh()))
    };
    let mut lengths = vec![0.0; count];
    let mut gapped = vec![false; count];
    for pair in &cuts.pairs {
        lengths[pair.last] = value(pair.kind)?;
    }
    for sheet in &cuts.sheets {
        for start in 0..3 {
            let (f, g, h) = (sheet[start], sheet[(start + 1) % 3], sheet[(start + 2) % 3]);
            let side = (f.last + 1) % count;
            if side != g.first || cuts.pair(side) {
                return None;
            }
            let ((f_corner, c_f, s_f), (g_corner, c_g, s_g), (_, c_h, _)) =
                (trig(f)?, trig(g)?, trig(h)?);
            let argument = (c_f * c_g + c_h) / (s_f * s_g);
            let gap = if f_corner == g_corner {
                argument.acosh()
            } else {
                argument.asinh()
            };
            if !(gap.is_finite() && gap > 0.0) {
                return None;
            }
            lengths[side] += gap;
            gapped[side] = true;
        }
    }
    let complete = (0..count).all(|side| gapped[side] != cuts.pair(side))
        && lengths
            .iter()
            .all(|&length| length.is_finite() && length > 0.0);
    complete.then_some(lengths)
}

pub(crate) fn default_length(vertices: Option<&[DVec3]>, feature: Feature) -> f64 {
    let exact = vertices.and_then(|vertices| exact_default(vertices, feature));
    rounded(exact.unwrap_or(f64::NAN))
}

fn rounded(value: f64) -> f64 {
    let decimals = (value * 1000.0).round() / 1000.0;
    if decimals.is_nan() {
        SHORTEST
    } else {
        decimals.clamp(SHORTEST, LONGEST)
    }
}

fn exact_default(vertices: &[DVec3], feature: Feature) -> Option<f64> {
    let count = vertices.len();
    if count == 0 {
        return None;
    }
    let vertex = |index: usize| vertices.get(index % count).copied();
    match feature.kind {
        Kind::Corner => None,
        Kind::CornerPair(_) => Some(distance(vertex(feature.first)?, vertex(feature.last)?)),
        Kind::Slice(_) => {
            let wall = |side: usize| wall_through(vertex(side + count - 1)?, vertex(side)?).ok();
            let (first, second) = (wall(feature.first)?, wall(feature.last + 1)?);
            Some(form(first, second).abs().acosh())
        }
    }
}

pub(crate) fn inscribed(orders: &[u32]) -> Option<Vec<DVec3>> {
    let halves: Vec<f64> = orders
        .iter()
        .map(|&order| PI / f64::from(order) / 2.0)
        .collect();
    let radius = inradius(&halves);
    let mut direction = 0.0;
    let mut previous = None;
    let mut vertices = Vec::with_capacity(orders.len());
    for &half in &halves {
        let alpha = (half.cos() / radius.cosh()).asin();
        if let Some(previous) = previous {
            direction += previous + alpha;
        }
        previous = Some(alpha);
        let distance = (1.0 / (alpha.tan() * half.tan())).acosh();
        vertices.push(rotation(direction) * (boost(distance) * ORIGIN));
    }
    let finite = radius.is_finite() && vertices.iter().all(|vertex| vertex.is_finite());
    finite.then_some(vertices)
}

fn angle_sum(halves: &[f64], radius: f64) -> f64 {
    halves
        .iter()
        .map(|half| (half.cos() / radius.cosh()).asin())
        .sum()
}

fn inradius(halves: &[f64]) -> f64 {
    let mut high: f64 = 1.0;
    while high.is_finite() && angle_sum(halves, high) >= PI {
        high *= 2.0;
    }
    let mut low = 0.0;
    for _ in 0..BISECTION_STEPS {
        let middle = f64::midpoint(low, high);
        if middle <= low || middle >= high {
            break;
        }
        if angle_sum(halves, middle) > PI {
            low = middle;
        } else {
            high = middle;
        }
    }
    f64::midpoint(low, high)
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;

    use escher_geometry::hyperbolic::{ORIGIN, boost, form, rotation};
    use glam::DVec3;

    use super::{
        Cuts, Feature, Kind, LONGEST, SHORTEST, angle_sum, default_length, exact_default, inradius,
        inscribed, rounded, side_lengths,
    };
    use crate::parameters::Parameter;
    use crate::shared::{NAMED_POLYGONS, corner_orders, small_polygons};

    fn corner(index: usize) -> Feature {
        Feature {
            kind: Kind::Corner,
            first: index,
            last: index,
        }
    }

    fn pair(number: usize, first: usize) -> Feature {
        Feature {
            kind: Kind::CornerPair(number),
            first,
            last: first + 1,
        }
    }

    fn slice(first: usize, last: usize) -> Feature {
        Feature {
            kind: Kind::Slice(0),
            first,
            last,
        }
    }

    fn orders(text: &str) -> Vec<u32> {
        corner_orders(&crate::shared::signature(text))
    }

    fn defaults(orders: &[u32], cuts: &Cuts) -> Vec<f64> {
        let vertices = inscribed(orders);
        cuts.parameters()
            .into_iter()
            .map(|(feature, _)| default_length(vertices.as_deref(), feature))
            .collect()
    }

    #[test]
    fn cuts_give_the_features_and_sheets_of_four_polygons() {
        for (text, pairs, slices, sheets) in [
            (
                "*3222",
                vec![pair(0, 1)],
                vec![],
                vec![[corner(0), pair(0, 1), corner(3)]],
            ),
            (
                "*3322",
                vec![pair(0, 2)],
                vec![],
                vec![[corner(0), corner(1), pair(0, 2)]],
            ),
            (
                "*3333",
                vec![],
                vec![slice(2, 3)],
                vec![
                    [corner(2), corner(3), slice(0, 1)],
                    [slice(2, 3), corner(0), corner(1)],
                ],
            ),
            (
                "*22222",
                vec![pair(0, 0), pair(1, 2)],
                vec![],
                vec![[pair(0, 0), pair(1, 2), corner(4)]],
            ),
        ] {
            let cuts = Cuts::of(&orders(text));
            assert_eq!(cuts.pairs, pairs, "{text}");
            assert_eq!(cuts.slices, slices, "{text}");
            assert_eq!(cuts.sheets, sheets, "{text}");
        }
    }

    #[test]
    fn parameters_list_the_corner_pairs_then_the_slices_with_their_features() {
        let cuts = Cuts::of(&orders("*2222222"));
        assert_eq!(
            cuts.parameters(),
            [
                (pair(0, 0), Parameter::CornerPair(1)),
                (pair(1, 2), Parameter::CornerPair(2)),
                (pair(2, 4), Parameter::CornerPair(3)),
                (slice(4, 6), Parameter::Slice(1)),
            ]
        );
    }

    #[test]
    fn every_gap_of_the_named_polygons_lies_between_its_features() {
        for text in ["*5432", "*65432", "*2222222", "*8765432"] {
            let orders = orders(text);
            let cuts = Cuts::of(&orders);
            let count = orders.len();
            for sheet in &cuts.sheets {
                for start in 0..3 {
                    let (f, g) = (sheet[start], sheet[(start + 1) % 3]);
                    assert_eq!((f.last + 1) % count, g.first, "{text} {sheet:?}");
                }
            }
            let defaults = defaults(&orders, &cuts);
            assert!(side_lengths(&orders, &cuts, &defaults).is_some(), "{text}");
        }
    }

    #[test]
    fn named_polygons_have_one_parameter_for_each_corner_beyond_three() {
        for text in NAMED_POLYGONS {
            let orders = orders(text);
            let cuts = Cuts::of(&orders);
            let (count, pairs) = (orders.len(), cuts.pairs.len());
            assert_eq!(cuts.parameters().len(), count - 3, "{text}");
            assert_eq!(cuts.sheets.len(), count - pairs - 2, "{text}");
        }
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the sweep of the small polygons runs loops too long for the interpreter"
    )]
    fn no_sheet_holds_three_corners() {
        let signatures = small_polygons();
        assert!(signatures.len() > 4000, "{}", signatures.len());
        let all = NAMED_POLYGONS
            .iter()
            .map(|text| orders(text))
            .chain(signatures.iter().map(corner_orders));
        for orders in all {
            for sheet in Cuts::of(&orders).sheets {
                let corners = sheet
                    .iter()
                    .filter(|feature| feature.kind == Kind::Corner)
                    .count();
                assert!(corners < 3, "{orders:?} {sheet:?}");
            }
        }
    }

    #[test]
    fn a_sheet_in_the_order_of_removal_fails_the_condition_on_its_gaps() {
        let orders = orders("*3333");
        let mut cuts = Cuts::of(&orders);
        let defaults = defaults(&orders, &cuts);
        assert!(side_lengths(&orders, &cuts, &defaults).is_some());
        cuts.sheets[0].swap(0, 1);
        assert_eq!(side_lengths(&orders, &cuts, &defaults), None);
    }

    #[test]
    fn side_lengths_refuse_missing_values_other_orders_and_a_degenerate_gap() {
        let orders = orders("*33222");
        let cuts = Cuts::of(&orders);
        assert!(side_lengths(&orders, &cuts, &[1.0, 2.0]).is_some());
        assert_eq!(side_lengths(&orders, &cuts, &[1.0]), None);
        assert_eq!(side_lengths(&orders, &cuts, &[]), None);
        assert_eq!(side_lengths(&orders[..4], &cuts, &[1.0, 2.0]), None);
        assert_eq!(side_lengths(&orders, &cuts, &[1.0, 0.0]), None);
    }

    #[test]
    fn rounded_keeps_three_decimals_inside_the_range_and_clamps_outside_it() {
        assert_eq!(rounded(1.3169), 1.317);
        assert_eq!(rounded(0.1), SHORTEST);
        assert_eq!(rounded(4.0), LONGEST);
        assert_eq!(rounded(f64::NAN), SHORTEST);
        assert_eq!(rounded(f64::INFINITY), LONGEST);
    }

    #[test]
    fn default_length_without_a_polygon_or_a_length_is_the_shortest() {
        let vertices = inscribed(&orders("*33222"));
        assert!(vertices.is_some());
        assert_eq!(default_length(None, pair(0, 2)), SHORTEST);
        assert_eq!(default_length(vertices.as_deref(), corner(0)), SHORTEST);
        assert_eq!(exact_default(&[], pair(0, 2)), None);
        assert_eq!(exact_default(&[], slice(2, 4)), None);
        let point = boost(1.0) * ORIGIN;
        assert_eq!(exact_default(&[point, point, point], slice(0, 1)), None);
    }

    #[test]
    fn inscribed_is_none_for_an_angle_that_is_not_finite() {
        assert_eq!(inscribed(&[0, 3, 3, 3]), None);
    }

    fn assert_defaults(texts: impl Iterator<Item = Vec<u32>>) -> usize {
        let mut largest = (0.0, Vec::new());
        let mut largest_residual: f64 = 0.0;
        let mut samples = 0;
        for orders in texts {
            let cuts = Cuts::of(&orders);
            let halves: Vec<f64> = orders
                .iter()
                .map(|&order| PI / f64::from(order) / 2.0)
                .collect();
            let residual = (angle_sum(&halves, inradius(&halves)) - PI).abs();
            assert!(residual <= 2e-14, "{orders:?}: {residual:e}");
            largest_residual = largest_residual.max(residual);
            let Some(vertices) = inscribed(&orders) else {
                panic!("{orders:?} has no inscribed polygon");
            };
            for (feature, _) in cuts.parameters() {
                let Some(value) = exact_default(&vertices, feature) else {
                    panic!("{orders:?} has no default for {feature:?}");
                };
                let default = default_length(Some(&vertices), feature);
                assert!((SHORTEST..=LONGEST).contains(&value), "{orders:?} {value}");
                let difference = (value - default).abs();
                assert!(difference <= 5.0e-4, "{orders:?} {value} {default}");
                if difference > largest.0 {
                    largest = (difference, orders.clone());
                }
                samples += 1;
            }
        }
        println!(
            "largest rounding of a default {:.3e} at {:?}, largest angle sum residual {largest_residual:.3e}, {samples} samples",
            largest.0, largest.1
        );
        samples
    }

    #[test]
    fn defaults_of_the_named_polygons_are_unclamped_and_rounded_to_three_decimals() {
        let samples = assert_defaults(NAMED_POLYGONS.iter().map(|text| orders(text)));
        assert!(samples >= 40, "{samples}");
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the sweep of the small polygons runs loops too long for the interpreter"
    )]
    fn defaults_of_the_small_polygons_are_unclamped_and_rounded_to_three_decimals() {
        let samples = assert_defaults(small_polygons().iter().map(corner_orders));
        assert!(samples >= 4000, "{samples}");
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "the interpreter adds rounding errors of its own to the inverse hyperbolic cosine, which can exceed the bound of four units in the last place"
    )]
    fn slice_default_is_the_distance_between_two_walls() {
        let vertices =
            [0.0, 1.0, 2.0, 3.0].map(|turn| rotation(turn * PI / 2.0) * (boost(1.0) * ORIGIN));
        let Some(value) = exact_default(&vertices, slice(2, 3)) else {
            panic!("the square has no slice");
        };
        let wall = |first: DVec3, second: DVec3| match escher_geometry::hyperbolic::wall_through(
            first, second,
        ) {
            Ok(wall) => wall,
            Err(error) => panic!("{error}"),
        };
        let expected = form(
            wall(vertices[1], vertices[2]),
            wall(vertices[3], vertices[0]),
        )
        .abs()
        .acosh();
        assert!(
            (value - expected).abs() <= 4.0 * f64::EPSILON * expected,
            "{value} {expected}"
        );
    }
}
