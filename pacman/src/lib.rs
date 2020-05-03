use anyhow::Result;
use ecs::Reg;
use engine::{
    orchestrator::Orchestrator,
    graphics::BackendTrait,
};
use std::path::Path;
use std::time::Duration;

type ResourceRegistry = Reg![
];

pub struct Pacman<B>
where
    B: BackendTrait,
{
    orchestrator: Orchestrator<B, ResourceRegistry>,
}

impl<B> Pacman<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self> {
        Ok(Pacman {
            orchestrator: Orchestrator::new(resource_path, delta_time, max_frame_time)?,
        })
    }

    pub fn load(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn run(&mut self, iterations: Option<usize>) {
        self.orchestrator.run(iterations)
    }
}
