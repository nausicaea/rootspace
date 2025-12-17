use std::time::Duration;

use griffon::winit::event::WindowEvent;

use crate::components::camera::Camera;
use ecs::{EventQueue, ReceiverId, Resources, System, WithResources};
use griffon::Graphics;

#[derive(Debug)]
pub struct CameraManager {
    receiver: ReceiverId<WindowEvent>,
}

impl WithResources for CameraManager {
    #[tracing::instrument(skip_all)]
    fn with_res(res: &Resources) -> anyhow::Result<Self> {
        let receiver = res.write::<EventQueue<WindowEvent>>().subscribe::<Self>();

        Ok(CameraManager { receiver })
    }
}

impl CameraManager {
    #[tracing::instrument(skip_all)]
    fn on_resize(&self, res: &Resources, width: u32, height: u32) {
        tracing::debug!("Updating the camera dimensions ({width}x{height})");

        res.write_components::<Camera>()
            .iter_mut()
            .for_each(|c| c.set_dimensions(width, height));
    }
}

impl System for CameraManager {
    #[tracing::instrument(skip_all)]
    fn run(&mut self, res: &Resources, _t: Duration, _dt: Duration) {
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
    use ecs::{End, Reg, SystemRegistry, World};

    #[test]
    fn camera_manager_reg_macro() {
        type _SR = Reg![CameraManager];
    }

    #[test]
    fn camera_manager_system_registry() {
        let res = Resources::with_dependencies::<Reg![EventQueue<WindowEvent>], _>(&()).unwrap();
        let _rr = SystemRegistry::push(End, CameraManager::with_res(&res).unwrap());
    }

    #[test]
    fn camera_manager_world() {
        let _w =
            World::with_dependencies::<Reg![EventQueue<WindowEvent>], Reg![], Reg![CameraManager], (), Reg![], _>(&())
                .unwrap();
    }
}
