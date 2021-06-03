use anyhow::{Context, Result};
use clap::{load_yaml, App};
use engine::{HeadlessBackend, GliumBackend};
use fern::Dispatch;
use log::{LevelFilter, SetLoggerError, debug};
use std::io;
use rootspace::Rootspace;

fn setup_logger(verbosity: u64) -> Result<(), SetLoggerError> {
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
    setup_logger(verbosity).context("Could not configure the logging system")?;

    if subcommand == "initialize" {
        let scm = maybe_subcommand_matches.context("No arguments were provided to the initialize subcommand")?;
        let name = scm.value_of("name").context("Missing required argument 'name'")?;

        // Configure the project-specific directories
        let project_dirs = directories::ProjectDirs::from("org", "nausicaea", name)
            .context("Could not find the project directories")?;

        let asset_database = project_dirs.data_local_dir().join("assets");
        debug!("Located the asset database at: {}", asset_database.display());
        if !asset_database.is_dir() {
            std::fs::create_dir_all(&asset_database).context("Could not create the asset database directory")?;
        }

        let state_dir = project_dirs.data_local_dir().join("states");
        debug!("Located the state directory at: {}", state_dir.display());
        if !state_dir.is_dir() {
            std::fs::create_dir_all(&state_dir).context("Could not create the state directory")?;
        }

        let main_state = state_dir.join("main.json");

        let g = Rootspace::<HeadlessBackend>::new().context("Could not create a new, empty game")?;
        g.save(&main_state)
            .context("Could not save the state for the new, empty game")?;
    } else if subcommand == "run" {
        let scm = maybe_subcommand_matches.context("No arguments were provided to the run subcommand")?;
        let headless = scm.is_present("headless");
        let _command = scm.value_of("command");
        let name = scm.value_of("name").context("Missing required argument 'name'")?;

        // Configure the project-specific directories
        let project_dirs = directories::ProjectDirs::from("org", "nausicaea", name)
            .context("Could not find the project directories")?;

        let state_dir = project_dirs.data_local_dir().join("states");
        debug!("Located the state directory at: {}", state_dir.display());

        let main_state = state_dir.join("main.json");

        if headless {
            let mut g = Rootspace::<HeadlessBackend>::load(&main_state)
                .context("Could not load a headless game from an existing state")?;
            g.run();
        } else {
            let mut g = Rootspace::<GliumBackend>::load(&main_state)
                .context("Could not load a game from an existing state")?;
            g.run();
        }
    }

    Ok(())
}
