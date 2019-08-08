use crate::{
    components::{Camera, Model, Renderable, Status, UiModel},
    event::EngineEvent,
    graphics::{BackendTrait, FrameTrait},
    resources::SceneGraph,
};
use ecs::{EventQueue, Resources, Storage, System};
use failure::Error;
#[cfg(any(test, feature = "diagnostics"))]
use std::time::Instant;
use std::{collections::VecDeque, time::Duration};
#[cfg(feature = "diagnostics")]
use typename::TypeName;

static DRAW_CALL_WINDOW: usize = 10;
static FRAME_TIME_WINDOW: usize = 10;

#[derive(Debug)]
pub struct Renderer<B> {
    pub backend: B,
    pub clear_color: [f32; 4],
    draw_calls: VecDeque<usize>,
    frame_times: VecDeque<Duration>,
    initialised: bool,
}

impl<B> Renderer<B>
where
    B: BackendTrait,
{
    pub fn new(
        events_loop: &B::EventsLoop,
        title: &str,
        dimensions: (u32, u32),
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, Error> {
        Ok(Renderer {
            backend: B::new(events_loop, title, dimensions, vsync, msaa)?,
            clear_color: [0.69, 0.93, 0.93, 1.0],
            draw_calls: VecDeque::with_capacity(DRAW_CALL_WINDOW),
            frame_times: VecDeque::with_capacity(FRAME_TIME_WINDOW),
            initialised: false,
        })
    }

    #[cfg(any(test, feature = "diagnostics"))]
    pub fn average_draw_calls(&self) -> f32 {
        self.draw_calls.iter().sum::<usize>() as f32 / DRAW_CALL_WINDOW as f32
    }

    #[cfg(any(test, feature = "diagnostics"))]
    pub fn average_frame_time(&self) -> Duration {
        self.frame_times.iter().sum::<Duration>() / FRAME_TIME_WINDOW as u32
    }

    #[cfg(any(test, feature = "diagnostics"))]
    fn update_draw_calls(&mut self, draw_calls: usize) {
        self.draw_calls.push_front(draw_calls);
        if self.draw_calls.len() > DRAW_CALL_WINDOW {
            self.draw_calls.truncate(DRAW_CALL_WINDOW);
        }
    }

    #[cfg(any(test, feature = "diagnostics"))]
    fn update_frame_time(&mut self, frame_time: Duration) {
        self.frame_times.push_front(frame_time);
        if self.frame_times.len() > FRAME_TIME_WINDOW {
            self.frame_times.truncate(FRAME_TIME_WINDOW);
        }
    }
}

#[cfg(not(feature = "diagnostics"))]
impl<B> System for Renderer<B>
where
    B: BackendTrait,
{
    fn name(&self) -> &'static str {
        "Renderer"
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        #[cfg(any(test, feature = "diagnostics"))]
        let start_mark = Instant::now();

        #[cfg(any(test, feature = "diagnostics"))]
        let mut draw_calls: usize = 0;

        if !self.initialised {
            res.borrow_mut::<EventQueue<EngineEvent>>()
                .send(EngineEvent::ChangeDpi(self.backend.dpi_factor()));
            self.initialised = true;
        }

        // Update the scene graphs.
        res.borrow_mut::<SceneGraph<Model>>()
            .update(&res.borrow_component::<Model>());
        res.borrow_mut::<SceneGraph<UiModel>>()
            .update(&res.borrow_component::<UiModel>());

        // Obtain a reference to the camera.
        let cameras = res.borrow_component::<Camera>();

        // Create a new frame.
        let mut target = self.backend.create_frame();
        target.initialize(self.clear_color, 1.0);

        let world_graph = res.borrow::<SceneGraph<Model>>();
        let ui_graph = res.borrow::<SceneGraph<UiModel>>();
        let statuses = res.borrow_component::<Status>();
        let renderables = res.borrow_component::<Renderable<B>>();

        for cam in cameras.iter() {
            // Render the world scene.
            for (entity, model) in world_graph.iter() {
                if statuses.get(entity).map(|s| s.enabled()) == Some(true) {
                    if let Some(data) = renderables.get(entity) {
                        #[cfg(any(test, feature = "diagnostics"))]
                        {
                            draw_calls += 1;
                        }
                        target
                            .render(&(cam.world_matrix() * model.matrix()), data)
                            .expect("Unable to render the world");
                    }
                }
            }

            // Render the ui scene.
            for (entity, model) in ui_graph.iter() {
                if statuses.get(entity).map(|s| s.enabled()) == Some(true) {
                    if let Some(data) = renderables.get(entity) {
                        #[cfg(any(test, feature = "diagnostics"))]
                        {
                            draw_calls += 1;
                        }
                        target
                            .render(&(cam.ui_matrix() * model.matrix()), data)
                            .expect("Unable to render the UI");
                    }
                }
            }
        }

        // Finalize the frame and thus swap the display buffers.
        target.finalize().expect("Unable to finalize the frame");

        #[cfg(any(test, feature = "diagnostics"))]
        self.update_draw_calls(draw_calls);

        #[cfg(any(test, feature = "diagnostics"))]
        self.update_frame_time(start_mark.elapsed());
    }
}

#[cfg(feature = "diagnostics")]
impl<B> System for Renderer<B>
where
    B: BackendTrait + TypeName,
{
    fn name(&self) -> &'static str {
        "Renderer"
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        #[cfg(any(test, feature = "diagnostics"))]
        let start_mark = Instant::now();

        #[cfg(any(test, feature = "diagnostics"))]
        let mut draw_calls: usize = 0;

        if !self.initialised {
            res.borrow_mut::<EventQueue<EngineEvent>>()
                .send(EngineEvent::ChangeDpi(self.backend.dpi_factor()));
            self.initialised = true;
        }

        // Update the scene graphs.
        res.borrow_mut::<SceneGraph<Model>>()
            .update(&res.borrow_component::<Model>());
        res.borrow_mut::<SceneGraph<UiModel>>()
            .update(&res.borrow_component::<UiModel>());

        // Obtain a reference to the camera.
        let cameras = res.borrow_component::<Camera>();

        // Create a new frame.
        let mut target = self.backend.create_frame();
        target.initialize(self.clear_color, 1.0);

        let world_graph = res.borrow::<SceneGraph<Model>>();
        let ui_graph = res.borrow::<SceneGraph<UiModel>>();
        let statuses = res.borrow_component::<Status>();
        let renderables = res.borrow_component::<Renderable<B>>();

        for cam in cameras.iter() {
            // Render the world scene.
            for (entity, model) in world_graph.iter() {
                if statuses.get(entity).map(|s| s.enabled()) == Some(true) {
                    if let Some(data) = renderables.get(entity) {
                        #[cfg(any(test, feature = "diagnostics"))]
                        {
                            draw_calls += 1;
                        }
                        target
                            .render(&(cam.world_matrix() * model.matrix()), data)
                            .expect("Unable to render the world");
                    }
                }
            }

            // Render the ui scene.
            for (entity, model) in ui_graph.iter() {
                if statuses.get(entity).map(|s| s.enabled()) == Some(true) {
                    if let Some(data) = renderables.get(entity) {
                        #[cfg(any(test, feature = "diagnostics"))]
                        {
                            draw_calls += 1;
                        }
                        target
                            .render(&(cam.ui_matrix() * model.matrix()), data)
                            .expect("Unable to render the UI");
                    }
                }
            }
        }

        // Finalize the frame and thus swap the display buffers.
        target.finalize().expect("Unable to finalize the frame");

        #[cfg(any(test, feature = "diagnostics"))]
        self.update_draw_calls(draw_calls);

        #[cfg(any(test, feature = "diagnostics"))]
        self.update_frame_time(start_mark.elapsed());
    }
}
