#![recursion_limit = "256"]

use clap::Parser;
use rootspace::App;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, help = "Select the game to run", default_value = "rootspace")]
    game: String,
}

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tokio-console")]
    console_subscriber::init();
    #[cfg(not(feature = "tokio-console"))]
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let app = App::new(&args.game);
    app.run()?;
    Ok(())
}
