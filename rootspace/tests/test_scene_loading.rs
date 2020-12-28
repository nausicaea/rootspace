use rootspace::Rootspace;
use engine::HeadlessBackend;
use anyhow::{Result, Context};
use std::env;
use std::path::PathBuf;

#[test]
fn test_segfault() -> Result<()> {
    let resource_dir = {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .context("Cannot find the `CARGO_MANIFEST_DIR` environment variable")?;

        PathBuf::from(manifest_dir).parent().unwrap().join("assets").join("rootspace")
    };

    let mut g: Rootspace<HeadlessBackend> = Rootspace::new(resource_dir, None)
        .context("Cannot create the game")?;

    g.set_main_scene(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/segfault.json"))?;

    g.load().context("Cannot load the game")?;

    // g.run();

    Ok(())
}