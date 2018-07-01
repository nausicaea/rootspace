extern crate game;

use game::Game;
use std::env;
use std::time::Duration;

#[test]
#[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
#[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
fn create_and_run_game() {
    let resource_path = env::temp_dir();
    let delta_time = Duration::from_millis(50);
    let max_frame_time = Duration::from_millis(250);
    let iterations = Some(1);
    let mut g = Game::new(&resource_path, delta_time, max_frame_time).unwrap();
    g.run(false, iterations).unwrap();
}

#[test]
fn create_and_run_game_headless() {
    let resource_path = env::temp_dir();
    let delta_time = Duration::from_millis(50);
    let max_frame_time = Duration::from_millis(250);
    let iterations = Some(1);
    let mut g = Game::new(&resource_path, delta_time, max_frame_time).unwrap();
    g.run(true, iterations).unwrap();
}
