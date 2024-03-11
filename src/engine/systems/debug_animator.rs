use crate::glamour::num::ToMatrix;
use std::time::Duration;

use async_trait::async_trait;

use crate::{
    ecs::{resources::Resources, system::System, with_resources::WithResources},
    engine::components::{renderable::Renderable, transform::Transform},
    glamour::{quat::Quat, unit::Unit},
};

#[derive(Debug)]
pub struct DebugAnimator;

impl WithResources for DebugAnimator {
    async fn with_res(_res: &crate::ecs::resources::Resources) -> Result<Self, anyhow::Error> {
        Ok(DebugAnimator)
    }
}

#[async_trait]
impl System for DebugAnimator {
    async fn run(&mut self, res: &Resources, _t: Duration, dt: Duration) {
        let angle = dt.as_secs_f32() * 0.21;
        let rotation = Unit::from(Quat::new(angle, 0.0, 1.0, 0.0)).to_matrix();
        for (_, _, t) in res.iter_rw::<Renderable, Transform>() {
            let t_mat = t.orientation().to_matrix();
            t.set_orientation(Unit::from(Into::<Quat<f32>>::into(t_mat * rotation)))
        }
    }
}
