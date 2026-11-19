use core::cell::OnceCell;
use std::collections::BTreeMap;

use escher_signature::Signature;

use crate::error::{Error, Result};
use glam::DVec3;

use crate::sheets::{Cuts, Feature, LONGEST, SHORTEST, default_length, inscribed};
use crate::triangle::{Shape, Supported, Wallpaper, admitted};

const ASPECT_DEFAULT: f64 = 1.0;

const SKEW_DEFAULT: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Parameter {
    Aspect,
    Skew,
    CornerPair(u8),
    Slice(u8),
}

impl Parameter {
    pub(crate) fn name(self) -> String {
        match self {
            Self::Aspect => "aspect".to_owned(),
            Self::Skew => "skew".to_owned(),
            Self::CornerPair(number) => format!("cornerpair{number}.length"),
            Self::Slice(number) => format!("slice{number}.length"),
        }
    }

    pub(crate) const fn minimum(self) -> f64 {
        match self {
            Self::Aspect => 0.125,
            Self::Skew => -0.5,
            Self::CornerPair(_) | Self::Slice(_) => SHORTEST,
        }
    }

    pub(crate) const fn maximum(self) -> f64 {
        match self {
            Self::Aspect => 8.0,
            Self::Skew => 0.5,
            Self::CornerPair(_) | Self::Slice(_) => LONGEST,
        }
    }
}

/// Returns the shape parameters of a signature, each with its name, range,
/// and default value, in a fixed order.
///
/// A shape parameter is a named number that changes the shape of the
/// fundamental domain without changing its symmetry group. Pass a value for
/// it to [`FundamentalDomain::build_with`](crate::FundamentalDomain::build_with)
/// through [`ShapeParameters`]. The rectangle of `*2222` has the one parameter
/// `aspect`, the length of its side 0 divided by the length of its side 1.
/// The domains of `**`, `xx`, `*x`, `22x`, `22*`, and `2*22` lie on the same
/// rectangle and have the same parameter `aspect`. The parallelogram of `o`
/// has `aspect` and then `skew`, from -0.5 to 0.5 with the default 0. Its
/// first lattice vector is `(w, 0)` and its second `(skew * w, 1 / w)`, with
/// `w` the square root of `aspect`, so `skew` is the first coordinate of the
/// second vector divided by the length of the first.
///
/// The domain of `2222` is two such rectangles side by side, and its
/// parameter `aspect` is that of each rectangle. It also has `skew`, with the
/// same range and default, which shears the two rectangles by
/// `(x, y) -> (x + skew * aspect * y, y)`. Its lattice vectors are then
/// `(2w, 0)` and `(2 * skew * w, 2 / w)`. A `skew` of exactly 0 leaves the two
/// rectangles as they are.
///
/// A hyperbolic polygon `*p1...pn` with four to eight corners, each of order
/// at most 1000, has `n - 3` parameters, each a hyperbolic length from 0.5 to
/// 3.5 at the curvature -1. Two consecutive corners of order 2 form a corner
/// pair. The pairs are numbered from 1 as the corners are scanned from corner
/// 0 without wrapping around. The parameter `cornerpair<k>.length` is the length of the side
/// between the two right angles of the `k`-th pair. The other parameters,
/// `slice<i>.length`, are the lengths of common perpendiculars of pairs of
/// sides. The polygon is cut along them into pieces. Each piece has three
/// features, and each feature is a corner, a corner pair, or a cut. The corner
/// pairs come first in the list of parameters, then the slices. The default of
/// each length is that of the polygon with an inscribed circle whose corners
/// have the angles of the signature, rounded to three decimals.
///
/// Every other signature has none. This includes `4*2` and `3*3`, whose
/// domains are the kites of `442` and `632`, the rotation signature of a
/// polygon, such as `3333`, whose domain doubles the polygon with every length
/// at its default, and a signature for which the crate builds no domain, such
/// as a polygon with a corner of order above 1000.
///
/// # Examples
///
/// ```
/// use escher_domain::parameters;
/// use escher_signature::Signature;
///
/// let specs = parameters(&"*2222".parse::<Signature>()?);
/// assert_eq!(specs.len(), 1);
/// assert_eq!(specs[0].name(), "aspect");
/// assert_eq!((specs[0].minimum(), specs[0].maximum()), (0.125, 8.0));
/// assert_eq!(specs[0].default_value(), 1.0);
/// assert_eq!(parameters(&"2222".parse::<Signature>()?)[..1], specs);
/// assert!(parameters(&"*632".parse::<Signature>()?).is_empty());
///
/// let torus = parameters(&"o".parse::<Signature>()?);
/// assert_eq!(torus.len(), 2);
/// assert_eq!(parameters(&"2222".parse::<Signature>()?), torus);
/// assert_eq!(torus[1].name(), "skew");
/// assert_eq!((torus[1].minimum(), torus[1].maximum()), (-0.5, 0.5));
/// assert_eq!(torus[1].default_value(), 0.0);
///
/// let square = parameters(&"*3333".parse::<Signature>()?);
/// assert_eq!(square.len(), 1);
/// assert_eq!(square[0].name(), "slice1.length");
/// assert_eq!(square[0].default_value(), 1.317);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn parameters(signature: &Signature) -> Vec<ParameterSpec> {
    let Ok(supported) = admitted(signature) else {
        return Vec::new();
    };
    let shape = supported.shape();
    let cuts = shape.and_then(Shape::cuts);
    let defaults = Defaults::new(shape);
    listed(supported, cuts.as_ref())
        .into_iter()
        .map(|(parameter, feature)| ParameterSpec {
            parameter,
            name: parameter.name(),
            default_value: defaults.of(parameter, feature),
        })
        .collect()
}

fn listed(supported: Supported, cuts: Option<&Cuts>) -> Vec<(Parameter, Option<Feature>)> {
    match supported {
        Supported::Reflection(Shape::Rectangle)
        | Supported::Wallpaper(
            Wallpaper::Annulus
            | Wallpaper::KleinBottle
            | Wallpaper::MobiusStrip
            | Wallpaper::TwoTwoMirror
            | Wallpaper::TwoTwoCrossCap
            | Wallpaper::TwoMirrorTwoTwo,
        ) => vec![(Parameter::Aspect, None)],
        Supported::Wallpaper(Wallpaper::Torus) | Supported::Rotation(Shape::Rectangle) => {
            vec![(Parameter::Aspect, None), (Parameter::Skew, None)]
        }
        Supported::Reflection(Shape::HyperbolicPolygon { .. }) => cuts
            .map_or_default(Cuts::parameters)
            .into_iter()
            .map(|(feature, parameter)| (parameter, Some(feature)))
            .collect(),
        Supported::Reflection(
            Shape::HyperbolicTriangle { .. }
            | Shape::EuclideanTriangle { .. }
            | Shape::SphericalTriangle { .. }
            | Shape::Lune { .. },
        )
        | Supported::Rotation(
            Shape::HyperbolicTriangle { .. }
            | Shape::HyperbolicPolygon { .. }
            | Shape::EuclideanTriangle { .. }
            | Shape::SphericalTriangle { .. }
            | Shape::Lune { .. },
        )
        | Supported::Wallpaper(Wallpaper::FourMirrorTwo | Wallpaper::ThreeMirrorThree) => {
            Vec::new()
        }
    }
}

struct Defaults {
    orders: Vec<u32>,
    inscribed: OnceCell<Option<Vec<DVec3>>>,
}

impl Defaults {
    fn new(shape: Option<Shape>) -> Self {
        Self {
            orders: shape.map_or_else(Vec::new, Shape::orders),
            inscribed: OnceCell::new(),
        }
    }

    fn of(&self, parameter: Parameter, feature: Option<Feature>) -> f64 {
        match (parameter, feature) {
            (Parameter::Aspect, _) => ASPECT_DEFAULT,
            (Parameter::Skew, _) => SKEW_DEFAULT,
            (Parameter::CornerPair(_) | Parameter::Slice(_), Some(feature)) => {
                let vertices = self.inscribed.get_or_init(|| inscribed(&self.orders));
                default_length(vertices.as_deref(), feature)
            }
            (Parameter::CornerPair(_) | Parameter::Slice(_), None) => SHORTEST,
        }
    }
}

/// The name, the range, and the default value of one shape parameter of a
/// signature.
///
/// [`parameters`] lists them. The minimum, the default value, and the maximum
/// are finite, and the minimum is at most the default value, which is at most
/// the maximum. A value of the parameter must lie in the closed range from the
/// minimum to the maximum.
///
/// # Examples
///
/// ```
/// use escher_domain::parameters;
/// use escher_signature::Signature;
///
/// for spec in parameters(&"*2222".parse::<Signature>()?) {
///     assert!(spec.minimum() <= spec.default_value());
///     assert!(spec.default_value() <= spec.maximum());
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterSpec {
    parameter: Parameter,
    name: String,
    default_value: f64,
}

impl ParameterSpec {
    /// Returns the name of the parameter, such as `aspect` or
    /// `slice1.length`.
    ///
    /// A setting in [`ShapeParameters`] matches the parameter only when its
    /// name equals this name byte for byte.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the smallest value of the parameter.
    #[must_use]
    pub fn minimum(&self) -> f64 {
        self.parameter.minimum()
    }

    /// Returns the largest value of the parameter.
    #[must_use]
    pub fn maximum(&self) -> f64 {
        self.parameter.maximum()
    }

    /// Returns the value that the parameter takes when no setting gives it.
    #[must_use]
    pub fn default_value(&self) -> f64 {
        self.default_value
    }
}

/// The settings of shape parameters that a caller passes to
/// [`FundamentalDomain::build_with`](crate::FundamentalDomain::build_with).
///
/// A setting is a name and a value. The set holds each name at most once, and
/// [`ShapeParameters::new`] gives the empty set, in which every parameter
/// takes its default value. The set checks nothing:
/// [`FundamentalDomain::build_with`](crate::FundamentalDomain::build_with)
/// rejects a name that is not a parameter of the signature and a value
/// outside the range of its parameter.
///
/// # Examples
///
/// ```
/// use escher_domain::{FundamentalDomain, ShapeParameters};
/// use escher_signature::Signature;
///
/// let mut settings = ShapeParameters::new();
/// assert_eq!(settings.set("aspect", 2.0), None);
/// assert_eq!(settings.set("aspect", 4.0), Some(2.0));
/// let domain = FundamentalDomain::build_with(&"*2222".parse::<Signature>()?, &settings)?;
/// let lengths = domain.side_lengths();
/// assert!((lengths[0] / lengths[1] - 4.0).abs() < 1e-12);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapeParameters {
    values: BTreeMap<String, f64>,
}

impl ShapeParameters {
    /// Returns the empty set of settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the value of the parameter of a name, and returns the value that
    /// it replaces, if the name was already set.
    ///
    /// The method accepts any name and any value, including NaN.
    pub fn set(&mut self, name: impl Into<String>, value: f64) -> Option<f64> {
        self.insert(name.into(), value)
    }

    fn insert(&mut self, name: String, value: f64) -> Option<f64> {
        self.values.insert(name, value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Shear {
    pub(crate) aspect: f64,
    pub(crate) skew: f64,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Values {
    pub(crate) generating: Vec<f64>,
    pub(crate) shear: Option<Shear>,
}

pub(crate) fn validate(
    signature: &Signature,
    supported: Supported,
    cuts: Option<&Cuts>,
    settings: &ShapeParameters,
) -> Result<Values> {
    let accepted = listed(supported, cuts);
    let named = |name: &str| {
        accepted
            .iter()
            .find(|(parameter, _)| parameter.name() == name)
    };
    if settings.values.keys().any(|name| named(name).is_none()) {
        return Err(Error::unknown_parameter(signature));
    }
    for (name, &value) in &settings.values {
        if let Some(&(parameter, _)) = named(name) {
            let within = value >= parameter.minimum() && value <= parameter.maximum();
            if !within {
                return Err(Error::parameter_out_of_range(parameter));
            }
        }
    }
    let generating = match supported {
        Supported::Rotation(shape) => Supported::Reflection(shape),
        Supported::Reflection(_) | Supported::Wallpaper(_) => supported,
    };
    let defaults = Defaults::new(supported.shape());
    let value = |parameter: Parameter, feature: Option<Feature>| {
        settings
            .values
            .get(&parameter.name())
            .copied()
            .unwrap_or_else(|| defaults.of(parameter, feature))
    };
    let shear = matches!(supported, Supported::Rotation(Shape::Rectangle)).then(|| Shear {
        aspect: value(Parameter::Aspect, None),
        skew: value(Parameter::Skew, None),
    });
    Ok(Values {
        generating: listed(generating, cuts)
            .into_iter()
            .map(|(parameter, feature)| value(parameter, feature))
            .collect(),
        shear,
    })
}
