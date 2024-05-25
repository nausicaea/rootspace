use builder::AffineBuilder;
use num_traits::{Float, NumAssign};
use serde::{Deserialize, Serialize};

use crate::{mat::Mat4, num::Zero, ops::cross::Cross, quat::Quat, unit::Unit, vec::Vec4};

mod approx;
pub mod builder;
mod convert;
mod iter;
mod num;
mod ops;

#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "R: serde::Serialize",
    deserialize = "R: Copy + num_traits::Zero + for<'r> serde::Deserialize<'r>"
))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine<R> {
    pub t: Vec4<R>,
    pub o: Unit<Quat<R>>,
    pub s: R,
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
            t: Vec4::zero(),
            o: Quat::identity().into(),
            s: R::one(),
        }
    }
}

impl<R> Affine<R>
where
    R: Float + NumAssign,
{
    pub fn inv_t(&self) -> Mat4<R> {
        let tmp = Affine {
            t: Vec4::zero(),
            o: self.o,
            s: R::one() / self.s,
        };

        tmp.into()
    }
}

impl<R> Affine<R>
where
    R: Float,
{
    pub fn with_look_at_rh(eye: Vec4<R>, target: Vec4<R>, up: Unit<Vec4<R>>) -> Self {
        let fwd: Unit<_> = (target - eye).into();
        let right: Unit<_> = Unit::from(-up.cross(fwd).0);
        let rotated_up: Unit<_> = fwd.cross(right);

        let eye = Vec4::new(-(eye * right.0), -(eye * rotated_up.0), eye * fwd.0, R::zero());

        Affine {
            t: eye,
            o: Quat::with_look_at_rh(fwd, up),
            s: R::one(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ::approx::ulps_eq;
    use cgmath::Matrix;
    use proptest::{prop_assert, proptest};
    use serde_test::{assert_tokens, Token};

    use super::*;
    use crate::{
        num::ToMatrix,
        test_helpers::proptest::{bounded_f32, vec4},
    };

    proptest! {
        #[test]
        fn with_look_at_rh_is_equal_to_cgmath(eye in vec4(bounded_f32(-24, 24)), cntr in vec4(bounded_f32(-32, 32))) {
            let up: Unit<Vec4<f32>> = Vec4::y();

            let glamour_look_at = Affine::with_look_at_rh(eye, cntr, up).to_matrix();

            let cgmath_look_at = cgmath::Matrix4::look_at_rh(
                cgmath::Point3::new(eye.x, eye.y, eye.z),
                cgmath::Point3::new(cntr.x, cntr.y, cntr.z),
                cgmath::Vector3::new(up.x, up.y, up.z),
            );

            prop_assert!(
                ulps_eq!(glamour_look_at, cgmath_look_at), 
                "\nglamour =   {glamour_look_at:?}\ncgmath = {:?}",
                cgmath_look_at.transpose(),
            );
        }

        #[test]
        fn with_look_at_rh_is_equal_to_nalgebra(eye in vec4(bounded_f32(-24, 24)), cntr in vec4(bounded_f32(-32, 32))) {
            let up: Unit<Vec4<f32>> = Vec4::y();

            let glamour_look_at = Affine::with_look_at_rh(eye, cntr, up).to_matrix();

            let nalgebra_look_at = nalgebra::Matrix4::look_at_rh(
                &nalgebra::Point3::new(eye.x, eye.y, eye.z),
                &nalgebra::Point3::new(cntr.x, cntr.y, cntr.z),
                &nalgebra::Vector3::new(up.x, up.y, up.z),
            );

            prop_assert!(
                ulps_eq!(glamour_look_at, nalgebra_look_at), 
                "\nglamour = {glamour_look_at:?}\nnalgebra =     {:?}",
                nalgebra_look_at.transpose(),
            );
        }
    }

    #[test]
    fn affine_provides_identity_constructor() {
        let a: Affine<f32> = Affine::identity();
        assert_eq!(a.t, Vec4::<f32>::zero());
        assert_eq!(a.o, Unit::from(Quat::<f32>::identity()));
        assert_eq!(a.s, 1.0f32);
    }

    #[test]
    fn affine_implements_serde() {
        let a: Affine<f32> = Affine::identity();

        assert_tokens(
            &a,
            &[
                Token::Struct { name: "Affine", len: 3 },
                Token::Str("t"),
                Token::Struct { name: "Vec4", len: 4 },
                Token::Str("x"),
                Token::F32(0.0),
                Token::Str("y"),
                Token::F32(0.0),
                Token::Str("z"),
                Token::F32(0.0),
                Token::Str("w"),
                Token::F32(0.0),
                Token::StructEnd,
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
                Token::F32(1.0),
                Token::StructEnd,
            ],
        );
    }
}
