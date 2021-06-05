use std::{
    cmp,
    time::{Duration, Instant},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use ecs::{LoopControl, Reg, ResourceRegistry, SystemRegistry, World};

use crate::{
    graphics::BackendTrait,
};

use self::type_registry::{RenderSystemTypes, ResourceTypes, UpdateSystemTypes};
use crate::resources::{AssetDatabase, Statistics};
use file_manipulation::{FilePathBuf, NewOrExFilePathBuf};
use std::{convert::TryFrom, fs::File, path::Path, thread::sleep};
use try_default::TryDefault;

pub mod type_registry;

const DELTA_TIME: u64 = 50;
const MIN_FRAME_DURATION: u64 = 15625;
const MAX_FRAME_DURATION: u64 = 250;

pub type EmptyGame<B> = Orchestrator<B, Reg![], Reg![], Reg![], Reg![]>;

type OrchestratorWorld<B, RR, FUSR, USR, RSR> =
    World<ResourceTypes<B, RR>, FUSR, UpdateSystemTypes<B, USR>, RenderSystemTypes<B, RSR>>;

pub struct Orchestrator<B: BackendTrait, RR, FUSR, USR, RSR> {
    world: OrchestratorWorld<B, RR, FUSR, USR, RSR>,
    delta_time: Duration,
    min_frame_duration: Duration,
    max_frame_duration: Duration,
}

impl<B, RR, FUSR, USR, RSR> Orchestrator<B, RR, FUSR, USR, RSR>
where
    B: BackendTrait,
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    pub fn new<S: AsRef<str>>(name: S) -> Result<Self> {
        let mut world = World::try_default()?;
        world.get_mut::<AssetDatabase>().initialize(name.as_ref())?;

        Ok(Orchestrator {
            world,
            delta_time: Duration::from_millis(DELTA_TIME),
            min_frame_duration: Duration::from_micros(MIN_FRAME_DURATION),
            max_frame_duration: Duration::from_millis(MAX_FRAME_DURATION),
        })
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
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

        Ok(Orchestrator {
            world,
            delta_time: Duration::from_millis(DELTA_TIME),
            min_frame_duration: Duration::from_micros(MIN_FRAME_DURATION),
            max_frame_duration: Duration::from_millis(MAX_FRAME_DURATION),
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Create the deserializer
        let state_path = NewOrExFilePathBuf::try_from(path.as_ref())?;
        let mut file = File::create(state_path)?;
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
            let frame_time = cmp::min(loop_time.elapsed(), self.max_frame_duration);
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

            // Artificially prolong the frame if it was too short
            if frame_time < self.min_frame_duration {
                sleep(self.min_frame_duration - frame_time);
            }

            // Update the frame time statistics
            self.world.borrow_mut::<Statistics>().update_loop_times(frame_time);
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
            self.world, self.delta_time, self.max_frame_duration,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{GliumBackend, HeadlessBackend, Orchestrator};
    use ecs::Reg;

    type TestGame<B> = Orchestrator<B, Reg![], Reg![], Reg![], Reg![]>;

    #[test]
    fn game_creation_headless() {
        let r: Result<TestGame<HeadlessBackend>> = TestGame::new("test");
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn game_creation_glium() {
        let r: Result<TestGame<GliumBackend>> = TestGame::new("test");
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    fn game_loading_and_saving_headless_headless() {
        // TODO: Extend the test to evaluate whether the loaded game equals the newly created game

        let first: TestGame<HeadlessBackend> = TestGame::new("test").unwrap();
        let r = first.save("test.json");
        assert!(r.is_ok(), "{}", r.unwrap_err());

        let r: Result<TestGame<HeadlessBackend>> = TestGame::load("test.json");
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn game_loading_and_saving_glium_glium() {
        // TODO: Extend the test to evaluate whether the loaded game equals the newly created game

        let first: TestGame<GliumBackend> = TestGame::new("test").unwrap();
        let r = first.save("test.json");
        assert!(r.is_ok(), "{}", r.unwrap_err());

        let r: Result<TestGame<GliumBackend>> = TestGame::load("test.json");
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn game_loading_and_saving_headless_glium() {
        // TODO: Extend the test to evaluate whether the loaded game equals the newly created game

        let first: TestGame<HeadlessBackend> = TestGame::new("test").unwrap();
        let r = first.save("test.json");
        assert!(r.is_ok(), "{}", r.unwrap_err());

        let r: Result<TestGame<GliumBackend>> = TestGame::load("test.json");
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn game_loading_and_saving_glium_headless() {
        // TODO: Extend the test to evaluate whether the loaded game equals the newly created game

        let first: TestGame<GliumBackend> = TestGame::new("test").unwrap();
        let r = first.save("test.json");
        assert!(r.is_ok(), "{}", r.unwrap_err());

        let r: Result<TestGame<HeadlessBackend>> = TestGame::load("test.json");
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }
}
