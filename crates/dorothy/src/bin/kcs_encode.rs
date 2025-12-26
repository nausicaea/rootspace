use std::fs::File;
use std::io::BufReader;
use clap::Parser;
use hound::SampleFormat;
use dorothy::shared::{CodecProperties, Kcs};

#[derive(Debug, Parser)]
struct Args {
    in_file: String,
    out_file: String,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let _spec = hound::WavSpec {
        channels: 1,
        sample_rate: Kcs::SAMPLE_RATE.get(),
        bits_per_sample: (size_of::<i8>() * 8) as u16,
        sample_format: SampleFormat::Int,
    };

    let mut in_file = BufReader::new(File::open(args.in_file)?);
    let mut writer = vec![];

    std::io::copy(&mut in_file, &mut writer)?;

    Ok(())
}
