use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

use ecs::{
    event_queue::receiver_id::ReceiverId, world::event::WorldEvent, EventQueue, Index, Resources, SerializationName,
    Storage, System, WithResources,
};
use log::debug;
use rose_tree::Hierarchy;
use serde::{Deserialize, Serialize};

use crate::{
    components::{Camera, Model, Renderable, Status, UiModel},
    graphics::{BackendTrait, FrameTrait},
    resources::{GraphicsBackend, Settings, Statistics},
};

#[derive(Serialize, Deserialize)]
pub struct Renderer<B> {
    receiver: ReceiverId<WorldEvent>,
    #[serde(skip)]
    _b: PhantomData<B>,
}

impl<B> std::fmt::Debug for Renderer<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Renderer {{ receiver: {:?} }}", self.receiver,)
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
            _b: PhantomData::default(),
        }
    }
}

impl<B> Renderer<B>
where
    B: BackendTrait,
{
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

        // Reload all renderables.
        let events = res.borrow_mut::<EventQueue<WorldEvent>>().receive(&self.receiver);
        if events.into_iter().any(|e| e == WorldEvent::DeserializationComplete) {
            self.reload_renderables(res);
        }

        // Grab the necessary resources
        let factory = res.borrow_mut::<GraphicsBackend<B>>();
        let settings = res.borrow::<Settings>();
        let hierarchy = res.borrow::<Hierarchy<Index>>();
        let cameras = res.borrow_components::<Camera>();
        let models = res.borrow_components::<Model>();
        let ui_models = res.borrow_components::<UiModel>();
        let statuses = res.borrow_components::<Status>();
        let renderables = res.borrow_components::<Renderable>();

        // Create a new frame.
        let mut target = factory.create_frame();
        target.initialize(settings.clear_color, 1.0);

        for (cam_idx, cam) in cameras.indexed_iter() {
            // Skip any inactive cameras
            let global_cam_status = hierarchy
                .ancestors(&cam_idx)
                .filter_map(|aidx| statuses.get(aidx))
                .product::<Status>();

            if !global_cam_status.enabled() {
                continue;
            }

            // Obtain the model component of the camera
            let global_cam_model = hierarchy
                .ancestors(&cam_idx)
                .filter_map(|aidx| models.get(aidx))
                .product::<Model>();
            let cam_world_matrix = cam.as_world_matrix() * global_cam_model.to_matrix();
            let cam_ui_matrix = cam.as_ui_matrix();

            // Render the world scene.
            for (idx, _) in models.indexed_iter() {
                let renderable = if let Some(rdrbl) = renderables.get(idx) {
                    rdrbl
                } else {
                    continue;
                };

                let global_status = hierarchy
                    .ancestors(&idx)
                    .filter_map(|aidx| statuses.get(aidx))
                    .product::<Status>();

                if !(global_status.enabled() && global_status.visible()) {
                    continue;
                }

                let global_model = hierarchy
                    .ancestors(&idx)
                    .filter_map(|aidx| models.get(aidx))
                    .product::<Model>();

                world_draw_calls += 1;
                target
                    .render(&(cam_world_matrix * global_model.to_matrix()), &factory, renderable)
                    .unwrap_or_else(|e| panic!("Unable to render the world entity {}: {}", idx, e));
            }

            // Render the ui scene.
            for (idx, _) in ui_models.indexed_iter() {
                let renderable = if let Some(rdrbl) = renderables.get(idx) {
                    rdrbl
                } else {
                    continue;
                };

                let global_status = hierarchy
                    .ancestors(&idx)
                    .filter_map(|aidx| statuses.get(aidx))
                    .product::<Status>();

                if !(global_status.enabled() && global_status.visible()) {
                    continue;
                }

                let global_ui_model = hierarchy
                    .ancestors(&idx)
                    .filter_map(|aidx| ui_models.get(aidx))
                    .product::<UiModel>();

                ui_draw_calls += 1;
                target
                    .render(&(cam_ui_matrix * global_ui_model.matrix()), &factory, renderable)
                    .unwrap_or_else(|e| panic!("Unable to render the UI entity {}: {}", idx, e));
            }
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
