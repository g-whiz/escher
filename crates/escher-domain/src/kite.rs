use crate::domain::{Closure, Corner, FundamentalDomain, Plane, Side};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MirrorPair {
    Low,
    High,
}

pub(crate) fn on_kite(
    doubled: &FundamentalDomain,
    pair: MirrorPair,
) -> Result<FundamentalDomain, Option<escher_geometry::Error>> {
    let (&[first, second, third, fourth], &[corner, _, cone, _]) =
        (&doubled.sides[..], &doubled.corners[..])
    else {
        return Err(None);
    };
    let mirror = |side: Side, index: usize| match doubled.plane {
        Plane::Euclidean => Ok(Side::euclidean_mirror(side.covector, index)),
        Plane::Spherical { .. } => Ok(Side::spherical_mirror(side.normal, index)),
        Plane::Hyperbolic { .. } => Err(None),
    };
    let (sides, orders) = match pair {
        MirrorPair::Low if corner.order % 2 != 0 => return Err(None),
        MirrorPair::Low => (
            [mirror(first, 0)?, mirror(second, 1)?, third, fourth],
            [corner.order / 2, 1, cone.order, 1],
        ),
        MirrorPair::High => (
            [first, second, mirror(third, 2)?, mirror(fourth, 3)?],
            [corner.order, 1, 1, 1],
        ),
    };
    Ok(FundamentalDomain {
        plane: doubled.plane,
        corners: orders.map(Corner::single).to_vec(),
        sides: sides.to_vec(),
        vertices: doubled.vertices.clone(),
        centre: doubled.centre,
        closure: Closure::Cycles,
    })
}
