extern crate ecs;
extern crate engine;
extern crate failure;
extern crate glium;
extern crate log;

use ecs::database::DatabaseTrait;
use ecs::event::EventManagerTrait;
use ecs::world::World;
use engine::components::model::Model;
use engine::components::renderable::{Renderable, HeadlessRenderData, GliumRenderData};
use engine::context::{Context, SceneGraphTrait};
use engine::event::Event;
use engine::file_manipulation::FileError;
use engine::orchestrator::Orchestrator;
use engine::systems::event_interface::EventInterface;
use engine::systems::event_monitor::EventMonitor;
use engine::systems::renderer::Renderer;
use engine::systems::SystemGroup;
use engine::wrappers::glium::{HeadlessDisplay, HeadlessEventsLoop};
use failure::Error;
use glium::glutin::EventsLoop;
use glium::Display;
use std::path::Path;
use std::time::Duration;

pub struct Game {
    orchestrator: Orchestrator<World<Event, Context, SystemGroup>>,
}

impl Game {
    pub fn new(
        resource_path: &Path,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, FileError> {
        let o = Orchestrator::new(resource_path, delta_time, max_frame_time)?;

        Ok(Game { orchestrator: o })
    }

    pub fn run(&mut self, headless: bool, iterations: Option<usize>) -> Result<(), Error> {
        let title = "Title";
        let dimensions = [1024, 768];
        let vsync = true;
        let msaa = 4;
        let clear_color = [0.2, 0.3, 0.0, 1.0];

        let event_monitor = EventMonitor::default();
        self.orchestrator.world.add_system(event_monitor);

        // Create and register the systems that depend on a graphics backend.
        if headless {
            let event_interface = EventInterface::new(HeadlessEventsLoop::default());
            let renderer: Renderer<Event, Context, HeadlessDisplay, Model, Renderable<HeadlessRenderData>> =
                Renderer::new(
                    &event_interface.events_loop,
                    title,
                    &dimensions,
                    vsync,
                    msaa,
                    clear_color,
                ).unwrap();

            self.orchestrator.world.add_system(event_interface);
            self.orchestrator.world.add_system(renderer);

            let ctx = &mut self.orchestrator.world.context;
            let a = ctx.create_entity();
            ctx.insert_node(a);
            ctx.add(a, Model::new(100.0)).unwrap();
            ctx.add(a, Renderable::<HeadlessRenderData>::new()).unwrap();
            let b = ctx.create_entity();
            ctx.insert_node(b);
            ctx.add(b, Model::new(50.0)).unwrap();
            let c = ctx.create_entity();
            ctx.add(c, Renderable::<HeadlessRenderData>::new()).unwrap();
            let d = ctx.create_entity();
            ctx.insert_node(d);
        } else {
            let event_interface = EventInterface::new(EventsLoop::new());
            let renderer: Renderer<Event, Context, Display, Model, Renderable<GliumRenderData>> = Renderer::new(
                &event_interface.events_loop,
                title,
                &dimensions,
                vsync,
                msaa,
                clear_color,
            ).unwrap();

            self.orchestrator.world.add_system(event_interface);
            self.orchestrator.world.add_system(renderer);

            let ctx = &mut self.orchestrator.world.context;
            let a = ctx.create_entity();
            ctx.insert_node(a);
            ctx.add(a, Model::new(100.0)).unwrap();
            ctx.add(a, Renderable::<GliumRenderData>::new()).unwrap();
            let b = ctx.create_entity();
            ctx.insert_node(b);
            ctx.add(b, Model::new(50.0)).unwrap();
            let c = ctx.create_entity();
            ctx.add(c, Renderable::<GliumRenderData>::new()).unwrap();
            let d = ctx.create_entity();
            ctx.insert_node(d);
        }

        self.orchestrator
            .world
            .context
            .dispatch_later(Event::ready());
        self.orchestrator.run(iterations)?;

        Ok(())
    }
}
