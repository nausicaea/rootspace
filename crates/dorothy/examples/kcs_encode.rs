use std::io::{BufReader, Cursor, Read, Write};

use clap::Parser;
use dorothy::{Spec, encode};

/// Read data from stdin and encode it as Kansas City Standard to stdout
#[derive(Debug, Parser)]
struct Args {
    /// Set how much padding is added to the front and back of the data stream
    #[arg(short, long, default_value_t = 5)]
    padding_factor: usize,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut kcs_spec = Spec::<i16>::with_kcs();
    kcs_spec.padding_factor = args.padding_factor;

    let mut source_data = Vec::new();
    BufReader::new(std::io::stdin()).read_to_end(&mut source_data)?;
    let encoded = encode(&kcs_spec, &source_data).collect::<Vec<_>>();

    let mut output_buffer = Cursor::new(Vec::new());
    let mut wav_writer = hound::WavWriter::new(
        &mut output_buffer,
        hound::WavSpec {
            channels: kcs_spec.channels,
            sample_rate: kcs_spec.sample_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )?;
    //let mut wav_writer_i16 = wav_writer.get_i16_writer(encoded.len() as u32);
    for sample in encoded {
        wav_writer.write_sample(sample)?;
    }
    //wav_writer_i16.flush().unwrap();
    wav_writer.finalize()?;
    std::io::stdout().write_all(&output_buffer.into_inner())?;

    Ok(())
}
