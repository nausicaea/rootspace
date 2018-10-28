extern crate game;

use game::Game;
use std::time::Duration;

#[test]
fn create_and_run_game_headless() {
    let resource_path = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/rootspace");
    let delta_time = Duration::from_millis(50);
    let max_frame_time = Duration::from_millis(250);
    let iterations = Some(1);
    let mut g = Game::new(&resource_path, delta_time, max_frame_time).unwrap();
    g.run(true, iterations).unwrap();
}
