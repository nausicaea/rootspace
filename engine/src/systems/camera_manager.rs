use crate::components::camera::Camera;
use crate::event::EngineEventTrait;
use ecs::{EventHandlerSystem, Resources};
use std::marker::PhantomData;

pub struct CameraManager<Evt> {
    _evt: PhantomData<Evt>,
}

impl<Evt> Default for CameraManager<Evt> {
    fn default() -> Self {
        CameraManager {
            _evt: PhantomData::default(),
        }
    }
}

impl<Evt> CameraManager<Evt> {
    fn on_resize(&self, res: &mut Resources, dims: (u32, u32)) {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Updating the camera dimensions (dims={:?})", dims);

        unimplemented!();
        // ctx.find_mut::<Camera>()
        //     .map(|c| c.set_dimensions(dims))
        //     .map_err(|e| format_err!("{} (Camera)", e))
        //     .expect("Could not update the camera dimensions");
    }

    fn on_change_dpi(&self, res: &mut Resources, factor: f64) {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Updating the camera dpi factor (factor={:?})", factor);

        unimplemented!();
        // ctx.find_mut::<Camera>()
        //     .map(|c| c.set_dpi_factor(factor))
        //     .map_err(|e| format_err!("{} (Camera)", e))
        //     .expect("Could not update the camera dpi factor");
    }
}

impl<Evt> EventHandlerSystem<Evt> for CameraManager<Evt>
where
    Evt: EngineEventTrait,
{
    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::resize() | Evt::change_dpi()
    }

    fn run(&mut self, res: &mut Resources, event: &Evt) -> bool {
        if let Some(dims) = event.resize_data() {
            self.on_resize(res, dims);
        } else if let Some(factor) = event.change_dpi_data() {
            self.on_change_dpi(res, factor);
        }

        true
    }
}
