// mod assets;
mod resources;

use ecs::{Component, EventQueue, LoopStage, Reg, WorldEvent};
use engine::{
    components::{Camera, Info, Model, Renderable, Status, UiModel},
    event::EngineEvent,
    graphics::{glium::GliumBackend, headless::HeadlessBackend, BackendTrait},
    resources::{BackendResource, SceneGraph},
    systems::{
        CameraManager, DebugConsole, DebugShell, EventCoordinator, EventInterface, EventMonitor, ForceShutdown,
        Renderer,
    },
    orchestrator::Orchestrator,
};
use failure::Error;
use nalgebra::{Vector2, Vector3};
use std::{f32, path::Path, time::Duration};

type ResourceRegistry = Reg![
    <Camera as Component>::Storage,
    <Info as Component>::Storage,
    <Model as Component>::Storage,
    <Renderable as Component>::Storage,
    <Status as Component>::Storage,
    <UiModel as Component>::Storage,
];

pub struct Game<B>
where
    B: BackendTrait,
{
    orchestrator: Orchestrator<B, ResourceRegistry>,
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
        Ok(Game {
            orchestrator: Orchestrator::new(resource_path, delta_time, max_frame_time)?,
        })
    }

    pub fn load(&mut self) -> Result<(), Error> {
        let camera = self.orchestrator.create_entity();
        self.orchestrator.insert_component(camera, Status::default());
        self.orchestrator.insert_component(camera, Camera::default());
        self.orchestrator
            .insert_component(camera, Info::new("Camera", "The main camera"));

        let ea = self.orchestrator.create_entity();
        self.orchestrator.get_mut::<SceneGraph<Model>>().insert(ea);
        self.orchestrator.insert_component(ea, Status::default());
        self.orchestrator
            .insert_component(ea, Info::new("Entity A", "Rotated cube example"));
        self.orchestrator.insert_component(
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
            let factory = self.orchestrator.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .font(f)
                .text_scale(16.0)
                .text_width(2.0, 200)
                .vertex_shader(vs)
                .fragment_shader(fs)
                .text(text)
                .build_text(factory)?
        };
        self.orchestrator.insert_component(ea, renderable);

        let eb = self.orchestrator.create_entity();
        self.orchestrator.get_mut::<SceneGraph<Model>>().insert(eb);
        self.orchestrator.insert_component(eb, Status::default());
        self.orchestrator
            .insert_component(eb, Info::new("Entity B", "Text example"));
        self.orchestrator.insert_component(
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
            let factory = self.orchestrator.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .mesh(m)
                .vertex_shader(vs)
                .fragment_shader(fs)
                .diffuse_texture(dt)
                .build_mesh(factory)?
        };
        self.orchestrator.insert_component(eb, renderable);

        let ec = self.orchestrator.create_entity();
        self.orchestrator.get_mut::<SceneGraph<UiModel>>().insert(ec);
        self.orchestrator.insert_component(ec, Status::default());
        self.orchestrator
            .insert_component(ec, Info::new("Entity C", "UI Text example"));
        self.orchestrator.insert_component(
            ec,
            UiModel::new(Vector2::new(0.0, 0.0), Vector2::new(800.0, 600.0), -1.0),
        );
        let renderable = {
            let m = self.orchestrator.file("meshes", "quad.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            let factory = self.orchestrator.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .mesh(m)
                .vertex_shader(vs)
                .fragment_shader(fs)
                .diffuse_texture(dt)
                .build_mesh(factory)?
        };
        self.orchestrator.insert_component(ec, renderable);

        // Handle the regular systems.
        let force_shutdown = ForceShutdown::default();
        self.orchestrator.add_system(LoopStage::Update, force_shutdown);

        let debug_console = DebugConsole::default();
        self.orchestrator.add_system(LoopStage::Update, debug_console);

        let event_interface: EventInterface<B> = EventInterface::default();
        self.orchestrator.add_system(LoopStage::Update, event_interface);

        let renderer: Renderer<B> = Renderer::new([0.69, 0.93, 0.93, 1.0]);
        self.orchestrator.add_system(LoopStage::Render, renderer);

        let queue = self.orchestrator.get_mut::<EventQueue<WorldEvent>>();
        let event_monitor: EventMonitor<WorldEvent> = EventMonitor::new(queue);
        self.orchestrator.add_system(LoopStage::Update, event_monitor);

        let queue = self.orchestrator.get_mut::<EventQueue<EngineEvent>>();
        let event_monitor: EventMonitor<EngineEvent> = EventMonitor::new(queue);
        self.orchestrator.add_system(LoopStage::Update, event_monitor);

        let queue = self.orchestrator.get_mut::<EventQueue<EngineEvent>>();
        let camera_manager = CameraManager::new(queue);
        self.orchestrator.add_system(LoopStage::Update, camera_manager);

        let queue = self.orchestrator.get_mut::<EventQueue<EngineEvent>>();
        let debug_shell = DebugShell::new(queue);
        self.orchestrator.add_system(LoopStage::Update, debug_shell);

        let queue = self.orchestrator.get_mut::<EventQueue<EngineEvent>>();
        let event_coordinator = EventCoordinator::new(queue);
        self.orchestrator.add_system(LoopStage::Update, event_coordinator);

        self.orchestrator
            .get_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::Startup);

        Ok(())
    }

    pub fn run(&mut self, iterations: Option<usize>) {
        self.orchestrator.run(iterations)
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
