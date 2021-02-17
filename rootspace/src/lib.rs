mod assets;
mod debug_commands;

use crate::debug_commands::FileSystemCommand;
use anyhow::Result;
use ecs::{LoopStage, Reg};
use engine::{
    graphics::BackendTrait, orchestrator::Orchestrator,
    systems::DebugShell,
};
use file_manipulation::DirPathBuf;
use std::{convert::TryFrom, path::Path};

type ResourceRegistry = Reg![];
type FixedUpdateSystemRegistry = Reg![];
type UpdateSystemRegistry = Reg![];
type RenderSystemRegistry = Reg![];

pub struct Rootspace<B>
where
    B: BackendTrait,
{
    orch: Orchestrator<B, ResourceRegistry, FixedUpdateSystemRegistry, UpdateSystemRegistry, RenderSystemRegistry>,
}

impl<B> Rootspace<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(asset_database: P) -> Result<Self> {
        let asset_database = DirPathBuf::try_from(asset_database.as_ref())?;
        let mut orch = Orchestrator::new(&asset_database)?;

        Ok(Rootspace { orch })
    }

    pub fn run(&mut self) {
        self.orch.run()
    }
}
