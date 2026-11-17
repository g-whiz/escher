use core::f64::consts::PI;

use escher_geometry::hyperbolic::{
    ORIGIN, boost, covector, inverse, normalize_point, reflection, rotation, side, translation,
    wall_through,
};
use escher_signature::{Geometry, Signature};
use glam::DVec3;

use crate::domain::{Orientation, PairingKind, Side};

pub(crate) const LARGEST_ORDER: u32 = 1000;

const TWO_TO_THE_32: f64 = 4_294_967_296.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Shape {
    HyperbolicTriangle { orders: [u32; 3], area: f64 },
    EuclideanTriangle { orders: [u32; 3] },
    Rectangle,
    SphericalTriangle { orders: [u32; 3], area: f64 },
    Lune { order: u32, area: f64 },
}

impl Shape {
    pub(crate) fn orders(self) -> Vec<u32> {
        match self {
            Self::HyperbolicTriangle { orders, .. }
            | Self::EuclideanTriangle { orders }
            | Self::SphericalTriangle { orders, .. } => orders.to_vec(),
            Self::Rectangle => vec![2; 4],
            Self::Lune { order, .. } => vec![order; 2],
        }
    }

    pub(crate) fn with_half_area(self) -> Self {
        match self {
            Self::HyperbolicTriangle { orders, area } => Self::HyperbolicTriangle {
                orders,
                area: area / 2.0,
            },
            Self::SphericalTriangle { orders, area } => Self::SphericalTriangle {
                orders,
                area: area / 2.0,
            },
            Self::Lune { order, area } => Self::Lune {
                order,
                area: area / 2.0,
            },
            Self::EuclideanTriangle { .. } | Self::Rectangle => self,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Supported {
    Reflection(Shape),
    Rotation(Shape),
}

impl Supported {
    pub(crate) fn shape(self) -> Shape {
        match self {
            Self::Reflection(shape) | Self::Rotation(shape) => shape,
        }
    }
}

pub(crate) fn supported(signature: &Signature) -> Option<Supported> {
    if signature.handles() != 0 || signature.crosscaps() != 0 {
        return None;
    }
    let cones = signature.cone_orders();
    let (orders, class): (_, fn(Shape) -> Supported) = match signature.boundaries() {
        [boundary] if cones.is_empty() => (boundary.corner_orders(), Supported::Reflection),
        [] => (cones, Supported::Rotation),
        _ => return None,
    };
    let shape = match (orders, signature.geometry()) {
        ([first, second, third], Geometry::Hyperbolic) => {
            let orders = [first.finite()?, second.finite()?, third.finite()?];
            let area = area(signature)?;
            Shape::HyperbolicTriangle { orders, area }
        }
        ([first, second, third], Geometry::Euclidean) => {
            let orders = [first.finite()?, second.finite()?, third.finite()?];
            Shape::EuclideanTriangle { orders }
        }
        ([first, second, third, fourth], Geometry::Euclidean) => {
            for order in [first, second, third, fourth] {
                order.finite()?;
            }
            Shape::Rectangle
        }
        ([first, second, third], Geometry::Spherical) => {
            let orders = [first.finite()?, second.finite()?, third.finite()?];
            let area = area(signature)?;
            Shape::SphericalTriangle { orders, area }
        }
        ([first, second], Geometry::Spherical) => {
            let order = first.finite()?;
            if second.finite()? != order {
                return None;
            }
            let area = area(signature)?;
            Shape::Lune { order, area }
        }
        _ => return None,
    };
    Some(class(shape))
}

fn area(signature: &Signature) -> Option<f64> {
    let area_over_pi = signature.area_over_pi()?;
    Some(
        PI * signed_to_f64(area_over_pi.numerator())
            / unsigned_to_f64(area_over_pi.denominator().get()),
    )
}

fn signed_to_f64(value: i64) -> f64 {
    let [b0, b1, b2, b3, b4, b5, b6, b7] = value.to_be_bytes();
    let high = i32::from_be_bytes([b0, b1, b2, b3]);
    let low = u32::from_be_bytes([b4, b5, b6, b7]);
    f64::from(high) * TWO_TO_THE_32 + f64::from(low)
}

fn unsigned_to_f64(value: u64) -> f64 {
    let [b0, b1, b2, b3, b4, b5, b6, b7] = value.to_be_bytes();
    let high = u32::from_be_bytes([b0, b1, b2, b3]);
    let low = u32::from_be_bytes([b4, b5, b6, b7]);
    f64::from(high) * TWO_TO_THE_32 + f64::from(low)
}

#[derive(Debug)]
pub(crate) struct Triangle {
    pub(crate) sides: [Side; 3],
    pub(crate) vertices: [DVec3; 3],
    pub(crate) centre: DVec3,
}

pub(crate) fn construct(orders: [u32; 3]) -> Result<Triangle, Option<escher_geometry::Error>> {
    let angles = orders.map(|order| PI / f64::from(order));
    let [(sin_0, cos_0), (sin_1, cos_1), (sin_2, cos_2)] = angles.map(f64::sin_cos);
    let first_cosh = (cos_1 + cos_2 * cos_0) / (sin_2 * sin_0);
    let second_cosh = (cos_0 + cos_2 * cos_1) / (sin_2 * sin_1);
    let first_length = first_cosh.acosh();
    let second_length = second_cosh.acosh();

    let placed = [
        boost(first_length) * ORIGIN,
        rotation(angles[2]) * (boost(second_length) * ORIGIN),
        ORIGIN,
    ];
    let target = normalize_point(placed[0] + placed[1] + placed[2]).map_err(Some)?;
    let move_to_origin = inverse(translation(target));
    let vertices = placed.map(|vertex| move_to_origin * vertex);

    let mut normals = [DVec3::ZERO; 3];
    for (index, slot) in normals.iter_mut().enumerate() {
        let opposite = vertices[(index + 1) % 3];
        let mut normal = wall_through(vertices[(index + 2) % 3], vertices[index]).map_err(Some)?;
        if side(covector(normal), opposite) > 0.0 {
            normal = -normal;
        }
        let inside = side(covector(normal), opposite) < 0.0;
        if !inside {
            return Err(None);
        }
        *slot = normal;
    }
    let sides = [0, 1, 2].map(|index| Side::mirror(normals[index], index));
    let centre = normalize_point(vertices[0] + vertices[1] + vertices[2]).map_err(Some)?;

    let finite = vertices.iter().all(|vertex| vertex.is_finite())
        && centre.is_finite()
        && sides.iter().all(|side| {
            side.normal.is_finite() && side.covector.is_finite() && side.pairing.is_finite()
        });
    if finite {
        Ok(Triangle {
            sides,
            vertices,
            centre,
        })
    } else {
        Err(None)
    }
}

impl Side {
    fn mirror(normal: DVec3, index: usize) -> Self {
        Self {
            normal,
            covector: covector(normal),
            pairing: reflection(normal),
            partner: index,
            kind: PairingKind::Mirror,
            orientation: Orientation::Reversing,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::PI;

    use escher_geometry::ErrorKind;
    use escher_signature::Signature;

    use super::{Shape, Supported, construct, signed_to_f64, supported, unsigned_to_f64};

    fn signature(text: &str) -> Signature {
        match text.parse() {
            Ok(signature) => signature,
            Err(error) => panic!("{text:?} is rejected: {error}"),
        }
    }

    #[test]
    fn construct_of_spherical_orders_is_numerical() {
        let error = construct([5, 3, 2]).err();
        assert_eq!(
            error.map(|source| source.map(|source| source.kind())),
            Some(Some(ErrorKind::NotAPoint)),
        );
    }

    #[test]
    fn supported_reads_the_canonical_order_and_the_area() {
        assert_eq!(
            supported(&signature("*237")),
            Some(Supported::Reflection(Shape::HyperbolicTriangle {
                orders: [7, 3, 2],
                area: PI / 42.0,
            })),
        );
    }

    #[test]
    fn supported_reads_the_euclidean_triangle_and_the_rectangle() {
        assert_eq!(
            supported(&signature("*236")),
            Some(Supported::Reflection(Shape::EuclideanTriangle {
                orders: [6, 3, 2]
            })),
        );
        assert_eq!(
            supported(&signature("*2222")),
            Some(Supported::Reflection(Shape::Rectangle))
        );
    }

    #[test]
    fn supported_reads_the_spherical_triangle_and_the_lune() {
        assert_eq!(
            supported(&signature("*532")),
            Some(Supported::Reflection(Shape::SphericalTriangle {
                orders: [5, 3, 2],
                area: PI / 30.0,
            })),
        );
        assert!(matches!(
            supported(&signature("*55")),
            Some(Supported::Reflection(Shape::Lune { order: 5, .. }))
        ));
    }

    #[test]
    fn supported_reads_the_rotation_signatures() {
        assert_eq!(
            supported(&signature("237")),
            Some(Supported::Rotation(Shape::HyperbolicTriangle {
                orders: [7, 3, 2],
                area: PI / 21.0,
            })),
        );
        assert_eq!(
            supported(&signature("2222")),
            Some(Supported::Rotation(Shape::Rectangle))
        );
        assert!(matches!(
            supported(&signature("22")),
            Some(Supported::Rotation(Shape::Lune { order: 2, .. }))
        ));
    }

    #[test]
    fn supported_rejects_a_signature_without_a_construction() {
        for text in ["3*2", "*3333", "3222", "*(inf)32"] {
            assert_eq!(supported(&signature(text)), None, "{text}");
        }
    }

    #[test]
    fn conversions_to_f64_round_to_nearest_over_the_whole_range() {
        assert_eq!(signed_to_f64(0), 0.0);
        assert_eq!(signed_to_f64(-1), -1.0);
        assert_eq!(signed_to_f64(-4_294_967_297), -4_294_967_297.0);
        assert_eq!(
            signed_to_f64(9_007_199_254_740_993),
            9_007_199_254_740_992.0
        );
        assert_eq!(signed_to_f64(i64::MAX), 9_223_372_036_854_775_808.0);
        assert_eq!(signed_to_f64(i64::MIN), -9_223_372_036_854_775_808.0);
        assert_eq!(unsigned_to_f64(1), 1.0);
        assert_eq!(unsigned_to_f64(4_294_967_297), 4_294_967_297.0);
        assert_eq!(unsigned_to_f64(u64::MAX), 18_446_744_073_709_551_616.0);
    }
}
