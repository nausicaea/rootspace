extern crate game;

use std::env;
use std::time::Duration;
use game::Game;

#[test]
#[cfg_attr(not(windows), ignore)]
fn create_and_run_game() {
    let resource_path = env::temp_dir();
    let delta_time = Duration::from_millis(50);
    let max_frame_time = Duration::from_millis(250);
    let iterations = Some(1);
    let mut g = Game::new(&resource_path, delta_time, max_frame_time).unwrap();
    g.run(iterations).unwrap();
}
