use num_traits::Float;
use crate::mat::Mat4;
use thiserror::Error;
use approx::{RelativeEq, relative_eq};

#[derive(Debug, PartialEq, Clone)]
pub struct Ortho<R>(Mat4<R>);

impl<R> Ortho<R> {
    pub fn as_matrix(&self) -> &Mat4<R> {
        self.as_ref()
    }

    pub fn builder() -> OrthoBuilder<R> {
        OrthoBuilder::default()
    }
}

impl<R> AsRef<Mat4<R>> for Ortho<R> {
    fn as_ref(&self) -> &Mat4<R> {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrthoBuilder<R> {
    aspect: Option<R>,
    fov_y: Option<R>,
    near_z: Option<R>,
    far_z: Option<R>,
}

impl<R> OrthoBuilder<R> {
    pub fn with_aspect(mut self, a: R) -> Self {
        self.aspect = Some(a);
        self
    }

    pub fn with_fov_y(mut self, f: R) -> Self {
        self.fov_y = Some(f);
        self
    }

    pub fn with_near_z(mut self, n: R) -> Self {
        self.near_z = Some(n);
        self
    }

    pub fn with_far_z(mut self, f: R) -> Self {
        self.far_z = Some(f);
        self
    }
}

impl<R> OrthoBuilder<R> 
where
    R: Float + RelativeEq,
{
    pub fn build(self) -> Result<Ortho<R>, Error> {
        let aspect = match self.aspect {
            None => Err(Error::MissingAspectRatio),
            Some(a) if a < R::zero() => Err(Error::AspectRatioMustBePositive),
            Some(a) if relative_eq!(a, R::zero()) => Err(Error::AspectRatioMustBePositive),
            Some(a) => Ok(a),
        }?;

        let fov_y = match self.fov_y {
            None => Err(Error::MissingFieldOfView),
            Some(f) if relative_eq!(f, R::zero()) => Err(Error::FieldOfViewMustBeNonZero),
            Some(f) => Ok(f),
        }?;

        let (near_z, far_z) = match (self.near_z, self.far_z) {
            (None, _) => Err(Error::MissingNearFrustumPlane),
            (_, None) => Err(Error::MissingFarFrustumPlane),
            (Some(n), Some(f)) if relative_eq!(n, f) => Err(Error::FrustumZPlanesMustNotSuperimpose),
            (Some(n), Some(f)) => Ok((n, f)),
        }?;

        let z = R::zero();
        let o = R::one();
        let t = o + o;
        let width = far_z * (fov_y / t).tan();
        let height = width / aspect;
        let m00 = t / width;
        let m11 = t / height;
        let m22 = -t / (far_z - near_z);
        let m23 = -(far_z + near_z) / (far_z - near_z);

        Ok(Ortho(Mat4::from([
            [m00, z, z, z],
            [z, m11, z, z],
            [z, z, m22, m23],
            [z, z, z, o],
        ])))
    }
}

impl<R> Default for OrthoBuilder<R> {
    fn default() -> Self {
        OrthoBuilder {
            aspect: None,
            fov_y: None,
            near_z: None,
            far_z: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum Error {
    #[error("missing parameter 'aspect'. Use the method `OrthoBuilder::with_aspect` to specify this parameter")]
    MissingAspectRatio,
    #[error("the 'aspect' parameter must be non-negative and larger than zero")]
    AspectRatioMustBePositive,
    #[error("missing parameter 'fov_y'. Use the method `OrthoBuilder::with_fov_y` to specify this parameter")]
    MissingFieldOfView,
    #[error("the 'fov_y' parameter must not be zero")]
    FieldOfViewMustBeNonZero,
    #[error("missing parameter 'near_z'. Use the method `OrthoBuilder::with_near_z` to specify this parameter")]
    MissingNearFrustumPlane,
    #[error("missing parameter 'far_z'. Use the method `OrthoBuilder::with_far_z` to specify this parameter")]
    MissingFarFrustumPlane,
    #[error("the 'near_z' and 'far_z' parameters must not be similar")]
    FrustumZPlanesMustNotSuperimpose,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn testing_ortho() -> Ortho<f32> {
        Ortho::builder()
            .with_aspect(1.5)
            .with_fov_y(std::f32::consts::PI / 4.0)
            .with_near_z(0.1)
            .with_far_z(1000.0)
            .build()
            .unwrap()
    }

    fn testing_ortho_builder() -> OrthoBuilder<f32> {
        OrthoBuilder::default()
    }

    fn testing_complete_ortho_builder() -> OrthoBuilder<f32> {
        Ortho::builder()
            .with_aspect(1.5)
            .with_fov_y(std::f32::consts::PI / 4.0)
            .with_near_z(0.1)
            .with_far_z(1000.0)
    }

    #[test]
    fn ortho_provides_builder_method() {
        let _: OrthoBuilder<f32> = Ortho::builder();
    }

    #[test]
    fn ortho_implements_as_ref_for_mat4_and_provides_as_matrix() {
        let p = testing_ortho();

        let _: &Mat4<f32> = AsRef::as_ref(&p);
        let _: &Mat4<f32> = p.as_matrix();
    }

    #[test]
    fn ortho_builder_provides_with_aspect_method() {
        let pb = testing_ortho_builder();

        let _: OrthoBuilder<f32> = pb.with_aspect(1.0f32);
    }

    #[test]
    fn ortho_builder_provides_with_fov_y_method() {
        let pb = testing_ortho_builder();

        let _: OrthoBuilder<f32> = pb.with_fov_y(1.0f32);
    }

    #[test]
    fn ortho_builder_provides_with_near_z_method() {
        let pb = testing_ortho_builder();

        let _: OrthoBuilder<f32> = pb.with_near_z(1.0f32);
    }

    #[test]
    fn ortho_builder_provides_with_far_z_method() {
        let pb = testing_ortho_builder();

        let _: OrthoBuilder<f32> = pb.with_far_z(1.0f32);
    }

    #[test]
    fn ortho_builder_provides_build_method() {
        let pb = testing_ortho_builder();

        let _: Result<Ortho<f32>, Error> = pb.build();
    }

    #[test]
    fn ortho_builder_fails_when_aspect_is_missing() {
        let mut pb = testing_complete_ortho_builder();
        pb.aspect = None;

        assert_eq!(pb.build(), Err(Error::MissingAspectRatio));
    }

    #[test]
    fn ortho_builder_fails_when_aspect_is_negative() {
        let pb = testing_complete_ortho_builder();

        assert_eq!(pb.with_aspect(-1.0).build(), Err(Error::AspectRatioMustBePositive));
    }

    #[test]
    fn ortho_builder_fails_when_aspect_is_zero() {
        let pb = testing_complete_ortho_builder();

        assert_eq!(pb.with_aspect(0.0).build(), Err(Error::AspectRatioMustBePositive));
    }

    #[test]
    fn ortho_builder_fails_when_fov_y_is_missing() {
        let mut pb = testing_complete_ortho_builder();
        pb.fov_y = None;

        assert_eq!(pb.build(), Err(Error::MissingFieldOfView));
    }

    #[test]
    fn ortho_builder_fails_when_fov_y_is_zero() {
        let pb = testing_complete_ortho_builder();

        assert_eq!(pb.with_fov_y(0.0).build(), Err(Error::FieldOfViewMustBeNonZero));
    }

    #[test]
    fn ortho_builder_fails_when_near_z_is_missing() {
        let mut pb = testing_complete_ortho_builder();
        pb.near_z = None;

        assert_eq!(pb.build(), Err(Error::MissingNearFrustumPlane));
    }

    #[test]
    fn ortho_builder_fails_when_far_z_is_missing() {
        let mut pb = testing_complete_ortho_builder();
        pb.far_z = None;

        assert_eq!(pb.build(), Err(Error::MissingFarFrustumPlane));
    }

    #[test]
    fn ortho_builder_fails_when_near_z_and_far_z_are_equal() {
        let pb = testing_complete_ortho_builder();

        assert_eq!(pb.with_near_z(2.0).with_far_z(2.0).build(), Err(Error::FrustumZPlanesMustNotSuperimpose));
    }

    #[test]
    fn ortho_contains_a_valid_mat4() {
        let p = Ortho::builder()
            .with_aspect(1.5)
            .with_fov_y(std::f32::consts::PI / 4.0)
            .with_near_z(0.1)
            .with_far_z(1000.0)
            .build()
            .unwrap();

        let m4: Mat4<f32> = p.0;

        assert_relative_eq!(m4[(0, 0)],  0.004828427216);
        assert_relative_eq!(m4[(0, 1)],  0.0);
        assert_relative_eq!(m4[(0, 2)],  0.0);
        assert_relative_eq!(m4[(0, 3)],  0.0);
        assert_relative_eq!(m4[(1, 0)],  0.0);
        assert_relative_eq!(m4[(1, 1)],  0.007242640824);
        assert_relative_eq!(m4[(1, 2)],  0.0);
        assert_relative_eq!(m4[(1, 3)],  0.0);
        assert_relative_eq!(m4[(2, 0)],  0.0);
        assert_relative_eq!(m4[(2, 1)],  0.0);
        assert_relative_eq!(m4[(2, 2)], -0.00200020002);
        assert_relative_eq!(m4[(2, 3)], -1.00020002);
        assert_relative_eq!(m4[(3, 0)],  0.0);
        assert_relative_eq!(m4[(3, 1)],  0.0);
        assert_relative_eq!(m4[(3, 2)],  0.0);
        assert_relative_eq!(m4[(3, 3)],  1.0);
    }
}
