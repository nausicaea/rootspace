use crate::components::{camera::Camera, model::Model, renderable::Renderable, ui_model::UiModel};
use crate::event::EngineEventTrait;
use crate::scene_graph::SceneGraph;
use ecs::{Component, Storage, System, Resources, EventManager};
use failure::Error;
use crate::graphics::{BackendTrait, FrameTrait};
use std::{marker::PhantomData, time::Duration};

#[derive(Debug)]
pub struct Renderer<Evt, B> {
    pub backend: B,
    pub clear_color: [f32; 4],
    frames: usize,
    draw_calls: usize,
    initialised: bool,
    _evt: PhantomData<Evt>,
}

impl<Evt, B> Renderer<Evt, B>
where
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
            initialised: false,
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
}

impl<Evt, B> System for Renderer<Evt, B>
where
    Evt: EngineEventTrait,
    B: BackendTrait + 'static,
{

    fn run(&mut self, res: &mut Resources, _t: &Duration, _dt: &Duration) {
        #[cfg(any(test, feature = "diagnostics"))]
        {
            self.frames += 1;
        }

        if !self.initialised {
            res.get_mut::<EventManager<Evt>>()
                .dispatch_later(Evt::new_change_dpi(self.backend.dpi_factor()));
            self.initialised = true;
        }

        // Update the scene graphs.
        res.borrow_mut::<SceneGraph<Model>>()
            .update(&res.borrow::<<Model as Component>::Storage>());
        res.borrow_mut::<SceneGraph<UiModel>>()
            .update(&res.borrow::<<UiModel as Component>::Storage>());

        // Obtain a reference to the camera.
        let cameras = res.borrow::<<Camera as Component>::Storage>();

        // Create a new frame.
        let mut target = self.backend.create_frame();
        target.initialize(self.clear_color, 1.0);

        let world_graph = res.borrow::<SceneGraph<Model>>();
        let ui_graph = res.borrow::<SceneGraph<UiModel>>();
        let renderables = res.borrow::<<Renderable<B> as Component>::Storage>();

        for cam in cameras.iter() {
            // Render the world scene.
            for (entity, model) in world_graph.iter() {
                if let Some(data) = renderables.get(entity) {
                    #[cfg(any(test, feature = "diagnostics"))]
                    {
                        self.draw_calls += 1;
                    }
                    target.render(&(cam.world_matrix() * model.matrix()), data)
                        .expect("Unable to render the world");
                }
            }

            // Render the ui scene.
            for (entity, model) in ui_graph.iter() {
                if let Some(data) = renderables.get(entity) {
                    #[cfg(any(test, feature = "diagnostics"))]
                    {
                        self.draw_calls += 1;
                    }
                    target.render(&(cam.ui_matrix() * model.matrix()), data)
                        .expect("Unable to render the UI");
                }
            }
        }

        // Finalize the frame and thus swap the display buffers.
        target.finalize()
            .expect("Unable to finalize the frame");
    }
}
