use std::{
    iter::{Product, Sum},
    ops::{Add, Mul},
};

use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use forward_ref::forward_ref_binop;
use num_traits::{Float, Inv, NumAssign, One, Zero};

use crate::{
    iter_float::IterFloat,
    mat::Mat4,
    ops::{dot::Dot, inv_elem::InvElem, mul_elem::MulElem, norm::Norm},
    quat::Quat,
    unit::Unit,
    vec::Vec4,
    Cross,
};

#[cfg_attr(any(test, feature = "serde_support"), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    any(test, feature = "serde_support"),
    serde(bound(
        serialize = "R: serde::Serialize",
        deserialize = "R: Copy + num_traits::Zero + for<'r> serde::Deserialize<'r>"
    ))
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine<R> {
    pub t: Vec3<R>,
    pub o: Unit<Quat<R>>,
    pub s: Vec3<R>,
}

impl<R> Affine<R> {
    pub fn builder() -> AffineBuilder<R> {
        AffineBuilder::default()
    }
}

impl<R> Affine<R>
where
    R: Float,
{
    pub fn identity() -> Self {
        Affine {
            t: Vec3::zero(),
            o: Quat::identity().into(),
            s: Vec3::one(),
        }
    }
}

impl<R> Affine<R>
where
    R: IterFloat,
{
    pub fn look_at_lh(eye: Vec3<R>, cntr: Vec3<R>, up: Unit<Vec3<R>>) -> Self {
        let fwd: Unit<_> = (&cntr - &eye).into();
        let side: Unit<_> = up.as_ref().cross(fwd.as_ref()).into();
        let rotated_up: Unit<_> = fwd.as_ref().cross(side.as_ref()).into();

        let eye_t = eye.t();
        let eye = Vec3::new(
            -(&eye_t * side.as_ref()),
            -(&eye_t * rotated_up.as_ref()),
            &eye_t * fwd.as_ref(),
        );

        Affine {
            t: eye,
            o: Quat::look_at_lh(fwd, up).into(),
            s: Vec3::one(),
        }
    }
}

impl<R> Affine<R>
where
    R: Float + Inv<Output = R>,
{
    pub fn inv(&self) -> Self {
        Affine {
            t: -(&self.t),
            o: self.o.as_ref().c().into(),
            s: (&self.s).inv_elem(),
        }
    }
}

impl<'a, R> Mul<&'a Vec4<R>> for &'a Affine<R>
where
    R: Float,
{
    type Output = Vec4<R>;

    fn mul(self, rhs: &'a Vec4<R>) -> Self::Output {
        self.dot(rhs)
    }
}

impl<'a, R> Dot<&'a Vec4<R>> for &'a Affine<R>
where
    R: Float,
{
    type Output = Vec4<R>;

    fn dot(self, rhs: &'a Vec4<R>) -> Self::Output {
        let scaled: Vec3<R> = (&self.s).mul_elem(&rhs.subset::<3, 1>(0, 0));
        let scaled: Vec4<R> = Vec4::new(scaled.x(), scaled.y(), scaled.z(), rhs.w());
        let rotated = self.o.as_ref() * &scaled;
        let t = Vec4::new(self.t.x(), self.t.y(), self.t.z(), R::zero());
        let translated = t + rotated;

        translated
    }
}

forward_ref_binop!(impl<R: Float> Dot, dot for Affine<R>, Vec4<R>, Vec4<R>);
forward_ref_binop!(impl<R: Float> Mul, mul for Affine<R>, Vec4<R>, Vec4<R>);

impl<'a, R> Mul for &'a Affine<R>
where
    R: Float,
{
    type Output = Affine<R>;

    fn mul(self, rhs: Self) -> Self::Output {
        self.dot(rhs)
    }
}

impl<'a, R> Dot for &'a Affine<R>
where
    R: Float,
{
    type Output = Affine<R>;

    fn dot(self, rhs: Self) -> Self::Output {
        Affine {
            t: (&self.t).add(&rhs.t),
            o: self.o.as_ref().mul(rhs.o.as_ref()).into(),
            s: (&self.s).mul_elem(&rhs.s),
        }
    }
}

forward_ref_binop!(impl<R: Float> Dot, dot for Affine<R>, Affine<R>, Affine<R>);
forward_ref_binop!(impl<R: Float> Mul, mul for Affine<R>, Affine<R>, Affine<R>);

impl<R> Product for Affine<R>
where
    R: Float,
{
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Affine::identity(), |state, value| state * value)
    }
}

impl<'a, R> Product<&'a Affine<R>> for Affine<R>
where
    R: Float,
{
    fn product<I: Iterator<Item = &'a Affine<R>>>(iter: I) -> Self {
        iter.fold(Affine::identity(), |state, value| &state * value)
    }
}

impl<R> Affine<R>
where
    R: Float + NumAssign,
{
    pub fn to_matrix(&self) -> Mat4<R> {
        self.into()
    }
}

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
        let mut m: Mat4<R> = v.o.as_ref().into();
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
    R: Copy + One + Zero + NumAssign + Float + Sum + std::fmt::Debug,
{
    type Error = ();

    fn try_from(v: Mat4<R>) -> Result<Self, Self::Error> {
        let t: Vec3<R> = v.subset::<3, 1>(0, 3);

        let s = Vec3::new(v.col(0).norm(), v.col(1).norm(), v.col(2).norm());

        let mut rot_m: Mat4<R> = v.clone();
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

        let o: Unit<Quat<R>> = Unit::from(Quat::from(rot_m));

        Ok(Affine { t, o, s })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AffineBuilder<R> {
    t: Option<Vec3<R>>,
    o: Option<Quat<R>>,
    s: Option<Vec3<R>>,
}

impl<R> AffineBuilder<R> {
    pub fn with_translation(mut self, v: Vec3<R>) -> Self {
        self.t = Some(v);
        self
    }

    pub fn with_orientation(mut self, q: Quat<R>) -> Self {
        self.o = Some(q);
        self
    }

    pub fn with_scale(mut self, v: Vec3<R>) -> Self {
        self.s = Some(v);
        self
    }
}

impl<R> AffineBuilder<R>
where
    R: Float,
{
    pub fn build(self) -> Affine<R> {
        Affine {
            t: self.t.unwrap_or_else(Vec3::zero),
            o: self
                .o
                .map(|o| Unit::from(o))
                .unwrap_or_else(|| Unit::from(Quat::identity())),
            s: self.s.unwrap_or_else(Vec3::one),
        }
    }
}

impl<R> Default for AffineBuilder<R> {
    fn default() -> Self {
        AffineBuilder {
            t: None,
            o: None,
            s: None,
        }
    }
}

impl<R> AbsDiffEq for Affine<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.t.abs_diff_eq(&rhs.t, epsilon)
            && self.o.abs_diff_eq(&rhs.o, epsilon)
            && self.s.abs_diff_eq(&rhs.s, epsilon)
    }
}

impl<R> RelativeEq for Affine<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.t.relative_eq(&rhs.t, epsilon, max_relative)
            && self.o.relative_eq(&rhs.o, epsilon, max_relative)
            && self.s.relative_eq(&rhs.s, epsilon, max_relative)
    }
}

impl<R> UlpsEq for Affine<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.t.ulps_eq(&rhs.t, epsilon, max_ulps)
            && self.o.ulps_eq(&rhs.o, epsilon, max_ulps)
            && self.s.ulps_eq(&rhs.s, epsilon, max_ulps)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn affine_provides_look_at_lh() {
        let eye = [0.0f32, 1.0, 2.0];
        let cntr = [0.0f32, 0.0, 0.0];
        let up = [0.0f32, 1.0, 0.0];

        let a = Affine::look_at_lh(Vec3::from(eye), Vec3::from(cntr), Unit::from(Vec3::from(up)));

        let expected = Mat4::new([
            [-1.0000001f32, -0.0, 0.0, -0.0],
            [0.0, 0.8944272, -0.44721365, -0.0],
            [0.0, -0.44721365, -0.8944273, -2.236068],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        assert_ulps_eq!(a.to_matrix(), expected);
    }

    #[test]
    fn affine_provides_identity_constructor() {
        let a: Affine<f32> = Affine::identity();
        assert_eq!(a.t, Vec3::<f32>::zero());
        assert_eq!(a.o, Unit::from(Quat::<f32>::identity()));
        assert_eq!(a.s, Vec3::<f32>::one());
    }

    #[test]
    fn affine_provides_builder() {
        let a: Affine<f32> = Affine::builder().build();
        assert_eq!(a, Affine::<f32>::identity());

        let a: Affine<f32> = Affine::builder().with_scale(Vec3::from([1.0, 2.0, 3.0])).build();

        assert_eq!(a.s, Vec3::from([1.0, 2.0, 3.0]));
    }

    #[test]
    fn affine_provides_to_matrix_method() {
        let a: Affine<f32> = Affine::identity();
        assert_eq!(a.to_matrix(), Mat4::<f32>::identity());
    }

    #[test]
    fn affine_implements_try_from_mat4() {
        let m: Mat4<f32> = Mat4::identity();
        assert_eq!(Affine::<f32>::try_from(m), Ok(Affine::<f32>::identity()));
    }

    #[test]
    fn affine_implements_mul_for_vec4() {
        let a: Affine<f32> = Affine::identity();
        let b: Vec4<f32> = Vec4::new(1.0, 1.0, 1.0, 1.0);
        assert_eq!(&a * &b, b);
    }

    #[test]
    fn affine_implements_serde() {
        let a: Affine<f32> = Affine::identity();

        assert_tokens(
            &a,
            &[
                Token::Struct { name: "Affine", len: 3 },
                Token::Str("t"),
                Token::Seq { len: Some(3) },
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::SeqEnd,
                Token::Str("o"),
                Token::Struct { name: "Quat", len: 4 },
                Token::Str("w"),
                Token::F32(1.0),
                Token::Str("i"),
                Token::F32(0.0),
                Token::Str("j"),
                Token::F32(0.0),
                Token::Str("k"),
                Token::F32(0.0),
                Token::StructEnd,
                Token::Str("s"),
                Token::Seq { len: Some(3) },
                Token::F32(1.0),
                Token::F32(1.0),
                Token::F32(1.0),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}
