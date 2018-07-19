extern crate ecs;
extern crate engine;
extern crate failure;
extern crate log;
extern crate nalgebra;

use ecs::{DatabaseTrait, EventManagerTrait, World};
use engine::{
    components::{camera::Camera, model::Model, renderable::Renderable},
    context::{Context, SceneGraphTrait},
    event::Event,
    file_manipulation::FileError,
    orchestrator::Orchestrator,
    systems::{
        event_coordinator::EventCoordinator,
        event_interface::{GliumEventInterface, HeadlessEventInterface},
        event_monitor::EventMonitor,
        renderer::{GliumRenderer, HeadlessRenderer},
        SystemGroup,
    },
};
use failure::Error;
use nalgebra::Vector3;
use std::{path::Path, time::Duration, f32};

pub struct Game {
    orchestrator: Orchestrator<World<Event, Context, SystemGroup>>,
}

impl Game {
    pub fn new(resource_path: &Path, delta_time: Duration, max_frame_time: Duration) -> Result<Self, FileError> {
        let o = Orchestrator::new(resource_path, delta_time, max_frame_time)?;

        Ok(Game { orchestrator: o })
    }

    pub fn run(&mut self, headless: bool, iterations: Option<usize>) -> Result<(), Error> {
        let camera = self.context_mut().create_entity();
        self.context_mut().add(camera, Camera::default()).unwrap();

        let ea = self.context_mut().create_entity();
        self.context_mut().insert_node(ea);
        self.context_mut().add(ea, Model::new(Vector3::new(0.0, 0.0, -10.0), Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0))).unwrap();

        let eb = self.context_mut().create_entity();
        self.context_mut().insert_node(eb);
        self.context_mut().add(eb, Model::new(Vector3::new(-2.0, 1.0, -7.0), Vector3::new(0.0, f32::consts::PI / 4.0, 0.0), Vector3::new(1.0, 1.0, 1.0))).unwrap();

        if headless {
            let event_interface = HeadlessEventInterface::default();
            let renderer = HeadlessRenderer::new(&event_interface.events_loop, "Title", [800, 600], true, 4).unwrap();

            self.context_mut()
                .add(ea, Renderable::triangle(&renderer.backend).unwrap())
                .unwrap();

            self.context_mut()
                .add(eb, Renderable::cube(&renderer.backend).unwrap())
                .unwrap();

            self.world_mut().add_system(event_interface);
            self.world_mut().add_system(renderer);
        } else {
            let event_interface = GliumEventInterface::default();
            let renderer = GliumRenderer::new(&event_interface.events_loop, "Title", [800, 600], true, 4).unwrap();

            self.context_mut()
                .add(ea, Renderable::triangle(&renderer.backend).unwrap())
                .unwrap();

            self.context_mut()
                .add(eb, Renderable::cube(&renderer.backend).unwrap())
                .unwrap();

            self.world_mut().add_system(event_interface);
            self.world_mut().add_system(renderer);
        }

        let event_monitor = EventMonitor::default();
        self.world_mut().add_system(event_monitor);

        let event_coordinator = EventCoordinator::default();
        self.world_mut().add_system(event_coordinator);

        self.orchestrator.world.context.dispatch_later(Event::startup());
        self.orchestrator.run(iterations)?;

        Ok(())
    }

    fn world_mut(&mut self) -> &mut World<Event, Context, SystemGroup> {
        &mut self.orchestrator.world
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.orchestrator.world.context
    }
}
