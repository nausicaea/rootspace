use std::io::{BufReader, Write};

use clap::Parser;
use dorothy::{Spec, decode};

/// Read data from stdin and decode it as Kansas City Standard to stdout
#[derive(Debug, Parser)]
struct Args;

fn main() -> anyhow::Result<()> {
    let wav_reader = hound::WavReader::new(BufReader::new(std::io::stdin()))?;

    let mut spec = Spec::<i16>::with_kcs();
    spec.channels = wav_reader.spec().channels;
    spec.sample_rate = wav_reader.spec().sample_rate;

    let output = decode(&spec, wav_reader.into_samples::<i16>().map(|s| s.unwrap())).unwrap();
    std::io::stdout().write_all(&output[0])?;

    Ok(())
}
