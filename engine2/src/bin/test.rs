use std::ops::Deref;

use ecs::Reg;
use engine2::{orchestrator::Orchestrator, resources::{graphics::GraphicsDeps, asset_database::AssetDatabaseDeps}};
use winit::event_loop::EventLoop;

struct Dependencies<'a, T: 'static> {
    event_loop: &'a EventLoop<T>,
    name: &'a str,
    force_init: bool,
}

impl<'a, T> GraphicsDeps for Dependencies<'a, T> {
    type CustomEvent = T;

    fn event_loop(&self) -> &winit::event_loop::EventLoopWindowTarget<Self::CustomEvent> {
        self.event_loop.deref()
    }
}

impl<'a, T> AssetDatabaseDeps for Dependencies<'a, T> {
    fn name(&self) -> &str {
        self.name
    }

    fn force_init(&self) -> bool {
        self.force_init
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new();

    let deps = Dependencies {
        event_loop: &event_loop,
        name: "test",
        force_init: false,
    };

    let state = Orchestrator::with_dependencies::<Reg![], Reg![], Reg![], Reg![], _>(&deps).unwrap();

    event_loop.run(state.run())
}
