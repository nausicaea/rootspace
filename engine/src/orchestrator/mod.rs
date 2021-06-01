use std::{
    cmp,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use log::debug;
use serde::{Deserialize, Serialize};

use ecs::{LoopControl, ResourceRegistry, SystemRegistry, World};

use crate::{
    components::{Model, UiModel},
    graphics::BackendTrait,
    resources::{GraphicsBackend, SceneGraph},
};

use self::type_registry::{RenderSystemTypes, ResourceTypes, UpdateSystemTypes};
use file_manipulation::{DirPathBuf, FilePathBuf, NewOrExFilePathBuf};
use std::{convert::TryFrom, fs::File, path::Path};

pub mod type_registry;

type OrchestratorWorld<B, RR, FUSR, USR, RSR> = World<ResourceTypes<B, RR>, FUSR, UpdateSystemTypes<B, USR>, RenderSystemTypes<B, RSR>>;

pub struct Orchestrator<B: BackendTrait, RR, FUSR, USR, RSR> {
    world: OrchestratorWorld<B, RR, FUSR, USR, RSR>,
    delta_time: Duration,
    max_frame_time: Duration,
}

impl<B, RR, FUSR, USR, RSR> Orchestrator<B, RR, FUSR, USR, RSR>
where
    B: BackendTrait,
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    pub fn new<P: AsRef<Path>>(asset_database: &P) -> Result<Self> {
        // Create the world
        let mut world = World::try_default()?;

        // Retrieve the settings and create the backend as a resource
        // FIXME: Can we make it so that the GraphicsBackend is also automatically initialized?
        let asset_database = DirPathBuf::try_from(asset_database.as_ref())?;
        let backend = GraphicsBackend::<B>::new(asset_database).context("Failed to initialise the graphics backend")?;
        world.insert(backend);

        Ok(Orchestrator {
            world,
            delta_time: Duration::from_millis(50),
            max_frame_time: Duration::from_millis(250),
        })
    }

    pub fn load<P: AsRef<Path>>(path: &P) -> Result<Self> {
        // Create the deserializer
        let file_path = FilePathBuf::try_from(path.as_ref())?;
        let mut file = File::open(file_path)?;
        let mut deserializer = serde_json::Deserializer::from_reader(&mut file);

        // Deserialize the entire world
        let world = World::deserialize(&mut deserializer)?;

        // // Add an additional command to the debug shell
        // // TODO: Create a registry of debug commands and serialize those as well
        // orch.world
        //     .get_system_mut::<DebugShell>(LoopStage::Update)
        //     .add_command(FileSystemCommand);

        // Update the scene graphs for the first time
        // FIXME: Do we really need to update the scene graphs?
        world
            .borrow_mut::<SceneGraph<Model>>()
            .update(&world.borrow_components::<Model>());
        world
            .borrow_mut::<SceneGraph<UiModel>>()
            .update(&world.borrow_components::<UiModel>());

        Ok(Orchestrator {
            world,
            delta_time: Duration::from_millis(50),
            max_frame_time: Duration::from_millis(250),
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: &P) -> Result<()> {
        debug!("Saving the game state to: {}", path.as_ref().display());

        // Create the deserializer
        let file_path = NewOrExFilePathBuf::try_from(path.as_ref())?;
        let mut file = File::create(file_path)?;
        let mut serializer = serde_json::Serializer::pretty(&mut file);

        // Serialize the World
        self.world.serialize(&mut serializer)?;

        Ok(())
    }

    pub fn run(&mut self) {
        // Initialize the timers
        let mut loop_time = Instant::now();
        let mut accumulator = Duration::default();
        let mut dynamic_game_time = Duration::default();
        let mut fixed_game_time = Duration::default();

        // Run the main game loop
        loop {
            // Assess the duration of the last frame
            let frame_time = cmp::min(loop_time.elapsed(), self.max_frame_time);
            loop_time = Instant::now();
            accumulator += frame_time;
            dynamic_game_time += frame_time;

            // Call fixed update functions until the accumulated time buffer is empty
            while accumulator >= self.delta_time {
                self.world.fixed_update(&fixed_game_time, &self.delta_time);
                accumulator -= self.delta_time;
                fixed_game_time += self.delta_time;
            }

            // Call the dynamic update and render functions
            self.world.update(&dynamic_game_time, &frame_time);
            self.world.render(&dynamic_game_time, &frame_time);

            // Perform maintenance tasks (both Orchestrator and World listen for events themselves)
            if self.maintain() == LoopControl::Abort {
                break;
            }
        }
    }

    fn maintain(&mut self) -> LoopControl {
        self.world.maintain()
    }
}

impl<B: BackendTrait, RR, FUSR, USR, RSR> std::fmt::Debug for Orchestrator<B, RR, FUSR, USR, RSR> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Orchestrator {{ world: {:?}, delta_time: {:?}, max_frame_time: {:?} }}",
            self.world, self.delta_time, self.max_frame_time,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{GliumBackend, HeadlessBackend, Orchestrator};
    use ecs::Reg;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    type TestGame<B> = Orchestrator<B, Reg![], Reg![], Reg![], Reg![]>;

    #[test]
    fn game_creation_headless() {
        let asset_database = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));

        let r: Result<TestGame<HeadlessBackend>> = TestGame::new(&asset_database);
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn game_creation_glium() {
        let asset_database = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));

        let r: Result<TestGame<GliumBackend>> = TestGame::new(&asset_database);
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    fn game_loading_and_saving_headless_headless() {
        // TODO: Extend the test to evaluate whether the loaded game equals the newly created game

        let asset_database = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let tf = NamedTempFile::new().unwrap();

        let first: TestGame<HeadlessBackend> = TestGame::new(&asset_database).unwrap();
        let r = first.save(&tf.path());
        assert!(r.is_ok(), "{}", r.unwrap_err());

        let r: Result<TestGame<HeadlessBackend>> = TestGame::load(&tf.path());
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn game_loading_and_saving_glium_glium() {
        // TODO: Extend the test to evaluate whether the loaded game equals the newly created game

        let asset_database = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let tf = NamedTempFile::new().unwrap();

        let first: TestGame<GliumBackend> = TestGame::new(&asset_database).unwrap();
        let r = first.save(&tf.path());
        assert!(r.is_ok(), "{}", r.unwrap_err());

        let r: Result<TestGame<GliumBackend>> = TestGame::load(&tf.path());
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn game_loading_and_saving_headless_glium() {
        // TODO: Extend the test to evaluate whether the loaded game equals the newly created game

        let asset_database = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let tf = NamedTempFile::new().unwrap();

        let first: TestGame<HeadlessBackend> = TestGame::new(&asset_database).unwrap();
        let r = first.save(&tf.path());
        assert!(r.is_ok(), "{}", r.unwrap_err());

        let r: Result<TestGame<GliumBackend>> = TestGame::load(&tf.path());
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn game_loading_and_saving_glium_headless() {
        // TODO: Extend the test to evaluate whether the loaded game equals the newly created game

        let asset_database = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let tf = NamedTempFile::new().unwrap();

        let first: TestGame<GliumBackend> = TestGame::new(&asset_database).unwrap();
        let r = first.save(&tf.path());
        assert!(r.is_ok(), "{}", r.unwrap_err());

        let r: Result<TestGame<HeadlessBackend>> = TestGame::load(&tf.path());
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }
}
