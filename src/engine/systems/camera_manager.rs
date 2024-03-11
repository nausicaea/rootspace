use async_trait::async_trait;
use std::time::Duration;

use crate::ecs::event_queue::receiver_id::ReceiverId;
use crate::ecs::event_queue::EventQueue;
use crate::ecs::resources::Resources;
use crate::ecs::system::System;
use crate::ecs::with_resources::WithResources;
use crate::engine::components::camera::Camera;
use crate::engine::resources::graphics::Graphics;
use log::debug;
use winit::event::WindowEvent;

#[derive(Debug)]
pub struct CameraManager {
    receiver: ReceiverId<WindowEvent>,
}

impl WithResources for CameraManager {
    async fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let receiver = res.write::<EventQueue<WindowEvent>>().subscribe::<Self>();

        Ok(CameraManager { receiver })
    }
}

impl CameraManager {
    fn on_resize(&self, res: &Resources, dims: (u32, u32)) {
        debug!("Updating the camera dimensions (dims={:?})", dims);

        res.write_components::<Camera>()
            .iter_mut()
            .for_each(|c| c.set_dimensions(dims));
    }
}

#[async_trait]
impl System for CameraManager {
    async fn run(&mut self, res: &Resources, _t: Duration, _dt: Duration) {
        let events = res.write::<EventQueue<WindowEvent>>().receive(&self.receiver);
        for event in events {
            if let WindowEvent::Resized(dims) = event {
                let max_dims = res.read::<Graphics>().max_window_size();
                if dims.width <= max_dims.width && dims.height <= max_dims.height {
                    self.on_resize(res, (dims.width, dims.height))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::registry::{End, SystemRegistry};
    use crate::ecs::world::World;
    use crate::Reg;

    use super::*;

    #[test]
    fn camera_manager_reg_macro() {
        type _SR = Reg![CameraManager];
    }

    #[tokio::test]
    async fn camera_manager_system_registry() {
        let res = Resources::with_dependencies::<Reg![EventQueue<WindowEvent>], _>(&())
            .await
            .unwrap();
        let _rr = SystemRegistry::push(End, CameraManager::with_res(&res).await.unwrap());
    }

    #[tokio::test]
    async fn camera_manager_world() {
        let _w =
            World::with_dependencies::<Reg![EventQueue<WindowEvent>], Reg![], Reg![CameraManager], (), Reg![], _>(&())
                .await
                .unwrap();
    }
}
