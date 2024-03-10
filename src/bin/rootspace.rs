use clap::Parser;
use std::ops::Deref;
use std::sync::Arc;
use tokio::runtime::Runtime;

use rootspace::engine::orchestrator::{Orchestrator, OrchestratorDeps};
use rootspace::engine::resources::asset_database::AssetDatabaseDeps;
use rootspace::engine::resources::graphics::settings::Settings;
use rootspace::engine::resources::graphics::GraphicsDeps;
use rootspace::engine::resources::rpc_settings::RpcDeps;
use rootspace::Reg;
use winit::event_loop::EventLoop;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, help = "Select the game to run", default_value = "rootspace")]
    game: String,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let event_loop = EventLoop::new()?;
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    let rt = Arc::new(rt);
    let deps = Dependencies::new(rt.clone(), &event_loop, &args.game);
    let state =
        rt.block_on(async move { Orchestrator::with_dependencies::<Reg![], Reg![], Reg![], _>(&deps).await })?;
    event_loop.run(state.start())?;
    Ok(())
}

struct Dependencies<'a, T: 'static> {
    rt: Arc<Runtime>,
    event_loop: &'a EventLoop<T>,
    name: &'a str,
    force_init: bool,
    graphics_settings: Settings,
}

impl<'a, T> Dependencies<'a, T> {
    fn new(rt: Arc<Runtime>, event_loop: &'a EventLoop<T>, name: &'a str) -> Dependencies<'a, T> {
        Dependencies {
            rt,
            event_loop,
            name,
            force_init: false,
            graphics_settings: Settings::default(),
        }
    }
}

impl<'a, T> GraphicsDeps for Dependencies<'a, T> {
    type CustomEvent = T;

    fn event_loop(&self) -> &winit::event_loop::EventLoopWindowTarget<Self::CustomEvent> {
        self.event_loop.deref()
    }

    fn settings(&self) -> &Settings {
        &self.graphics_settings
    }
}

impl<'a, T> AssetDatabaseDeps for Dependencies<'a, T> {
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

impl<'a, T> OrchestratorDeps for Dependencies<'a, T> {
    fn runtime(&self) -> Arc<Runtime> {
        self.rt.clone()
    }

    fn main_scene(&self) -> Option<&str> {
        None
    }
}

impl<'a, T> RpcDeps for Dependencies<'a, T> {}
