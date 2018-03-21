extern crate log;
extern crate failure;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate bitflags;
extern crate ecs;
extern crate engine;

mod event;
mod systems;
mod auxiliary;

use std::path::Path;
use std::time::Duration;

use ecs::world::World;
use ecs::database::Database;
use engine::error::Error as RootEngineError;
use engine::orchestrator::Orchestrator;
use engine::file_manipulation::FileError;
use self::event::Event;
use self::systems::SystemGroup;
use self::auxiliary::Auxiliary;

pub struct Game {
    orchestrator: Orchestrator<World<Auxiliary, Database, Event, SystemGroup>>,
}

impl Game {
    pub fn new(resource_path: &Path, delta_time: Duration, max_frame_time: Duration) -> Result<Self, FileError> {
        let o = Orchestrator::new(resource_path, delta_time, max_frame_time)?;

        Ok(Game {
            orchestrator: o,
        })
    }
    pub fn run(&mut self, iterations: Option<usize>) -> Result<(), Error> {
        self.orchestrator.run(iterations)?;

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    EngineError(#[cause] RootEngineError),
}

impl From<RootEngineError> for Error {
    fn from(value: RootEngineError) -> Self {
        Error::EngineError(value)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    /// This test is effectively an integration test, because it causes the entire game to run for
    /// one iteration.
    #[test]
    fn create_and_run_game() {
        let resource_path = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let iterations = Some(1);
        let mut g = Game::new(&resource_path, delta_time, max_frame_time).unwrap();
        g.run(iterations).unwrap();
    }
}
