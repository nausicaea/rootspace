use approx::ulps_eq;
use num_traits::{Float, Inv, Zero};

use crate::mat::Mat4;

impl<R: Float + approx::UlpsEq> Inv for &Mat4<R> {
    type Output = Mat4<R>;

    fn inv(self) -> Self::Output {
        match self.svd() {
            Ok(svd) => {
                if ulps_eq!(svd.det_abs(), R::zero()) {
                    tracing::error!("non-invertible matrix");
                    Mat4::nan()
                } else {
                    let mut s_inv: Mat4<R> = Mat4::identity();
                    s_inv[(0, 0)] = R::one() / svd.sigma[(0, 0)];
                    s_inv[(1, 1)] = R::one() / svd.sigma[(1, 1)];
                    s_inv[(2, 2)] = R::one() / svd.sigma[(2, 2)];
                    s_inv[(3, 3)] = R::one() / svd.sigma[(3, 3)];

                    let u_inv = svd.u.t();
                    let vt_inv = svd.vt.t();

                    u_inv * s_inv * vt_inv
                }
            }
            Err(e) => {
                tracing::error!("singular value decomposition error: {}", e);
                Mat4::nan()
            }
        }
    }
}

impl<R: Float + approx::UlpsEq> Inv for Mat4<R> {
    type Output = Mat4<R>;

    fn inv(self) -> Self::Output {
        (&self).inv()
    }
}

#[cfg(test)]
mod tests {
    use approx::abs_diff_eq;

    use super::*;
    use crate::{affine::Affine, quat::Quat, test_helpers::diff, vec::Vec4};

    #[test]
    fn svd_inv() {
        let m: Mat4<f32> = Affine::builder()
            .with_scale(1.2)
            .with_translation([4.0, 3.0, 5.0, 0.0].into())
            .with_orientation(Quat::with_axis_angle(Vec4::y(), 1.5))
            .build()
            .into();

        assert!(
            abs_diff_eq!(m * m.inv(), Mat4::<f32>::identity(), epsilon = 10.0 * f32::EPSILON),
            "m * m.inv() != Mat4::<f32>::identity()\ndiff:\n{}",
            diff(&(m * m.inv()), &Mat4::<f32>::identity(), |a, b| abs_diff_eq!(
                a,
                b,
                epsilon = 10.0 * f32::EPSILON
            ))
        );
    }
}
