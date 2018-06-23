extern crate clap;
#[macro_use]
extern crate log;
extern crate fern;
extern crate game;

use clap::{App, Arg};
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
        .expect("Unable to configure the logger");

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("headless")
                .long("headless")
                .help("Disables the graphical backend"),
        )
        .arg(
            Arg::with_name("iterations")
                .short("i")
                .long("iterations")
                .takes_value(true)
                .help("Specifies the number of iterations to run"),
        )
        .get_matches();

    let headless = matches.is_present("headless");
    let iterations: Option<usize> = matches.value_of("iterations").and_then(|i| i.parse().ok());

    let r = Game::new(
        &env::temp_dir(),
        Duration::from_millis(50),
        Duration::from_millis(250),
    );
    match r {
        Ok(mut game) => if let Err(e) = game.run(headless, iterations) {
            error!("The game aborted with a runtime error: {}", e)
        },
        Err(e) => error!("Creation of the game failed with: {}", e),
    }
}
