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
        let w1 = self.w;
        let i1 = self.i;
        let j1 = self.j;
        let k1 = self.k;
        let w2 = rhs.w;
        let i2 = rhs.i;
        let j2 = rhs.j;
        let k2 = rhs.k;

        Quat::new(
            w1 * w2 - i1 * i2 - j1 * j2 - k1 * k2,
            w1 * i2 + i1 * w2 + j1 * k2 - k1 * j2,
            w1 * j2 - i1 * k2 + j1 * w2 + k1 * i2,
            w1 * k2 + i1 * j2 - j1 * i2 + k1 * w2,
        )
    }
}

forward_ref_binop!(impl<R: Float> Mul, mul for Quat<R>, Quat<R>, Quat<R>);

#[cfg(test)]
mod tests {

    use proptest::{collection::vec, num::f32::NORMAL, prelude::*};

    use crate::glamour::{num::ToMatrix, unit::Unit};

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

    proptest! {
        #[test]
        fn f32_mul_behavior(a in NORMAL, b in NORMAL) {
            let result = b * a;
            prop_assert!(!result.is_nan());
        }

        #[test]
        fn f32_add_behavior(a in NORMAL, b in NORMAL) {
            let result = b + a;
            prop_assert!(!result.is_nan());
        }

        #[test]
        fn f32_sub_behavior(a in NORMAL, b in NORMAL) {
            let result = b - a;
            prop_assert!(!result.is_nan());
        }

        #[test]
        fn quat_mul_behavior(a in vec(NORMAL, 4), b in vec(NORMAL, 4)) {
            let first: Quat<f32> = Quat::new(a[0], a[1], a[2], a[3]);
            let second: Quat<f32> = Quat::new(b[0], b[1], b[2], b[3]);
            let result = second * first;
            prop_assert!(!result.is_nan());
        }

        #[test]
        fn unit_quat_mul_is_the_same_as_rot_mat_mul(a in vec(NORMAL, 4), b in vec(NORMAL, 4)) {
            let first: Unit<Quat<f32>> = Unit::from(Quat::new(a[0], a[1], a[2], a[3]));
            let second: Unit<Quat<f32>> = Unit::from(Quat::new(b[0], b[1], b[2], b[3]));

            let qp = second * first;
            let mp = Into::<Unit<_>>::into(Into::<Quat<f32>>::into(second.to_matrix() * first.to_matrix()));
            prop_assert!(approx::relative_eq!(qp, mp));
        }
    }
}
