#[macro_use] extern crate log;
extern crate fern;
extern crate game;

use std::env;
use std::io;
use std::time::Duration;
use log::LevelFilter;
use fern::Dispatch;
use game::Game;

fn main() {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} @{}: {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Trace)
        .chain(io::stdout())
        .apply()
        .unwrap_or_else(|e| error!("Error setting up the logger: {}", e));

    match Game::new(&env::temp_dir(), Duration::from_millis(50), Duration::from_millis(250)) {
        Ok(mut game) => if let Err(e) = game.run(None) {
            error!("The game aborted with a runtime error: {}", e)
        },
        Err(e) => error!("Creation of the game failed with: {}", e),
    }
}
