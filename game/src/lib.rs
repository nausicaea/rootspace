extern crate ecs;
extern crate engine;
extern crate failure;
extern crate glium;
extern crate log;

use ecs::database::DatabaseTrait;
use ecs::event::EventManagerTrait;
use ecs::world::World;
use engine::components::camera::Camera;
use engine::context::Context;
use engine::event::Event;
use engine::file_manipulation::FileError;
use engine::orchestrator::Orchestrator;
use engine::systems::SystemGroup;
use engine::systems::event_coordinator::EventCoordinator;
use engine::systems::event_interface::{HeadlessEventInterface, GliumEventInterface};
use engine::systems::event_monitor::EventMonitor;
use engine::systems::renderer::{HeadlessRenderer, GliumRenderer};
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
        let camera = self.orchestrator.world.context.create_entity();
        self.orchestrator.world.context.add(camera, Camera::default()).unwrap();

        if headless {
            let event_interface = HeadlessEventInterface::default();
            let renderer = HeadlessRenderer::new(&event_interface.events_loop, "Title", [800, 600], true, 4).unwrap();

            self.orchestrator.world.add_system(event_interface);
            self.orchestrator.world.add_system(renderer);
        } else {
            let event_interface = GliumEventInterface::default();
            let renderer = GliumRenderer::new(&event_interface.events_loop, "Title", [800, 600], true, 4).unwrap();

            self.orchestrator.world.add_system(event_interface);
            self.orchestrator.world.add_system(renderer);
        }

        let event_monitor = EventMonitor::default();
        self.orchestrator.world.add_system(event_monitor);

        let event_coordinator = EventCoordinator::default();
        self.orchestrator.world.add_system(event_coordinator);

        self.orchestrator
            .world
            .context
            .dispatch_later(Event::startup());
        self.orchestrator.run(iterations)?;

        Ok(())
    }
}
