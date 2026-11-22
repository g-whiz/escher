use escher_domain::{FundamentalDomain, Orientation, Side};
use escher_signature::Geometry;
use glam::{DMat3, DVec3};

const TOLERANCE: f64 = 4e-15;

/// The result of folding a point into a fundamental domain.
///
/// Get one from [`fold`]. It holds the folded point, the number of pairings
/// that the fold applied, how many of them reverse orientation, and whether
/// the fold converged, which means that it stopped at a finite point with a
/// positive point size that lies inside every side of the domain. The point
/// size is the last coordinate for a hyperbolic or Euclidean domain, and the
/// largest absolute coordinate for a spherical domain.
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
    /// 1 keeps that last coordinate through every step. For a spherical domain,
    /// the point is not scaled back onto the sphere, and it stands for its ray.
    /// The point is finite and has a positive point size when the fold
    /// converged, so for a spherical domain it is not the zero vector. A point
    /// that needed no step is the input, bit for bit.
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

    /// Returns whether the fold stopped at a finite point with a positive point
    /// size that lies inside every side of the domain.
    ///
    /// It is false when the point has a coordinate that is not finite or a
    /// point size that is not positive, at the start or after a step, when a
    /// side test gives NaN, and when the point still lies outside a side after
    /// the cap on the number of steps. The point size is the last coordinate
    /// for a hyperbolic or Euclidean domain, and the largest absolute
    /// coordinate for a spherical domain.
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
/// coordinate 1. For a spherical domain, it is any vector other than the zero
/// vector, which stands for the point of the sphere of
/// [`escher_geometry::spherical`] on its ray. The pairing of a side is the
/// isometry that maps the neighbouring copy of the domain across that side onto
/// the domain. At each step the fold tests the sides in the order of
/// [`FundamentalDomain::sides`] and applies the pairing of the first side whose
/// test is positive beyond a tolerance. After the step it calls `visit` with
/// the index of that side in the same order. A point within that tolerance of a
/// side counts as inside it, so a vertex of the domain folds in zero steps. The
/// tolerance is relative to the size of the side's covector and to the point
/// size, which is the last coordinate for a hyperbolic or Euclidean domain and
/// the largest absolute coordinate for a spherical domain, so scaling the point
/// by a power of two changes no decision as long as no product of the fold
/// underflows or overflows.
///
/// For a spherical domain, the tolerance product is already subnormal at a
/// point size of `1e-300`, so exact arithmetic does not guarantee that
/// invariance for a small vector. A measurement found no changed decision for
/// any of 42,878 unit vectors of 17 spherical domains, scaled by exact powers
/// of two from `2^-1000` to `2^1023`, and the first change at `2^-1001`. The
/// accuracy stated below therefore holds only for a vector whose largest
/// absolute coordinate is at least `1e-300`, a limit that rests on that
/// measurement. A smaller vector can take other decisions, and a vector on or
/// near a wall may not converge.
///
/// The fold stops without converging when the point still lies outside a side
/// after `max_folds` steps. It also stops without converging as soon as the
/// point has a coordinate that is not finite or a point size that is not
/// positive, because such a vector is not a point. So the zero vector and, for
/// a hyperbolic or Euclidean domain, a vector with a negative last coordinate
/// fold in zero steps. For a spherical domain, every other finite vector is a
/// point, including one with a last coordinate that is zero or negative.
///
/// The function never panics, for any input. For a hyperbolic triangle and
/// its double, the result is accurate up to a disk radius of `1 - 1e-6`. For a
/// hyperbolic polygon with four to eight corners and its double, the accuracy
/// depends on the shape, because the walls of a wide shape lie far from the
/// centre and the rounding of a reflection is expected to grow with the
/// distance of its wall. The accuracy up to the same radius is measured on a
/// fixed set of shapes only. For any other shape, the function only returns and
/// reports whether the fold converged. For a Euclidean reflection domain and
/// its double, it is accurate up to a distance of 65 from the centre of the
/// domain. For the domains of `o`, `**`, `xx`, `*x`, `22x`, `22*`, `2*22`,
/// `4*2`, and `3*3`, whose sides are mirrors or are paired by translations,
/// glide reflections, half-turns, or rotations, and for the domain of `2222`
/// with a `skew` other than 0, the accuracy up to the same distance depends on
/// `aspect` and `skew`, and it is measured on a fixed set of settings only. For
/// any other setting, the function only returns and reports whether the fold
/// converged. For a spherical
/// domain, it is accurate for every vector whose largest absolute coordinate is
/// at least `1e-300` and whose fold converges.
/// A point very close to a vertex of high order can still lie outside a side
/// after the default cap of 1500 steps, and its fold then does not converge. A
/// measurement found no such point within `1e-14` radians of the vertex of
/// order 100 of `*(100)22`. Beyond those limits the function still returns,
/// but its result carries no accuracy guarantee.
///
/// A domain whose side pairings are rotations and translations, such as the
/// domain of `732`, folds by the same steps. Each pairing of a double is a
/// product of two reflections, so its error bounds are its own, and the
/// ranges of accuracy above hold for it with one exception. A translation, a
/// glide reflection, or a half-turn of the domain of `o`, `**`, `xx`, `*x`,
/// `22x`, `22*`, or `2*22`, and a half-turn of the domain of `2222` with a
/// `skew` other than 0, has exact entries, and the domains of `**`, `*x`,
/// `22*`, and `2*22` also have mirror sides. The domains of `4*2` and `3*3`
/// keep the rotations of the kites of `442` and `632` and have two mirror
/// sides.
/// For a hyperbolic domain
/// whose corner orders are all near 100, such as the domain of
/// `(100)(100)(100)` or `(100)(100)(99)`, a point very close to a vertex of
/// order 100 may still lie outside a side after the default cap, and its fold
/// then does not converge. Measurements on two machines found such folds at
/// vertices of both domains up to a hyperbolic distance of `1e-10`, and which
/// points fail depends on the platform mathematics library.
///
/// A domain with no reversing side counts no reversal, and the step counts of
/// a point and of its image under a pairing need not differ by one.
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
    let size = PointSize::of(domain.geometry());
    fold_carrying(&walls, size, point, [DVec3::ZERO; 2], max_folds, visit).0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PointSize {
    LastCoordinate,
    LargestCoordinate,
}

impl PointSize {
    pub(crate) const fn of(geometry: Geometry) -> Self {
        match geometry {
            Geometry::Hyperbolic | Geometry::Euclidean => Self::LastCoordinate,
            Geometry::Spherical => Self::LargestCoordinate,
        }
    }

    fn measure(self, point: DVec3) -> f64 {
        match self {
            Self::LastCoordinate => point.z,
            Self::LargestCoordinate => point.abs().max_element(),
        }
    }
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
    size: PointSize,
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
        let measured = size.measure(point);
        let positive = measured > 0.0;
        if !point.is_finite() || !positive {
            return (stop(point, steps, reversals, false), tangents);
        }
        let mut violated = None;
        for (index, wall) in walls.iter().enumerate() {
            let value = wall.covector.dot(point);
            if value.is_nan() {
                return (stop(point, steps, reversals, false), tangents);
            }
            if value > TOLERANCE * wall.scale * measured {
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

    use super::{PointSize, Wall, fold_carrying};

    #[test]
    fn fold_carries_the_tangents_through_each_pairing() {
        let flip = DMat3::from_diagonal(DVec3::new(-1.0, 1.0, 1.0));
        let walls = [Wall::from_parts(DVec3::X, flip, Orientation::Reversing)];
        let point = DVec3::new(0.5, 0.0, 1.25);
        let tangents = [DVec3::new(1.0, 2.0, 3.0), DVec3::new(-4.0, 5.0, 6.0)];
        let (result, carried) = fold_carrying(
            &walls,
            PointSize::LastCoordinate,
            point,
            tangents,
            10,
            |_| {},
        );
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
            PointSize::LastCoordinate,
            DVec3::new(0.5, 0.0, 1.25),
            [DVec3::ZERO; 2],
            10,
            |_| {},
        );
        assert_eq!((result.steps(), result.reversals()), (1, 0));
    }

    #[test]
    fn fold_on_the_sphere_reads_the_largest_absolute_coordinate_as_the_point_size() {
        let flip = DMat3::from_diagonal(DVec3::new(-1.0, 1.0, 1.0));
        let walls = [Wall::from_parts(DVec3::X, flip, Orientation::Reversing)];
        let below = DVec3::new(0.5, 0.0, -1.25);
        let fold = |size, point| fold_carrying(&walls, size, point, [DVec3::ZERO; 2], 10, |_| {}).0;
        let sheet = fold(PointSize::LastCoordinate, below);
        assert_eq!((sheet.converged(), sheet.steps()), (false, 0));
        let sphere = fold(PointSize::LargestCoordinate, below);
        assert_eq!((sphere.converged(), sphere.steps()), (true, 1));
        assert_eq!(sphere.point(), DVec3::new(-0.5, 0.0, -1.25));
        let zero = fold(PointSize::LargestCoordinate, DVec3::ZERO);
        assert_eq!((zero.converged(), zero.steps()), (false, 0));
        let threshold = fold(PointSize::LargestCoordinate, DVec3::new(1.0e-15, 0.0, -1.0));
        assert_eq!((threshold.converged(), threshold.steps()), (true, 0));
    }
}
