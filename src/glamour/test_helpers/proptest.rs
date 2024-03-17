use crate::glamour::affine::Affine;
use crate::glamour::mat::Mat4;
use crate::glamour::quat::Quat;
use crate::glamour::unit::Unit;
use crate::Vec4;
use ::proptest::{
    collection::vec,
    num::f32::ZERO,
    prop_compose,
    strategy::{Strategy, Union},
};
use proptest::num::f32::{NEGATIVE, NORMAL, POSITIVE, SUBNORMAL};

pub fn bounded_f32(lower_exp: i32, upper_exp: i32) -> impl Clone + Strategy<Value = f32> {
    let neg_lower = -(2.0_f32).powi(upper_exp);
    let neg_upper = -(2.0_f32).powi(lower_exp);
    let pos_lower = (2.0_f32).powi(lower_exp);
    let pos_upper = (2.0_f32).powi(upper_exp);
    Union::new([
        (neg_lower..neg_upper).boxed(),
        ZERO.boxed(),
        (pos_lower..pos_upper).boxed(),
    ])
}

pub fn bounded_nonzero_f32(lower_exp: i32, upper_exp: i32) -> impl Clone + Strategy<Value = f32> {
    let neg_lower = -(2.0_f32).powi(upper_exp);
    let neg_upper = -(2.0_f32).powi(lower_exp);
    let pos_lower = (2.0_f32).powi(lower_exp);
    let pos_upper = (2.0_f32).powi(upper_exp);
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
    pub fn affine(s: impl Clone + Strategy<Value = f32>)(t in vec4(s.clone()), o in unit_quat(s.clone()), x in (POSITIVE | NEGATIVE | NORMAL | SUBNORMAL)) -> Affine<f32> {
        Affine::builder()
            .with_translation(t)
            .with_orientation(o)
            .with_scale(x)
            .build()
    }
}
