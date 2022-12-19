use std::{fs::File, path::Path};

use ecs::{Reg, RegAdd, ResourceRegistry, SystemRegistry};
use engine2::resources::{asset_database::AssetDatabase, graphics::Graphics};
use file_manipulation::FilePathBuf;
use log::trace;
use winit::{
    event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();

    let state: Orchestrator<Reg![], Reg![], Reg![], Reg![]> = Orchestrator::new().unwrap();

    event_loop.run(state.run(String::from("test"), false))
}

type Resources<S> = RegAdd![AssetDatabase, Graphics, S];

type World<S, F, D, R> = ecs::World<Resources<S>, F, D, R>;

struct Orchestrator<S, F, D, R> {
    world: World<S, F, D, R>,
}

impl<S, F, D, R> Orchestrator<S, F, D, R>
where
    S: 'static + ResourceRegistry,
    F: 'static + SystemRegistry,
    D: 'static + SystemRegistry,
    R: 'static + SystemRegistry,
{
    fn new() -> anyhow::Result<Self> {
        use try_default::TryDefault;

        let world = World::try_default()?;

        Ok(Orchestrator { world })
    }

    fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        use serde::Deserialize;

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

        Ok(Orchestrator { world })
    }

    fn init<T>(&mut self, event_loop: &EventLoopWindowTarget<T>, name: &str, force_init: bool) {
        use pollster::FutureExt;

        self.world.get_mut::<AssetDatabase>().initialize(name, force_init).unwrap();
        self.world.get_mut::<Graphics>().initialize(event_loop).block_on();
    }

    fn run(
        mut self, name: String, force_init: bool,
    ) -> impl 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow) {
        move |event, event_loop, control_flow| {
            trace!("{:?}", &event);
            match event {
                Event::NewEvents(StartCause::Init) => self.init(event_loop, &name, force_init),
                Event::WindowEvent {
                    event: ref e,
                    ..
                } => match e {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
}
