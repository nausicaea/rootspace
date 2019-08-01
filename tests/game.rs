extern crate failure;
extern crate game;

use failure::Error;
use game::Game;
use std::time::Duration;

#[test]
fn create_and_run_game_headless() -> Result<(), Error> {
    let resource_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/rootspace");
    let delta_time = Duration::from_millis(50);
    let max_frame_time = Duration::from_millis(250);
    let headless = true;
    let iterations = Some(1);

    let mut g = Game::new(&resource_path, delta_time, max_frame_time)?;
    g.load(headless)?;
    g.run(iterations);

    Ok(())
}
