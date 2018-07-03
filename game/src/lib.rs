extern crate ecs;
extern crate engine;
extern crate failure;
extern crate glium;
extern crate log;

use ecs::{DatabaseTrait, EventManagerTrait, World};
use engine::components::camera::Camera;
use engine::components::model::Model;
use engine::context::{Context, SceneGraphTrait};
use engine::event::Event;
use engine::file_manipulation::FileError;
use engine::graphics::glium::GliumRenderData;
use engine::graphics::headless::HeadlessRenderData;
use engine::graphics::RenderDataTrait;
use engine::orchestrator::Orchestrator;
use engine::systems::event_coordinator::EventCoordinator;
use engine::systems::event_interface::{GliumEventInterface, HeadlessEventInterface};
use engine::systems::event_monitor::EventMonitor;
use engine::systems::renderer::{GliumRenderer, HeadlessRenderer};
use engine::systems::SystemGroup;
use failure::Error;
use std::path::Path;
use std::time::Duration;

pub struct Game {
    orchestrator: Orchestrator<World<Event, Context, SystemGroup>>,
}

impl Game {
    pub fn new(
        resource_path: &Path,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, FileError> {
        let o = Orchestrator::new(resource_path, delta_time, max_frame_time)?;

        Ok(Game { orchestrator: o })
    }

    pub fn run(&mut self, headless: bool, iterations: Option<usize>) -> Result<(), Error> {
        let camera = self.context_mut().create_entity();
        self.context_mut().add(camera, Camera::default()).unwrap();

        let test_entity = self.context_mut().create_entity();
        self.context_mut().insert_node(test_entity);
        self.context_mut()
            .add(test_entity, Model::default())
            .unwrap();

        if headless {
            let event_interface = HeadlessEventInterface::default();
            let renderer =
                HeadlessRenderer::new(&event_interface.events_loop, "Title", [800, 600], true, 4)
                    .unwrap();

            self.context_mut()
                .add(
                    test_entity,
                    HeadlessRenderData::triangle(&renderer.backend).unwrap(),
                )
                .unwrap();

            self.world_mut().add_system(event_interface);
            self.world_mut().add_system(renderer);
        } else {
            let event_interface = GliumEventInterface::default();
            let renderer =
                GliumRenderer::new(&event_interface.events_loop, "Title", [800, 600], true, 4)
                    .unwrap();

            self.context_mut()
                .add(
                    test_entity,
                    GliumRenderData::triangle(&renderer.backend).unwrap(),
                )
                .unwrap();

            self.world_mut().add_system(event_interface);
            self.world_mut().add_system(renderer);
        }

        let event_monitor = EventMonitor::default();
        self.world_mut().add_system(event_monitor);

        let event_coordinator = EventCoordinator::default();
        self.world_mut().add_system(event_coordinator);

        self.orchestrator
            .world
            .context
            .dispatch_later(Event::startup());
        self.orchestrator.run(iterations)?;

        Ok(())
    }

    fn world_mut(&mut self) -> &mut World<Event, Context, SystemGroup> {
        &mut self.orchestrator.world
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.orchestrator.world.context
    }
}
