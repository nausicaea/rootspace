use crate::mat::Mat4;
use num_traits::{Float, NumCast, Zero};
use std::any::type_name;

impl<R: Float> Mat4<R> {
    /// Based on
    /// [LAPACK-dgesvd](https://www.netlib.org/lapack/explore-html/d1/d7f/group__gesvd_gac6bd5d4e645049e49bb70691180abf07.html#gac6bd5d4e645049e49bb70691180abf07)
    pub fn svd(&self) -> Result<Svd<R>, Error> {
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

        if info < 0 {
            let arg_name = match info {
                -1 => "jobu",
                -2 => "jobvt",
                -3 => "m",
                -4 => "n",
                -5 => "a",
                -6 => "lda",
                -7 => "s",
                -8 => "u",
                -9 => "ldu",
                -10 => "vt",
                -11 => "ldvt",
                -12 => "work",
                -13 => "lwork",
                _ => "unknown",
            };
            return Err(Error::LapackDgeSvdIllegalArgument {
                docs_url: "https://www.netlib.org/lapack/explore-html/d1/d7f/group__gesvd_gac6bd5d4e645049e49bb70691180abf07.html#gac6bd5d4e645049e49bb70691180abf07",
                info,
                arg_name,
            });
        } else if info > 0 {
            return Err(Error::LapackDgeSvdNonConvergentDbdSqr {
                docs_url: "https://www.netlib.org/lapack/explore-html/d1/d7f/group__gesvd_gac6bd5d4e645049e49bb70691180abf07.html#gac6bd5d4e645049e49bb70691180abf07",
                info,
                num_superdiagonals: info,
            });
        }

        let mut sigma = Mat4::identity();
        sigma[(0, 0)] = num_traits::cast(s[0]).ok_or(Error::NumCast(type_name::<R>()))?;
        sigma[(1, 1)] = num_traits::cast(s[1]).ok_or(Error::NumCast(type_name::<R>()))?;
        sigma[(2, 2)] = num_traits::cast(s[2]).ok_or(Error::NumCast(type_name::<R>()))?;
        sigma[(3, 3)] = num_traits::cast(s[3]).ok_or(Error::NumCast(type_name::<R>()))?;

        let u = try_into_r(u)?;
        let vt = try_into_r(vt)?;

        Ok(Svd { sigma, u, vt })
    }
}

#[derive(Debug)]
pub struct Svd<R> {
    pub sigma: Mat4<R>,
    pub u: Mat4<R>,
    pub vt: Mat4<R>,
}

impl<R: std::ops::Mul<R, Output = R> + Copy> Svd<R> {
    pub fn det_abs(&self) -> R {
        self.sigma[(0, 0)] * self.sigma[(1, 1)] * self.sigma[(2, 2)] * self.sigma[(3, 3)]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to cast {0} to f64")]
    NumCast(&'static str),
    #[error(
        "Argument '{arg_name}' to function 'lapack::dgesvd' has illegal value: info={info}. See for more information: {docs_url}"
    )]
    LapackDgeSvdIllegalArgument {
        arg_name: &'static str,
        info: i32,
        docs_url: &'static str,
    },
    #[error(
        "{num_superdiagonals} superdiagonals did not converge in function 'lapack::dgesvd': info={info}. See for more information: {docs_url}"
    )]
    LapackDgeSvdNonConvergentDbdSqr {
        docs_url: &'static str,
        info: i32,
        num_superdiagonals: i32,
    },
}

fn try_into_f64<R: NumCast + Copy>(value: Mat4<R>) -> Result<[f64; 16], Error> {
    let generic_data: [R; 16] = value.into();

    let mut f64_data: [f64; 16] = [0.0; 16];
    for (i, element) in generic_data.into_iter().enumerate() {
        f64_data[i] = num_traits::cast::<R, f64>(element).ok_or(Error::NumCast(type_name::<R>()))?;
    }

    Ok(f64_data)
}

fn try_into_r<R: NumCast + Copy + Zero>(value: [f64; 16]) -> Result<Mat4<R>, Error> {
    let mut generic_data: [R; 16] = [R::zero(); 16];
    for (i, element) in value.into_iter().enumerate() {
        generic_data[i] = num_traits::cast::<f64, R>(element).ok_or(Error::NumCast(type_name::<R>()))?;
    }

    let mat: Mat4<R> = generic_data.into();

    Ok(mat)
}

#[cfg(test)]
mod tests {
    use crate::affine::Affine;
    use crate::mat::Mat4;
    use crate::mat::ops::svd::Svd;
    use crate::quat::Quat;
    use crate::test_helpers::diff;
    use crate::vec::Vec4;
    use approx::{assert_ulps_ne, relative_eq, ulps_eq};

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
