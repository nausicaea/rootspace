use std::{
    iter::Sum,
    ops::{Div, Mul},
};

use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use forward_ref::{forward_ref_binop, forward_ref_unop};
use num_traits::{Float, One, Zero};

use crate::{
    mat::{Mat4, Vec4},
    ops::norm::Norm,
    Mat3, Unit, Vec3,
};

#[cfg_attr(any(test, feature = "serde_support"), derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct Quat<R> {
    pub w: R,
    pub i: R,
    pub j: R,
    pub k: R,
}

impl<R> std::fmt::Display for Quat<R>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} + i{} + j{} + k{}", self.w, self.i, self.j, self.k)
    }
}

impl<R> Quat<R> {
    pub const fn new(w: R, i: R, j: R, k: R) -> Self {
        Quat { w, i, j, k }
    }
}

impl<R> Quat<R>
where
    R: Float + Sum,
{
    pub fn look_at_lh(fwd: Unit<Vec3<R>>, up: Unit<Vec3<R>>) -> Self {
        Mat3::look_at_lh(fwd, up).into()
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn c(&self) -> Self {
        Quat::new(self.w, -self.i, -self.j, -self.k)
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn inv(&self) -> Self {
        self.c() / self.abssq()
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn abssq(&self) -> R {
        self.w.powi(2) + self.i.powi(2) + self.j.powi(2) + self.k.powi(2)
    }
}

impl<'a, R> Norm for &'a Quat<R>
where
    R: Float,
{
    type Output = R;

    fn norm(self) -> Self::Output {
        self.abssq().sqrt()
    }
}

forward_ref_unop!(impl<R: Float> Norm, norm for Quat<R>, R);

impl<R> Quat<R>
where
    R: Zero + One,
{
    pub fn identity() -> Self {
        Quat {
            w: R::one(),
            i: R::zero(),
            j: R::zero(),
            k: R::zero(),
        }
    }
}

macro_rules! impl_scalar_quatops {
    ($($Op:ident, $op:ident, [$($tgt:ident),+ $(,)*]);+ $(;)*) => {
        $(
        impl<'a, 'b, R> $Op<&'b R> for &'a Quat<R>
        where
            R: Float,
        {
            type Output = Quat<R>;

            fn $op(self, rhs: &'b R) -> Self::Output {
                Quat {
                    w: self.w.$op(*rhs),
                    i: self.i.$op(*rhs),
                    j: self.j.$op(*rhs),
                    k: self.k.$op(*rhs),
                }
            }
        }

        forward_ref_binop!(impl<R: Float> $Op, $op for Quat<R>, R, Quat<R>);

        $(
        impl<'a, 'b> $Op<&'b Quat<$tgt>> for &'a $tgt {
            type Output = Quat<$tgt>;

            fn $op(self, rhs: &'b Quat<$tgt>) -> Self::Output {
                Quat {
                    w: self.$op(rhs.w),
                    i: self.$op(rhs.i),
                    j: self.$op(rhs.j),
                    k: self.$op(rhs.k),
                }
            }
        }

        forward_ref_binop!(impl $Op, $op for $tgt, Quat<$tgt>, Quat<$tgt>);
        )*
        )+
    }
}

impl_scalar_quatops! (
    Mul, mul, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Div, div, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
);

impl<'a, 'b, R> Mul<&'b Vec4<R>> for &'a Quat<R>
where
    R: Float,
{
    type Output = Vec4<R>;

    fn mul(self, rhs: &'b Vec4<R>) -> Self::Output {
        let q = Quat::new(R::zero(), rhs.x(), rhs.y(), rhs.z());
        let rhs_1 = self * &q * self.c();
        Vec4::new(rhs_1.i, rhs_1.j, rhs_1.k, rhs.w())
    }
}

forward_ref_binop!(impl<R: Float> Mul, mul for Quat<R>, Vec4<R>, Vec4<R>);

impl<'a, 'b, R> Mul<&'b Quat<R>> for &'a Quat<R>
where
    R: Float,
{
    type Output = Quat<R>;

    fn mul(self, rhs: &'b Quat<R>) -> Self::Output {
        let a1 = self.w;
        let b1 = self.i;
        let c1 = self.j;
        let d1 = self.k;
        let a2 = rhs.w;
        let b2 = rhs.i;
        let c2 = rhs.j;
        let d2 = rhs.k;

        Quat::new(
            a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
            a1 * b2 + b1 * a2 + c1 * d2 - d1 * c2,
            a1 * c2 - b1 * d2 + c1 * a2 + d1 * b2,
            a1 * d2 + b1 * c2 - c1 * b2 + d1 * a2,
        )
    }
}

forward_ref_binop!(impl<R: Float> Mul, mul for Quat<R>, Quat<R>, Quat<R>);

impl<R> From<Mat3<R>> for Quat<R>
where
    R: Float,
{
    fn from(v: Mat3<R>) -> Self {
        let half: R = R::one() / (R::one() + R::one());

        if v[(2, 2)] < v[(0, 0)] {
            if v[(0, 0)] > v[(1, 1)] {
                let t = R::one() + v[(0, 0)] - v[(1, 1)] - v[(2, 2)];
                Quat::new(v[(1, 2)] - v[(2, 1)], t, v[(0, 1)] + v[(1, 0)], v[(2, 0)] + v[(0, 2)]) * (half / t.sqrt())
            } else {
                let t = R::one() - v[(0, 0)] + v[(1, 1)] - v[(2, 2)];
                Quat::new(v[(2, 0)] - v[(0, 2)], v[(0, 1)] + v[(1, 0)], t, v[(1, 2)] + v[(2, 1)]) * (half / t.sqrt())
            }
        } else {
            if v[(0, 0)] < -v[(1, 1)] {
                let t = R::one() - v[(0, 0)] - v[(1, 1)] + v[(2, 2)];
                Quat::new(v[(0, 1)] - v[(1, 0)], v[(2, 0)] + v[(0, 2)], v[(1, 2)] + v[(2, 1)], t) * (half / t.sqrt())
            } else {
                let t = R::one() + v[(0, 0)] + v[(1, 1)] + v[(2, 2)];
                Quat::new(t, v[(1, 2)] - v[(2, 1)], v[(2, 0)] - v[(0, 2)], v[(0, 1)] - v[(1, 0)]) * (half / t.sqrt())
            }
        }
    }
}

impl<R> From<Mat4<R>> for Quat<R>
where
    R: Float,
{
    fn from(v: Mat4<R>) -> Self {
        v.subset::<3, 3>(0, 0).into()
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    /// Based on information from the [Euclidean Space Blog](https://www.euclideanspace.com/maths/geometry/rotations/conversions/quaternionToMatrix/index.htm)
    pub fn to_matrix(&self) -> Mat4<R> {
        self.into()
    }
}

impl<R> From<Quat<R>> for Mat4<R>
where
    R: Float,
{
    fn from(v: Quat<R>) -> Self {
        From::from(&v)
    }
}

impl<'a, R> From<&'a Quat<R>> for Mat4<R>
where
    R: Float,
{
    fn from(v: &'a Quat<R>) -> Self {
        let v_norm = v.norm();
        let w = v.w / v_norm;
        let i = v.i / v_norm;
        let j = v.j / v_norm;
        let k = v.k / v_norm;

        let z = R::zero();
        let o = R::one();
        let t = o + o;

        Mat4::from([
            o - t * j * j - t * k * k,
            t * i * j - t * k * w,
            t * i * k + t * j * w,
            z,
            t * i * j + t * k * w,
            o - t * i * i - t * k * k,
            t * j * k - t * i * w,
            z,
            t * i * k - t * j * w,
            t * j * k + t * i * w,
            o - t * i * i - t * j * j,
            z,
            z,
            z,
            z,
            o,
        ])
    }
}

impl<R> AbsDiffEq for Quat<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.w.abs_diff_eq(&rhs.w, epsilon)
            && self.i.abs_diff_eq(&rhs.i, epsilon)
            && self.j.abs_diff_eq(&rhs.j, epsilon)
            && self.k.abs_diff_eq(&rhs.k, epsilon)
    }
}

impl<R> RelativeEq for Quat<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.w.relative_eq(&rhs.w, epsilon, max_relative)
            && self.i.relative_eq(&rhs.i, epsilon, max_relative)
            && self.j.relative_eq(&rhs.j, epsilon, max_relative)
            && self.k.relative_eq(&rhs.k, epsilon, max_relative)
    }
}

impl<R> UlpsEq for Quat<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.w.ulps_eq(&rhs.w, epsilon, max_ulps)
            && self.i.ulps_eq(&rhs.i, epsilon, max_ulps)
            && self.j.ulps_eq(&rhs.j, epsilon, max_ulps)
            && self.k.ulps_eq(&rhs.k, epsilon, max_ulps)
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn quat_provides_identity_constructor() {
        let q: Quat<f32> = Quat::identity();
        assert_eq!(q.w, 1.0f32);
        assert_eq!(q.i, 0.0f32);
        assert_eq!(q.j, 0.0f32);
        assert_eq!(q.k, 0.0f32);
    }

    #[test]
    fn quat_provides_new_constructor() {
        let q: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.w, 1.0f32);
        assert_eq!(q.i, 2.0f32);
        assert_eq!(q.j, 3.0f32);
        assert_eq!(q.k, 4.0f32);
    }

    #[test]
    fn quat_supports_scalar_multiplication() {
        let a: Quat<f32> = Quat::identity();
        let b: f32 = 2.0;
        assert_eq!(&a * &b, Quat::<f32>::new(2.0, 0.0, 0.0, 0.0));
        assert_eq!(&b * &a, Quat::<f32>::new(2.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn quat_implements_from_mat4() {
        let a: Mat4<f32> = Mat4::identity();
        assert_eq!(Quat::<f32>::from(a), Quat::<f32>::identity());
    }

    #[test]
    fn quat_provides_to_matrix_method() {
        let q = Quat::<f32>::identity();
        assert_eq!(q.to_matrix(), Mat4::<f32>::identity());

        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(
            q.to_matrix(),
            Mat4::<f32>::from([0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0])
        );

        let q = Quat::new(0.0f32, 0.0, 0.0, 0.0);
        assert!(q.to_matrix().is_nan());
    }

    #[test]
    fn mat4_from_quat_results_in_nan_for_zero_norm() {
        let q = Quat::new(0.0f32, 0.0, 0.0, 0.0);
        let m = q.to_matrix();
        assert!(m.is_nan());
    }

    #[test]
    fn quat_implements_abssq_and_norm_methods() {
        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(q.abssq(), 4.0f32);
        assert_eq!(q.norm(), 2.0f32);
    }

    #[test]
    fn quat_implements_mul_for_vec4() {
        let q: Quat<f32> = Quat::identity();
        let v: Vec4<f32> = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(&q * &v, v);
    }

    #[test]
    fn quat_implements_conjugation() {
        let q: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        let c: Quat<f32> = Quat::new(1.0, -2.0, -3.0, -4.0);
        assert_eq!(q.c(), c);
    }

    #[test]
    fn quat_implements_inversion() {
        let q: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        let i: Quat<f32> = Quat::new(1.0, -2.0, -3.0, -4.0) / 30.0;
        assert_eq!(q.inv(), i);
    }

    #[test]
    fn quat_implements_mul_for_quat() {
        let a: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        let b: Quat<f32> = Quat::new(5.0, 6.0, 7.0, 8.0);

        assert_eq!(a * b, Quat::new(-60.0, 12.0, 30.0, 24.0))
    }

    #[test]
    fn quat_implements_serde() {
        let a: Quat<f32> = Quat::identity();

        assert_tokens(
            &a,
            &[
                Token::Struct { name: "Quat", len: 4 },
                Token::Str("w"),
                Token::F32(1.0f32),
                Token::Str("i"),
                Token::F32(0.0f32),
                Token::Str("j"),
                Token::F32(0.0f32),
                Token::Str("k"),
                Token::F32(0.0f32),
                Token::StructEnd,
            ],
        );
    }
}
