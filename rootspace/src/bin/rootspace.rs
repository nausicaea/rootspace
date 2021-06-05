use anyhow::{Context, Result};
use clap::{load_yaml, App};
use engine::{GliumBackend, HeadlessBackend};
use fern::Dispatch;
use log::{LevelFilter, SetLoggerError};
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
        .chain(std::io::stderr())
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
        let force = scm.is_present("force");
        let name = scm.value_of("name").context("Missing required argument 'name'")?;

        // Configure the project-specific directories
        let project_dirs = directories::ProjectDirs::from("org", "nausicaea", name)
            .context("Could not find the project directories")?;
        let asset_dir = project_dirs.data_local_dir().join("assets");
        let scene_dir = asset_dir.join("scenes");
        let main_scene = scene_dir.join("main.json");

        if force || !asset_dir.exists() {
            let g = Rootspace::<HeadlessBackend>::new(name).context("Could not create a new, empty game")?;
            g.save(main_scene)
                .context("Could create the new main scene")?;
        }
    } else if subcommand == "run" {
        let scm = maybe_subcommand_matches.context("No arguments were provided to the run subcommand")?;
        let headless = scm.is_present("headless");
        let _command = scm.value_of("command");
        let name = scm.value_of("name").context("Missing required argument 'name'")?;

        // Configure the project-specific directories
        let project_dirs = directories::ProjectDirs::from("org", "nausicaea", name)
            .context("Could not find the project directories")?;
        let scene_dir = project_dirs.data_local_dir().join("assets").join("scenes");
        let main_scene = scene_dir.join("main.json");

        if headless {
            let mut g = Rootspace::<HeadlessBackend>::load(main_scene)
                .context("Could not load a headless game from an existing state")?;
            g.run();
        } else {
            let mut g =
                Rootspace::<GliumBackend>::load(main_scene).context("Could not load a game from an existing state")?;
            g.run();
        }
    }

    Ok(())
}
