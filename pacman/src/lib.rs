mod player_character;

use crate::player_character::{PlayerCharacter, PlayerCharacterMarker};
use anyhow::Result;
use ecs::{Component, EventQueue, LoopStage, Reg};
use engine::{
    components::{Camera, Info, Model, Projection, Renderable, RenderableType, Status},
    graphics::BackendTrait,
    orchestrator::Orchestrator,
    resources::{BackendResource, SceneGraph},
    EngineEvent,
};
use nalgebra::Vector3;
use std::path::Path;

type ResourceRegistry = Reg![<PlayerCharacterMarker as Component>::Storage,];
type FixedUpdateSystemRegistry = Reg![];
type UpdateSystemRegistry = Reg![PlayerCharacter];
type RenderSystemRegistry = Reg![];

pub struct Pacman<B>
where
    B: BackendTrait,
{
    orch: Orchestrator<B, ResourceRegistry, FixedUpdateSystemRegistry, UpdateSystemRegistry, RenderSystemRegistry>,
}

impl<B> Pacman<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(resource_path: P, command: Option<&str>) -> Result<Self> {
        Ok(Pacman {
            orch: Orchestrator::new(resource_path, command)?,
        })
    }

    pub fn load(&mut self) -> Result<()> {
        // Create the camera
        let camera = self.orch.create_entity();
        self.orch.get_mut::<SceneGraph<Model>>().insert(camera);
        self.orch.insert_component(camera, Status::default());
        self.orch
            .insert_component(camera, Info::new("Camera", "The main camera"));
        self.orch.insert_component(
            camera,
            Camera::new(
                Projection::Orthographic,
                (800, 600),
                std::f32::consts::PI / 2.0,
                (0.1, 1000.0),
                1.0,
            ),
        );
        self.orch.insert_component(camera, Model::identity());

        // Create the player character
        let pacman = self.orch.create_entity();
        self.orch.get_mut::<SceneGraph<Model>>().insert(pacman);
        self.orch.insert_component(pacman, Status::default());
        self.orch
            .insert_component(pacman, Info::new("Pacman", "The player character"));
        self.orch.insert_component(
            pacman,
            Model::new(
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );
        let factory = self.orch.get_mut::<BackendResource<B>>();
        let renderable = Renderable::builder()
            .with_mesh("meshes/quad.ply")
            .with_vertex_shader("shaders/base-vertex.glsl")
            .with_fragment_shader("shaders/base-fragment.glsl")
            .with_diffuse_texture("textures/sprites.png")
            .with_type(RenderableType::Mesh)
            .build(factory)?;
        self.orch.insert_component(pacman, renderable);
        self.orch.insert_component(pacman, PlayerCharacterMarker);

        // Add the systems
        let queue = self.orch.get_mut::<EventQueue<EngineEvent>>();
        let pc = PlayerCharacter::new(queue);
        self.orch.add_system(LoopStage::FixedUpdate, pc);

        Ok(())
    }

    pub fn run(&mut self) {
        self.orch.run()
    }
}
