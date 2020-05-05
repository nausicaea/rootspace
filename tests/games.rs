use anyhow::Result;
use rootspace::Rootspace;
use pacman::Pacman;
use engine::HeadlessBackend;
use std::time::Duration;

#[test]
fn create_and_run_rootspace_headless() -> Result<()> {
    let resource_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/rootspace");
    let delta_time = Duration::from_millis(50);
    let max_frame_time = Duration::from_millis(250);
    let iterations = Some(1);

    let mut g: Rootspace<HeadlessBackend> = Rootspace::new(&resource_path, delta_time, max_frame_time)?;
    g.load()?;
    g.run(iterations);

    Ok(())
}

#[test]
fn create_and_run_pacman_headless() -> Result<()> {
    let resource_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/pacman");
    let delta_time = Duration::from_millis(50);
    let max_frame_time = Duration::from_millis(250);
    let iterations = Some(1);

    let mut g: Pacman<HeadlessBackend> = Pacman::new(&resource_path, delta_time, max_frame_time)?;
    g.load()?;
    g.run(iterations);

    Ok(())
}
