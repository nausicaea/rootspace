#[macro_use]
extern crate bitflags;
extern crate ecs;
extern crate engine;
extern crate failure;
extern crate glium;
extern crate log;
extern crate nalgebra;

mod event;

use ecs::{DatabaseTrait, EventManagerTrait};
use engine::{
    components::{camera::Camera, info::Info, model::Model, renderable::Renderable},
    context::{Context, Layer, SceneGraphTrait},
    event::EngineEventTrait,
    systems::{DebugConsole, DebugShell, EventCoordinator, EventMonitor},
    DefaultOrchestrator, DefaultWorld, GliumEventInterface, GliumRenderer, HeadlessEventInterface, HeadlessRenderer,
};
use event::Event;
use failure::Error;
use nalgebra::Vector3;
use std::{f32, io, path::Path, time::Duration};

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

    pub fn run(&mut self, headless: bool, iterations: Option<usize>) -> Result<(), Error> {
        let camera = self.context_mut().create_entity();
        self.context_mut().add(camera, Camera::default())?;

        let ea = self.context_mut().create_entity();
        self.context_mut().insert_node(ea, Layer::World);
        self.context_mut()
            .add(ea, Info::new("Entity A", "Rotated cube example"))?;
        self.context_mut().add(
            ea,
            Model::new(
                Vector3::new(0.0, 0.0, -10.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        )?;

        let eb = self.context_mut().create_entity();
        self.context_mut().insert_node(eb, Layer::World);
        self.context_mut().add(eb, Info::new("Entity B", "Text example"))?;
        self.context_mut().add(
            eb,
            Model::new(
                Vector3::new(-2.0, 1.0, -7.0),
                Vector3::new(0.0, f32::consts::PI / 4.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        )?;

        let ec = self.context_mut().create_entity();
        self.context_mut().insert_node(ec, Layer::Ui);
        self.context_mut().add(ec, Info::new("Entity C", "UI Text example"))?;
        self.context_mut().add(
            ec,
            Model::new(
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(800.0, 600.0, 1.0),
            ),
        )?;

        if headless {
            let event_interface = HeadlessEventInterface::default();
            let renderer = HeadlessRenderer::new(&event_interface.events_loop, "Title", (800, 600), true, 4)?;

            let f = self.orchestrator.file("fonts", "SourceSansPro-Regular.ttf")?;
            let vs = self.orchestrator.file("shaders", "text-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "text-fragment.glsl")?;
            let text = "Hello, World!";
            self.context_mut().add(
                ea,
                Renderable::builder()
                    .font(f)
                    .text_scale(16.0)
                    .text_width(2.0, 200)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .text(text)
                    .build_text_headless(&renderer.backend)?,
            )?;

            let m = self.orchestrator.file("meshes", "cube.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            self.context_mut().add(
                eb,
                Renderable::builder()
                    .mesh(m)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .diffuse_texture(dt)
                    .build_mesh_headless(&renderer.backend)?,
            )?;

            let m = self.orchestrator.file("meshes", "quad.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            self.context_mut().add(
                ec,
                Renderable::builder()
                    .mesh(m)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .diffuse_texture(dt)
                    .build_mesh_headless(&renderer.backend)?,
            )?;

            self.world_mut().add_system(event_interface);
            self.world_mut().add_system(renderer);
        } else {
            let event_interface = GliumEventInterface::default();
            let renderer = GliumRenderer::new(&event_interface.events_loop, "Title", (800, 600), true, 4)?;

            let f = self.orchestrator.file("fonts", "SourceSansPro-Regular.ttf")?;
            let vs = self.orchestrator.file("shaders", "text-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "text-fragment.glsl")?;
            let text = "Hello, World!";
            self.context_mut().add(
                ea,
                Renderable::builder()
                    .font(f)
                    .text_scale(16.0)
                    .text_width(2.0, 200)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .text(text)
                    .build_text_glium(&renderer.backend)?,
            )?;

            let m = self.orchestrator.file("meshes", "cube.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            self.context_mut().add(
                eb,
                Renderable::builder()
                    .mesh(m)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .diffuse_texture(dt)
                    .build_mesh_glium(&renderer.backend)?,
            )?;

            let m = self.orchestrator.file("meshes", "quad.ply")?;
            let vs = self.orchestrator.file("shaders", "base-vertex.glsl")?;
            let fs = self.orchestrator.file("shaders", "base-fragment.glsl")?;
            let dt = self.orchestrator.file("textures", "tv-test-image.png")?;
            self.context_mut().add(
                ec,
                Renderable::builder()
                    .mesh(m)
                    .vertex_shader(vs)
                    .fragment_shader(fs)
                    .diffuse_texture(dt)
                    .build_mesh_glium(&renderer.backend)?,
            )?;

            self.world_mut().add_system(event_interface);
            self.world_mut().add_system(renderer);
        }

        let event_monitor = EventMonitor::default();
        self.world_mut().add_system(event_monitor);

        let event_coordinator = EventCoordinator::default();
        self.world_mut().add_system(event_coordinator);

        let debug_console = DebugConsole::new(io::stdin(), None, None);
        self.world_mut().add_system(debug_console);

        let debug_shell = DebugShell::default();
        self.world_mut().add_system(debug_shell);

        self.orchestrator.world.context.dispatch_later(Event::new_startup());
        self.orchestrator.run(iterations)?;

        Ok(())
    }

    fn world_mut(&mut self) -> &mut DefaultWorld<Event> {
        &mut self.orchestrator.world
    }

    fn context_mut(&mut self) -> &mut Context<Event> {
        &mut self.orchestrator.world.context
    }
}
