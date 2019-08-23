extern crate ecs;
extern crate engine;
extern crate failure;
extern crate glium;
extern crate log;
extern crate nalgebra;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
extern crate daggy;

mod file_system;

use ecs::{EventQueue, LoopStage, Resources, World, WorldTrait, Persistence};
use engine::{
    components::{Camera, Info, Model, Status, UiModel, Renderable},
    event::EngineEvent,
    graphics::{glium::GliumBackend, headless::HeadlessBackend, BackendTrait},
    resources::{SceneGraph, Backend},
    systems::{
        CameraManager, DebugConsole, DebugShell, EventCoordinator, EventInterface, EventMonitor, ForceShutdown,
        Renderer,
    },
    DefaultOrchestrator,
};
use failure::Error;
use nalgebra::{Vector2, Vector3};
use std::{f32, path::Path, time::Duration};
use self::file_system::FileSystem;

pub struct Game<B> {
    orchestrator: DefaultOrchestrator<B>,
}

impl<B> Game<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, Error> {
        let mut o = DefaultOrchestrator::new(resource_path, delta_time, max_frame_time)?;
        o.world.add_resource(FileSystem::default(), Persistence::Runtime);

        Ok(Game { orchestrator: o })
    }

    pub fn load(&mut self) -> Result<(), Error> {
        self.orchestrator.reset();

        let event_interface: EventInterface<B> = EventInterface::default();
        let renderer: Renderer<B> = Renderer::new(&event_interface.events_loop, "Title", (800, 600), true, 4)?;

        let camera = self.world_mut().create_entity();
        self.world_mut().add_component(camera, Status::default());
        self.world_mut().add_component(camera, Camera::default());
        self.world_mut()
            .add_component(camera, Info::new("Camera", "The main camera"));

        let ea = self.world_mut().create_entity();
        self.world_mut().get_resource_mut::<SceneGraph<Model>>().insert(ea);
        self.world_mut().add_component(ea, Status::default());
        self.world_mut()
            .add_component(ea, Info::new("Entity A", "Rotated cube example"));
        self.world_mut().add_component(
            ea,
            Model::new(
                Vector3::new(0.0, 0.0, -10.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );
        let renderable = {
            let f = self.orchestrator.file("fonts", "SourceSansPro-Regular.ttf")?;
            let vs = self.orchestrator.file("shaders", "text-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "text-fragment.glsl")?;
            let text = "Hello, World!";
            let factory = self.res_mut().get_mut::<Backend<B>>();
            Renderable::builder()
                .font(f)
                .text_scale(16.0)
                .text_width(2.0, 200)
                .vertex_shader(vs)
                .fragment_shader(fs)
                .text(text)
                .build_text(&renderer.backend, factory)?
        };
        self.world_mut().add_component(
            ea,
            renderable,
        );

        let eb = self.world_mut().create_entity();
        self.world_mut().get_resource_mut::<SceneGraph<Model>>().insert(eb);
        self.world_mut().add_component(eb, Status::default());
        self.world_mut()
            .add_component(eb, Info::new("Entity B", "Text example"));
        self.world_mut().add_component(
            eb,
            Model::new(
                Vector3::new(-2.0, 1.0, -7.0),
                Vector3::new(0.0, f32::consts::PI / 4.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );
        let renderable = {
            let m = self.orchestrator.file("meshes", "cube.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            let factory = self.res_mut().get_mut::<Backend<B>>();
            Renderable::builder()
                .mesh(m)
                .vertex_shader(vs)
                .fragment_shader(fs)
                .diffuse_texture(dt)
                .build_mesh(&renderer.backend, factory)?
        };
        self.world_mut().add_component(
            eb,
            renderable,
        );

        let ec = self.world_mut().create_entity();
        self.world_mut().get_resource_mut::<SceneGraph<UiModel>>().insert(ec);
        self.world_mut().add_component(ec, Status::default());
        self.world_mut()
            .add_component(ec, Info::new("Entity C", "UI Text example"));
        self.world_mut().add_component(
            ec,
            UiModel::new(Vector2::new(0.0, 0.0), Vector2::new(800.0, 600.0), -1.0),
        );
        let renderable = {
            let m = self.orchestrator.file("meshes", "quad.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            let factory = self.res_mut().get_mut::<Backend<B>>();
            Renderable::builder()
                .mesh(m)
                .vertex_shader(vs)
                .fragment_shader(fs)
                .diffuse_texture(dt)
                .build_mesh(&renderer.backend, factory)?
        };
        self.world_mut().add_component(
            ec,
            renderable,
        );

        // Handle the regular systems.
        let force_shutdown = ForceShutdown::default();
        self.world_mut().add_system(LoopStage::Update, force_shutdown);

        let debug_console = DebugConsole::default();
        self.world_mut().add_system(LoopStage::Update, debug_console);

        self.world_mut().add_system(LoopStage::Update, event_interface);
        self.world_mut().add_system(LoopStage::Render, renderer);

        let queue = self.res_mut().get_mut::<EventQueue<EngineEvent>>();
        let event_monitor: EventMonitor<EngineEvent> = EventMonitor::new(queue);
        self.world_mut().add_system(LoopStage::Update, event_monitor);

        let queue = self.res_mut().get_mut::<EventQueue<EngineEvent>>();
        let camera_manager = CameraManager::new(queue);
        self.world_mut().add_system(LoopStage::Update, camera_manager);

        let queue = self.res_mut().get_mut::<EventQueue<EngineEvent>>();
        let debug_shell = DebugShell::new(queue);
        self.world_mut().add_system(LoopStage::Update, debug_shell);

        let queue = self.res_mut().get_mut::<EventQueue<EngineEvent>>();
        let event_coordinator = EventCoordinator::new(queue);
        self.world_mut().add_system(LoopStage::Update, event_coordinator);

        self.world_mut()
            .get_resource_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::Startup);

        Ok(())
    }

    pub fn run(&mut self, iterations: Option<usize>) {
        self.orchestrator.run(iterations)
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.orchestrator.world
    }

    fn res_mut(&mut self) -> &mut Resources {
        &mut self.orchestrator.world.resources
    }
}

impl Game<HeadlessBackend> {
    pub fn new_headless<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, Error> {
        Self::new(resource_path, delta_time, max_frame_time)
    }
}

impl Game<GliumBackend> {
    pub fn new_glium<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, Error> {
        Self::new(resource_path, delta_time, max_frame_time)
    }
}
