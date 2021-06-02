mod assets;
mod debug_commands;

use ecs::Reg;
use engine::orchestrator::Orchestrator;

pub type EmptyGame<B> = Orchestrator<B, Reg![], Reg![], Reg![], Reg![]>;
