use std::ops::Deref;

use rootspace::engine::orchestrator::{Orchestrator, OrchestratorDeps};
use rootspace::engine::resources::asset_database::AssetDatabaseDeps;
use rootspace::engine::resources::graphics::settings::Settings;
use rootspace::engine::resources::graphics::GraphicsDeps;
use rootspace::Reg;
use winit::event_loop::EventLoop;

struct Dependencies<'a, T: 'static> {
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;

    let deps = Dependencies {
        event_loop: &event_loop,
        name: "test",
        main_scene: "test.cbor",
        force_init: false,
        within_repo: true,
        graphics_settings: Settings::default(),
    };

    let state = Orchestrator::with_dependencies::<Reg![], Reg![], Reg![], _>(&deps).await?;

    event_loop.run(state.start())?;

    Ok(())
}
