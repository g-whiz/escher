use escher_domain::{FundamentalDomain, Orientation, Side};
use glam::{DMat3, DVec3};

const TOLERANCE: f64 = 4e-15;

/// The result of folding a point into a fundamental domain.
///
/// Get one from [`fold`]. It holds the folded point, the number of pairings
/// that the fold applied, how many of them reverse orientation, and whether
/// the fold converged, which means that it stopped at a finite point with a
/// positive last coordinate that lies inside every side of the domain.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_geometry::hyperbolic::{ORIGIN, boost};
/// use escher_render::fold;
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*732".parse::<Signature>()?)?;
/// let result = fold(&domain, boost(2.0) * ORIGIN, 1500, |_| {});
/// assert!(result.converged());
/// assert!(result.steps() > 0);
/// assert_eq!(result.reversals(), result.steps());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fold {
    point: DVec3,
    steps: u32,
    reversals: u32,
    converged: bool,
}

impl Fold {
    /// Returns the folded point.
    ///
    /// For a hyperbolic domain, the fold does not scale the point back onto the
    /// hyperboloid, because near the limit circle the scale factor is mostly
    /// rounding noise. For a Euclidean domain, a point whose last coordinate is
    /// 1 keeps that last coordinate through every step. The point is finite and
    /// has a positive last coordinate when the fold converged. A point that
    /// needed no step is the input, bit for bit.
    #[must_use]
    pub fn point(self) -> DVec3 {
        self.point
    }

    /// Returns the number of pairings that the fold applied, at most the cap.
    #[must_use]
    pub fn steps(self) -> u32 {
        self.steps
    }

    /// Returns the number of applied pairings that reverse orientation, at
    /// most [`steps`](Fold::steps).
    #[must_use]
    pub fn reversals(self) -> u32 {
        self.reversals
    }

    /// Returns whether the fold stopped at a finite point with a positive last
    /// coordinate that lies inside every side of the domain.
    ///
    /// It is false when the point has a coordinate that is not finite or a
    /// last coordinate that is not positive, at the start or after a step,
    /// when a side test gives NaN, and when the point still lies outside a side
    /// after the cap on the number of steps.
    #[must_use]
    pub fn converged(self) -> bool {
        self.converged
    }
}

/// Folds a point into a fundamental domain by applying the pairing of the
/// first side that the point lies outside, until it lies inside every side.
///
/// Use it to find the tile of a point and the copy of the point in the
/// domain. For a hyperbolic domain, the point is a vector of the linear model
/// of [`escher_geometry::hyperbolic`]. For a Euclidean domain, it is a point of
/// the affine plane of [`escher_geometry::euclidean`], which has last
/// coordinate 1. The pairing of a side is the isometry that maps the
/// neighbouring copy of the domain across that side onto the domain. At each
/// step the fold tests the sides in the order of [`FundamentalDomain::sides`]
/// and applies the pairing of the first side whose test is positive beyond a
/// tolerance. After the step it calls `visit` with the index of that side in
/// the same order. A point within that tolerance of a side counts as inside it,
/// so a vertex of the domain folds in zero steps. The tolerance is relative to
/// the size of the side's covector and of the point, so scaling the point by a
/// power of two changes no decision. The fold stops without converging when the
/// point still lies outside a side after `max_folds` steps. It also stops
/// without converging as soon as the point has a coordinate that is not finite
/// or a last coordinate that is not positive, because such a vector is not a
/// point of the plane. So the zero vector and, for a hyperbolic domain, a
/// vector on the other sheet of the hyperboloid fold in zero steps.
///
/// The function never panics, for any input. For a hyperbolic domain, the
/// result is accurate up to a disk radius of `1 - 1e-6`. For a Euclidean
/// domain, it is accurate up to a distance of 65 from the centre of the
/// domain. Beyond those limits the function still returns, but its result
/// carries no accuracy guarantee.
///
/// # Examples
///
/// ```
/// use escher_domain::FundamentalDomain;
/// use escher_geometry::hyperbolic::{ORIGIN, boost, rotation};
/// use escher_render::fold;
/// use escher_signature::Signature;
///
/// let domain = FundamentalDomain::build(&"*542".parse::<Signature>()?)?;
/// let mut sides = Vec::new();
/// let result = fold(&domain, rotation(1.0) * boost(3.0) * ORIGIN, 1500, |side| sides.push(side));
/// assert!(result.converged());
/// assert_eq!(sides.len(), usize::try_from(result.steps())?);
///
/// let inside = fold(&domain, ORIGIN, 1500, |_| {});
/// assert_eq!((inside.steps(), inside.point()), (0, ORIGIN));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn fold(
    domain: &FundamentalDomain,
    point: DVec3,
    max_folds: u32,
    visit: impl FnMut(usize),
) -> Fold {
    let walls: Vec<Wall> = domain.sides().iter().map(|&side| Wall::new(side)).collect();
    fold_carrying(&walls, point, [DVec3::ZERO; 2], max_folds, visit).0
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Wall {
    covector: DVec3,
    pairing: DMat3,
    reverses: bool,
    scale: f64,
}

impl Wall {
    pub(crate) fn new(side: Side) -> Self {
        Self::from_parts(side.covector(), side.pairing(), side.orientation())
    }

    pub(crate) fn from_parts(covector: DVec3, pairing: DMat3, orientation: Orientation) -> Self {
        Self {
            covector,
            pairing,
            reverses: orientation == Orientation::Reversing,
            scale: covector.abs().element_sum(),
        }
    }

    pub(crate) fn covector(&self) -> DVec3 {
        self.covector
    }
}

pub(crate) fn fold_carrying(
    walls: &[Wall],
    mut point: DVec3,
    mut tangents: [DVec3; 2],
    max_folds: u32,
    mut visit: impl FnMut(usize),
) -> (Fold, [DVec3; 2]) {
    let mut steps = 0;
    let mut reversals = 0;
    let stop = |point, steps, reversals, converged| Fold {
        point,
        steps,
        reversals,
        converged,
    };
    loop {
        let on_upper_sheet = point.z > 0.0;
        if !point.is_finite() || !on_upper_sheet {
            return (stop(point, steps, reversals, false), tangents);
        }
        let mut violated = None;
        for (index, wall) in walls.iter().enumerate() {
            let value = wall.covector.dot(point);
            if value.is_nan() {
                return (stop(point, steps, reversals, false), tangents);
            }
            if value > TOLERANCE * wall.scale * point.z {
                violated = Some((index, wall));
                break;
            }
        }
        let Some((index, wall)) = violated else {
            return (stop(point, steps, reversals, true), tangents);
        };
        if steps == max_folds {
            return (stop(point, steps, reversals, false), tangents);
        }
        point = wall.pairing * point;
        tangents = tangents.map(|tangent| wall.pairing * tangent);
        steps += 1;
        if wall.reverses {
            reversals += 1;
        }
        visit(index);
    }
}

#[cfg(test)]
mod tests {
    use escher_domain::Orientation;
    use glam::{DMat3, DVec3};

    use super::{Wall, fold_carrying};

    #[test]
    fn fold_carries_the_tangents_through_each_pairing() {
        let flip = DMat3::from_diagonal(DVec3::new(-1.0, 1.0, 1.0));
        let walls = [Wall::from_parts(DVec3::X, flip, Orientation::Reversing)];
        let point = DVec3::new(0.5, 0.0, 1.25);
        let tangents = [DVec3::new(1.0, 2.0, 3.0), DVec3::new(-4.0, 5.0, 6.0)];
        let (result, carried) = fold_carrying(&walls, point, tangents, 10, |_| {});
        assert!(result.converged());
        assert_eq!(result.steps(), 1);
        assert_eq!(result.point(), DVec3::new(-0.5, 0.0, 1.25));
        assert_eq!(
            carried,
            [DVec3::new(-1.0, 2.0, 3.0), DVec3::new(4.0, 5.0, 6.0)]
        );
    }

    #[test]
    fn fold_counts_only_reversing_pairings_as_reversals() {
        let flip = DMat3::from_diagonal(DVec3::new(-1.0, 1.0, 1.0));
        let walls = [Wall::from_parts(DVec3::X, flip, Orientation::Preserving)];
        let (result, _) = fold_carrying(
            &walls,
            DVec3::new(0.5, 0.0, 1.25),
            [DVec3::ZERO; 2],
            10,
            |_| {},
        );
        assert_eq!((result.steps(), result.reversals()), (1, 0));
    }
}
