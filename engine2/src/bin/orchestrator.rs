use std::{fs::File, path::Path};

use ecs::{Reg, RegAdd, ResourceRegistry, SystemRegistry};
use engine2::resources::asset_database::AssetDatabase;
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
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let state: Orchestrator<Reg![], Reg![], Reg![], Reg![]> = Orchestrator::new(window, "wgpu", false).unwrap();

    event_loop.run(state.event_handler_factory())
}

type Resources<S> = RegAdd![AssetDatabase, S];

type World<S, F, D, R> = ecs::World<Resources<S>, F, D, R>;

struct Orchestrator<S, F, D, R> {
    world: World<S, F, D, R>,
    window: Window,
}

impl<S, F, D, R> Orchestrator<S, F, D, R>
where
    S: 'static + ResourceRegistry,
    F: 'static + SystemRegistry,
    D: 'static + SystemRegistry,
    R: 'static + SystemRegistry,
{
    fn new<N: AsRef<str>>(window: Window, name: N, force_init: bool) -> anyhow::Result<Self> {
        use try_default::TryDefault;

        let mut world = World::try_default()?;
        world.get_mut::<AssetDatabase>().initialize(name.as_ref(), force_init)?;

        Ok(Orchestrator { world, window })
    }

    pub fn load<P: AsRef<Path>>(window: Window, path: P) -> anyhow::Result<Self> {
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

        Ok(Orchestrator { world, window })
    }

    async fn init(&mut self) {
        let _size = self.window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&self.window) };
        let _adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
    }

    fn event_handler_factory(
        mut self,
    ) -> impl 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow) {
        move |event, _event_loop, control_flow| {
            use pollster::FutureExt;

            trace!("{:?}", &event);
            match event {
                Event::NewEvents(StartCause::Init) => self.init().block_on(),
                Event::WindowEvent {
                    window_id,
                    event: ref e,
                } if window_id == self.window.id() => match e {
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
