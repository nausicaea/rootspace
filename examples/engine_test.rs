use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use tokio::runtime::Runtime;

use rootspace::engine::orchestrator::{Orchestrator, OrchestratorDeps};
use rootspace::engine::resources::asset_database::AssetDatabaseDeps;
use rootspace::engine::resources::graphics::settings::Settings;
use rootspace::engine::resources::graphics::GraphicsDeps;
use rootspace::Reg;
use winit::event_loop::EventLoop;
use rootspace::engine::resources::rpc_settings::RpcDeps;

struct Dependencies<'a, T: 'static> {
    rt: Arc<Runtime>,
    event_loop: &'a EventLoop<T>,
    name: &'a str,
    main_scene: &'a str,
    force_init: bool,
    within_repo: bool,
    graphics_settings: Settings,
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
        self.within_repo
    }
}

impl<'a, T> OrchestratorDeps for Dependencies<'a, T> {
    fn main_scene(&self) -> &str {
        self.main_scene
    }

    fn runtime(&self) -> Arc<Runtime> {
        self.rt.clone()
    }
}

impl<'a, T> RpcDeps for Dependencies<'a, T> {}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let rt = Arc::new(rt);

    let deps = Dependencies {
        rt: rt.clone(),
        event_loop: &event_loop,
        name: "test",
        main_scene: "test.cbor",
        force_init: false,
        within_repo: true,
        graphics_settings: Settings::default(),
    };

    let state = rt.block_on(async move {Orchestrator::with_dependencies::<Reg![], Reg![], Reg![], _>(&deps).await })?;

    event_loop.run(state.start())?;

    Ok(())
}
