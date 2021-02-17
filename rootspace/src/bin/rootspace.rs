use directories;
use anyhow::{Context, Result};
use clap::{App, load_yaml};
use fern::Dispatch;
use log::{LevelFilter, SetLoggerError};
use std::io;

fn setup_logger(verbosity: u64) -> Result<(), SetLoggerError> {
    let log_level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} @{}: {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(io::stderr())
        .apply()
}

fn main() -> Result<()> {
    // Parse the command line arguments
    let app_yaml = load_yaml!("rootspace.yaml");
    let matches = App::from_yaml(app_yaml).get_matches();
    let verbosity = matches.occurrences_of("verbosity");
    let (subcommand, maybe_subcommand_matches) = matches.subcommand();

    // Configure the logger
    setup_logger(verbosity)
        .context("Could not configure the logging system")?;

    // Configure the project-specific directories
    let config_dir = directories::ProjectDirs::from(
        "org",
        "nausicaea",
        "rootspace",
    ).context("Could not find the project directories")?;

    if subcommand == "initialize" {
        let scm = maybe_subcommand_matches
            .context("No arguments were provided to the initialize subcommand")?;
        let name = scm.value_of("name")
            .context("Missing required argument 'name'")?;

        todo!("Implement the initialize subcommand");

    } else if subcommand == "run" {
        let scm = maybe_subcommand_matches
            .context("No arguments were provided to the run subcommand")?;
        let headless = scm.is_present("headless");
        let command = scm.value_of("command");
        let name = scm.value_of("name")
            .context("Missing required argument 'name'")?;

        todo!("Implement the run subcommand");
    }

    Ok(())
}
