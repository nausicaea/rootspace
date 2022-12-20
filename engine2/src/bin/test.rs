use ecs::Reg;
use engine2::orchestrator::Orchestrator;
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new();

    let state: Orchestrator<Reg![], Reg![], Reg![], Reg![]> = Orchestrator::new()?;

    event_loop.run(state.run(String::from("test"), false))
}
