use clap::Parser;
use ecs::with_dependencies::WithDependencies;
use engine2::assets::scene::Scene;
use engine2::components::camera::Camera;
use engine2::components::renderable::RenderableSource;
use engine2::components::transform::Transform;
use engine2::resources::asset_database::{AssetDatabase, AssetDatabaseDeps};

#[derive(Debug, Parser)]
#[command(name = "scene", author, version, about = "Creates a new test scene via asset database", long_about = None)]
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
            [0.0, 0.0, -2.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.0],
        ))
        .submit();

    scene
        .create_entity()
        .with_transform(Transform::default())
        .with_renderable(RenderableSource::with_model("models", "triangle.ply"))
        .submit();

    adb.save_asset(&scene, "scenes", "test.cbor")?;

    Ok(())
}
