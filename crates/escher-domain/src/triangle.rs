use core::f64::consts::PI;

use escher_geometry::hyperbolic::{
    ORIGIN, boost, covector, inverse, normalize_point, reflection, rotation, side, translation,
    wall_through,
};
use escher_signature::{Geometry, Order, Signature};
use glam::DVec3;

use crate::domain::{Orientation, PairingKind, Side};
use crate::error::Error;
use crate::sheets::Cuts;

pub(crate) const LARGEST_ORDER: u32 = 1000;

pub(crate) const MOST_ORDERS: usize = 8;

const TWO_TO_THE_32: f64 = 4_294_967_296.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Shape {
    HyperbolicTriangle {
        orders: [u32; 3],
        area: f64,
    },
    HyperbolicPolygon {
        orders: [u32; MOST_ORDERS],
        count: u8,
        area: f64,
    },
    EuclideanTriangle {
        orders: [u32; 3],
    },
    Rectangle,
    SphericalTriangle {
        orders: [u32; 3],
        area: f64,
    },
    Lune {
        order: u32,
        area: f64,
    },
}

impl Shape {
    pub(crate) fn orders(self) -> Vec<u32> {
        match self {
            Self::HyperbolicTriangle { orders, .. }
            | Self::EuclideanTriangle { orders }
            | Self::SphericalTriangle { orders, .. } => orders.to_vec(),
            Self::HyperbolicPolygon { orders, count, .. } => {
                orders.into_iter().take(usize::from(count)).collect()
            }
            Self::Rectangle => vec![2; 4],
            Self::Lune { order, .. } => vec![order; 2],
        }
    }

    pub(crate) fn cuts(self) -> Option<Cuts> {
        match self {
            Self::HyperbolicPolygon { .. } => Some(Cuts::of(&self.orders())),
            Self::HyperbolicTriangle { .. }
            | Self::EuclideanTriangle { .. }
            | Self::Rectangle
            | Self::SphericalTriangle { .. }
            | Self::Lune { .. } => None,
        }
    }

    pub(crate) fn with_half_area(self) -> Self {
        match self {
            Self::HyperbolicTriangle { orders, area } => Self::HyperbolicTriangle {
                orders,
                area: area / 2.0,
            },
            Self::HyperbolicPolygon {
                orders,
                count,
                area,
            } => Self::HyperbolicPolygon {
                orders,
                count,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Wallpaper {
    Torus,
    Annulus,
    KleinBottle,
    MobiusStrip,
    TwoTwoMirror,
    TwoTwoCrossCap,
    TwoMirrorTwoTwo,
    FourMirrorTwo,
    ThreeMirrorThree,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Supported {
    Reflection(Shape),
    Rotation(Shape),
    Wallpaper(Wallpaper),
}

impl Supported {
    pub(crate) fn shape(self) -> Option<Shape> {
        match self {
            Self::Reflection(shape) | Self::Rotation(shape) => Some(shape),
            Self::Wallpaper(_) => None,
        }
    }
}

pub(crate) fn supported(signature: &Signature) -> Result<Supported, Error> {
    reflection_or_rotation(signature)
        .or_else(|error| wallpaper(signature).map(Supported::Wallpaper).ok_or(error))
}

fn reflection_or_rotation(signature: &Signature) -> Result<Supported, Error> {
    let unsupported = || Error::unsupported(signature);
    if signature.handles() != 0 || signature.crosscaps() != 0 {
        return Err(unsupported());
    }
    let cones = signature.cone_orders();
    let (orders, class): (_, fn(Shape) -> Supported) = match signature.boundaries() {
        [boundary] if cones.is_empty() => (boundary.corner_orders(), Supported::Reflection),
        [] => (cones, Supported::Rotation),
        _ => return Err(unsupported()),
    };
    let shape = match (orders, signature.geometry()) {
        ([first, second, third], Geometry::Hyperbolic) => {
            let orders = finite([first, second, third]).ok_or_else(unsupported)?;
            let area = area(signature).ok_or_else(unsupported)?;
            Shape::HyperbolicTriangle { orders, area }
        }
        (orders, Geometry::Hyperbolic) if orders.len() > 3 => {
            let finite: Option<Vec<u32>> = orders.iter().map(|order| order.finite()).collect();
            let finite = finite.ok_or_else(unsupported)?;
            let area = area(signature).ok_or_else(unsupported)?;
            polygon(&finite, area)?
        }
        ([first, second, third], Geometry::Euclidean) => {
            let orders = finite([first, second, third]).ok_or_else(unsupported)?;
            Shape::EuclideanTriangle { orders }
        }
        ([first, second, third, fourth], Geometry::Euclidean) => {
            finite([first, second, third, fourth]).ok_or_else(unsupported)?;
            Shape::Rectangle
        }
        ([first, second, third], Geometry::Spherical) => {
            let orders = finite([first, second, third]).ok_or_else(unsupported)?;
            let area = area(signature).ok_or_else(unsupported)?;
            Shape::SphericalTriangle { orders, area }
        }
        ([first, second], Geometry::Spherical) => {
            let [order, other] = finite([first, second]).ok_or_else(unsupported)?;
            if other != order {
                return Err(unsupported());
            }
            let area = area(signature).ok_or_else(unsupported)?;
            Shape::Lune { order, area }
        }
        _ => return Err(unsupported()),
    };
    Ok(class(shape))
}

fn wallpaper(signature: &Signature) -> Option<Wallpaper> {
    if signature.geometry() != Geometry::Euclidean {
        return None;
    }
    let cones: Vec<u32> = signature
        .cone_orders()
        .iter()
        .map(|order| order.finite())
        .collect::<Option<_>>()?;
    let mut boundaries = Vec::new();
    for boundary in signature.boundaries() {
        let corners: Option<Vec<u32>> = boundary
            .corner_orders()
            .iter()
            .map(|order| order.finite())
            .collect();
        boundaries.push(corners?);
    }
    let structure = (
        signature.handles(),
        signature.crosscaps(),
        cones.as_slice(),
        boundaries.as_slice(),
    );
    Some(match structure {
        (1, 0, [], []) => Wallpaper::Torus,
        (0, 0, [], [first, second]) if first.is_empty() && second.is_empty() => Wallpaper::Annulus,
        (0, 2, [], []) => Wallpaper::KleinBottle,
        (0, 1, [], [boundary]) if boundary.is_empty() => Wallpaper::MobiusStrip,
        (0, 0, [2, 2], [boundary]) if boundary.is_empty() => Wallpaper::TwoTwoMirror,
        (0, 1, [2, 2], []) => Wallpaper::TwoTwoCrossCap,
        (0, 0, [2], [boundary]) if boundary.as_slice() == [2, 2] => Wallpaper::TwoMirrorTwoTwo,
        (0, 0, [4], [boundary]) if boundary.as_slice() == [2] => Wallpaper::FourMirrorTwo,
        (0, 0, [3], [boundary]) if boundary.as_slice() == [3] => Wallpaper::ThreeMirrorThree,
        _ => return None,
    })
}

pub(crate) fn admitted(signature: &Signature) -> Result<Supported, Error> {
    let supported = supported(signature)?;
    match supported {
        Supported::Reflection(shape) | Supported::Rotation(shape) => {
            let largest = shape.orders().into_iter().fold(0, u32::max);
            if largest > LARGEST_ORDER {
                return Err(Error::limit_exceeded(largest));
            }
        }
        Supported::Wallpaper(_) => {}
    }
    Ok(supported)
}

fn finite<const N: usize>(orders: [&Order; N]) -> Option<[u32; N]> {
    let mut finite = [0; N];
    for (slot, order) in finite.iter_mut().zip(orders) {
        *slot = order.finite()?;
    }
    Some(finite)
}

fn polygon(orders: &[u32], area: f64) -> Result<Shape, Error> {
    let mut stored = [0; MOST_ORDERS];
    let count = u8::try_from(orders.len())
        .ok()
        .filter(|&count| usize::from(count) <= MOST_ORDERS)
        .ok_or_else(|| Error::too_many_orders(orders.len()))?;
    for (slot, &order) in stored.iter_mut().zip(orders) {
        *slot = order;
    }
    Ok(Shape::HyperbolicPolygon {
        orders: stored,
        count,
        area,
    })
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
    pub(crate) fn mirror(normal: DVec3, index: usize) -> Self {
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

    use escher_geometry::ErrorKind as GeometryErrorKind;
    use escher_signature::Signature;

    use crate::ErrorKind;

    use super::{
        Shape, Supported, Wallpaper, admitted, construct, signed_to_f64, supported, unsigned_to_f64,
    };

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
            Some(Some(GeometryErrorKind::NotAPoint)),
        );
    }

    #[test]
    fn supported_reads_the_canonical_order_and_the_area() {
        assert_eq!(
            supported(&signature("*237")).ok(),
            Some(Supported::Reflection(Shape::HyperbolicTriangle {
                orders: [7, 3, 2],
                area: PI / 42.0,
            })),
        );
    }

    #[test]
    fn supported_reads_the_euclidean_triangle_and_the_rectangle() {
        assert_eq!(
            supported(&signature("*236")).ok(),
            Some(Supported::Reflection(Shape::EuclideanTriangle {
                orders: [6, 3, 2]
            })),
        );
        assert_eq!(
            supported(&signature("*2222")).ok(),
            Some(Supported::Reflection(Shape::Rectangle))
        );
    }

    #[test]
    fn supported_reads_the_spherical_triangle_and_the_lune() {
        assert_eq!(
            supported(&signature("*532")).ok(),
            Some(Supported::Reflection(Shape::SphericalTriangle {
                orders: [5, 3, 2],
                area: PI / 30.0,
            })),
        );
        assert!(matches!(
            supported(&signature("*55")).ok(),
            Some(Supported::Reflection(Shape::Lune { order: 5, .. }))
        ));
    }

    #[test]
    fn supported_reads_the_rotation_signatures() {
        assert_eq!(
            supported(&signature("237")).ok(),
            Some(Supported::Rotation(Shape::HyperbolicTriangle {
                orders: [7, 3, 2],
                area: PI / 21.0,
            })),
        );
        assert_eq!(
            supported(&signature("2222")).ok(),
            Some(Supported::Rotation(Shape::Rectangle))
        );
        assert!(matches!(
            supported(&signature("22")).ok(),
            Some(Supported::Rotation(Shape::Lune { order: 2, .. }))
        ));
    }

    #[test]
    fn supported_reads_each_wallpaper_signature_by_its_structure() {
        for (text, wallpaper) in [
            ("o", Wallpaper::Torus),
            ("**", Wallpaper::Annulus),
            ("xx", Wallpaper::KleinBottle),
            ("*x", Wallpaper::MobiusStrip),
            ("x*", Wallpaper::MobiusStrip),
            ("22*", Wallpaper::TwoTwoMirror),
            ("22x", Wallpaper::TwoTwoCrossCap),
            ("x22", Wallpaper::TwoTwoCrossCap),
            ("2x2", Wallpaper::TwoTwoCrossCap),
            ("2*22", Wallpaper::TwoMirrorTwoTwo),
            ("4*2", Wallpaper::FourMirrorTwo),
            ("3*3", Wallpaper::ThreeMirrorThree),
        ] {
            assert_eq!(
                supported(&signature(text)).ok(),
                Some(Supported::Wallpaper(wallpaper)),
                "{text}"
            );
        }
    }

    #[test]
    fn admitted_admits_every_wallpaper_signature() {
        for text in [
            "o", "**", "xx", "*x", "x*", "22x", "x22", "2x2", "22*", "2*22", "4*2", "3*3",
        ] {
            assert!(admitted(&signature(text)).is_ok(), "{text}");
        }
    }

    #[test]
    fn supported_rejects_a_signature_without_a_construction() {
        for text in [
            "3*2",
            "*(inf)32",
            "*(inf)222",
            "(inf)222",
            "3*22",
            "4*3",
            "oo",
            "xxx",
            "*",
            "x",
            "2*",
            "1",
            "*22*",
            "o*",
            "ox",
            "*(inf)(inf)",
            "22(inf)",
        ] {
            assert_eq!(supported(&signature(text)).ok(), None, "{text}");
        }
    }

    #[test]
    fn supported_reads_the_polygon_in_its_canonical_order() {
        assert_eq!(
            supported(&signature("*3333")).ok(),
            Some(Supported::Reflection(Shape::HyperbolicPolygon {
                orders: [3, 3, 3, 3, 0, 0, 0, 0],
                count: 4,
                area: PI * 2.0 / 3.0,
            })),
        );
        assert_eq!(
            supported(&signature("3222")).ok(),
            Some(Supported::Rotation(Shape::HyperbolicPolygon {
                orders: [3, 2, 2, 2, 0, 0, 0, 0],
                count: 4,
                area: PI / 3.0,
            })),
        );
        let polygon = supported(&signature("*2345")).ok();
        assert_eq!(
            polygon.and_then(Supported::shape).map(Shape::orders),
            Some(vec![5, 4, 3, 2])
        );
    }

    #[test]
    fn supported_counts_the_orders_after_the_predicate() {
        let kind = |text: &str| supported(&signature(text)).map_err(|error| error.kind());
        for star in ["*", ""] {
            assert!(kind(&format!("{star}22222222")).is_ok(), "{star}");
            assert_eq!(
                kind(&format!("{star}222222222")).err(),
                Some(ErrorKind::LimitExceeded),
                "{star}"
            );
            assert_eq!(
                kind(&format!("{star}(inf)22222222")).err(),
                Some(ErrorKind::UnsupportedSignature),
                "{star}"
            );
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
