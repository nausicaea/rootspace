use components::AsMatrix;
use context::SceneGraphTrait;
use ecs::{DatabaseTrait, Entity, EventTrait, LoopStage, SystemTrait};
use failure::Error;
use graphics::{
    glium::{GliumBackend as GB, GliumEventsLoop as GEL, GliumFrame as GF, GliumRenderData as GRD},
    headless::{HeadlessBackend as HB, HeadlessEventsLoop as HEL, HeadlessFrame as HF, HeadlessRenderData as HRD},
    BackendTrait, FrameTrait,
};
use std::{marker::PhantomData, time::Duration};

pub type HeadlessRenderer<Ctx, Evt, Cam, Mdl> = Renderer<Ctx, Evt, Cam, Mdl, HRD, HF, HEL, HB>;
pub type GliumRenderer<Ctx, Evt, Cam, Mdl> = Renderer<Ctx, Evt, Cam, Mdl, GRD, GF, GEL, GB>;

#[derive(Debug)]
pub struct Renderer<Ctx, Evt, Cam, Mdl, R, F, E, B> {
    pub backend: B,
    pub clear_color: [f32; 4],
    frames: usize,
    draw_calls: usize,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
    _cam: PhantomData<Cam>,
    _mdl: PhantomData<Mdl>,
    _r: PhantomData<R>,
    _f: PhantomData<F>,
    _e: PhantomData<E>,
}

impl<Ctx, Evt, Cam, Mdl, R, F, E, B> Renderer<Ctx, Evt, Cam, Mdl, R, F, E, B>
where
    B: BackendTrait<E, F>,
{
    pub fn new(events_loop: &E, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error> {
        Ok(Renderer {
            backend: B::new(events_loop, title, dimensions, vsync, msaa)?,
            clear_color: [0.69, 0.93, 0.93, 1.0],
            frames: 0,
            draw_calls: 0,
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
            _cam: PhantomData::default(),
            _mdl: PhantomData::default(),
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

impl<Ctx, Evt, Cam, Mdl, R, F, E, B> SystemTrait<Ctx, Evt> for Renderer<Ctx, Evt, Cam, Mdl, R, F, E, B>
where
    Ctx: DatabaseTrait + SceneGraphTrait<Entity, Mdl>,
    Evt: EventTrait,
    Cam: AsMatrix + 'static,
    Mdl: Default + Clone + AsMatrix + 'static,
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

        // Obtain a reference to the camera.
        let cam = match ctx.find::<Cam>() {
            Ok(cam) => cam,
            Err(e) => return Err(format_err!("{} (Camera)", e)),
        };

        // Create a new frame.
        let mut target = self.backend.create_frame();
        target.initialize(self.clear_color, 1.0);

        // Render the scene.
        for (entity, model) in nodes {
            if ctx.has::<Mdl>(entity) {
                if let Ok(data) = ctx.borrow::<R>(entity) {
                    #[cfg(any(test, feature = "diagnostics"))]
                    {
                        self.draw_calls += 1;
                    }
                    target.render(&(cam.as_matrix() * model.as_matrix()), data)?;
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
    use components::{camera::Camera, model::Model};
    use ecs::mock::MockEvt;
    use graphics::{glium::GliumRenderData as GRD, headless::HeadlessRenderData as HRD, RenderDataTrait};
    use mock::MockCtx;
    use std::f32;

    #[test]
    fn new_headless() {
        assert_ok!(
            HeadlessRenderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model>::new(
                &Default::default(),
                "Title",
                [800, 600],
                false,
                0
            )
        );
    }

    #[test]
    fn get_stage_filter_headless() {
        let r = HeadlessRenderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model>::new(
            &Default::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

        assert_eq!(r.get_stage_filter(), LoopStage::RENDER);
    }

    #[test]
    fn render_headless() {
        let mut ctx: MockCtx<MockEvt, Model> = MockCtx::default();
        let mut r = HeadlessRenderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model>::new(
            &Default::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

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
        ctx.add(c, Camera::default()).unwrap();

        assert_ok!(r.render(&mut ctx, &Default::default(), &Default::default()));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 1);
        assert_ulps_eq!(r.average_draw_calls(), 1.0, epsilon = f32::EPSILON);
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn new_glium() {
        assert_ok!(GliumRenderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model>::new(
            &Default::default(),
            "Title",
            [800, 600],
            false,
            0
        ));
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn get_stage_filter_glium() {
        let r = GliumRenderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model>::new(
            &Default::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

        assert_eq!(r.get_stage_filter(), LoopStage::RENDER);
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn render_glium() {
        let mut ctx: MockCtx<MockEvt, Model> = MockCtx::default();
        let mut r = GliumRenderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model>::new(
            &Default::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

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
        ctx.add(c, Camera::default()).unwrap();

        assert_ok!(r.render(&mut ctx, &Default::default(), &Default::default()));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 1);
        assert_ulps_eq!(r.average_draw_calls(), 1.0, epsilon = f32::EPSILON);
    }
}
