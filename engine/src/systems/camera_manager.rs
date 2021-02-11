use crate::{components::camera::Camera, event::EngineEvent};
use ecs::{EventQueue, ReceiverId, Resources, System, MaybeDefault, WithResources};
#[cfg(any(test, debug_assertions))]
use log::{debug, trace};
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraManager {
    receiver: ReceiverId<EngineEvent>,
}

impl WithResources for CameraManager {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>()
            .subscribe::<Self>();

        CameraManager {
            receiver,
        }
    }
}

impl CameraManager {
    fn on_resize(&self, res: &Resources, dims: (u32, u32)) {
        #[cfg(any(test, debug_assertions))]
        debug!("Updating the camera dimensions (dims={:?})", dims);

        res.borrow_components_mut::<Camera>()
            .iter_mut()
            .for_each(|c| c.set_dimensions(dims));
    }

    fn on_change_dpi(&self, res: &Resources, factor: f64) {
        #[cfg(any(test, debug_assertions))]
        debug!("Updating the camera dpi factor (factor={:?})", factor);

        res.borrow_components_mut::<Camera>()
            .iter_mut()
            .for_each(|c| c.set_dpi_factor(factor));
    }
}

impl MaybeDefault for CameraManager {
    fn maybe_default() -> Option<Self> {
        None
    }
}

impl System for CameraManager {
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res
            .borrow_mut::<EventQueue<EngineEvent>>()
            .receive(&self.receiver);
        for event in events {
            match event {
                EngineEvent::Resize(dims) => self.on_resize(res, dims),
                EngineEvent::ChangeDpi(factor) => self.on_change_dpi(res, factor),
                _ => (),
            }
        }
    }
}
