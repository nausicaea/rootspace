#[cfg(any(test, debug_assertions))]
use std::time::Instant;
use std::{collections::VecDeque, marker::PhantomData, time::Duration};
use serde::{Serialize, Deserialize};

#[cfg(any(test, debug_assertions))]
use log::debug;
use log::trace;

use ecs::{world::event::WorldEvent, Entities, EventQueue, ReceiverId, Resources, Storage, System, MaybeDefault, WithResources};

use crate::{
    components::{Camera, Model, Renderable, Status, UiModel},
    event::EngineEvent,
    graphics::{BackendTrait, FrameTrait},
    resources::{GraphicsBackend, SceneGraph},
};
use crate::resources::SettingsTrait;

static DRAW_CALL_WINDOW: usize = 10;
static FRAME_TIME_WINDOW: usize = 10;

#[derive(Serialize, Deserialize)]
pub struct Renderer<S, B> {
    receiver: ReceiverId<WorldEvent>,
    #[serde(skip)]
    initialised: bool,
    #[serde(skip)]
    draw_calls: VecDeque<(usize, usize)>,
    #[serde(skip)]
    frame_times: VecDeque<Duration>,
    #[serde(skip)]
    _b: PhantomData<B>,
    #[serde(skip)]
    _s: PhantomData<S>,
}

impl<S, B> MaybeDefault for Renderer<S, B> {}

impl<S, B> std::fmt::Debug for Renderer<S, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Renderer {{ receiver: {:?}, initialised: {:?}, draw_calls: {:?}, frame_times: {:?} }}",
            self.receiver,
            self.initialised,
            self.draw_calls,
            self.frame_times,
        )
    }
}

impl<S, B> WithResources for Renderer<S, B>
where
    B: BackendTrait,
{
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<WorldEvent>>()
            .subscribe::<Self>();

        Renderer {
            receiver,
            initialised: false,
            draw_calls: VecDeque::with_capacity(DRAW_CALL_WINDOW),
            frame_times: VecDeque::with_capacity(FRAME_TIME_WINDOW),
            _b: PhantomData::default(),
            _s: PhantomData::default(),
        }
    }
}


impl<S, B> Renderer<S, B>
where
    B: BackendTrait,
{
    fn set_dpi_factor(&self, res: &Resources) {
        let dpi_factor = res.borrow::<GraphicsBackend<B>>().dpi_factor();
        res.borrow_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::ChangeDpi(dpi_factor));
    }

    fn reload_renderables(&self, res: &Resources) {
        #[cfg(any(test, debug_assertions))]
        debug!("Reloading all renderables");
        #[cfg(any(test, debug_assertions))]
        let reload_mark = Instant::now();
        let mut backend = res.borrow_mut::<GraphicsBackend<B>>();
        backend
            .reload_assets(&mut res.borrow_components_mut::<Renderable>())
            .expect("Could not reload all renderable assets");
        #[cfg(any(test, debug_assertions))]
        debug!(
            "Completed reloading all renderables after {:?}",
            reload_mark.elapsed()
        );
    }

    #[cfg(any(test, debug_assertions))]
    pub fn average_world_draw_calls(&self) -> f32 {
        self.draw_calls.iter().map(|(wdc, _)| wdc).sum::<usize>() as f32 / DRAW_CALL_WINDOW as f32
    }

    #[cfg(any(test, debug_assertions))]
    pub fn average_ui_draw_calls(&self) -> f32 {
        self.draw_calls.iter().map(|(_, udc)| udc).sum::<usize>() as f32 / DRAW_CALL_WINDOW as f32
    }

    #[cfg(any(test, debug_assertions))]
    pub fn average_frame_time(&self) -> Duration {
        self.frame_times.iter().sum::<Duration>() / FRAME_TIME_WINDOW as u32
    }

    #[cfg(any(test, debug_assertions))]
    fn update_draw_calls(&mut self, world_draw_calls: usize, ui_draw_calls: usize) {
        self.draw_calls
            .push_front((world_draw_calls, ui_draw_calls));
        if self.draw_calls.len() > DRAW_CALL_WINDOW {
            self.draw_calls.truncate(DRAW_CALL_WINDOW);
        }
    }

    #[cfg(any(test, debug_assertions))]
    fn update_frame_time(&mut self, frame_time: Duration) {
        self.frame_times.push_front(frame_time);
        if self.frame_times.len() > FRAME_TIME_WINDOW {
            self.frame_times.truncate(FRAME_TIME_WINDOW);
        }
    }
}

impl<S, B> System for Renderer<S, B>
where
    S: SettingsTrait,
    B: BackendTrait,
{
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        #[cfg(any(test, debug_assertions))]
        let start_mark = Instant::now();

        #[cfg(any(test, debug_assertions))]
        let mut world_draw_calls: usize = 0;

        #[cfg(any(test, debug_assertions))]
        let mut ui_draw_calls: usize = 0;

        // The following is just a workaround for the DPI factor not being set properly by the
        // graphics_backend at initialisation.
        if !self.initialised {
            #[cfg(any(test, debug_assertions))]
            debug!("Initialising the renderer");
            self.set_dpi_factor(res);
            self.initialised = true;
        }

        // Reload all renderables.
        let events = res
            .borrow_mut::<EventQueue<WorldEvent>>()
            .receive(&self.receiver);
        if events
            .into_iter()
            .any(|e| e == WorldEvent::DeserializationComplete)
        {
            self.reload_renderables(res);
        }

        // Update the scene graphs.
        res.borrow_mut::<SceneGraph<Model>>()
            .update(&res.borrow_components::<Model>());
        res.borrow_mut::<SceneGraph<UiModel>>()
            .update(&res.borrow_components::<UiModel>());

        // Obtain a reference to the camera.
        let cameras = res.borrow_components::<Camera>();

        // Grab the necessary resources
        let settings = res.borrow::<S>();
        let entities = res.borrow::<Entities>();
        let world_graph = res.borrow::<SceneGraph<Model>>();
        let ui_graph = res.borrow::<SceneGraph<UiModel>>();
        let factory = res.borrow::<GraphicsBackend<B>>();
        let statuses = res.borrow_components::<Status>();
        let renderables = res.borrow_components::<Renderable>();

        // Create a new frame.
        let mut target = factory.create_frame();
        target.initialize(settings.clear_color(), 1.0);

        for (cam_idx, cam) in cameras.iter_enum() {
            // Skip any inactive cameras
            if statuses.get(cam_idx).map_or(true, |s| !s.enabled()) {
                continue;
            }

            // Obtain the model component of the camera
            let cam_entity = entities.get(cam_idx);
            let cam_model = world_graph.get(&cam_entity);
            let cam_matrix = cam.world_matrix() * cam_model.matrix();

            // Render the world scene.
            world_graph
                .iter()
                .filter(|&(entity, _)| {
                    statuses
                        .get(entity)
                        .map_or(false, |s| s.enabled() && s.visible())
                })
                .filter_map(|(entity, model)| {
                    renderables
                        .get(entity)
                        .map(|renderable| (model, renderable))
                })
                .for_each(|(model, renderable)| {
                    #[cfg(any(test, debug_assertions))]
                    {
                        world_draw_calls += 1;
                    }
                    target
                        .render(&(cam_matrix * model.matrix()), &factory, renderable)
                        .expect("Unable to render the world");
                });

            // Render the ui scene.
            ui_graph
                .iter()
                .filter(|&(entity, _)| {
                    statuses
                        .get(entity)
                        .map_or(false, |s| s.enabled() && s.visible())
                })
                .filter_map(|(entity, model)| {
                    renderables
                        .get(entity)
                        .map(|renderable| (model, renderable))
                })
                .for_each(|(model, renderable)| {
                    #[cfg(any(test, debug_assertions))]
                    {
                        ui_draw_calls += 1;
                    }
                    target
                        .render(&(cam.ui_matrix() * model.matrix()), &factory, renderable)
                        .expect("Unable to render the UI");
                });
        }

        // Finalize the frame and thus swap the display buffers.
        target.finalize().expect("Unable to finalize the frame");

        #[cfg(any(test, debug_assertions))]
        self.update_draw_calls(world_draw_calls, ui_draw_calls);

        #[cfg(any(test, debug_assertions))]
        self.update_frame_time(start_mark.elapsed());
    }
}
