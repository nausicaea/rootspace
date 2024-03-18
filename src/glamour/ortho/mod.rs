use num_traits::Float;

use super::mat::Mat4;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "R: serde::Serialize",
    deserialize = "R: Copy + num_traits::Zero + for<'r> serde::Deserialize<'r>"
))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ortho<R>(pub(crate) Mat4<R>);

impl<R> Ortho<R> {
    pub fn as_matrix(&self) -> &Mat4<R> {
        self.as_ref()
    }
}

impl<R> Ortho<R>
where
    R: Float,
{
    pub fn new(width: R, height: R, near_z: R, far_z: R) -> Self {
        let z = R::zero();
        let o = R::one();
        let t = o + o;

        let r0c0 = t / width;
        let r1c1 = t / height;
        let r2c2 = -t / (far_z - near_z);
        let r2c3 = -(far_z + near_z) / (far_z - near_z);

        Ortho(Mat4::new([
            [r0c0, z, z, z],
            [z, r1c1, z, z],
            [z, z, r2c2, r2c3],
            [z, z, z, o],
        ]))
    }

    pub fn set_dimensions(&mut self, width: R, height: R) {
        let o = R::one();
        let t = o + o;

        self.0[(0, 0)] = t / width;
        self.0[(1, 1)] = t / height;
    }
}

impl<R> AsRef<Mat4<R>> for Ortho<R> {
    fn as_ref(&self) -> &Mat4<R> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glamour::ortho::Ortho;
    use crate::glamour::test_helpers::proptest::bounded_positive_f32;
    use approx::ulps_eq;
    use proptest::{prop_assert, proptest};

    fn testing_ortho() -> Ortho<f32> {
        Ortho::new(1.5, std::f32::consts::PI / 4.0, 0.1, 1000.0)
    }

    #[test]
    fn ortho_implements_as_ref_for_mat4_and_provides_as_matrix() {
        let p = testing_ortho();

        let _: &Mat4<f32> = AsRef::as_ref(&p);
        let _: &Mat4<f32> = p.as_matrix();
    }

    #[test]
    fn ortho_comparison_to_nalgebra_and_cgmath() {
        use cgmath::Matrix;

        let width = 800.0_f32;
        let height = 600.0;
        let nz = 0.001;
        let dz = 1000.0;
        let glamour_ortho = Ortho::new(width, height, nz, nz + dz);
        let nalgebra_ortho =
            nalgebra::Orthographic3::new(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, nz, nz + dz);
        let cgmath_ortho = cgmath::ortho(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, nz, nz + dz);
        assert!(
            ulps_eq!(*glamour_ortho.as_matrix(), nalgebra_ortho.to_homogeneous()),
            "glamour\t\t\t=    {:?}\nnalgebra (transposed)\t=         {:?}\ncgmath (transposed)\t= {:?}",
            *glamour_ortho.as_matrix(),
            nalgebra_ortho.to_homogeneous().transpose(),
            cgmath_ortho.transpose()
        );
    }

    proptest! {
        #[test]
        fn ortho_is_equal_to_nalgebra(width in bounded_positive_f32(-22, 63), height in bounded_positive_f32(-22, 64), nz in bounded_positive_f32(-22, 63), dz in bounded_positive_f32(-22, 63)) {
            let glamour_ortho = Ortho::new(width, height, nz, nz + dz);
            let nalgebra_ortho = nalgebra::Orthographic3::new(-width/2.0, width/2.0, -height/2.0, height/2.0, nz, nz + dz);
            prop_assert!(ulps_eq!(*glamour_ortho.as_matrix(), nalgebra_ortho.to_homogeneous()), "glamour = {:?}\nnalgebra = {:?}", *glamour_ortho.as_matrix(), nalgebra_ortho.to_homogeneous());
        }

        #[test]
        fn ortho_is_equal_to_cgmath(width in bounded_positive_f32(-22, 63), height in bounded_positive_f32(-22, 64), nz in bounded_positive_f32(-22, 63), dz in bounded_positive_f32(-22, 63)) {
            let glamour_ortho = Ortho::new(width, height, nz, nz + dz);
            let cgmath_ortho = cgmath::ortho(-width/2.0, width/2.0, -height/2.0, height/2.0, nz, nz + dz);

            prop_assert!(ulps_eq!(*glamour_ortho.as_matrix(), cgmath_ortho), "glamour = {:?}\ncgmath = {:?}", *glamour_ortho.as_matrix(), cgmath_ortho);
        }
    }
}
