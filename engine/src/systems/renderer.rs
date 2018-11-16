use components::{camera::Camera, renderable::Renderable, model::Model};
use context::{Layer, SceneGraphTrait};
use ecs::{DatabaseTrait, LoopStage, SystemTrait};
use event::EngineEventTrait;
use failure::Error;
use graphics::{BackendTrait, FrameTrait};
use std::{marker::PhantomData, time::Duration};

#[derive(Debug)]
pub struct Renderer<Ctx, Evt, B> {
    pub backend: B,
    pub clear_color: [f32; 4],
    frames: usize,
    draw_calls: usize,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt, B> Renderer<Ctx, Evt, B>
where
    Ctx: DatabaseTrait,
    B: BackendTrait,
{
    pub fn new(
        events_loop: &B::Loop,
        title: &str,
        dimensions: (u32, u32),
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, Error> {
        Ok(Renderer {
            backend: B::new(events_loop, title, dimensions, vsync, msaa)?,
            clear_color: [0.69, 0.93, 0.93, 1.0],
            frames: 0,
            draw_calls: 0,
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
        })
    }

    #[cfg(any(test, feature = "diagnostics"))]
    pub fn average_draw_calls(&self) -> f32 {
        if self.frames > 0 {
            self.draw_calls as f32 / self.frames as f32
        } else {
            0.0
        }
    }

    fn on_startup(&self, ctx: &mut Ctx) -> Result<bool, Error> {
        self.on_change_dpi(ctx, self.backend.dpi_factor())
    }

    fn on_resize(&self, ctx: &mut Ctx, dims: (u32, u32)) -> Result<bool, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Updating the camera dimensions (dims={:?})", dims);

        ctx.find_mut::<Camera>()
            .map(|c| c.set_dimensions(dims))
            .map_err(|e| format_err!("{} (Camera)", e))?;

        Ok(true)
    }

    fn on_change_dpi(&self, ctx: &mut Ctx, factor: f64) -> Result<bool, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Updating the camera dpi factor (factor={:?})", factor);

        ctx.find_mut::<Camera>()
            .map(|c| c.set_dpi_factor(factor))
            .map_err(|e| format_err!("{} (Camera)", e))?;

        Ok(true)
    }
}

impl<Ctx, Evt, B> SystemTrait<Ctx, Evt> for Renderer<Ctx, Evt, B>
where
    Ctx: DatabaseTrait + SceneGraphTrait,
    Evt: EngineEventTrait,
    B: BackendTrait + 'static,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::RENDER | LoopStage::HANDLE_EVENTS
    }

    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::startup() | Evt::resize() | Evt::change_dpi()
    }

    fn handle_event(&mut self, ctx: &mut Ctx, event: &Evt) -> Result<bool, Error> {
        if event.matches_filter(Evt::startup()) {
            self.on_startup(ctx)
        } else if let Some(dims) = event.resize_data() {
            self.on_resize(ctx, dims)
        } else if let Some(factor) = event.change_dpi_data() {
            self.on_change_dpi(ctx, factor)
        } else {
            Ok(true)
        }
    }

    fn render(&mut self, ctx: &mut Ctx, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        {
            self.frames += 1;
        }

        // Update the scene graphs.
        ctx.update_graph(Layer::World)?;
        ctx.update_graph(Layer::Ui)?;

        // Obtain a reference to the camera.
        let cam = ctx.find::<Camera>().map_err(|e| format_err!("{} (Camera)", e))?;

        // Create a new frame.
        let mut target = self.backend.create_frame();
        target.initialize(self.clear_color, 1.0);

        // Render the world scene.
        for (entity, model) in ctx.get_nodes(Layer::World) {
            if ctx.has::<Model>(entity) {
                if let Ok(data) = ctx.get::<Renderable<B>>(entity) {
                    #[cfg(any(test, feature = "diagnostics"))]
                    {
                        self.draw_calls += 1;
                    }
                    target.render(&(cam.world_matrix() * model.matrix()), data)?;
                }
            }
        }

        // Render the ui scene.
        for (entity, model) in ctx.get_nodes(Layer::Ui) {
            if ctx.has::<Model>(entity) {
                if let Ok(data) = ctx.get::<Renderable<B>>(entity) {
                    #[cfg(any(test, feature = "diagnostics"))]
                    {
                        self.draw_calls += 1;
                    }
                    target.render(&(cam.ui_matrix() * model.matrix()), data)?;
                }
            }
        }

        // Finalize the frame and thus swap the display buffers.
        target.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context::{Layer, Context};
    use graphics::headless::HeadlessBackend as HB;
    use mock::{MockEvt, MockEvtFlag};
    use std::f32;

    #[test]
    fn new_headless() {
        assert_ok!(Renderer::<Context<MockEvt>, MockEvt, HB>::new(
            &Default::default(),
            "Title",
            (800, 600),
            false,
            0
        ));
    }

    #[test]
    fn get_stage_filter_headless() {
        let r =
            Renderer::<Context<MockEvt>, MockEvt, HB>::new(&Default::default(), "Title", (800, 600), false, 0).unwrap();

        assert_eq!(r.get_stage_filter(), LoopStage::RENDER | LoopStage::HANDLE_EVENTS);
    }

    #[test]
    fn get_event_filter_headless() {
        let r =
            Renderer::<Context<MockEvt>, MockEvt, HB>::new(&Default::default(), "Title", (800, 600), false, 0).unwrap();

        assert_eq!(
            r.get_event_filter(),
            MockEvtFlag::STARTUP | MockEvtFlag::RESIZE | MockEvtFlag::CHANGE_DPI
        );
    }

    #[test]
    fn render_headless() {
        let mut ctx: Context<MockEvt> = Context::default();
        let mut r =
            Renderer::<Context<MockEvt>, MockEvt, HB>::new(&Default::default(), "Title", (800, 600), false, 0).unwrap();

        let a = ctx.create_entity();
        ctx.insert_node(a, Layer::World);
        ctx.add(a, Model::default()).unwrap();
        ctx.add(a, Renderable::default()).unwrap();
        let b = ctx.create_entity();
        ctx.insert_node(b, Layer::World);
        ctx.add(b, Model::default()).unwrap();
        let c = ctx.create_entity();
        ctx.insert_node(c, Layer::World);
        ctx.add(c, Renderable::default()).unwrap();
        let d = ctx.create_entity();
        ctx.insert_node(d, Layer::World);
        ctx.add(c, Camera::default()).unwrap();

        assert_ok!(r.render(&mut ctx, &Default::default(), &Default::default()));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 1);
        assert_ulps_eq!(r.average_draw_calls(), 1.0, epsilon = f32::EPSILON);
    }
}
