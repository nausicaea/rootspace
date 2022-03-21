use num_traits::{Zero, One, Float, Num};
use crate::mat::{Mat4, Mat3};
use std::ops::Mul;

#[derive(Debug, PartialEq)]
pub struct Quat<R> {
    w: R,
    i: R,
    j: R,
    k: R,
}

impl<R> Quat<R> {
    pub fn new(w: R, i: R, j: R, k: R) -> Self {
        Quat { w, i, j, k }
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn norm(&self) -> R {
        (self.w.powi(2) + self.i.powi(2) + self.j.powi(2) + self.k.powi(2)).sqrt()
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

macro_rules! impl_scalar_quatops {
    ($($Op:ident, $op:ident, [$($tgt:ident),+ $(,)*]);+ $(;)*) => {
        $(
            impl<R> $Op<R> for Quat<R>
            where
                R: Num + Copy,
            {
                type Output = Quat<R>;

                fn $op(self, rhs: R) -> Self::Output {
                    (&self).$op(&rhs)
                }
            }

            impl<'a, R> $Op<&'a R> for &'a Quat<R>
            where
                R: Num + Copy,
            {
                type Output = Quat<R>;

                fn $op(self, rhs: &'a R) -> Self::Output {
                    Quat {
                        w: self.w.$op(*rhs),
                        i: self.i.$op(*rhs),
                        j: self.j.$op(*rhs),
                        k: self.k.$op(*rhs),
                    }
                }
            }

            $(
                impl $Op<Quat<$tgt>> for $tgt {
                    type Output = Quat<$tgt>;

                    fn $op(self, rhs: Quat<$tgt>) -> Self::Output {
                        (&self).$op(&rhs)
                    }
                }

                impl<'a> $Op<&'a Quat<$tgt>> for &'a $tgt {
                    type Output = Quat<$tgt>;

                    fn $op(self, rhs: &'a Quat<$tgt>) -> Self::Output {
                        Quat {
                            w: self.$op(rhs.w),
                            i: self.$op(rhs.i),
                            j: self.$op(rhs.j),
                            k: self.$op(rhs.k),
                        }
                    }
                }
            )*
        )+
    }
}

impl_scalar_quatops! (
    Mul, mul, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
);

impl<'a, R> From<&'a Mat3<R>> for Quat<R>
where
    R: Float + One,
{
    fn from(v: &'a Mat3<R>) -> Self {
        let half: R = R::one() / (R::one() + R::one());

        if v[(2, 2)] < v[(0, 0)] {
            if v[(0, 0)] > v[(1, 1)] {
                let t = R::one() + v[(0, 0)] - v[(1, 1)] - v[(2, 2)];
                Quat::new(
                    v[(1, 2)] - v[(2, 1)],
                    t, 
                    v[(0, 1)] + v[(1, 0)], 
                    v[(2, 0)] + v[(0, 2)], 
                ) * (half / t.sqrt())
            } else {
                let t = R::one() - v[(0, 0)] + v[(1, 1)] - v[(2, 2)];
                Quat::new(
                    v[(2, 0)] - v[(0, 2)],
                    v[(0, 1)] + v[(1, 0)], 
                    t, 
                    v[(1, 2)] + v[(2, 1)], 
                ) * (half / t.sqrt())
            }
        } else {
            if v[(0, 0)] < -v[(1, 1)] {
                let t = R::one() - v[(0, 0)] - v[(1, 1)] + v[(2, 2)];
                Quat::new( 
                    v[(0, 1)] - v[(1, 0)],
                    v[(2, 0)] + v[(0, 2)], 
                    v[(1, 2)] + v[(2, 1)], 
                    t, 
                ) * (half / t.sqrt())
            } else {
                let t = R::one() + v[(0, 0)] + v[(1, 1)] + v[(2, 2)];
                Quat::new(
                    t,
                    v[(1, 2)] - v[(2, 1)], 
                    v[(2, 0)] - v[(0, 2)], 
                    v[(0, 1)] - v[(1, 0)], 
                ) * (half / t.sqrt())
            }
        }
    }
}

/// Based on information from the [Euclidean Space Blog](https://www.euclideanspace.com/maths/geometry/rotations/conversions/quaternionToMatrix/index.htm)
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
            o - t*j*j - t*k*k, t*i*j - t*k*w, t*i*k + t*j*w, z,
            t*i*j + t*k*w, o - t*i*i - t*k*k, t*j*k - t*i*w, z,
            t*i*k - t*j*w, t*j*k + t*i*w, o - t*i*i - t*j*j, z,
            z, z, z, o,
        ])
    }
}

#[cfg(test)]
mod tests {
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
    fn quat_implements_from_ref_mat3() {
        let a: Mat3<f32> = Mat3::identity();
        assert_eq!(Quat::<f32>::from(&a), Quat::<f32>::identity());
    }

    #[test]
    fn mat4_implements_from_ref_quat() {
        let q = Quat::<f32>::identity();
        assert_eq!(Mat4::<f32>::from(&q), Mat4::<f32>::identity());

        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(Mat4::<f32>::from(&q), Mat4::<f32>::from([0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]));

        let q = Quat::new(0.0f32, 0.0, 0.0, 0.0);
        assert!(Mat4::<f32>::from(&q).is_nan());
    }

    #[test]
    fn mat4_from_quat_results_in_nan_for_zero_norm() {
        let q = Quat::new(0.0f32, 0.0, 0.0, 0.0);
        let m = Mat4::<f32>::from(&q);
        assert!(m.is_nan());
    }

    #[test]
    fn quat_implements_norm_method() {
        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(q.norm(), 2.0f32);
    }
}
