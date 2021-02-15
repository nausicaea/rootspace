mod assets;
mod debug_commands;
mod settings;

use crate::debug_commands::FileSystemCommand;
use anyhow::{Context, Result};
use ecs::{LoopStage, Reg};
use engine::{
    graphics::BackendTrait, orchestrator::Orchestrator, resources::GraphicsBackend,
    systems::DebugShell,
};
use file_manipulation::{FilePathBuf, DirPathBuf};
use std::{convert::TryFrom, path::Path};
use crate::settings::Settings;

type ResourceRegistry = Reg![];
type FixedUpdateSystemRegistry = Reg![];
type UpdateSystemRegistry = Reg![];
type RenderSystemRegistry = Reg![];

pub struct Rootspace<B>
where
    B: BackendTrait,
{
    orch: Orchestrator<Settings, B, ResourceRegistry, FixedUpdateSystemRegistry, UpdateSystemRegistry, RenderSystemRegistry>,
}

impl<B> Rootspace<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(resource_path: P, command: Option<&str>) -> Result<Self> {
        let resource_path = DirPathBuf::try_from(resource_path.as_ref())?;
        let settings = Settings::builder(resource_path).build();
        let mut orch = Orchestrator::new(settings, command)?;

        // Add an additional command to the debug shell
        orch.world
            .get_system_mut::<DebugShell<Settings>>(LoopStage::Update)
            .add_command(FileSystemCommand);

        Ok(Rootspace { orch })
    }

    pub fn load(&mut self) -> Result<()> {
        // FIXME: Replace with a proper deserialization of Orchestrator
        // self.orch.load(&self.main_scene);

        Ok(())
    }

    pub fn run(&mut self) {
        self.orch.run()
    }
}
