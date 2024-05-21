use num_traits::Float;

use super::mat::Mat4;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "R: serde::Serialize",
    deserialize = "R: Copy + num_traits::Zero + for<'r> serde::Deserialize<'r>"
))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Persp<R>(pub(crate) Mat4<R>);

impl<R> Persp<R> {
    pub fn as_matrix(&self) -> &Mat4<R> {
        self.as_ref()
    }
}

impl<R> Persp<R>
where
    R: Float,
{
    pub fn new(aspect: R, fov_y: R, near_z: R, far_z: R) -> Self {
        let zero = R::zero();
        let one = R::one();
        let two = one + one;

        let cot = one / (fov_y / two).tan();

        let r0c0 = cot / aspect;
        let r1c1 = cot;
        let r2c2 = (far_z + near_z) / (near_z - far_z);
        let r2c3 = (two * far_z * near_z) / (near_z - far_z);
        let r3c2 = -one;

        Persp(Mat4::new([
            [r0c0, zero, zero, zero],
            [zero, r1c1, zero, zero],
            [zero, zero, r2c2, r2c3],
            [zero, zero, r3c2, zero],
        ]))
    }

    pub fn set_aspect(&mut self, aspect: R) {
        self.0[(0, 0)] = aspect * self.0[(1, 1)];
    }
}

impl<R> AsRef<Mat4<R>> for Persp<R> {
    fn as_ref(&self) -> &Mat4<R> {
        &self.0
    }
}

#[allow(unused_parens)]
#[cfg(test)]
mod tests {
    use approx::ulps_eq;
    use proptest::{prop_assert, proptest};

    use super::*;
    use crate::test_helpers::proptest::bounded_positive_f32;

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
    fn persp_comparison_to_nalgebra_and_cgmath() {
        use cgmath::Matrix;

        let a = 0.8_f32;
        let f = std::f32::consts::PI * 0.75;
        let nz = 0.001;
        let dz = 1000.0;
        let glamour_persp = Persp::new(a, f, nz, nz + dz);
        let nalgebra_persp = nalgebra::Perspective3::new(a, f, nz, nz + dz);
        let cgmath_persp = cgmath::perspective(cgmath::Rad(f), a, nz, nz + dz);
        assert!(
            ulps_eq!(*glamour_persp.as_matrix(), nalgebra_persp.to_homogeneous()),
            "glamour\t\t\t=    {:?}\nnalgebra (transposed)\t=         {:?}\ncgmath (transposed)\t= {:?}",
            *glamour_persp.as_matrix(),
            nalgebra_persp.to_homogeneous().transpose(),
            cgmath_persp.transpose()
        );
    }

    proptest! {
        #[test]
        fn persp_is_equal_to_nalgebra(a in bounded_positive_f32(-22, 63), f in (2.0*f32::EPSILON)..(std::f32::consts::PI), nz in bounded_positive_f32(-22, 63), dz in bounded_positive_f32(-22, 63)) {
            let glamour_persp = Persp::new(a, f, nz, nz + dz);
            let nalgebra_persp = nalgebra::Perspective3::new(a, f, nz, nz + dz);
            prop_assert!(ulps_eq!(*glamour_persp.as_matrix(), nalgebra_persp.to_homogeneous()), "glamour = {:?}\nnalgebra = {:?}", *glamour_persp.as_matrix(), nalgebra_persp.to_homogeneous());
        }

        #[test]
        fn persp_is_equal_to_cgmath(a in bounded_positive_f32(-22, 63), f in (2.0*f32::EPSILON)..(std::f32::consts::PI), nz in bounded_positive_f32(-22, 63), dz in bounded_positive_f32(-22, 63)) {
            let glamour_persp = Persp::new(a, f, nz, nz + dz);
            let cgmath_persp = cgmath::perspective(cgmath::Rad(f), a, nz, nz + dz);

            prop_assert!(ulps_eq!(*glamour_persp.as_matrix(), cgmath_persp), "glamour = {:?}\ncgmath = {:?}", *glamour_persp.as_matrix(), cgmath_persp);
        }
    }
}
