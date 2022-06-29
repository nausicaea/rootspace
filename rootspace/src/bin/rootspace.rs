use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use engine::{GliumBackend, HeadlessBackend, APP_ORGANIZATION, APP_QUALIFIER};
use fern::Dispatch;
use log::{debug, LevelFilter, SetLoggerError};
use rootspace::Rootspace;

/// Encodes the command line arguments for the Rootspace binary
#[derive(Debug, Parser)]
#[clap(name = "rootspace")]
#[clap(about = "Runs a game with the Rootspace engine", long_about = None, version)]
struct Args {
    #[clap(short, long, help = "Increases the output of the program", parse(from_occurrences))]
    verbose: u64,
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Create a new game")]
    Initialize {
        #[clap(
            short,
            long,
            help = "Instruct the initializer to overwrite all data at the local data assets directory",
            parse(from_flag)
        )]
        force: bool,
        #[clap(help = "Specify the name of the game")]
        name: String,
    },
    #[clap(about = "Run an existing game")]
    #[clap(alias = "r")]
    Run {
        #[clap(short = 'H', long, help = "Disable the graphical backend", parse(from_flag))]
        headless: bool,
        #[clap(short, long, help = "Execute a command as soon as the game has loaded")]
        command: Option<String>,
        #[clap(help = "Specify the name of the game")]
        name: String,
    },
}

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
    let matches = Args::parse();
    let verbosity = matches.verbose;

    // Configure the logger
    setup_logger(verbosity).context("Could not configure the logging system")?;

    match &matches.subcommand {
        Subcommands::Initialize { force, name } => {
            // Configure the project-specific directories
            let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, name)
                .context("Could not find the project directories")?;
            let asset_dir = project_dirs.data_local_dir().join("assets");
            let scene_dir = asset_dir.join("scenes");
            let main_scene = scene_dir.join("main.json");

            if *force || !asset_dir.exists() {
                let g =
                    Rootspace::<HeadlessBackend>::new(name, *force).context("Could not create a new, empty game")?;
                g.save(main_scene).context("Could create the new main scene")?;
            }
        }
        Subcommands::Run {
            headless,
            command,
            name,
        } => {
            // Configure the project-specific directories
            let project_dirs = directories::ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, name)
                .context("Could not find the project directories")?;
            let scene_dir = project_dirs.data_local_dir().join("assets").join("scenes");
            let main_scene = scene_dir.join("main.json");
            debug!("Loading the main scene from {}", main_scene.display());

            if *headless {
                let mut g = Rootspace::<HeadlessBackend>::load(main_scene)
                    .context("Could not load a headless game from an existing state")?;
                if let Some(cmd) = command {
                    g.with_command(cmd);
                }
                g.run();
            } else {
                let mut g = Rootspace::<GliumBackend>::load(main_scene)
                    .context("Could not load a game from an existing state")?;
                if let Some(cmd) = command {
                    g.with_command(cmd);
                }
                g.run();
            }
        }
    }

    Ok(())
}
