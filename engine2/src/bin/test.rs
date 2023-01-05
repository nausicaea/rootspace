use ecs::Reg;
use engine2::orchestrator::Orchestrator;
use try_default::TryDefault;
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new();

    let state: Orchestrator<Reg![], Reg![], Reg![], Reg![]> = Orchestrator::try_default()?;

    event_loop.run(state.run(String::from("test"), false))
}
