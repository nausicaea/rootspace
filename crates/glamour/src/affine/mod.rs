use builder::AffineBuilder;
use num_traits::Float;
use serde::{Deserialize, Serialize};

use crate::{mat::Mat4, num::Zero, ops::cross::Cross, quat::Quat, unit::Unit, vec::Vec4};

mod approx;
pub mod builder;
mod convert;
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
    R: Float,
{
    pub fn with_look_at_rh(eye: Vec4<R>, target: Vec4<R>, up: Unit<Vec4<R>>) -> Self {
        let eye = Vec4::new(eye.x, eye.y, eye.z, R::one());
        let target = Vec4::new(target.x, target.y, target.z, R::one());
        let up: Unit<_> = Vec4::new(up.x, up.y, up.z, R::zero()).into();

        let dir: Unit<_> = (target - eye).into();
        let right: Unit<_> = dir.cross(up);
        let rotated_up: Unit<_> = right.cross(dir);

        let mat = Mat4([
            [right.x, right.y, right.z, R::zero()],
            [rotated_up.x, rotated_up.y, rotated_up.z, R::zero()],
            [-dir.x, -dir.y, -dir.z, R::zero()],
            [R::zero(), R::zero(), R::zero(), R::one()],
        ]);

        let o: Unit<Quat<R>> = mat.into();

        let mut t = Vec4::zero() - eye;
        t.w = R::zero();
        let qt = Quat::from(t);
        let t: Vec4<R> = (o.0 * qt * o.0.c()).into();

        Affine { t, o, s: R::one() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::ToMatrix;
    use crate::test_helpers::proptest::{affine, bounded_f32, bounded_nonzero_f32, vec4};
    use ::approx::relative_eq;
    use cgmath::Transform;
    use proptest::{prop_assert, proptest};
    use serde_test::{Token, assert_tokens};

    #[test]
    #[ignore = "the results of cgmath and nalgebra don't agree"]
    fn with_look_at_rh_comparison() {
        let eye = Vec4::new(0.0, 5.0, -10.0, 1.0);
        let cntr = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let up: Unit<Vec4<f32>> = Vec4::y();

        let glamour_look_at = Affine::with_look_at_rh(eye, cntr, up);

        let cgmath_look_at = cgmath::Decomposed::look_at_rh(
            cgmath::Point3::new(eye.x, eye.y, eye.z),
            cgmath::Point3::new(cntr.x, cntr.y, cntr.z),
            cgmath::Vector3::new(up.x, up.y, up.z),
        );

        let nalgebra_look_at = nalgebra::Similarity3::look_at_rh(
            &nalgebra::Point3::new(eye.x, eye.y, eye.z),
            &nalgebra::Point3::new(cntr.x, cntr.y, cntr.z),
            &nalgebra::Vector3::new(up.x, up.y, up.z),
            1.0,
        );

        assert!(
            relative_eq!(glamour_look_at, cgmath_look_at) && relative_eq!(glamour_look_at, nalgebra_look_at),
            "\nglamour  = [t.x = {:+.05e}, t.y = {:+.05e}, t.z = {:+.05e}, o.w = {:+.05e}, o.i = {:+.05e}, o.j = {:+.05e}, o.k = {:+.05e}]\ncgmath   = [t.x = {:+.05e}, t.y = {:+.05e}, t.z = {:+.05e}, o.w = {:+.05e}, o.i = {:+.05e}, o.j = {:+.05e}, o.k = {:+.05e}]\nnalgebra = [t.x = {:+.05e}, t.y = {:+.05e}, t.z = {:+.05e}, o.w = {:+.05e}, o.i = {:+.05e}, o.j = {:+.05e}, o.k = {:+.05e}]",
            glamour_look_at.t.x,
            glamour_look_at.t.y,
            glamour_look_at.t.z,
            glamour_look_at.o.w,
            glamour_look_at.o.i,
            glamour_look_at.o.j,
            glamour_look_at.o.k,
            cgmath_look_at.disp.x,
            cgmath_look_at.disp.y,
            cgmath_look_at.disp.z,
            cgmath_look_at.rot.s,
            cgmath_look_at.rot.v.x,
            cgmath_look_at.rot.v.y,
            cgmath_look_at.rot.v.z,
            nalgebra_look_at.isometry.translation.x,
            nalgebra_look_at.isometry.translation.y,
            nalgebra_look_at.isometry.translation.z,
            nalgebra_look_at.isometry.rotation.w,
            nalgebra_look_at.isometry.rotation.i,
            nalgebra_look_at.isometry.rotation.j,
            nalgebra_look_at.isometry.rotation.k,
        );
    }

    proptest! {
        #[test]
        fn affine_conversion_to_quat_erases_all_but_rotational_components(a in affine(bounded_f32(-32, 32), bounded_nonzero_f32(-32, 32))) {
            let a_rot = Into::<Unit<Quat<f32>>>::into(a.to_matrix()).to_matrix();
            let a_identity = a_rot.t() * a_rot;
            prop_assert!(relative_eq!(a_identity, Mat4::identity(), max_relative = 1.0),
                "Orthogonality didn't hold for extracted quaternion. Expected an identity matrix, got: {a_identity:?}"
            )
        }

        #[test]
        #[ignore = "our implementation has significant differences to cgmath for the translational part"]
        fn with_look_at_rh_is_equal_to_cgmath(eye in vec4(bounded_nonzero_f32(-16, 16))) {
            let cntr = Vec4::zero();
            let up: Unit<Vec4<f32>> = Vec4::y();

            let glamour_look_at = Affine::with_look_at_rh(eye, cntr, up);

            let cgmath_look_at = cgmath::Decomposed::look_at_rh(
                cgmath::Point3::new(eye.x, eye.y, eye.z),
                cgmath::Point3::new(cntr.x, cntr.y, cntr.z),
                cgmath::Vector3::new(up.x, up.y, up.z),
            );

            prop_assert!(
                relative_eq!(glamour_look_at, cgmath_look_at),
                "\nglamour = [t.x = {:+.05e}, t.y = {:+.05e}, t.z = {:+.05e}, o.w = {:+.05e}, o.i = {:+.05e}, o.j = {:+.05e}, o.k = {:+.05e}]\ncgmath  = [t.x = {:+.05e}, t.y = {:+.05e}, t.z = {:+.05e}, o.w = {:+.05e}, o.i = {:+.05e}, o.j = {:+.05e}, o.k = {:+.05e}]",
                glamour_look_at.t.x,
                glamour_look_at.t.y,
                glamour_look_at.t.z,
                glamour_look_at.o.w,
                glamour_look_at.o.i,
                glamour_look_at.o.j,
                glamour_look_at.o.k,
                cgmath_look_at.disp.x,
                cgmath_look_at.disp.y,
                cgmath_look_at.disp.z,
                cgmath_look_at.rot.s,
                cgmath_look_at.rot.v.x,
                cgmath_look_at.rot.v.y,
                cgmath_look_at.rot.v.z,
            );
        }

        #[test]
        #[ignore = "our implementation has significant differences to nalgebra for the translational part, and somehow, the nalgebra orientation part is equal to -1 times the glamour version"]
        fn with_look_at_rh_is_equal_to_nalgebra(eye in vec4(bounded_f32(-24, 24)), cntr in vec4(bounded_f32(-32, 32))) {
            let up: Unit<Vec4<f32>> = Vec4::y();

            let glamour_look_at = Affine::with_look_at_rh(eye, cntr, up);

            let nalgebra_look_at = nalgebra::Similarity3::look_at_rh(
                &nalgebra::Point3::new(eye.x, eye.y, eye.z),
                &nalgebra::Point3::new(cntr.x, cntr.y, cntr.z),
                &nalgebra::Vector3::new(up.x, up.y, up.z),
                1.0,
            );

            prop_assert!(
                relative_eq!(glamour_look_at, nalgebra_look_at),
                "\nglamour  = [t.x = {:+.05e}, t.y = {:+.05e}, t.z = {:+.05e}, o.w = {:+.05e}, o.i = {:+.05e}, o.j = {:+.05e}, o.k = {:+.05e}]\nnalgebra = [t.x = {:+.05e}, t.y = {:+.05e}, t.z = {:+.05e}, o.w = {:+.05e}, o.i = {:+.05e}, o.j = {:+.05e}, o.k = {:+.05e}]",
                glamour_look_at.t.x,
                glamour_look_at.t.y,
                glamour_look_at.t.z,
                glamour_look_at.o.w,
                glamour_look_at.o.i,
                glamour_look_at.o.j,
                glamour_look_at.o.k,
                nalgebra_look_at.isometry.translation.x,
                nalgebra_look_at.isometry.translation.y,
                nalgebra_look_at.isometry.translation.z,
                nalgebra_look_at.isometry.rotation.w,
                nalgebra_look_at.isometry.rotation.i,
                nalgebra_look_at.isometry.rotation.j,
                nalgebra_look_at.isometry.rotation.k,
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
