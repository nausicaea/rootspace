use clap::{App, Arg};
use fern::Dispatch;
use game::Game;
use log::{error, LevelFilter};
use std::{env, io, path::PathBuf, time::Duration};
use anyhow::{Result, Context};

fn main() -> Result<()> {
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
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Increases the output of the program"),
        )
        .get_matches();

    let headless = matches.is_present("headless");
    let iterations: Option<usize> = matches.value_of("iterations").and_then(|i| i.parse().ok());
    let verbosity = matches.occurrences_of("verbosity");

    let log_level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Trace,
    };

    Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("{} @{}: {}", record.level(), record.target(), message)))
        .level(log_level)
        .chain(io::stdout())
        .apply()
        .context("Unable to configure the logger")?;

    let resource_dir = {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .context("Cannot find the `CARGO_MANIFEST_DIR` environment variable")?;

        PathBuf::from(manifest_dir).join("assets").join("rootspace")
    };

    if headless {
        let mut game = Game::new_headless(resource_dir, Duration::from_millis(50), Duration::from_millis(250))
            .context("Cannot initialise the game")?;

        game.load().context("Cannot load the game")?;

        game.run(iterations);
    } else {
        let mut game = Game::new_glium(resource_dir, Duration::from_millis(50), Duration::from_millis(250))
            .context("Cannot initialise the game")?;

        game.load().context("Cannot load the game")?;

        game.run(iterations);
    }

    Ok(())
}
