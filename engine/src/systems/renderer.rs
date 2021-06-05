use serde::{Deserialize, Serialize};
use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

use log::debug;

use ecs::{
    world::event::WorldEvent, Entities, EventQueue, ReceiverId, Resources, SerializationName, Storage, System,
    WithResources,
};

use crate::{
    components::{Camera, Model, Renderable, Status, UiModel},
    event::EngineEvent,
    graphics::{BackendTrait, FrameTrait},
    resources::{GraphicsBackend, SceneGraph, Settings, Statistics},
};

#[derive(Serialize, Deserialize)]
pub struct Renderer<B> {
    receiver: ReceiverId<WorldEvent>,
    #[serde(skip)]
    initialised: bool,
    #[serde(skip)]
    _b: PhantomData<B>,
}

impl<B> std::fmt::Debug for Renderer<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Renderer {{ receiver: {:?}, initialised: {:?} }}",
            self.receiver, self.initialised,
        )
    }
}

impl<B> WithResources for Renderer<B>
where
    B: BackendTrait,
{
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<WorldEvent>>().subscribe::<Self>();

        Renderer {
            receiver,
            initialised: false,
            _b: PhantomData::default(),
        }
    }
}

impl<B> Renderer<B>
where
    B: BackendTrait,
{
    fn set_dpi_factor(&self, res: &Resources) {
        let dpi_factor = res.borrow::<GraphicsBackend<B>>().dpi_factor();
        res.borrow_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::ChangeDpi(dpi_factor));
    }

    fn reload_renderables(&self, res: &Resources) {
        debug!("Reloading all renderables");

        let reload_mark = Instant::now();
        let mut backend = res.borrow_mut::<GraphicsBackend<B>>();
        backend
            .reload_assets(&mut res.borrow_components_mut::<Renderable>())
            .expect("Could not reload all renderable assets");

        debug!("Completed reloading all renderables after {:?}", reload_mark.elapsed());
    }
}

impl<B> System for Renderer<B>
where
    B: BackendTrait,
{
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let start_mark = Instant::now();

        let mut world_draw_calls: usize = 0;

        let mut ui_draw_calls: usize = 0;

        // The following is just a workaround for the DPI factor not being set properly by the
        // graphics_backend at initialisation.
        if !self.initialised {
            debug!("Initialising the renderer");
            self.set_dpi_factor(res);
            self.initialised = true;
        }

        // Reload all renderables.
        let events = res.borrow_mut::<EventQueue<WorldEvent>>().receive(&self.receiver);
        if events.into_iter().any(|e| e == WorldEvent::DeserializationComplete) {
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
        let factory = res.borrow_mut::<GraphicsBackend<B>>();
        let settings = res.borrow::<Settings>();
        let entities = res.borrow::<Entities>();
        let world_graph = res.borrow::<SceneGraph<Model>>();
        let ui_graph = res.borrow::<SceneGraph<UiModel>>();
        let statuses = res.borrow_components::<Status>();
        let renderables = res.borrow_components::<Renderable>();

        // Create a new frame.
        let mut target = factory.create_frame();
        target.initialize(settings.clear_color, 1.0);

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
                .filter(|&(entity, _)| statuses.get(entity).map_or(false, |s| s.enabled() && s.visible()))
                .filter_map(|(entity, model)| renderables.get(entity).map(|renderable| (model, renderable)))
                .for_each(|(model, renderable)| {
                    world_draw_calls += 1;
                    target
                        .render(&(cam_matrix * model.matrix()), &factory, renderable)
                        .expect("Unable to render the world");
                });

            // Render the ui scene.
            ui_graph
                .iter()
                .filter(|&(entity, _)| statuses.get(entity).map_or(false, |s| s.enabled() && s.visible()))
                .filter_map(|(entity, model)| renderables.get(entity).map(|renderable| (model, renderable)))
                .for_each(|(model, renderable)| {
                    ui_draw_calls += 1;
                    target
                        .render(&(cam.ui_matrix() * model.matrix()), &factory, renderable)
                        .expect("Unable to render the UI");
                });
        }

        // Finalize the frame and thus swap the display buffers.
        target.finalize().expect("Unable to finalize the frame");

        let mut stats = res.borrow_mut::<Statistics>();
        stats.update_draw_calls(world_draw_calls, ui_draw_calls);
        stats.update_frame_time(start_mark.elapsed());
    }
}

impl<B> SerializationName for Renderer<B> {
    fn name() -> String {
        String::from("Renderer")
    }
}
