use std::time::Duration;

use ecs::{event_queue::receiver_id::ReceiverId, EventQueue, Resources, System, WithResources};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{components::camera::Camera, events::window_event::WindowEvent};

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraManager {
    receiver: ReceiverId<WindowEvent>,
}

impl WithResources for CameraManager {
    fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let receiver = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();

        Ok(CameraManager { receiver })
    }
}

impl CameraManager {
    fn on_resize(&self, res: &Resources, dims: (u32, u32)) {
        debug!("Updating the camera dimensions (dims={:?})", dims);

        res.borrow_components_mut::<Camera>()
            .iter_mut()
            .for_each(|c| c.set_dimensions(dims));
    }

    fn on_change_dpi(&self, res: &Resources, factor: f64) {
        debug!("Updating the camera dpi factor (factor={:?})", factor);

        res.borrow_components_mut::<Camera>()
            .iter_mut()
            .for_each(|c| c.set_dpi_factor(factor));
    }
}

impl System for CameraManager {
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<WindowEvent>>().receive(&self.receiver);
        for event in events {
            match event {
                WindowEvent::Resized(dims) => self.on_resize(res, (dims.width, dims.height)),
                WindowEvent::ScaleFactorChanged { scale_factor } => self.on_change_dpi(res, scale_factor),
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ecs::{SystemRegistry, Reg, End, World};

    use super::*;

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
        let _w = World::with_dependencies::<Reg![EventQueue<WindowEvent>], Reg![], Reg![CameraManager], Reg![], _>(&()).unwrap();
    }
}
