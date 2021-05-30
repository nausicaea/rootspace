mod assets;
mod debug_commands;

use ecs::{LoopStage, Reg};
use engine::{graphics::BackendTrait, orchestrator::Orchestrator, systems::DebugShell};
use file_manipulation::DirPathBuf;
use std::{convert::TryFrom, path::Path};

pub type EmptyGame<B> = Orchestrator<B, Reg![], Reg![], Reg![], Reg![]>;
