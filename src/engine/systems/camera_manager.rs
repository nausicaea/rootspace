use std::time::Duration;

use async_trait::async_trait;
use winit::event::WindowEvent;

use crate::{
    ecs::{
        event_queue::{receiver_id::ReceiverId, EventQueue},
        resources::Resources,
        system::System,
        with_resources::WithResources,
    },
    engine::{components::camera::Camera, resources::graphics::Graphics},
};

#[derive(Debug)]
pub struct CameraManager {
    receiver: ReceiverId<WindowEvent>,
}

impl WithResources for CameraManager {
    #[tracing::instrument(skip_all)]
    async fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let receiver = res.write::<EventQueue<WindowEvent>>().subscribe::<Self>();

        Ok(CameraManager { receiver })
    }
}

impl CameraManager {
    fn on_resize(&self, res: &Resources, width: u32, height: u32) {
        tracing::debug!("Updating the camera dimensions ({width}x{height})");

        res.write_components::<Camera>()
            .iter_mut()
            .for_each(|c| c.set_dimensions(width, height));
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
                    self.on_resize(res, dims.width, dims.height)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ecs::{
            registry::{End, SystemRegistry},
            world::World,
        },
        Reg,
    };

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
