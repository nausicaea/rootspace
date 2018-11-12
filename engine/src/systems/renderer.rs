use context::SceneGraphTrait;
use components::{Layer, TransformTrait, camera::Camera, model::Model};
use ecs::{DatabaseTrait, Entity, LoopStage, SystemTrait};
use event::{Event, EventFlag, EventData};
use failure::Error;
use graphics::{glium::GliumBackend, headless::HeadlessBackend, BackendTrait, FrameTrait};
use std::{borrow::Borrow, marker::PhantomData, time::Duration};

pub type HeadlessRenderer<Ctx, Ren> = Renderer<Ctx, Ren, HeadlessBackend>;
pub type GliumRenderer<Ctx, Ren> = Renderer<Ctx, Ren, GliumBackend>;

#[derive(Debug)]
pub struct Renderer<Ctx, Ren, B> {
    pub backend: B,
    pub clear_color: [f32; 4],
    frames: usize,
    draw_calls: usize,
    _ctx: PhantomData<Ctx>,
    _ren: PhantomData<Ren>,
}

impl<Ctx, Ren, B> Renderer<Ctx, Ren, B>
where
    B: BackendTrait,
    Ctx: DatabaseTrait,
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
            _ren: PhantomData::default(),
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

    fn on_resize(&self, ctx: &mut Ctx, dims: (u32, u32)) -> Result<bool, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Updating the camera dimensions (dims={:?})", dims);

        ctx.find_mut::<Camera>()
            .map(|c| c.set_dimensions(dims))
            .map_err(|e| format_err!("{} (Camera)", e))?;

        Ok(true)
    }

    fn on_dpi_change(&self, ctx: &mut Ctx, factor: f64) -> Result<bool, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Updating the camera dpi factor (factor={:?})", factor);

        ctx.find_mut::<Camera>()
            .map(|c| c.set_dpi_factor(factor))
            .map_err(|e| format_err!("{} (Camera)", e))?;

        Ok(true)
    }
}

impl<Ctx, Ren, B> SystemTrait<Ctx, Event> for Renderer<Ctx, Ren, B>
where
    Ctx: DatabaseTrait + SceneGraphTrait<Entity, Model>,
    Ren: Borrow<<B as BackendTrait>::Data> + 'static,
    B: BackendTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::RENDER | LoopStage::HANDLE_EVENTS
    }

    fn get_event_filter(&self) -> EventFlag {
        EventFlag::RESIZE | EventFlag::CHANGE_DPI
    }

    fn handle_event(&mut self, ctx: &mut Ctx, event: &Event) -> Result<bool, Error> {
        match event.data() {
            EventData::Resize(dims) => self.on_resize(ctx, *dims),
            EventData::ChangeDpi(factor) => self.on_dpi_change(ctx, *factor),
            _ => Ok(true),
        }
    }

    fn render(&mut self, ctx: &mut Ctx, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        {
            self.frames += 1;
        }

        // Update the scene graph and obtain the nodes (while sorting for z-value).
        ctx.update_graph()?;
        let nodes = ctx.get_nodes(true);

        // Obtain a reference to the camera.
        let cam = ctx.find::<Camera>().map_err(|e| format_err!("{} (Camera)", e))?;

        // Create a new frame.
        let mut target = self.backend.create_frame();
        target.initialize(self.clear_color, 1.0);

        // Render the scene.
        for (entity, model) in nodes {
            if ctx.has::<Model>(entity) {
                if let Ok(data) = ctx.get::<Ren>(entity) {
                    #[cfg(any(test, feature = "diagnostics"))]
                    {
                        self.draw_calls += 1;
                    }
                    let transform = match model.layer() {
                        Layer::World => cam.matrix() * model.matrix(),
                        Layer::Ndc => model.matrix().clone(),
                    };
                    target.render(&transform, data)?;
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
    use graphics::headless::{HeadlessBackend as HB, HeadlessRenderData as HRD};
    use mock::MockCtx;
    use std::f32;

    #[test]
    fn new_headless() {
        assert_ok!(
            Renderer::<MockCtx<Event, Model>, HRD, HB>::new(
                &Default::default(),
                "Title",
                (800, 600),
                false,
                0
            )
        );
    }

    #[test]
    fn get_stage_filter_headless() {
        let r = Renderer::<MockCtx<Event, Model>, HRD, HB>::new(
            &Default::default(),
            "Title",
            (800, 600),
            false,
            0,
        )
        .unwrap();

        assert_eq!(r.get_stage_filter(), LoopStage::RENDER | LoopStage::HANDLE_EVENTS);
    }

    #[test]
    fn get_event_filter_headless() {
        let r = Renderer::<MockCtx<Event, Model>, HRD, HB>::new(
            &Default::default(),
            "Title",
            (800, 600),
            false,
            0,
        )
        .unwrap();

        assert_eq!(r.get_event_filter(), EventFlag::RESIZE | EventFlag::CHANGE_DPI);
    }

    #[test]
    fn render_headless() {
        let mut ctx: MockCtx<Event, Model> = MockCtx::default();
        let mut r = Renderer::<MockCtx<Event, Model>, HRD, HB>::new(
            &Default::default(),
            "Title",
            (800, 600),
            false,
            0,
        )
        .unwrap();

        let a = ctx.create_entity();
        ctx.insert_node(a);
        ctx.add(a, Model::default()).unwrap();
        ctx.add(a, HRD::default()).unwrap();
        let b = ctx.create_entity();
        ctx.insert_node(b);
        ctx.add(b, Model::default()).unwrap();
        let c = ctx.create_entity();
        ctx.insert_node(c);
        ctx.add(c, HRD::default()).unwrap();
        let d = ctx.create_entity();
        ctx.insert_node(d);
        ctx.add(c, Camera::default()).unwrap();

        assert_ok!(r.render(&mut ctx, &Default::default(), &Default::default()));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 1);
        assert_ulps_eq!(r.average_draw_calls(), 1.0, epsilon = f32::EPSILON);
    }
}
