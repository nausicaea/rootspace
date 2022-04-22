use num_traits::{Num, Zero, One, Float, NumAssign, Signed};
use crate::mat::{Vec3, Vec4, Mat4, Mat3};
use crate::quat::Quat;
use std::iter::Sum;
use std::ops::Mul;

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde_support",
    serde(bound(
        serialize = "R: serde::Serialize",
        deserialize = "R: Copy + num_traits::Zero + for<'r> serde::Deserialize<'r>"
    ))
)]
#[derive(Debug, PartialEq)]
pub struct Affine<R> {
    t: Vec3<R>,
    o: Quat<R>,
    s: Vec3<R>,
}

impl<R> Affine<R> {
    pub fn builder() -> AffineBuilder<R> {
        AffineBuilder::default()
    }
}

impl<R> Affine<R> 
where
    R: Zero + One + Copy,
{
    pub fn identity() -> Self {
        AffineBuilder::default().build()
    }
}

impl<R> Mul<Vec4<R>> for Affine<R> 
where
    R: Num + Copy + Sum + One + Signed,
{
    type Output = Vec4<R>;

    fn mul(self, rhs: Vec4<R>) -> Self::Output {
        (&self).mul(&rhs)
    }
}

impl<'a, R> Mul<&'a Vec4<R>> for &'a Affine<R> 
where
    R: Num + Copy + Sum + One + Signed + Zero,
{
    type Output = Vec4<R>;

    fn mul(self, rhs: &'a Vec4<R>) -> Self::Output {
        let scaled: Vec3<R> = self.s.mul_elementwise(&rhs.subset::<3, 1>(0, 0));
        let scaled: Vec4<R> = Vec4::new(scaled.x(), scaled.y(), scaled.z(), rhs.w());
        let rotated = &self.o * &scaled;
        let t = Vec4::new(self.t.x(), self.t.y(), self.t.z(), R::zero());
        let translated = t + rotated;
        
        translated
    }
}

impl<'a, R> From<&'a Affine<R>> for Mat4<R>
where
    R: Float + NumAssign,
{
    fn from(value: &'a Affine<R>) -> Self {
        let mut m: Mat4<R> = (&value.o).into();
        m[(0, 0)] *= value.s[0];
        m[(1, 1)] *= value.s[1];
        m[(2, 2)] *= value.s[2];
        m[(0, 3)] = value.t[0];
        m[(1, 3)] = value.t[1];
        m[(2, 3)] = value.t[2];
        m
    }
}

impl<'a, R> From<&'a Mat4<R>> for Affine<R> 
where
    R: Copy + Zero + NumAssign + Float + Sum + std::fmt::Debug,
{
    fn from(v: &'a Mat4<R>) -> Self {
        let t: Vec3<R> = v.subset::<3, 1>(0, 3);

        let s = Vec3::new(
            v.col(0).norm(),
            v.col(1).norm(),
            v.col(2).norm(),
        );

        let mut rot_m: Mat3<R> = v.subset::<3, 3>(0, 0);
        rot_m[(0, 0)] /= s[0];
        rot_m[(1, 0)] /= s[0];
        rot_m[(2, 0)] /= s[0];
        rot_m[(0, 1)] /= s[1];
        rot_m[(1, 1)] /= s[1];
        rot_m[(2, 1)] /= s[1];
        rot_m[(0, 2)] /= s[2];
        rot_m[(1, 2)] /= s[2];
        rot_m[(2, 2)] /= s[2];

        let o = Quat::from(&rot_m);

        Affine {
            t, o, s,
        }
    }
}

impl<R> From<AffineBuilder<R>> for Affine<R> 
where
    R: One + Zero + Copy,
{
    fn from(v: AffineBuilder<R>) -> Self {
        Affine {
            t: v.t.unwrap_or_else(Vec3::zero),
            o: v.o.unwrap_or_else(Quat::identity),
            s: v.s.unwrap_or_else(Vec3::one),
        }
    }
}

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
    R: One + Zero + Copy,
{
    pub fn build(self) -> Affine<R> {
        self.into()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn affine_provides_identity_constructor() {
        let a: Affine<f32> = Affine::identity();
        assert_eq!(a.t, Vec3::<f32>::zero());
        assert_eq!(a.o, Quat::<f32>::identity());
        assert_eq!(a.s, Vec3::<f32>::one());
    }

    #[test]
    fn affine_provides_builder() {
        let a: Affine<f32> = Affine::builder().build();
        assert_eq!(a, Affine::<f32>::identity());

        let a: Affine<f32> = Affine::builder()
            .with_scale(Vec3::from([1.0, 2.0, 3.0]))
            .build();

        assert_eq!(a.s, Vec3::from([1.0, 2.0, 3.0]));
    }

    #[test]
    fn mat4_implements_from_ref_affine() {
        let a: Affine<f32> = Affine::identity();
        assert_eq!(Mat4::<f32>::from(&a), Mat4::<f32>::identity());
    }

    #[test]
    fn affine_implements_from_ref_mat4() {
        let m: Mat4<f32> = Mat4::identity();
        assert_eq!(Affine::<f32>::from(&m), Affine::<f32>::identity());
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
