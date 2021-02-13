mod player_character;
mod settings;

use crate::player_character::{PlayerCharacter, PlayerCharacterMarker};
use anyhow::Result;
use ecs::{Component, EventQueue, LoopStage, Reg};
use engine::{
    components::{Camera, Info, Model, Projection, Renderable, RenderableType, Status},
    graphics::BackendTrait,
    orchestrator::Orchestrator,
    resources::{GraphicsBackend, SceneGraph},
    EngineEvent,
};
use nalgebra::Vector3;
use std::path::Path;
use crate::settings::Settings;
use file_manipulation::DirPathBuf;
use std::convert::TryFrom;

type ResourceRegistry = Reg![<PlayerCharacterMarker as Component>::Storage,];
type FixedUpdateSystemRegistry = Reg![];
type UpdateSystemRegistry = Reg![PlayerCharacter];
type RenderSystemRegistry = Reg![];

pub struct Pacman<B>
where
    B: BackendTrait,
{
    orch: Orchestrator<Settings, B, ResourceRegistry, FixedUpdateSystemRegistry, UpdateSystemRegistry, RenderSystemRegistry>,
}

impl<B> Pacman<B>
where
    B: BackendTrait,
{
    pub fn new<P: AsRef<Path>>(resource_path: P, command: Option<&str>) -> Result<Self> {
        let resource_path = DirPathBuf::try_from(resource_path.as_ref())?;
        let settings = Settings::builder(resource_path).build();

        Ok(Pacman {
            orch: Orchestrator::new(settings, command)?,
        })
    }

    pub fn load(&mut self) -> Result<()> {
        let world = &mut self.orch.world;

        // Create the camera
        let camera = world.create_entity();
        world.get_mut::<SceneGraph<Model>>().insert(camera);
        world.insert_component(camera, Status::default());
        world
            .insert_component(camera, Info::new("Camera", "The main camera"));
        world.insert_component(
            camera,
            Camera::new(
                Projection::Orthographic,
                (800, 600),
                std::f32::consts::PI / 2.0,
                (0.1, 1000.0),
                1.0,
            ),
        );
        world.insert_component(camera, Model::identity());

        // Create the player character
        let pacman = world.create_entity();
        world.get_mut::<SceneGraph<Model>>().insert(pacman);
        world.insert_component(pacman, Status::default());
        world
            .insert_component(pacman, Info::new("Pacman", "The player character"));
        world.insert_component(
            pacman,
            Model::new(
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );
        let factory = world.get_mut::<GraphicsBackend<B>>();
        let renderable = Renderable::builder()
            .with_mesh("meshes/quad.ply")
            .with_vertex_shader("shaders/base-vertex.glsl")
            .with_fragment_shader("shaders/base-fragment.glsl")
            .with_diffuse_texture("textures/sprites.png")
            .with_type(RenderableType::Mesh)
            .build(factory)?;
        world.insert_component(pacman, renderable);
        world.insert_component(pacman, PlayerCharacterMarker);

        Ok(())
    }

    pub fn run(&mut self) {
        self.orch.run()
    }
}
