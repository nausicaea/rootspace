// mod assets;
// mod resources;

use ecs::Reg;
use engine::{
    components::{Camera, Info, Model, Renderable, Status, UiModel, renderable::RenderableType},
    graphics::BackendTrait,
    orchestrator::Orchestrator,
    resources::{BackendResource, SceneGraph},
};
use anyhow::Result;
use nalgebra::{Vector2, Vector3, Point3};
use std::{f32, path::Path};

type ResourceRegistry = Reg![
];

pub struct Rootspace<B>
where
    B: BackendTrait,
{
    orch: Orchestrator<B, ResourceRegistry>,
}

impl<B> Rootspace<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        command: Option<&str>,
    ) -> Result<Self> {
        Ok(Rootspace {
            orch: Orchestrator::new(resource_path, command)?,
        })
    }

    pub fn load(&mut self) -> Result<()> {
        let camera = self.orch.create_entity();
        self.orch.get_mut::<SceneGraph<Model>>().insert(camera);
        self.orch.insert_component(camera, Status::default());
        self.orch.insert_component(camera, Camera::default());
        self.orch
            .insert_component(camera, Info::new("Camera", "The main camera"));
        self.orch.insert_component(camera, Model::look_at(
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::y(), Vector3::new(1.0, 1.0, 1.0)
        ));

        let ea = self.orch.create_entity();
        self.orch.get_mut::<SceneGraph<Model>>().insert(ea);
        self.orch.insert_component(ea, Status::default());
        self.orch
            .insert_component(ea, Info::new("Entity A", "Rotated cube example"));
        self.orch.insert_component(
            ea,
            Model::new(
                Vector3::new(0.0, 0.0, -10.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );
        let renderable = {
            let factory = self.orch.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .with_font("fonts/SourceSansPro-Regular.ttf")
                .with_text_scale(16.0)
                .with_text_width(2.0, 200)
                .with_vertex_shader("shaders/text-vertex.glsl")
                .with_fragment_shader("shaders/text-fragment.glsl")
                .with_text("Hello, World!")
                .with_type(RenderableType::Text)
                .build(factory)?
        };
        self.orch.insert_component(ea, renderable);

        let eb = self.orch.create_entity();
        self.orch.get_mut::<SceneGraph<Model>>().insert(eb);
        self.orch.insert_component(eb, Status::default());
        self.orch
            .insert_component(eb, Info::new("Entity B", "Text example"));
        self.orch.insert_component(
            eb,
            Model::new(
                Vector3::new(-2.0, 1.0, -7.0),
                Vector3::new(0.0, f32::consts::PI / 4.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );
        let renderable = {
            let factory = self.orch.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .with_mesh("meshes/cube.ply")
                .with_vertex_shader("shaders/base-vertex.glsl")
                .with_fragment_shader("shaders/base-fragment.glsl")
                .with_diffuse_texture("textures/tv-test-image.png")
                .with_type(RenderableType::Mesh)
                .build(factory)?
        };
        self.orch.insert_component(eb, renderable);

        let ec = self.orch.create_entity();
        self.orch.get_mut::<SceneGraph<UiModel>>().insert(ec);
        self.orch.insert_component(ec, Status::default());
        self.orch
            .insert_component(ec, Info::new("Entity C", "UI Text example"));
        self.orch.insert_component(
            ec,
            UiModel::new(Vector2::new(0.0, 0.0), Vector2::new(800.0, 600.0), -1.0),
        );
        let renderable = {
            let factory = self.orch.get_mut::<BackendResource<B>>();
            Renderable::builder()
                .with_mesh("meshes/quad.ply")
                .with_vertex_shader("shaders/base-vertex.glsl")
                .with_fragment_shader("shaders/base-fragment.glsl")
                .with_diffuse_texture("textures/tv-test-image.png")
                .with_type(RenderableType::Mesh)
                .build(factory)?
        };
        self.orch.insert_component(ec, renderable);

        Ok(())
    }

    pub fn run(&mut self) {
        self.orch.run()
    }
}
