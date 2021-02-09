use crate::{components::camera::Camera, event::EngineEvent};
use ecs::{EventQueue, ReceiverId, Resources, System, MaybeDefault};
#[cfg(any(test, debug_assertions))]
use log::{debug, trace};
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraManager {
    receiver: ReceiverId<EngineEvent>,
}

impl CameraManager {
    pub fn new(queue: &mut EventQueue<EngineEvent>) -> Self {
        trace!("CameraManager subscribing to EventQueue<EngineEvent>");

        CameraManager {
            receiver: queue.subscribe(),
        }
    }

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
    fn name(&self) -> &'static str {
        stringify!(CameraManager)
    }

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
