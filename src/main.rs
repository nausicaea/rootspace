use clap::{App, Arg};
use fern::Dispatch;
use rootspace::Rootspace;
use log::{error, LevelFilter};
use std::{env, io, path::PathBuf, time::Duration};
use anyhow::{Result, Context};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("The argument {0} is missing")]
    MissingArgument(&'static str),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Do not know the game {0}")]
    ParseGameError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Game {
    Rootspace,
    Pacman,
}

impl std::str::FromStr for Game {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "rootspace" => Ok(Game::Rootspace),
            "pacman" => Ok(Game::Pacman),
            _ => Err(Error::ParseGameError(value.to_string())),
        }
    }
}

impl Into<&'static str> for Game {
    fn into(self) -> &'static str {
        (&self).into()
    }
}

impl Into<&'static str> for &Game {
    fn into(self) -> &'static str {
        match self {
            Game::Rootspace => "Rootspace",
            Game::Pacman => "Pacman",
        }
    }
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
            Arg::with_name("game")
                .possible_values(&[Game::Rootspace.into(), Game::Pacman.into()])
                .required(true)
                .help("Which game should I start?"),
        )
        .get_matches();

    let headless = matches.is_present("headless");
    let verbosity = matches.occurrences_of("verbosity");
    let iterations = matches
        .value_of("iterations")
        .map(|i: &str| i.parse::<usize>())
        .transpose()?;
    let game = matches
        .value_of("game")
        .ok_or(Error::MissingArgument("game"))
        .and_then(|g| g.parse::<Game>().map_err(|e| e.into()))?;

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

        PathBuf::from(manifest_dir).join("assets").join(Into::<&'static str>::into(game))
    };

    if game == Game::Rootspace {
        if headless {
            let mut g = Rootspace::new_headless(resource_dir, Duration::from_millis(50), Duration::from_millis(250))
                .context("Cannot initialise the game")?;

            g.load().context("Cannot load the game")?;

            g.run(iterations);
        } else {
            let mut g = Rootspace::new_glium(resource_dir, Duration::from_millis(50), Duration::from_millis(250))
                .context("Cannot initialise the game")?;

            g.load().context("Cannot load the game")?;

            g.run(iterations);
        }
    } else if game == Game::Pacman {
        todo!()
    }

    Ok(())
}
