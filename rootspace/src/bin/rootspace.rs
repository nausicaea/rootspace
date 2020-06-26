use clap::{App, Arg};
use fern::Dispatch;
use rootspace::Rootspace;
use log::{error, LevelFilter};
use std::{env, io, path::PathBuf, time::Duration};
use anyhow::{Result, Context};
use thiserror::Error;
use engine::{HeadlessBackend, GliumBackend};

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

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
            Arg::with_name("verbosity")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Increases the output of the program"),
        )
        .arg(
            Arg::with_name("iterations")
                .short("i")
                .long("iterations")
                .takes_value(true)
                .help("Specifies the number of iterations to run"),
        )
        .arg(
            Arg::with_name("command")
                .short("c")
                .long("command")
                .takes_value(true)
                .help("Execute a command within the game context"),
        )
        .get_matches();

    let headless = matches.is_present("headless");
    let verbosity = matches.occurrences_of("verbosity");
    let iterations = matches
        .value_of("iterations")
        .map(|i: &str| i.parse::<usize>())
        .transpose()?;
    let command = matches.value_of("command");

    let log_level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
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

        PathBuf::from(manifest_dir).parent().unwrap().join("assets").join("rootspace")
    };

    if headless {
        let mut g: Rootspace<HeadlessBackend> = Rootspace::new(resource_dir, command)
            .context("Cannot initialise the game")?;

        g.load().context("Cannot load the game")?;

        g.run(iterations);
    } else {
        let mut g: Rootspace<GliumBackend> = Rootspace::new(resource_dir, command)
            .context("Cannot initialise the game")?;

        g.load().context("Cannot load the game")?;

        g.run(iterations);
    }

    Ok(())
}
