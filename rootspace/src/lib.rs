mod assets;
mod debug_commands;

use crate::debug_commands::FileSystemCommand;
use anyhow::{Context, Result};
use ecs::{LoopStage, Reg};
use engine::{
    graphics::BackendTrait, orchestrator::Orchestrator, resources::GraphicsBackend,
    systems::DebugShell,
};
use file_manipulation::FilePathBuf;
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
    main_scene: FilePathBuf,
}

impl<B> Rootspace<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(resource_path: P, command: Option<&str>) -> Result<Self> {
        let mut orch = Orchestrator::new(resource_path, command)?;

        let main_scene = orch
            .get_mut::<GraphicsBackend<B>>()
            .find_asset("scenes/rootspace.json")
            .context("Could not find the main scene asset")?;

        Ok(Rootspace { orch, main_scene })
    }

    #[cfg(any(test, debug_assertions))]
    pub fn set_main_scene<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.main_scene =
            FilePathBuf::try_from(path.as_ref()).context("Could not find the main scene asset")?;

        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        // FIXME: Replace with a proper deserialization of Orchestrator
        // self.orch.load(&self.main_scene);

        // Add an additional command
        let debug_shell = self
            .orch
            .world
            .find_system_mut::<DebugShell>(LoopStage::Update)
            .context("Could not find the system DebugShell")?;
        debug_shell.add_command(FileSystemCommand);

        Ok(())
    }

    pub fn run(&mut self) {
        self.orch.run()
    }
}
