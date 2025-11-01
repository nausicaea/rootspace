use std::sync::Arc;
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};

use crate::{Orchestrator, OrchestratorDeps, RpcDeps};
use assam::AssetDatabaseDeps;
use ecs::Reg;
use griffon::{GraphicsDeps, Settings};

#[derive(Debug)]
pub struct App {
    name: String,
    force_init: bool,
    graphics_settings: Settings,
}

impl App {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        App {
            name: name.as_ref().to_owned(),
            force_init: false,
            graphics_settings: Settings::default(),
        }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        let App {
            name,
            force_init,
            graphics_settings,
        } = self;
        let rt = Arc::new(RuntimeBuilder::new_multi_thread().enable_all().build()?);
        let event_loop = EventLoop::new()?;

        let deps = Deps {
            rt: rt.clone(),
            event_loop: &event_loop,
            name: &name,
            force_init,
            graphics_settings: &graphics_settings,
        };
        let state =
            rt.block_on(
                async move { Orchestrator::with_dependencies::<Reg![], Reg![], Reg![], Reg![], _>(&deps).await },
            )?;

        // Creates and returns a closure that is run by
        // [`EventLoop::run`](winit::event_loop::EventLoop::run) every time `winit` received an event
        // from the operating system. Internally, the closure instructs the asynchronous runtime to
        // block on [`Orchestrator::run`](Orchestrator::run), which does
        // the actual work.
        event_loop.run(state.start())?;
        Ok(())
    }
}

#[derive(Debug)]
struct Deps<'a> {
    rt: Arc<Runtime>,
    event_loop: &'a EventLoop<()>,
    name: &'a str,
    force_init: bool,
    graphics_settings: &'a Settings,
}

impl<'a> GraphicsDeps for Deps<'a> {
    type CustomEvent = ();

    fn event_loop(&self) -> &EventLoopWindowTarget<Self::CustomEvent> {
        self.event_loop
    }

    fn settings(&self) -> &Settings {
        self.graphics_settings
    }
}

impl<'a> AssetDatabaseDeps for Deps<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn force_init(&self) -> bool {
        self.force_init
    }

    fn within_repo(&self) -> bool {
        cfg!(debug_assertions)
    }
}

impl<'a> OrchestratorDeps for Deps<'a> {
    fn runtime(&self) -> Arc<Runtime> {
        self.rt.clone()
    }

    fn main_scene(&self) -> Option<&str> {
        None
    }
}

impl<'a> RpcDeps for Deps<'a> {}
