use clap::Parser;
use rootspace::ecs::with_dependencies::WithDependencies;
use rootspace::engine::assets::scene::Scene;
use rootspace::engine::components::camera::Camera;
use rootspace::engine::components::renderable::RenderableSource;
use rootspace::engine::components::transform::Transform;
use rootspace::engine::resources::asset_database::{AssetDatabase, AssetDatabaseDeps};
use rootspace::glamour::num::One;
use rootspace::glamour::vec::Vec4;

#[derive(Debug, Parser)]
#[command(name = "create_scene", author, version, about = "Creates a new test scene via asset database", long_about = None)]
struct Args {
    #[arg(short = 'r', long, help = "Load and save assets from within the repository", action = clap::ArgAction::SetTrue)]
    within_repo: bool,
}

struct Dependencies<'a> {
    name: &'a str,
    force_init: bool,
    within_repo: bool,
}

impl<'a> AssetDatabaseDeps for Dependencies<'a> {
    fn name(&self) -> &str {
        self.name
    }

    fn force_init(&self) -> bool {
        self.force_init
    }

    fn within_repo(&self) -> bool {
        self.within_repo
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let matches = Args::parse();

    let deps = Dependencies {
        name: "test",
        force_init: false,
        within_repo: matches.within_repo,
    };

    let adb = AssetDatabase::with_deps(&deps)?;

    let mut scene = Scene::default();
    scene
        .create_entity()
        .with_camera(Camera::default())
        .with_transform(Transform::look_at_lh(
            [0.0, 0.0, -10.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.0],
        ))
        .submit();

    let tri1 = scene
        .create_entity()
        .with_transform(
            Transform::builder()
                .with_translation(Vec4::new(1.0, 0.0, 0.0, 0.0))
                .build(),
        )
        .with_renderable(RenderableSource::with_model("models", "quad.ply"))
        .submit();

    scene
        .create_entity()
        .with_parent(tri1)
        .with_transform(
            Transform::builder()
                .with_translation(Vec4::new(-1.0, 0.0, 0.1, 0.0))
                .with_scale(Vec4::one() * 0.5)
                .build(),
        )
        .with_renderable(RenderableSource::with_model("models", "triangle.ply"))
        .submit();

    adb.save_asset(&scene, "scenes", "test.cbor")?;

    Ok(())
}
