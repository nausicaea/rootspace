use std::time::Duration;

use crate::components::{debug_animate::DebugAnimate, transform::Transform};
use ecs::{Resources, System, WithResources};
use glamour::{quat::Quat, vec::Vec4};

#[derive(Debug)]
pub struct DebugAnimator;

impl WithResources for DebugAnimator {
    #[tracing::instrument(skip_all)]
    fn with_res(_res: &Resources) -> anyhow::Result<Self> {
        Ok(DebugAnimator)
    }
}

impl System for DebugAnimator {
    #[tracing::instrument(skip_all)]
    fn run(&mut self, res: &Resources, _t: Duration, dt: Duration) {
        let angle = dt.as_secs_f32() * 0.20;
        let rotation = Quat::with_axis_angle(Vec4::y(), angle);
        for (_, _, t) in res.iter_rw::<DebugAnimate, Transform>().filter(|(_, _, t)| !t.ui) {
            let t_quat = t.affine.o;
            let new_t_quat = rotation * t_quat;
            t.affine.o = new_t_quat;
        }
    }
}
