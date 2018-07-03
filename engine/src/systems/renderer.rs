use context::SceneGraphTrait;
use graphics::{BackendTrait, FrameTrait};
use ecs::database::DatabaseTrait;
use ecs::entity::Entity;
use ecs::event::EventTrait;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use failure::Error;
use std::marker::PhantomData;
use std::time::Duration;

#[derive(Debug)]
pub struct Renderer<Ctx, Evt, L, R, F, E, B> {
    backend: B,
    frames: usize,
    draw_calls: usize,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
    _l: PhantomData<L>,
    _r: PhantomData<R>,
    _f: PhantomData<F>,
    _e: PhantomData<E>,
}

impl<Ctx, Evt, L, R, F, E, B> Renderer<Ctx, Evt, L, R, F, E, B>
where
    Evt: EventTrait,
    L: AsRef<[[f32; 4]; 4]>,
    F: FrameTrait<R>,
    B: BackendTrait<E, F>,
{
    pub fn new(events_loop: &E, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error> {
        Ok(Renderer {
            backend: B::new(events_loop, title, dimensions, vsync, msaa)?,
            frames: 0,
            draw_calls: 0,
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
            _l: PhantomData::default(),
            _r: PhantomData::default(),
            _f: PhantomData::default(),
            _e: PhantomData::default(),
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
}

impl<Ctx, Evt, L, R, F, E, B> SystemTrait<Ctx, Evt> for Renderer<Ctx, Evt, L, R, F, E, B>
where
    Ctx: DatabaseTrait + SceneGraphTrait<Entity, L>,
    Evt: EventTrait,
    L: Default + Clone + AsRef<[[f32; 4]; 4]> + 'static,
    R: 'static,
    F: FrameTrait<R>,
    B: BackendTrait<E, F>,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::RENDER
    }

    fn render(&mut self, ctx: &mut Ctx, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        {
            self.frames += 1;
        }

        // Update the scene graph and obtain the nodes (while sorting for z-value).
        ctx.update_graph()?;
        let nodes = ctx.get_nodes(true);

        // Create a new frame.
        let mut target = self.backend.create_frame();
        target.initialize([0.0, 0.0, 0.0, 1.0], 1.0);

        // Render the scene.
        for (entity, model) in nodes {
            if ctx.has::<L>(entity) {
                if let Ok(data) = ctx.borrow::<R>(entity) {
                    #[cfg(any(test, feature = "diagnostics"))]
                    {
                        self.draw_calls += 1;
                    }
                    target.render(model, data)?;
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
    use components::model::Model;
    use graphics::RenderDataTrait;
    use graphics::headless::{HeadlessBackend as HB, HeadlessFrame as HF, HeadlessRenderData as HRD, HeadlessEventsLoop as HEL};
    use graphics::glium::{GliumBackend as GB, GliumFrame as GF, GliumRenderData as GRD, GliumEventsLoop as GEL};
    use mock::context::MockCtx;
    use ecs::mock::event::MockEvt;
    use std::f32;

    #[test]
    fn new_headless() {
        assert_ok!(Renderer::<MockCtx<MockEvt, Model>, MockEvt, Model, HRD, HF, HEL, HB>::new(&Default::default(), "Title", [800, 600], false, 0));
    }

    #[test]
    fn render_headless() {
        let mut ctx: MockCtx<MockEvt, Model> = MockCtx::default();
        let mut r = Renderer::<MockCtx<MockEvt, Model>, MockEvt, Model, HRD, HF, HEL, HB>::new(&Default::default(), "Title", [800, 600], false, 0).unwrap();

        let a = ctx.create_entity();
        ctx.insert_node(a);
        ctx.add(a, Model::default()).unwrap();
        ctx.add(a, HRD::triangle(&r.backend).unwrap()).unwrap();
        let b = ctx.create_entity();
        ctx.insert_node(b);
        ctx.add(b, Model::default()).unwrap();
        let c = ctx.create_entity();
        ctx.insert_node(c);
        ctx.add(c, HRD::triangle(&r.backend).unwrap()).unwrap();
        let d = ctx.create_entity();
        ctx.insert_node(d);

        assert_ok!(r.render(&mut ctx, &Default::default(), &Default::default()));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 1);
        assert_ulps_eq!(r.average_draw_calls(), 1.0, epsilon = f32::EPSILON);
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn new_glium() {
        assert_ok!(Renderer::<MockCtx<MockEvt, Model>, MockEvt, Model, GRD, GF, GEL, GB>::new(&Default::default(), "Title", [800, 600], false, 0));
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn render_glium() {
        let mut ctx: MockCtx<MockEvt, Model> = MockCtx::default();
        let mut r = Renderer::<MockCtx<MockEvt, Model>, MockEvt, Model, GRD, GF, GEL, GB>::new(&Default::default(), "Title", [800, 600], false, 0).unwrap();

        let a = ctx.create_entity();
        ctx.insert_node(a);
        ctx.add(a, Model::default()).unwrap();
        ctx.add(a, GRD::triangle(&r.backend).unwrap()).unwrap();
        let b = ctx.create_entity();
        ctx.insert_node(b);
        ctx.add(b, Model::default()).unwrap();
        let c = ctx.create_entity();
        ctx.insert_node(c);
        ctx.add(c, GRD::triangle(&r.backend).unwrap()).unwrap();
        let d = ctx.create_entity();
        ctx.insert_node(d);

        assert_ok!(r.render(&mut ctx, &Default::default(), &Default::default()));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 1);
        assert_ulps_eq!(r.average_draw_calls(), 1.0, epsilon = f32::EPSILON);
    }
}
