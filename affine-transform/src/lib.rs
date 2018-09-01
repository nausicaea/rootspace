//! This crate provides a decomposed version of an affine TRS matrix, one missing in `nalgebra`.
#![deny(missing_docs)]

extern crate alga;
extern crate nalgebra;

use alga::linear::ProjectiveTransformation;
use nalgebra::{
    norm, one, zero, Affine3, Matrix4, Point3, Real, Rotation3, Scalar, Translation3, UnitQuaternion, Vector3, U1, U3,
};

/// Unfortunately, `nalgebra` does not provide a decomposed affine matrix representation
/// (equivalent to Isometry but with translational, rotational, and non-uniform scaling
/// components). `AffineTransform` implements this instead. `Affine3` instances can be converted to
/// and from `AffineTransform`, assuming they contain no shear component.
///
/// # Example
///
/// ```
/// extern crate affine_transform;
/// extern crate nalgebra;
///
/// use affine_transform::AffineTransform;
/// use nalgebra::Affine3;
///
/// let a: AffineTransform<f32> = AffineTransform::identity();
/// let b: Affine3<f32> = Affine3::identity();
///
/// assert_eq!(Into::<Affine3<_>>::into(a), b);
/// assert_eq!(a, AffineTransform::<_>::from(b));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineTransform<N>
where
    N: Scalar + Real,
{
    /// Holds the translational component of the TRS matrix.
    pub translation: Translation3<N>,
    /// Holds the rotational component of the TRS matrix.
    pub rotation: UnitQuaternion<N>,
    /// Holds the non-uniform scale component of the TRS matrix.
    pub scale: Vector3<N>,
}

impl<N> AffineTransform<N>
where
    N: Scalar + Real,
{
    /// Creates a new, identity `AffineTransform` matrix.
    pub fn identity() -> Self {
        AffineTransform {
            translation: Translation3::identity(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(one(), one(), one()),
        }
    }

    /// Creates a new instance of `AffineTransform` from its parts.
    pub fn from_parts(translation: Translation3<N>, rotation: UnitQuaternion<N>, scale: Vector3<N>) -> Self {
        AffineTransform {
            translation,
            rotation,
            scale,
        }
    }

    /// Transforms the specified point.
    pub fn transform_point(&self, point: &Point3<N>) -> Point3<N> {
        self.translation * self.rotation * Point3::from_coordinates(self.scale.component_mul(&point.coords))
    }

    /// Transforms the specified vector.
    pub fn transform_vector(&self, vector: &Vector3<N>) -> Vector3<N> {
        self.rotation * self.scale.component_mul(vector)
    }

    /// Applies the inverse transformation to the specified point.
    pub fn inverse_transform_point(&self, point: &Point3<N>) -> Point3<N> {
        let untranslated = self.translation.inverse_transform_point(point);
        let unrotated = self.rotation.inverse_transform_point(&untranslated);
        Point3::from_coordinates(unrotated.coords.component_div(&self.scale))
    }

    /// Applies the inverse transformation to the specified vector.
    pub fn inverse_transform_vector(&self, vector: &Vector3<N>) -> Vector3<N> {
        let unrotated = self.rotation.inverse_transform_vector(vector);
        unrotated.component_div(&self.scale)
    }

    /// Assembles the internal scale vector into an `Affine3` matrix.
    fn scale_matrix(&self) -> Affine3<N> {
        Affine3::from_matrix_unchecked(Matrix4::new(
            self.scale.x,
            zero(),
            zero(),
            zero(),
            zero(),
            self.scale.y,
            zero(),
            zero(),
            zero(),
            zero(),
            self.scale.z,
            zero(),
            zero(),
            zero(),
            zero(),
            one(),
        ))
    }
}

impl<N> From<Affine3<N>> for AffineTransform<N>
where
    N: Scalar + Real,
{
    /// Decomposes an affine T*R*S matrix into their constituents, where T corresponds to the
    /// translational component, R refers to a rotation, and S refers to non-uniform scaling
    /// (without shear).
    fn from(value: Affine3<N>) -> Self {
        // Obtain the translational component.
        let t = Translation3::from_vector(value.matrix().fixed_slice::<U3, U1>(0, 3).into_owned());

        // Obtain the non-uniform scaling component.
        let s = Vector3::new(
            norm(&value.matrix().column(0).into_owned()),
            norm(&value.matrix().column(1).into_owned()),
            norm(&value.matrix().column(2).into_owned()),
        );

        // Obtain the rotational component.
        let mut r = value.matrix().fixed_slice::<U3, U3>(0, 0).into_owned();
        s.iter().enumerate().for_each(|(i, scale_component)| {
            let mut temp = r.column_mut(i);
            temp /= *scale_component;
        });

        let r = UnitQuaternion::from_rotation_matrix(&Rotation3::from_matrix_unchecked(r));

        AffineTransform {
            translation: t,
            rotation: r,
            scale: s,
        }
    }
}

impl<N> Into<Affine3<N>> for AffineTransform<N>
where
    N: Scalar + Real,
{
    /// Recomposes a TRS matrix (`AffineTransform`) into an `Affine3` matrix.
    fn into(self) -> Affine3<N> {
        self.translation * self.rotation * self.scale_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let a: AffineTransform<f32> = AffineTransform::identity();

        assert_eq!(a.translation.vector, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(a.rotation, UnitQuaternion::identity());
        assert_eq!(a.scale, Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_from_and_into_affine3() {
        let a = Affine3::from_matrix_unchecked(Matrix4::new(
            1.0, 0.0, 0.0, 0.1, 0.0, 2.0, 0.0, 0.2, 0.0, 0.0, 3.0, 0.3, 0.0, 0.0, 0.0, 1.0,
        ));

        let b: AffineTransform<f32> = a.into();

        assert_eq!(b.translation.vector, Vector3::new(0.1, 0.2, 0.3));
        assert_eq!(b.rotation, UnitQuaternion::identity());

        let c: Affine3<f32> = b.into();

        assert_eq!(c, a);
    }

    #[test]
    fn test_transform_point() {
        let a = AffineTransform::from_parts(
            Translation3::from_vector(Vector3::new(0.1, 0.2, 0.3)),
            UnitQuaternion::identity(),
            Vector3::new(1.0, 2.0, 3.0),
        );
        let b = Point3::new(1.0, 1.0, 1.0);
        let c = a.transform_point(&b);

        assert_eq!(c, Point3::new(1.1, 2.2, 3.3));
    }

    #[test]
    fn test_transform_vector() {
        let a = AffineTransform::from_parts(
            Translation3::from_vector(Vector3::new(0.1, 0.2, 0.3)),
            UnitQuaternion::identity(),
            Vector3::new(1.0, 2.0, 3.0),
        );
        let b = Vector3::new(1.0, 1.0, 1.0);
        let c = a.transform_vector(&b);

        assert_eq!(c, Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_inverse_transform_point() {
        let a = AffineTransform::from_parts(
            Translation3::from_vector(Vector3::new(0.1, 0.2, 0.3)),
            UnitQuaternion::identity(),
            Vector3::new(1.0, 2.0, 3.0),
        );
        let b = Point3::new(1.1, 2.2, 3.3);
        let c = a.inverse_transform_point(&b);

        assert_eq!(c, Point3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_inverse_transform_vector() {
        let a = AffineTransform::from_parts(
            Translation3::from_vector(Vector3::new(0.1, 0.2, 0.3)),
            UnitQuaternion::identity(),
            Vector3::new(1.0, 2.0, 3.0),
        );
        let b = Vector3::new(1.0, 2.0, 3.0);
        let c = a.inverse_transform_vector(&b);

        assert_eq!(c, Vector3::new(1.0, 1.0, 1.0));
    }
}
