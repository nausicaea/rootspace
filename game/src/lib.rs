extern crate ecs;
extern crate engine;
extern crate failure;
extern crate glium;
extern crate log;

use ecs::event::EventManagerTrait;
use ecs::world::World;
use engine::context::Context;
use engine::event::Event;
use engine::file_manipulation::FileError;
use engine::orchestrator::Orchestrator;
use engine::systems::event_monitor::EventMonitor;
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

    pub fn run(&mut self, _headless: bool, iterations: Option<usize>) -> Result<(), Error> {
        let event_monitor = EventMonitor::default();
        self.orchestrator.world.add_system(event_monitor);

        self.orchestrator
            .world
            .context
            .dispatch_later(Event::ready());
        self.orchestrator.run(iterations)?;

        Ok(())
    }
}
