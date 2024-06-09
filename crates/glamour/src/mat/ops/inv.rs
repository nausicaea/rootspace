use approx::ulps_eq;
use num_traits::{Float, Inv, NumCast, Zero};

use crate::mat::Mat4;

impl<R: Float> Mat4<R> {
    /// Based on
    /// [LAPACK-dgesvd](https://www.netlib.org/lapack/explore-html/d1/d7f/group__gesvd_gac6bd5d4e645049e49bb70691180abf07.html#gac6bd5d4e645049e49bb70691180abf07)
    fn svd(&self) -> Result<Svd<R>, Error> {
        let mut a = try_into_f64(*self)?;
        let mut s = [0f64; 4];
        let mut u = [0f64; 16];
        let mut vt = [0f64; 16];
        let mut work = [0f64; 16];
        let mut info = 0i32;

        unsafe {
            lapack::dgesvd(
                b'A', b'A', 4, 4, &mut a, 4, &mut s, &mut u, 4, &mut vt, 4, &mut work, 268, &mut info,
            )
        };

        if info != 0 {
            return Err(Error::Lapack(info));
        }

        let mut sigma = Mat4::identity();
        sigma[(0, 0)] = num_traits::cast(s[0]).ok_or(Error::NumCast)?;
        sigma[(1, 1)] = num_traits::cast(s[1]).ok_or(Error::NumCast)?;
        sigma[(2, 2)] = num_traits::cast(s[2]).ok_or(Error::NumCast)?;
        sigma[(3, 3)] = num_traits::cast(s[3]).ok_or(Error::NumCast)?;

        let u = try_into_r(u)?;
        let vt = try_into_r(vt)?;

        Ok(Svd { sigma, u, vt })
    }
}

#[derive(Debug)]
struct Svd<R> {
    pub sigma: Mat4<R>,
    pub u: Mat4<R>,
    pub vt: Mat4<R>,
}

impl<R: std::ops::Mul<R, Output = R> + Copy> Svd<R> {
    fn det_abs(&self) -> R {
        self.sigma[(0, 0)] * self.sigma[(1, 1)] * self.sigma[(2, 2)] * self.sigma[(3, 3)]
    }
}

impl<'a, R: Float + approx::UlpsEq> Inv for &'a Mat4<R> {
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

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Unable to cast a generic float type to f64")]
    NumCast,
    #[error("Non-zero return code of lapack::dgesvd: {}", .0)]
    Lapack(i32),
}

fn try_into_f64<R: NumCast + Copy>(value: Mat4<R>) -> Result<[f64; 16], Error> {
    let generic_data: [R; 16] = value.into();

    let mut f64_data: [f64; 16] = [0.0; 16];
    for (i, element) in generic_data.into_iter().enumerate() {
        f64_data[i] = num_traits::cast::<R, f64>(element).ok_or(Error::NumCast)?;
    }

    Ok(f64_data)
}

fn try_into_r<R: NumCast + Copy + Zero>(value: [f64; 16]) -> Result<Mat4<R>, Error> {
    let mut generic_data: [R; 16] = [R::zero(); 16];
    for (i, element) in value.into_iter().enumerate() {
        generic_data[i] = num_traits::cast::<f64, R>(element).ok_or(Error::NumCast)?;
    }

    let mat: Mat4<R> = generic_data.into();

    Ok(mat)
}

#[cfg(test)]
mod tests {
    use approx::{abs_diff_eq, assert_ulps_ne, relative_eq, ulps_eq};

    use crate::{affine::Affine, quat::Quat, test_helpers::diff, vec::Vec4};

    use super::*;

    #[test]
    fn svd_inv() {
        let m: Mat4<f32> = Affine::builder()
            .with_scale(1.2)
            .with_translation([4.0, 3.0, 5.0, 0.0].into())
            .with_orientation(Quat::with_axis_angle(Vec4::y(), 1.5))
            .build()
            .into();

        assert!(
            abs_diff_eq!(m * m.inv(), Mat4::<f32>::identity(), epsilon=10.0*f32::EPSILON),
            "m * m.inv() != Mat4::<f32>::identity()\ndiff:\n{}",
            diff(&(m * m.inv()), &Mat4::<f32>::identity(), |a, b| abs_diff_eq!(a, b, epsilon=10.0*f32::EPSILON))
        );
    }

    #[test]
    fn svd() {
        let m: Mat4<f32> = Affine::builder()
            .with_scale(1.2)
            .with_translation([4.0, 3.0, 5.0, 0.0].into())
            .with_orientation(Quat::with_axis_angle(Vec4::y(), 1.5))
            .build()
            .into();

        let Svd { sigma, u, vt } = m.svd().unwrap();

        let mut s_inv: Mat4<f32> = Mat4::identity();
        s_inv[(0, 0)] = 1.0 / sigma[(0, 0)];
        s_inv[(1, 1)] = 1.0 / sigma[(1, 1)];
        s_inv[(2, 2)] = 1.0 / sigma[(2, 2)];
        s_inv[(3, 3)] = 1.0 / sigma[(3, 3)];

        assert!(
            ulps_eq!(sigma * s_inv, Mat4::<f32>::identity()),
            "s * s_inv != Mat4::<f32>::identity()\ndiff:\n{}",
            diff(&(sigma * s_inv), &Mat4::<f32>::identity(), |a, b| ulps_eq!(a, b))
        );

        let u_inv = u.t();

        assert!(
            relative_eq!(u * u_inv, Mat4::<f32>::identity()),
            "u * u_inv != Mat4::<f32>::identity()\ndiff:\n{}",
            diff(&(u * u_inv), &Mat4::<f32>::identity(), |a, b| relative_eq!(a, b))
        );

        let vt_inv = vt.t();

        assert!(
            ulps_eq!(vt * vt_inv, Mat4::<f32>::identity()),
            "vt * vt_inv != Mat4::<f32>::identity()\ndiff:\n{}",
            diff(&(vt * vt_inv), &Mat4::<f32>::identity(), |a, b| ulps_eq!(a, b))
        );
    }

    #[test]
    fn svd_det() {
        let m: Mat4<f32> = Affine::builder()
            .with_scale(1.2)
            .with_translation([4.0, 3.0, 5.0, 0.0].into())
            .with_orientation(Quat::with_axis_angle(Vec4::y(), 1.5))
            .build()
            .into();

        let svd = m.svd().unwrap();

        assert_ulps_ne!(svd.det_abs(), 0.0f32);
    }

}
