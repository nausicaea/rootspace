mod camera;
mod instance;
mod light;
mod model;
mod render;
mod state;
mod texture;
mod util;
mod vertex;

use griffon::winit::event_loop::{EventLoop, EventLoopWindowTarget};
use griffon::{GraphicsDeps, Settings};
use state::State;
use std::sync::Arc;
use tokio::runtime::Builder as RuntimeBuilder;

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tokio-console")]
    console_subscriber::init();
    #[cfg(not(feature = "tokio-console"))]
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new()?;
    let rt = Arc::new(RuntimeBuilder::new_multi_thread().enable_all().build()?);
    let state = rt.block_on(State::new(&event_loop))?;
    event_loop.run(state.run())?;
    Ok(())
}

#[derive(Debug)]
pub struct Dependencies<'a>(&'a EventLoopWindowTarget<()>, &'a Settings);

impl GraphicsDeps for Dependencies<'_> {
    type CustomEvent = ();

    fn event_loop(&self) -> &EventLoopWindowTarget<()> {
        self.0
    }

    fn settings(&self) -> &Settings {
        self.1
    }
}
