extern crate clap;
extern crate log;
extern crate failure;
extern crate fern;
extern crate game;

use clap::{App, Arg};
use fern::Dispatch;
use failure::Error;
use game::Game;
use log::LevelFilter;
use std::{env, io, time::Duration, path::PathBuf};

fn main() -> Result<(), String> {
    Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("{} @{}: {}", record.level(), record.target(), message)))
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
        ).arg(
            Arg::with_name("iterations")
                .short("i")
                .long("iterations")
                .takes_value(true)
                .help("Specifies the number of iterations to run"),
        ).get_matches();

    let headless = matches.is_present("headless");
    let iterations: Option<usize> = matches.value_of("iterations").and_then(|i| i.parse().ok());

    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("resources")
        .join("rootspace");

    let mut game = Game::new(dir, Duration::from_millis(50), Duration::from_millis(250))
        .map_err(|e| format!("{}", e))?;

    game.run(headless, iterations)
        .map_err(|e| format!("{}", e))
}
