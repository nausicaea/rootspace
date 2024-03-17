use builder::AffineBuilder;
use num_traits::{Float, Inv};
use serde::{Deserialize, Serialize};

use crate::glamour::{num::Zero, ops::cross::Cross, quat::Quat, unit::Unit, vec::Vec4};

pub mod builder;
mod convert;
mod num;

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
    pub fn look_at_lh(eye: Vec4<R>, target: Vec4<R>, up: Unit<Vec4<R>>) -> Self {
        let fwd: Unit<_> = (target - eye).into();
        let side: Unit<_> = up.cross(fwd);
        let rotated_up: Unit<_> = fwd.cross(side);

        let eye = Vec4::new(-(eye * side.0), -(eye * rotated_up.0), eye * fwd.0, R::zero());

        Affine {
            t: eye,
            o: Quat::look_at_lh(fwd, up),
            s: R::one(),
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
            s: self.s.inv(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::glamour::num::ToMatrix;
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn affine_provides_look_at_lh() {
        let eye = Vec4::from([0.0f32, 1.0, 2.0, 1.0]);
        let cntr = Vec4::from([0.0f32, 0.0, 0.0, 1.0]);
        let up = Vec4::from([0.0f32, 1.0, 0.0, 0.0]);

        let a = Affine::look_at_lh(eye, cntr, Unit::from(up));

        let comparison = cgmath::Matrix4::look_at_lh(
            cgmath::Point3::new(eye.x, eye.y, eye.z),
            cgmath::Point3::new(cntr.x, cntr.y, cntr.z),
            cgmath::Vector3::new(up.x, up.y, up.z),
        );

        eprintln!("{} = {:?}", a.to_matrix(), comparison);

        // let expected = Mat4::new([
        //     [-1.0000001f32, -0.0, 0.0, -0.0],
        //     [0.0, 0.8944272, -0.44721365, -0.0],
        //     [0.0, -0.44721365, -0.8944273, -2.236068],
        //     [0.0, 0.0, 0.0, 1.0],
        // ]);

        // assert_ulps_eq!(a.to_matrix(), expected);
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
