use std::collections::{BTreeSet, HashMap};

use escher_signature::{Geometry, Signature};

use crate::common::{Model, Value, parse, signature};

const SCALE: i64 = 1680;

struct Search {
    model: Model,
    found: BTreeSet<String>,
}

impl Search {
    fn cones(&mut self, budget: i64) {
        self.boundaries(budget);
        if self.model.cones.len() == 5 {
            return;
        }
        let largest = match self.model.cones.last() {
            Some(Value::Finite(order)) => *order,
            _ => 8,
        };
        for order in 2..=largest {
            let rest = budget - (SCALE - SCALE / i64::from(order));
            if rest >= 0 {
                self.model.cones.push(Value::Finite(order));
                self.cones(rest);
                self.model.cones.pop();
            }
        }
    }

    fn boundaries(&mut self, budget: i64) {
        if budget == 0 {
            let text = self.model.text();
            let signature = signature(&text);
            assert_eq!(signature.geometry(), Geometry::Euclidean, "{text:?}");
            assert!(signature.is_compact(), "{text:?}");
            self.found.insert(signature.to_string());
        }
        if self.model.boundaries.len() < 3 && budget >= SCALE {
            self.model.boundaries.push(Vec::new());
            self.corners(budget - SCALE);
            self.model.boundaries.pop();
        }
    }

    fn corners(&mut self, budget: i64) {
        self.boundaries(budget);
        let Some(corners) = self.model.boundaries.last() else {
            return;
        };
        if corners.len() == 5 {
            return;
        }
        for order in 2..=8 {
            let rest = budget - (SCALE - SCALE / i64::from(order)) / 2;
            if rest >= 0 {
                if let Some(corners) = self.model.boundaries.last_mut() {
                    corners.push(Value::Finite(order));
                }
                self.corners(rest);
                if let Some(corners) = self.model.boundaries.last_mut() {
                    corners.pop();
                }
            }
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn enumeration_of_euclidean_compact_signatures_finds_exactly_the_seventeen_plane_signatures() {
    let mut search = Search {
        model: Model::default(),
        found: BTreeSet::new(),
    };
    for handles in 0..=2 {
        for crosscaps in 0..=3 {
            let budget = 2 * SCALE - 2 * SCALE * i64::from(handles) - SCALE * i64::from(crosscaps);
            if budget >= 0 {
                search.model.handles = handles;
                search.model.crosscaps = crosscaps;
                search.cones(budget);
            }
        }
    }
    let expected = [
        "*632", "632", "*442", "4*2", "442", "*333", "3*3", "333", "*2222", "2*22", "22*", "2222",
        "**", "*x", "xx", "22x", "o",
    ]
    .map(str::to_owned)
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(search.found, expected);
}

fn sequences(values: &[Value], longest: usize) -> Vec<Vec<Value>> {
    let mut all = vec![Vec::new()];
    let mut last = vec![Vec::new()];
    for _ in 0..longest {
        last = last
            .iter()
            .flat_map(|sequence: &Vec<Value>| {
                values.iter().map(move |&value| {
                    let mut longer = sequence.clone();
                    longer.push(value);
                    longer
                })
            })
            .collect();
        all.extend(last.iter().cloned());
    }
    all
}

fn boundary_lists(values: &[Value], boundaries: usize, corners: usize) -> Vec<Vec<Vec<Value>>> {
    let mut lists = vec![Vec::new()];
    if boundaries == 0 {
        return lists;
    }
    for first in sequences(values, corners) {
        for mut rest in boundary_lists(values, boundaries - 1, corners - first.len()) {
            rest.insert(0, first.clone());
            lists.push(rest);
        }
    }
    lists
}

struct Bounds<'values> {
    cone_values: &'values [Value],
    cones: usize,
    corner_values: &'values [Value],
    boundaries: usize,
    corners: usize,
    weight: u32,
}

fn universe(bounds: &Bounds<'_>) -> Vec<Model> {
    let cone_lists = sequences(bounds.cone_values, bounds.cones);
    let boundary_lists = boundary_lists(bounds.corner_values, bounds.boundaries, bounds.corners);
    let mut models = Vec::new();
    for handles in 0..=bounds.weight / 2 {
        for crosscaps in 0..=bounds.weight - 2 * handles {
            for cones in &cone_lists {
                for boundaries in &boundary_lists {
                    models.push(Model {
                        handles,
                        crosscaps,
                        cones: cones.clone(),
                        boundaries: boundaries.clone(),
                    });
                }
            }
        }
    }
    models
}

fn root(parents: &mut [usize], mut node: usize) -> usize {
    while parents[node] != node {
        parents[node] = parents[parents[node]];
        node = parents[node];
    }
    node
}

fn assert_one_canonical_string_per_class(models: &[Model]) {
    let index = models
        .iter()
        .enumerate()
        .map(|(position, model)| (model, position))
        .collect::<HashMap<_, _>>();
    let mut parents = (0..models.len()).collect::<Vec<_>>();
    for (position, model) in models.iter().enumerate() {
        for next in model.moves() {
            let other = index[&next];
            let (first, second) = (root(&mut parents, position), root(&mut parents, other));
            parents[first] = second;
        }
    }
    let mut results = HashMap::<usize, (String, Result<Signature, String>)>::new();
    let mut classes_of_strings = HashMap::<String, usize>::new();
    for (position, model) in models.iter().enumerate() {
        let class = root(&mut parents, position);
        let text = model.text();
        let result = parse(&text).map_err(|error| format!("{:?}", error.kind()));
        let key = result.as_ref().map(ToString::to_string);
        if let Some((first_text, first)) = results.get(&class) {
            let first_key = first.as_ref().map(ToString::to_string);
            assert_eq!(
                key, first_key,
                "{text:?} and {first_text:?} are in one class"
            );
        } else {
            if let Ok(signature) = &result {
                let canonical = signature.to_string();
                let previous = classes_of_strings.insert(canonical.clone(), class);
                assert_eq!(previous, None, "two classes share {canonical:?}");
                let own = index[&Model::of(signature)];
                assert_eq!(
                    root(&mut parents, own),
                    class,
                    "{canonical:?} leaves its class"
                );
            }
            results.insert(class, (text, result));
        }
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn equivalence_classes_with_cone_points_have_one_canonical_string_each() {
    let models = universe(&Bounds {
        cone_values: &[Value::Finite(2), Value::Finite(3), Value::Infinite],
        cones: 2,
        corner_values: &[
            Value::Finite(2),
            Value::Finite(3),
            Value::Finite(4),
            Value::Infinite,
        ],
        boundaries: 3,
        corners: 4,
        weight: 4,
    });
    assert_one_canonical_string_per_class(&models);
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the enumeration runs a loop too long for the interpreter"
)]
fn equivalence_classes_with_long_boundaries_have_one_canonical_string_each() {
    let models = universe(&Bounds {
        cone_values: &[],
        cones: 0,
        corner_values: &[Value::Finite(2), Value::Finite(3), Value::Finite(4)],
        boundaries: 2,
        corners: 6,
        weight: 3,
    });
    assert_one_canonical_string_per_class(&models);
}
