extern crate clap;
extern crate combine;
extern crate log;
extern crate fern;
extern crate ply;

use clap::{App, Arg};
use combine::{parser::Parser, stream::{ReadStream, buffered::BufferedStream, state::State}};
use fern::Dispatch;
use log::LevelFilter;
use std::{io::{self, Read}, fs::File, path::PathBuf};

fn is_extant_file(value: String) -> Result<(), String> {
    let p = PathBuf::from(value);
    if p.exists() && p.is_file() {
        Ok(())
    } else {
        Err("The value '{0}' does not refer to an existing file".into())
    }
}

fn main() {
    Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("{} @{}: {}", record.level(), record.target(), message)))
        .level(LevelFilter::Trace)
        .chain(io::stdout())
        .apply()
        .expect("Unable to configure the logger");

    let matches = App::new("ply")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Parses stanford ply files and verifies or converts them.")
        .arg(Arg::with_name("files")
             .multiple(true)
             .validator(is_extant_file)
             .help("Specify the file(s) to be loaded by the ply parser"))
        .get_matches();

    let file_paths = matches.values_of("files")
        .expect("The argument 'files' was not found")
        .map(|p| PathBuf::from(p))
        .collect::<Vec<_>>();

    for path in &file_paths {
        let file = File::open(path)
            .unwrap();

        let stream = BufferedStream::new(State::new(ReadStream::new(file)), 32);

        let ply_data = ply::parsers::ply().parse(stream)
            .expect("Parse error")
            .0;

        println!("{:?}", ply_data);
    }
}
