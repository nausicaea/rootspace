use num_traits::float::Float;
use num_traits::{NumAssign};
use std::iter::Sum;
use crate::glamour::affine::Affine;
use crate::glamour::mat::Mat4;
use crate::glamour::ops::norm::Norm;
use crate::glamour::quat::Quat;
use crate::glamour::unit::Unit;
use crate::Vec4;

impl<R> From<Affine<R>> for Mat4<R>
where
    R: Float + NumAssign,
{
    fn from(v: Affine<R>) -> Self {
        From::from(&v)
    }
}

impl<'a, R> From<&'a Affine<R>> for Mat4<R>
where
    R: Float + NumAssign,
{
    fn from(v: &'a Affine<R>) -> Self {
        let mut m: Mat4<R> = v.o.into();
        m[(0, 0)] *= v.s[0];
        m[(1, 1)] *= v.s[1];
        m[(2, 2)] *= v.s[2];
        m[(0, 3)] = v.t[0];
        m[(1, 3)] = v.t[1];
        m[(2, 3)] = v.t[2];
        m
    }
}

impl<R> TryFrom<Mat4<R>> for Affine<R>
where
    R: Copy + num_traits::One + num_traits::Zero + NumAssign + Float + Sum,
{
    type Error = ();

    fn try_from(v: Mat4<R>) -> Result<Self, Self::Error> {
        let mut t: Vec4<R> = v.col(3);
        t.w = R::zero();

        let s = Vec4::new(v.col(0).norm(), v.col(1).norm(), v.col(2).norm(), R::zero());

        let mut rot_m: Mat4<R> = v;
        rot_m[(0, 0)] /= s[0];
        rot_m[(1, 0)] /= s[0];
        rot_m[(2, 0)] /= s[0];
        rot_m[(0, 1)] /= s[1];
        rot_m[(1, 1)] /= s[1];
        rot_m[(2, 1)] /= s[1];
        rot_m[(0, 2)] /= s[2];
        rot_m[(1, 2)] /= s[2];
        rot_m[(2, 2)] /= s[2];
        rot_m[(0, 3)] = R::zero();
        rot_m[(1, 3)] = R::zero();
        rot_m[(2, 3)] = R::zero();
        rot_m[(3, 0)] = R::zero();
        rot_m[(3, 1)] = R::zero();
        rot_m[(3, 2)] = R::zero();
        rot_m[(3, 3)] = R::one();

        let o: Unit<Quat<R>> = rot_m.into();

        Ok(Affine { t, o, s })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affine_implements_try_from_mat4() {
        let m: Mat4<f32> = Mat4::identity();
        assert_eq!(Affine::<f32>::try_from(m), Ok(Affine::<f32>::identity()));
    }
}
