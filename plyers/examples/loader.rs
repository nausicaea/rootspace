use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "loader", author, version, about = "Loads files as Stanford PLY model files", long_about = None)]
struct Args {
    #[arg(short, long, help = "Increases the output of the program", action = clap::ArgAction::Count)]
    verbose: u8,
    #[arg(help = "Specify the path of the file to be parsed")]
    path: std::path::PathBuf,
}

fn main() {
    let matches = Args::parse();

    match plyers::load_ply(&matches.path) {
        Err(e) => panic!("{}", e),
        Ok(ply) => {
            if matches.verbose > 0 {
                print!("{:?}", ply)
            }
        }
    }
}
