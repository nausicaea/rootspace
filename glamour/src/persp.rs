use approx::{relative_eq, AbsDiffEq, RelativeEq, UlpsEq};
use num_traits::Float;

use crate::mat::Mat4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Persp<R>(Mat4<R>);

impl<R> Persp<R> {
    pub fn as_matrix(&self) -> &Mat4<R> {
        self.as_ref()
    }
}

impl<R> Persp<R>
where
    R: Float + RelativeEq,
{
    pub fn new(aspect: R, fov_y: R, near_z: R, far_z: R) -> Self {
        let z = R::zero();
        let o = R::one();
        let t = o + o;

        if aspect < z || relative_eq!(aspect, z) {
            panic!("the 'aspect' parameter must positive and non-zero");
        }

        if relative_eq!(fov_y, z) {
            panic!("the 'fov_y' parameter must positive and non-zero");
        }

        if relative_eq!(near_z, far_z) {
            panic!("the 'near_z' and 'far_z' parameters must not be similar");
        } else if near_z > far_z {
            panic!("the 'near_z' parameter must not be larger than 'far_z'");
        }

        let tan = o / (fov_y / t).tan();
        let f_rel = -far_z / (far_z - near_z);

        let m00 = aspect * tan;
        let m11 = tan;
        let m22 = f_rel;
        let m23 = near_z * f_rel;
        let m32 = -o;

        Persp(Mat4::new([
            [m00, z, z, z],
            [z, m11, z, z],
            [z, z, m22, m23],
            [z, z, m32, z],
        ]))
    }
}

impl<R> Persp<R>
where
    R: Float + RelativeEq,
{
    pub fn set_aspect(&mut self, aspect: R) {
        let z = R::zero();

        if aspect < z || relative_eq!(aspect, z) {
            panic!("the 'aspect' parameter must positive and non-zero");
        }

        self.0[(0, 0)] = aspect * self.0[(1, 1)];
    }
}

impl<R> Persp<R>
where
    R: Float,
{
    pub fn inv(&self) -> Self {
        let z = R::zero();
        let o = R::one();
        let m00 = o / self.0[(0, 0)];
        let m11 = o / self.0[(1, 1)];
        let m32 = o / self.0[(2, 3)];
        let m33 = self.0[(2, 2)] / self.0[(2, 3)];

        Persp(Mat4::from([
            [m00, z, z, z],
            [z, m11, z, z],
            [z, z, z, -o],
            [z, z, m32, m33],
        ]))
    }
}

impl<R> AsRef<Mat4<R>> for Persp<R> {
    fn as_ref(&self) -> &Mat4<R> {
        &self.0
    }
}

impl<R> AbsDiffEq for Persp<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.0.abs_diff_eq(&rhs.0, epsilon)
    }
}

impl<R> RelativeEq for Persp<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.0.relative_eq(&rhs.0, epsilon, max_relative)
    }
}

impl<R> UlpsEq for Persp<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.0.ulps_eq(&rhs.0, epsilon, max_ulps)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    fn testing_persp() -> Persp<f32> {
        Persp::new(1.5, std::f32::consts::PI / 4.0, 0.1, 1000.0)
    }

    #[test]
    fn persp_implements_as_ref_for_mat4_and_provides_as_matrix() {
        let p = testing_persp();

        let _: &Mat4<f32> = AsRef::as_ref(&p);
        let _: &Mat4<f32> = p.as_matrix();
    }

    #[test]
    fn persp_supports_inversion() {
        let i = testing_persp();

        let m4: Mat4<f32> = i.0;

        assert_relative_eq!(m4[(0, 0)], 0.621320332);
        assert_relative_eq!(m4[(0, 1)], 0.0);
        assert_relative_eq!(m4[(0, 2)], 0.0);
        assert_relative_eq!(m4[(0, 3)], 0.0);
        assert_relative_eq!(m4[(1, 0)], 0.0);
        assert_relative_eq!(m4[(1, 1)], 0.414213555);
        assert_relative_eq!(m4[(1, 2)], 0.0);
        assert_relative_eq!(m4[(1, 3)], 0.0);
        assert_relative_eq!(m4[(2, 0)], 0.0);
        assert_relative_eq!(m4[(2, 1)], 0.0);
        assert_relative_eq!(m4[(2, 2)], 0.0);
        assert_relative_eq!(m4[(2, 3)], -1.0);
        assert_relative_eq!(m4[(3, 0)], 0.0);
        assert_relative_eq!(m4[(3, 1)], 0.0);
        assert_relative_eq!(m4[(3, 2)], -4.99950000);
        assert_relative_eq!(m4[(3, 3)], 5.00050000);
    }
}
