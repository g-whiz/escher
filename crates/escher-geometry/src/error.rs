/// A specialized [`Result`](core::result::Result) whose error is [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// The reason why a vector is not a point of a geometry, why two points span
/// no wall, or why two walls meet in no point.
///
/// Match on [`Error::kind`] to tell the causes apart. The
/// [`Display`](core::fmt::Display) text starts with `cannot `, names the
/// operation that failed, and is meant for a person, not for matching. The
/// error holds no value of the input.
///
/// # Examples
///
/// ```
/// use escher_geometry::ErrorKind;
/// use escher_geometry::hyperbolic::normalize_point;
/// use glam::DVec3;
///
/// let error = normalize_point(DVec3::new(2.0, 0.0, 1.0)).err();
/// assert_eq!(error.as_ref().map(|error| error.kind()), Some(ErrorKind::NotAPoint));
/// assert_eq!(
///     error.map(|error| error.to_string()).as_deref(),
///     Some("cannot normalize point: the vector is not timelike with a positive last coordinate"),
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("cannot {operation}: {detail}")]
pub struct Error {
    operation: &'static str,
    detail: Detail,
}

impl Error {
    pub(crate) fn not_a_point() -> Self {
        Self {
            operation: "normalize point",
            detail: Detail::NotAPoint,
        }
    }

    pub(crate) fn no_length() -> Self {
        Self {
            operation: "normalize point",
            detail: Detail::NoLength,
        }
    }

    pub(crate) fn degenerate_wall() -> Self {
        Self {
            operation: "find wall through two points",
            detail: Detail::DegenerateWall,
        }
    }

    pub(crate) fn walls_do_not_meet() -> Self {
        Self {
            operation: "find point where two walls meet",
            detail: Detail::WallsDoNotMeet,
        }
    }

    /// Returns the cause of the error as an [`ErrorKind`].
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self.detail {
            Detail::NotAPoint | Detail::NoLength => ErrorKind::NotAPoint,
            Detail::DegenerateWall | Detail::WallsDoNotMeet => ErrorKind::DegenerateWall,
        }
    }
}

/// The cause of an [`Error`], for a caller to match on.
///
/// The enum is `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
///
/// # Examples
///
/// ```
/// use escher_geometry::ErrorKind;
/// use escher_geometry::hyperbolic::{ORIGIN, wall_through};
///
/// let kind = wall_through(ORIGIN, ORIGIN).map_err(|error| error.kind());
/// assert_eq!(kind, Err(ErrorKind::DegenerateWall));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// No point of the geometry lies on the ray of the vector.
    ///
    /// In the hyperbolic plane, the vector is not timelike with a positive
    /// last coordinate. A vector is timelike when its Minkowski form with
    /// itself is negative. On the sphere, the vector has no finite, positive
    /// length. This kind also covers a vector with a NaN or infinite
    /// coordinate, and a vector whose form or dot product with itself
    /// underflows to zero or overflows to infinity.
    NotAPoint,
    /// The two points span no wall, such as two equal points, or the two
    /// walls meet in no point, such as two equal walls or two parallel lines.
    ///
    /// The hyperbolic plane, the Euclidean plane, and the sphere all report
    /// this kind, and on the sphere it also covers two antipodal points and
    /// two equal or opposite normals.
    /// It also covers two points so close that rounding leaves the squared
    /// length of the normal found from them not positive and finite, and a
    /// NaN or infinite coordinate. In the hyperbolic plane that squared length
    /// is the form of the normal with itself.
    DegenerateWall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
enum Detail {
    #[error("the vector is not timelike with a positive last coordinate")]
    NotAPoint,
    #[error("the vector has no finite, positive length")]
    NoLength,
    #[error("the points do not span a wall")]
    DegenerateWall,
    #[error("the walls do not meet in a point")]
    WallsDoNotMeet,
}

#[cfg(test)]
mod tests {
    use super::{Error, ErrorKind};

    #[test]
    fn display_of_not_a_point_names_the_operation_and_the_cause() {
        assert_eq!(
            Error::not_a_point().to_string(),
            "cannot normalize point: the vector is not timelike with a positive last coordinate",
        );
    }

    #[test]
    fn display_of_no_length_names_the_operation_and_the_cause() {
        assert_eq!(
            Error::no_length().to_string(),
            "cannot normalize point: the vector has no finite, positive length",
        );
    }

    #[test]
    fn display_of_degenerate_wall_names_the_operation_and_the_cause() {
        assert_eq!(
            Error::degenerate_wall().to_string(),
            "cannot find wall through two points: the points do not span a wall",
        );
    }

    #[test]
    fn display_of_walls_that_do_not_meet_names_the_operation_and_the_cause() {
        assert_eq!(
            Error::walls_do_not_meet().to_string(),
            "cannot find point where two walls meet: the walls do not meet in a point",
        );
    }

    #[test]
    fn kind_matches_each_constructor() {
        assert_eq!(Error::not_a_point().kind(), ErrorKind::NotAPoint);
        assert_eq!(Error::no_length().kind(), ErrorKind::NotAPoint);
        assert_eq!(Error::degenerate_wall().kind(), ErrorKind::DegenerateWall);
        assert_eq!(Error::walls_do_not_meet().kind(), ErrorKind::DegenerateWall);
    }

    #[test]
    fn source_is_none() {
        assert!(core::error::Error::source(&Error::not_a_point()).is_none());
        assert!(core::error::Error::source(&Error::no_length()).is_none());
        assert!(core::error::Error::source(&Error::degenerate_wall()).is_none());
        assert!(core::error::Error::source(&Error::walls_do_not_meet()).is_none());
    }
}
