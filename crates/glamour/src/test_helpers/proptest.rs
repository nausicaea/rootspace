use ::proptest::{
    collection::vec,
    num::f32::ZERO,
    prop_compose,
    strategy::{Strategy, Union},
};
use std::f32::consts::PI;

use crate::{affine::Affine, mat::Mat4, quat::Quat, unit::Unit, vec::Vec4};

pub fn bounded_positive_f32(lower_exp: i32, upper_exp: i32) -> impl Clone + Strategy<Value = f32> {
    let pos_lower = 2.0_f32.powi(lower_exp);
    let pos_upper = 2.0_f32.powi(upper_exp);
    pos_lower..pos_upper
}

pub fn bounded_f32(lower_exp: i32, upper_exp: i32) -> impl Clone + Strategy<Value = f32> {
    let neg_lower = -(2.0_f32.powi(upper_exp));
    let neg_upper = -(2.0_f32.powi(lower_exp));
    let pos_lower = 2.0_f32.powi(lower_exp);
    let pos_upper = 2.0_f32.powi(upper_exp);
    Union::new([
        (neg_lower..neg_upper).boxed(),
        ZERO.boxed(),
        (pos_lower..pos_upper).boxed(),
    ])
}

pub fn bounded_nonzero_f32(lower_exp: i32, upper_exp: i32) -> impl Clone + Strategy<Value = f32> {
    let neg_lower = -(2.0_f32.powi(upper_exp));
    let neg_upper = -(2.0_f32.powi(lower_exp));
    let pos_lower = 2.0_f32.powi(lower_exp);
    let pos_upper = 2.0_f32.powi(upper_exp);
    Union::new([(neg_lower..neg_upper).boxed(), (pos_lower..pos_upper).boxed()])
}

prop_compose! {
    pub fn quat(s: impl Strategy<Value = f32>)(v in vec(s, 4)) -> Quat<f32> {
        Quat::new(v[0], v[1], v[2], v[3])
    }
}

prop_compose! {
    pub fn unit_quat(s: impl Strategy<Value = f32>)(q in quat(s)) -> Unit<Quat<f32>> {
        Unit::from(q)
    }
}

prop_compose! {
    pub fn vec4(s: impl Strategy<Value = f32>)(v in vec(s, 3)) -> Vec4<f32> {
        Vec4::new(v[0], v[1], v[2], 0.0_f32)
    }
}

prop_compose! {
    pub fn unit_vec4(s: impl Strategy<Value = f32>)(v in vec4(s)) -> Unit<Vec4<f32>> {
        Unit::from(v)
    }
}

prop_compose! {
    pub fn point4(s: impl Strategy<Value = f32>)(v in vec(s, 3)) -> Vec4<f32> {
        Vec4::new(v[0], v[1], v[2], 1.0_f32)
    }
}

prop_compose! {
    pub fn mat4(s: impl Strategy<Value = f32>)(v in vec(s, 16)) -> Mat4<f32> {
        Mat4::try_from(v).unwrap()
    }
}

prop_compose! {
    pub fn rot_mat4()(angles in vec(-PI..=PI, 4)) -> Mat4<f32> {
        let sin_a = angles[0].sin();
        let cos_a = angles[0].cos();
        let sin_b = angles[1].sin();
        let cos_b = angles[1].cos();
        let sin_c = angles[2].sin();
        let cos_c = angles[2].cos();

        Mat4([
            [cos_b * cos_c, sin_a * sin_b * cos_c - cos_a * sin_c, cos_a * sin_b * cos_c + sin_a * sin_c, 0.0],
            [cos_b * sin_c, sin_a * sin_b * sin_c + cos_a * cos_c, cos_a * sin_b * sin_c - sin_a * cos_c, 0.0],
            [-sin_b, sin_a * cos_b, cos_a * cos_b, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

prop_compose! {
    pub fn affine(s: impl Strategy<Value = f32>, s2: impl Clone + Strategy<Value = f32>)(t in vec4(s), o in unit_quat(s2.clone()), x in s2) -> Affine<f32> {
        Affine::builder()
            .with_translation(t)
            .with_orientation(o)
            .with_scale(x)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::diff;

    use super::*;
    use approx::relative_eq;
    use proptest::{prop_assert, proptest};

    proptest! {
        #[test]
        fn verify_rot_mat4(m in rot_mat4()) {
            prop_assert!(
                relative_eq!(m * m.t(), Mat4::<f32>::identity(), max_relative=10000000.0 * f32::EPSILON),
                "m * m.t() != Mat4::<f32>::identity()\ndiff:\n{}",
                diff(&(m * m.t()), &Mat4::<f32>::identity(), |a, b| relative_eq!(a, b, max_relative=10000000.0 * f32::EPSILON))
            );
        }
    }
}
