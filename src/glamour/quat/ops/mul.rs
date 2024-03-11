use std::ops::Mul;

use num_traits::Float;

use crate::{
    forward_ref_binop,
    glamour::{quat::Quat, vec::Vec4},
};

impl<'a, 'b, R> Mul<&'b Vec4<R>> for &'a Quat<R>
where
    R: Float,
{
    type Output = Vec4<R>;

    fn mul(self, rhs: &'b Vec4<R>) -> Self::Output {
        let q = Quat::new(R::zero(), rhs.x, rhs.y, rhs.z);
        let rhs_1 = self * q * self.c();
        Vec4::new(rhs_1.i, rhs_1.j, rhs_1.k, rhs.w)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quat_implements_mul_for_vec4() {
        let q: Quat<f32> = Quat::identity();
        let v: Vec4<f32> = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q * v, v);
    }

    #[test]
    fn quat_implements_mul_for_quat() {
        let a: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        let b: Quat<f32> = Quat::new(5.0, 6.0, 7.0, 8.0);

        assert_eq!(a * b, Quat::new(-60.0, 12.0, 30.0, 24.0))
    }
}
