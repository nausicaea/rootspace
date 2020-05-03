// mod assets;
// mod resources;

use ecs::Reg;
use engine::{
    components::{Camera, Info, Model, Renderable, Status, UiModel},
    graphics::BackendTrait,
    orchestrator::Orchestrator,
    resources::{BackendResource, SceneGraph},
};
use anyhow::Result;
use nalgebra::{Vector2, Vector3};
use std::{f32, path::Path, time::Duration};

type ResourceRegistry = Reg![
];

pub struct Rootspace<B>
where
    B: BackendTrait,
{
    orchestrator: Orchestrator<B, ResourceRegistry>,
}

impl<B> Rootspace<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self> {
        Ok(Rootspace {
            orchestrator: Orchestrator::new(resource_path, delta_time, max_frame_time)?,
        })
    }

    pub fn load(&mut self) -> Result<()> {
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
            let factory = self.orchestrator.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .font("fonts/SourceSansPro-Regular.ttf")
                .text_scale(16.0)
                .text_width(2.0, 200)
                .vertex_shader("shaders/text-vertex.glsl")
                .fragment_shader("shaders/text-fragment.glsl")
                .text("Hello, World!")
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
            let factory = self.orchestrator.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .mesh("meshes/cube.ply")
                .vertex_shader("shaders/base-vertex.glsl")
                .fragment_shader("shaders/base-fragment.glsl")
                .diffuse_texture("textures/tv-test-image.png")
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
            let factory = self.orchestrator.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .mesh("meshes/quad.ply")
                .vertex_shader("shaders/base-vertex.glsl")
                .fragment_shader("shaders/base-fragment.glsl")
                .diffuse_texture("textures/tv-test-image.png")
                .build_mesh(factory)?
        };
        self.orchestrator.insert_component(ec, renderable);

        Ok(())
    }

    pub fn run(&mut self, iterations: Option<usize>) {
        self.orchestrator.run(iterations)
    }
}
