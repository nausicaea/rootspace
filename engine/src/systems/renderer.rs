use context::SceneGraphTrait;
use ecs::{DatabaseTrait, Entity, EventTrait, LoopStage, SystemTrait};
use failure::Error;
use graphics::{BackendTrait, FrameTrait, headless::HeadlessBackend, glium::GliumBackend};
use nalgebra::Matrix4;
use std::{borrow::Borrow, marker::PhantomData, time::Duration};

pub type HeadlessRenderer<Ctx, Evt, Cam, Mdl, Ren> = Renderer<Ctx, Evt, Cam, Mdl, Ren, HeadlessBackend>;
pub type GliumRenderer<Ctx, Evt, Cam, Mdl, Ren> = Renderer<Ctx, Evt, Cam, Mdl, Ren, GliumBackend>;

#[derive(Debug)]
pub struct Renderer<Ctx, Evt, Cam, Mdl, Ren, B> {
    pub backend: B,
    pub clear_color: [f32; 4],
    frames: usize,
    draw_calls: usize,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
    _cam: PhantomData<Cam>,
    _mdl: PhantomData<Mdl>,
    _ren: PhantomData<Ren>,
}

impl<Ctx, Evt, Cam, Mdl, Ren, B> Renderer<Ctx, Evt, Cam, Mdl, Ren, B>
where
    B: BackendTrait,
{
    pub fn new(events_loop: &B::Loop, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error> {
        Ok(Renderer {
            backend: B::new(events_loop, title, dimensions, vsync, msaa)?,
            clear_color: [0.69, 0.93, 0.93, 1.0],
            frames: 0,
            draw_calls: 0,
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
            _cam: PhantomData::default(),
            _mdl: PhantomData::default(),
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
}

impl<Ctx, Evt, Cam, Mdl, Ren, B> SystemTrait<Ctx, Evt> for Renderer<Ctx, Evt, Cam, Mdl, Ren, B>
where
    Ctx: DatabaseTrait + SceneGraphTrait<Entity, Mdl>,
    Evt: EventTrait,
    Cam: Borrow<Matrix4<f32>> + 'static,
    Mdl: Default + Clone + Borrow<Matrix4<f32>> + 'static,
    Ren: Borrow<<B as BackendTrait>::Data> + 'static,
    B: BackendTrait,
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
        let cam = ctx.find::<Cam>()
            .map_err(|e| format_err!("{} (Camera", e))?;

        // Create a new frame.
        let mut target = self.backend.create_frame();
        target.initialize(self.clear_color, 1.0);

        // Render the scene.
        for (entity, model) in nodes {
            if ctx.has::<Mdl>(entity) {
                if let Ok(data) = ctx.get::<Ren>(entity) {
                    #[cfg(any(test, feature = "diagnostics"))]
                    {
                        self.draw_calls += 1;
                    }
                    target.render(&(cam.borrow() * model.borrow()), data)?;
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
    use graphics::{
        glium::{triangle, GliumBackend as GB, GliumRenderData as GRD},
        headless::{HeadlessBackend as HB, HeadlessRenderData as HRD},
    };
    use mock::MockCtx;
    use std::f32;

    #[test]
    fn new_headless() {
        assert_ok!(Renderer::<
            MockCtx<MockEvt, Model>,
            MockEvt,
            Camera,
            Model,
            HRD,
            HB,
        >::new(&Default::default(), "Title", [800, 600], false, 0));
    }

    #[test]
    fn get_stage_filter_headless() {
        let r = Renderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model, HRD, HB>::new(
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
        let mut r = Renderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model, HRD, HB>::new(
            &Default::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

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

    #[test]
    #[cfg_attr(
        feature = "wsl",
        should_panic(
            expected = "Failed to initialize any backend!\n    Wayland status: NoCompositorListening\n    X11 status: XOpenDisplayFailed\n"
        )
    )]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn new_glium() {
        assert_ok!(Renderer::<
            MockCtx<MockEvt, Model>,
            MockEvt,
            Camera,
            Model,
            GRD,
            GB,
        >::new(&Default::default(), "Title", [800, 600], false, 0));
    }

    #[test]
    #[cfg_attr(
        feature = "wsl",
        should_panic(
            expected = "Failed to initialize any backend!\n    Wayland status: NoCompositorListening\n    X11 status: XOpenDisplayFailed\n"
        )
    )]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn get_stage_filter_glium() {
        let r = Renderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model, GRD, GB>::new(
            &Default::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

        assert_eq!(r.get_stage_filter(), LoopStage::RENDER);
    }

    #[test]
    #[cfg_attr(
        feature = "wsl",
        should_panic(
            expected = "Failed to initialize any backend!\n    Wayland status: NoCompositorListening\n    X11 status: XOpenDisplayFailed\n"
        )
    )]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn render_glium() {
        let mut ctx: MockCtx<MockEvt, Model> = MockCtx::default();
        let mut r = Renderer::<MockCtx<MockEvt, Model>, MockEvt, Camera, Model, GRD, GB>::new(
            &Default::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

        let a = ctx.create_entity();
        ctx.insert_node(a);
        ctx.add(a, Model::default()).unwrap();
        ctx.add(a, triangle(&r.backend).unwrap()).unwrap();
        let b = ctx.create_entity();
        ctx.insert_node(b);
        ctx.add(b, Model::default()).unwrap();
        let c = ctx.create_entity();
        ctx.insert_node(c);
        ctx.add(c, triangle(&r.backend).unwrap()).unwrap();
        let d = ctx.create_entity();
        ctx.insert_node(d);
        ctx.add(c, Camera::default()).unwrap();

        assert_ok!(r.render(&mut ctx, &Default::default(), &Default::default()));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 1);
        assert_ulps_eq!(r.average_draw_calls(), 1.0, epsilon = f32::EPSILON);
    }
}
