mod player_character;

use anyhow::Result;
use ecs::{EventQueue, Reg, LoopStage};
use engine::{
    components::{Camera, Info, Status, camera::Projection, Model, Renderable, renderable::RenderableType},
    resources::{SceneGraph, BackendResource},
    orchestrator::Orchestrator,
    graphics::BackendTrait,
    EngineEvent
};
use crate::player_character::{PlayerCharacter, PlayerCharacterMarker};
use std::path::Path;
use std::time::Duration;
use nalgebra::{Vector3, Point3};

type ResourceRegistry = Reg![
];

pub struct Pacman<B>
where
    B: BackendTrait,
{
    orch: Orchestrator<B, ResourceRegistry>,
}

impl<B> Pacman<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self> {
        Ok(Pacman {
            orch: Orchestrator::new(resource_path, delta_time, max_frame_time)?,
        })
    }

    pub fn load(&mut self) -> Result<()> {
        // Create the camera
        let camera = self.orch.create_entity();
        self.orch.insert_component(camera, Status::default());
        self.orch
            .insert_component(camera, Info::new("Camera", "The main camera"));
        self.orch.insert_component(camera, Camera::new(Projection::Orthographic, (800, 600),
            std::f32::consts::PI / 4.0,
            (0.1, 1000.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::y(),
            1.0));

        // Create the player character
        let pacman = self.orch.create_entity();
        self.orch.get_mut::<SceneGraph<Model>>().insert(pacman);
        self.orch.insert_component(pacman, Status::default());
        self.orch.insert_component(pacman, Info::new("Pacman", "The player character"));
        self.orch.insert_component(pacman, Model::identity());
        let factory = self.orch.get_mut::<BackendResource<B>>();
        let renderable = Renderable::builder()
            .with_mesh("meshes/quad.ply")
            .with_vertex_shader("shaders/base-vertex.glsl")
            .with_fragment_shader("shaders/base-fragment.glsl")
            .with_diffuse_texture("textures/pacman.png")
            .with_type(RenderableType::Mesh)
            .build(factory)?;
        self.orch.insert_component(pacman, renderable);
        self.orch.insert_component(pacman, PlayerCharacterMarker);

        // Add the systems
        let queue = self.orch.get_mut::<EventQueue<EngineEvent>>();
        let pc = PlayerCharacter::new(queue);
        self.orch.add_system(LoopStage::Update, pc);

        Ok(())
    }

    pub fn run(&mut self, iterations: Option<usize>) {
        self.orch.run(iterations)
    }
}
