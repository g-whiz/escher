use std::collections::BTreeMap;

use escher_signature::Signature;

use crate::error::{Error, Result};
use crate::triangle::{Supported, supported};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Parameter {
    Aspect,
}

impl Parameter {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Aspect => "aspect",
        }
    }

    pub(crate) const fn minimum(self) -> f64 {
        match self {
            Self::Aspect => 0.125,
        }
    }

    pub(crate) const fn maximum(self) -> f64 {
        match self {
            Self::Aspect => 8.0,
        }
    }

    const fn default_value(self) -> f64 {
        match self {
            Self::Aspect => 1.0,
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
/// Every other signature has none, whether or not the crate builds a domain
/// for it.
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
/// assert!(parameters(&"*632".parse::<Signature>()?).is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn parameters(signature: &Signature) -> Vec<ParameterSpec> {
    supported(signature)
        .map_or_default(of)
        .into_iter()
        .map(|parameter| ParameterSpec { parameter })
        .collect()
}

fn of(supported: Supported) -> Vec<Parameter> {
    match supported {
        Supported::Rectangle => vec![Parameter::Aspect],
        Supported::HyperbolicTriangle { .. }
        | Supported::EuclideanTriangle { .. }
        | Supported::SphericalTriangle { .. }
        | Supported::Lune { .. } => Vec::new(),
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
}

impl ParameterSpec {
    /// Returns the name of the parameter, such as `aspect`.
    ///
    /// A setting in [`ShapeParameters`] matches the parameter only when its
    /// name equals this name byte for byte.
    #[must_use]
    pub fn name(&self) -> &str {
        self.parameter.name()
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
        self.parameter.default_value()
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

pub(crate) fn validate(
    signature: &Signature,
    supported: Supported,
    settings: &ShapeParameters,
) -> Result<()> {
    let accepted = of(supported);
    let named = |name: &str| {
        accepted
            .iter()
            .copied()
            .find(|parameter| parameter.name() == name)
    };
    if settings.values.keys().any(|name| named(name).is_none()) {
        return Err(Error::unknown_parameter(signature));
    }
    for (name, &value) in &settings.values {
        if let Some(parameter) = named(name) {
            let within = value >= parameter.minimum() && value <= parameter.maximum();
            if !within {
                return Err(Error::parameter_out_of_range(parameter));
            }
        }
    }
    Ok(())
}

pub(crate) fn value(settings: &ShapeParameters, parameter: Parameter) -> f64 {
    settings
        .values
        .get(parameter.name())
        .copied()
        .unwrap_or(parameter.default_value())
}
