#[macro_use]
extern crate log;
extern crate fern;
extern crate game;

use fern::Dispatch;
use game::Game;
use log::LevelFilter;
use std::env;
use std::io;
use std::time::Duration;

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
        .expect("Error setting up the logger");

    let r = Game::new(
        &env::temp_dir(),
        Duration::from_millis(50),
        Duration::from_millis(250),
    );
    match r {
        Ok(mut game) => if let Err(e) = game.run(false, None) {
            error!("The game aborted with a runtime error: {}", e)
        },
        Err(e) => error!("Creation of the game failed with: {}", e),
    }
}
