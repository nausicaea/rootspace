extern crate log;
extern crate failure;
extern crate winit;
extern crate ecs;
extern crate engine;

use std::path::Path;
use std::time::Duration;
use failure::Error;
use winit::EventsLoop;
use ecs::world::World;
use ecs::event::EventManagerTrait;
use engine::orchestrator::Orchestrator;
use engine::file_manipulation::FileError;
use engine::event::Event;
use engine::context::Context;
use engine::systems::SystemGroup;
use engine::systems::event_monitor::EventMonitor;
use engine::systems::event_interface::EventInterface;
use engine::systems::open_gl_renderer::OpenGlRenderer;

pub struct Game {
    orchestrator: Orchestrator<World<Event, Context, SystemGroup>>,
}

impl Game {
    pub fn new(resource_path: &Path, delta_time: Duration, max_frame_time: Duration) -> Result<Self, FileError> {
        let o = Orchestrator::new(resource_path, delta_time, max_frame_time)?;

        Ok(Game {
            orchestrator: o,
        })
    }
    pub fn run(&mut self, iterations: Option<usize>) -> Result<(), Error> {
        self.orchestrator.world.add_system(EventMonitor::default());
        self.orchestrator.world.add_system(EventInterface::new(EventsLoop::new()));
        self.orchestrator.world.add_system(OpenGlRenderer::new());
        self.orchestrator.world.context.dispatch_later(Event::Ready);

        self.orchestrator.run(iterations)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    /// This test is effectively an integration test, because it causes the entire game to run for
    /// one iteration.
    #[test]
    #[ignore]
    fn create_and_run_game() {
        let resource_path = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let iterations = Some(1);
        let mut g = Game::new(&resource_path, delta_time, max_frame_time).unwrap();
        g.run(iterations).unwrap();
    }
}
