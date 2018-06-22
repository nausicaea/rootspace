extern crate ecs;
extern crate engine;
extern crate failure;
extern crate glium;
extern crate log;

use ecs::event::EventManagerTrait;
use ecs::world::World;
use engine::components::model::Model;
use engine::context::Context;
use engine::event::Event;
use engine::file_manipulation::FileError;
use engine::wrappers::glium::{HeadlessDisplay, HeadlessEventsLoop};
use engine::orchestrator::Orchestrator;
use engine::systems::event_interface::EventInterface;
use engine::systems::event_monitor::EventMonitor;
use engine::systems::renderer::Renderer;
use engine::systems::SystemGroup;
use failure::Error;
use glium::Display;
use glium::glutin::EventsLoop;
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
        // Create and register the systems that depend on a graphics backend.
        if headless {
            let event_interface = EventInterface::new(HeadlessEventsLoop::default());
            let renderer: Renderer<Event, Context, HeadlessDisplay, Model> = Renderer::new(
                &event_interface.events_loop,
                "Title",
                &[1024, 768],
                true,
                4,
                [0.2, 0.3, 0.0, 1.0],
            ).unwrap();

            self.orchestrator.world.add_system(event_interface);
            self.orchestrator.world.add_system(renderer);

        } else {
            let event_interface = EventInterface::new(EventsLoop::new());
            let renderer: Renderer<Event, Context, Display, Model> = Renderer::new(
                &event_interface.events_loop,
                "Title",
                &[1024, 768],
                true,
                4,
                [0.2, 0.3, 0.0, 1.0],
            ).unwrap();

            self.orchestrator.world.add_system(event_interface);
            self.orchestrator.world.add_system(renderer);
        }

        let event_monitor = EventMonitor::default();
        self.orchestrator.world.add_system(event_monitor);

        self.orchestrator.world.context.dispatch_later(Event::Ready);
        self.orchestrator.run(iterations)?;

        Ok(())
    }
}
