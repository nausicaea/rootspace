#[macro_use]
extern crate bitflags;
extern crate ecs;
extern crate engine;
extern crate failure;
extern crate glium;
extern crate log;
extern crate nalgebra;

mod event;

use crate::event::Event;
use ecs::{EventManager, LoopStage, World};
use engine::{
    components::{camera::Camera, info::Info, model::Model, renderable::Renderable, ui_model::UiModel},
    event::EngineEventTrait,
    scene_graph::SceneGraph,
    systems::{
        camera_manager::CameraManager, debug_console::DebugConsole, debug_shell::DebugShell,
        event_coordinator::EventCoordinator, event_monitor::EventMonitor, force_shutdown::ForceShutdown,
    },
    DefaultOrchestrator, GliumEventInterface, GliumRenderer, HeadlessEventInterface, HeadlessRenderer,
};
use failure::Error;
use nalgebra::{Vector2, Vector3};
use std::{f32, path::Path, time::Duration};

pub struct Game {
    orchestrator: DefaultOrchestrator<Event>,
}

impl Game {
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, Error> {
        let o = DefaultOrchestrator::new(resource_path, delta_time, max_frame_time)?;

        Ok(Game { orchestrator: o })
    }

    pub fn load(&mut self, headless: bool) -> Result<(), Error> {
        self.orchestrator.reset();

        let camera = self.world_mut().create_entity();
        self.world_mut().add_component(camera, Camera::default());

        let ea = self.world_mut().create_entity();
        self.world_mut().get_resource_mut::<SceneGraph<Model>>().insert(ea);
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

        let eb = self.world_mut().create_entity();
        self.world_mut().get_resource_mut::<SceneGraph<Model>>().insert(eb);
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

        let ec = self.world_mut().create_entity();
        self.world_mut().get_resource_mut::<SceneGraph<UiModel>>().insert(ec);
        self.world_mut()
            .add_component(ec, Info::new("Entity C", "UI Text example"));
        self.world_mut().add_component(
            ec,
            UiModel::new(Vector2::new(0.0, 0.0), Vector2::new(800.0, 600.0), -1.0),
        );

        // Handle the regular systems.
        let event_monitor = EventMonitor::<Event>::default();
        self.world_mut().add_event_handler_system(event_monitor);

        let force_shutdown = ForceShutdown::<Event>::default();
        self.world_mut().add_system(LoopStage::Update, force_shutdown);

        let camera_manager = CameraManager::<Event>::default();
        self.world_mut().add_event_handler_system(camera_manager);

        let debug_console = DebugConsole::<Event>::default();
        self.world_mut().add_system(LoopStage::Update, debug_console);

        let debug_shell = DebugShell::<Event>::default();
        self.world_mut().add_event_handler_system(debug_shell);

        // Handle the systems that depend on a backend.
        if headless {
            let event_interface = HeadlessEventInterface::<Event>::default();
            let renderer = HeadlessRenderer::<Event>::new(&event_interface.events_loop, "Title", (800, 600), true, 4)?;

            let f = self.orchestrator.file("fonts", "SourceSansPro-Regular.ttf")?;
            let vs = self.orchestrator.file("shaders", "text-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "text-fragment.glsl")?;
            let text = "Hello, World!";
            self.world_mut().add_component(
                ea,
                Renderable::builder()
                    .font(f)
                    .text_scale(16.0)
                    .text_width(2.0, 200)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .text(text)
                    .build_text_headless(&renderer.backend)?,
            );

            let m = self.orchestrator.file("meshes", "cube.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            self.world_mut().add_component(
                eb,
                Renderable::builder()
                    .mesh(m)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .diffuse_texture(dt)
                    .build_mesh_headless(&renderer.backend)?,
            );

            let m = self.orchestrator.file("meshes", "quad.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            self.world_mut().add_component(
                ec,
                Renderable::builder()
                    .mesh(m)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .diffuse_texture(dt)
                    .build_mesh_headless(&renderer.backend)?,
            );

            self.world_mut().add_system(LoopStage::Update, event_interface);
            self.world_mut().add_system(LoopStage::Render, renderer);
        } else {
            let event_interface = GliumEventInterface::<Event>::default();
            let renderer = GliumRenderer::<Event>::new(&event_interface.events_loop, "Title", (800, 600), true, 4)?;

            let f = self.orchestrator.file("fonts", "SourceSansPro-Regular.ttf")?;
            let vs = self.orchestrator.file("shaders", "text-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "text-fragment.glsl")?;
            let text = "Hello, World!";
            self.world_mut().add_component(
                ea,
                Renderable::builder()
                    .font(f)
                    .text_scale(16.0)
                    .text_width(2.0, 200)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .text(text)
                    .build_text_glium(&renderer.backend)?,
            );

            let m = self.orchestrator.file("meshes", "cube.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            self.world_mut().add_component(
                eb,
                Renderable::builder()
                    .mesh(m)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .diffuse_texture(dt)
                    .build_mesh_glium(&renderer.backend)?,
            );

            let m = self.orchestrator.file("meshes", "quad.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            self.world_mut().add_component(
                ec,
                Renderable::builder()
                    .mesh(m)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .diffuse_texture(dt)
                    .build_mesh_glium(&renderer.backend)?,
            );

            self.world_mut().add_system(LoopStage::Update, event_interface);
            self.world_mut().add_system(LoopStage::Render, renderer);
        }

        // The event coordinator should run last, because it can affect the shutdown of the engine.
        let event_coordinator = EventCoordinator::<Event>::default();
        self.world_mut().add_event_handler_system(event_coordinator);

        self.world_mut()
            .get_resource_mut::<EventManager<Event>>()
            .dispatch_later(Event::new_startup());

        Ok(())
    }

    pub fn run(&mut self, iterations: Option<usize>) {
        self.orchestrator.run(iterations)
    }

    fn world_mut(&mut self) -> &mut World<Event> {
        &mut self.orchestrator.world
    }
}
