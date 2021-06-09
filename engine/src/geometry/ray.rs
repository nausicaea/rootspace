use std::fmt;

use affine_transform::AffineTransform;
use nalgebra::{Point3, RealField, Scalar, Unit, Vector3};

/// A `Ray` characterises a ray (a line segment with an origin, direction and infinite length in
/// that direction).
#[derive(Debug, Clone, PartialEq)]
pub struct Ray<N>
where
    N: Scalar + RealField,
{
    /// Specifies the origin of the `Ray`.
    pub origin: Point3<N>,
    /// Specifies the direction of the `Ray`.
    pub direction: Unit<Vector3<N>>,
}

impl<N> Ray<N>
where
    N: Scalar + RealField,
{
    /// Creates a new `Ray`.
    pub fn new(origin: Point3<N>, direction: Unit<Vector3<N>>) -> Self {
        Ray { origin, direction }
    }

    /// Extends the `Ray` to the specified position and returns the resulting point.
    pub fn at(&self, position: N) -> Point3<N> {
        self.origin + self.direction.as_ref() * position
    }

    /// Transforms the `Ray` into a new coordinate system determined by an `AffineTransform`
    /// matrix.
    pub fn transform(&self, transform: &AffineTransform<N>) -> Option<Self> {
        let new_origin = transform.transform_point(&self.origin);
        let new_direction = Unit::try_new(transform.transform_vector(&self.direction), N::default_epsilon())?;

        Some(Ray {
            origin: new_origin,
            direction: new_direction,
        })
    }

    /// Applies the inverse of the supplied `AffineTransform` matrix to the `Ray`.
    pub fn inverse_transform(&self, transform: &AffineTransform<N>) -> Option<Self> {
        let new_origin = transform.inverse_transform_point(&self.origin);
        let new_direction = Unit::try_new(
            transform.inverse_transform_vector(&self.direction),
            N::default_epsilon(),
        )?;

        Some(Ray {
            origin: new_origin,
            direction: new_direction,
        })
    }
}

impl<N> fmt::Display for Ray<N>
where
    N: Scalar + RealField,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Ray {{origin: {}, direction: {}}}",
            self.origin,
            self.direction.into_inner()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let o = Point3::new(0.0, 0.0, 0.0);
        let d = Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0));
        let _: Ray<f32> = Ray::new(o, d);
    }

    #[test]
    fn at() {
        let o = Point3::new(0.0, 0.0, 0.0);
        let d = Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0));
        let r = Ray::new(o, d);

        assert_eq!(r.at(0.0), o);
        assert_eq!(r.at(1.0), Point3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn transform() {
        let o = Point3::new(0.0, 0.0, 0.0);
        let d = Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0));
        let r = Ray::new(o, d);

        let s = r.transform(&AffineTransform::identity()).unwrap();

        assert_eq!(s, r);
    }

    #[test]
    fn inverse_transform() {
        let o = Point3::new(0.0, 0.0, 0.0);
        let d = Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0));
        let r = Ray::new(o, d);

        let s = r.inverse_transform(&AffineTransform::identity()).unwrap();

        assert_eq!(s, r);
    }
}
