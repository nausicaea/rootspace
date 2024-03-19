use std::time::Duration;

use async_trait::async_trait;

use crate::{
    ecs::{resources::Resources, system::System, with_resources::WithResources},
    engine::components::{renderable::Renderable, transform::Transform},
    glamour::{quat::Quat, unit::Unit},
    Vec4,
};

#[derive(Debug)]
pub struct DebugAnimator;

impl WithResources for DebugAnimator {
    #[tracing::instrument(skip_all)]
    async fn with_res(_res: &crate::ecs::resources::Resources) -> Result<Self, anyhow::Error> {
        Ok(DebugAnimator)
    }
}

#[async_trait]
impl System for DebugAnimator {
    #[tracing::instrument(skip_all)]
    async fn run(&mut self, res: &Resources, _t: Duration, dt: Duration) {
        let angle = dt.as_secs_f32() * 0.20;
        let rotation = Quat::with_axis_angle(Unit::from(Vec4::new(0.0, 1.0, 0.0, 0.0)), angle);
        for (_, _, t) in res.iter_rw::<Renderable, Transform>() {
            let t_quat = t.0.o;
            let new_t_quat = rotation * t_quat;
            t.0.o = new_t_quat;
        }
    }
}
