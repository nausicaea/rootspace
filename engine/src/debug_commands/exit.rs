use ecs::{EventQueue, Resources};

use crate::{CommandTrait, EngineEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExitCommand;

impl CommandTrait for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Shuts down the engine (can also be done with Ctrl-C. Tap Ctrl-C twice to force a shutdown)"
    }

    fn run(&self, res: &Resources, _: &[String]) -> anyhow::Result<()> {
        res.borrow_mut::<EventQueue<EngineEvent>>().send(EngineEvent::Shutdown);
        Ok(())
    }
}
