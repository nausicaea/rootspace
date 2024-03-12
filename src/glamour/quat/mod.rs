use std::iter::Sum;

use crate::glamour::mat::Mat4;
use crate::glamour::unit::Unit;
use crate::glamour::vec::Vec4;
use num_traits::{Float, One, Zero};

pub mod approx;
pub mod convert;
pub mod num;
pub mod ops;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Quat<R> {
    pub w: R,
    pub i: R,
    pub j: R,
    pub k: R,
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
    pub fn look_at_lh(fwd: Unit<Vec4<R>>, up: Unit<Vec4<R>>) -> Self {
        Mat4::look_at_lh(fwd, up).into()
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn c(&self) -> Self {
        Quat::new(self.w, -self.i, -self.j, -self.k)
    }

    pub fn is_nan(&self) -> bool {
        self.w.is_nan() || self.i.is_nan() || self.j.is_nan() || self.k.is_nan()
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

impl<R> std::fmt::Display for Quat<R>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} + i{} + j{} + k{}", self.w, self.i, self.j, self.k)
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
    fn quat_implements_abssq_methods() {
        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(q.abssq(), 4.0f32);
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
