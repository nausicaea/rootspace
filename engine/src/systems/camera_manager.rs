use crate::components::camera::Camera;
use crate::context::SceneGraphTrait;
use crate::event::EngineEventTrait;
use ecs::{DatabaseTrait, EventHandlerSystem};
use std::marker::PhantomData;

pub struct CameraManager<Ctx, Evt> {
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt> Default for CameraManager<Ctx, Evt> {
    fn default() -> Self {
        CameraManager {
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
        }
    }
}

impl<Ctx, Evt> CameraManager<Ctx, Evt>
where
    Ctx: DatabaseTrait,
{
    fn on_resize(&self, ctx: &mut Ctx, dims: (u32, u32)) {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Updating the camera dimensions (dims={:?})", dims);

        ctx.find_mut::<Camera>()
            .map(|c| c.set_dimensions(dims))
            .map_err(|e| format_err!("{} (Camera)", e))
            .expect("Could not update the camera dimensions");
    }

    fn on_change_dpi(&self, ctx: &mut Ctx, factor: f64) {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Updating the camera dpi factor (factor={:?})", factor);

        ctx.find_mut::<Camera>()
            .map(|c| c.set_dpi_factor(factor))
            .map_err(|e| format_err!("{} (Camera)", e))
            .expect("Could not update the camera dpi factor");
    }
}

impl<Ctx, Evt> EventHandlerSystem<Ctx, Evt> for CameraManager<Ctx, Evt>
where
    Ctx: DatabaseTrait + SceneGraphTrait,
    Evt: EngineEventTrait,
{
    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::resize() | Evt::change_dpi()
    }

    fn run(&mut self, ctx: &mut Ctx, event: &Evt) -> bool {
        if let Some(dims) = event.resize_data() {
            self.on_resize(ctx, dims);
        } else if let Some(factor) = event.change_dpi_data() {
            self.on_change_dpi(ctx, factor);
        }

        true
    }
}
