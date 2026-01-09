use std::io::{BufReader, Read};

use dorothy::{SquareWaveSpec, encode};

const fn spec() -> SquareWaveSpec {
    SquareWaveSpec {
        offset: 0,
        amplitude: i8::MAX,
        sample_rate: 9600,
        target_freq: 2400,
        num_periods: 8,
    }
}

fn main() {
    let source = std::env::args().nth(1).unwrap();
    let destination = std::env::args().nth(2).unwrap();
    let channels = 1;
    let kcs_spec = spec();

    let mut source_data = Vec::new();
    BufReader::new(std::fs::File::open(source).unwrap())
        .read_to_end(&mut source_data)
        .unwrap();
    let encoded = encode(kcs_spec, 5, &source_data).collect::<Vec<_>>();

    let mut wav_writer = hound::WavWriter::create(
        &destination,
        hound::WavSpec {
            channels,
            sample_rate: kcs_spec.sample_rate as u32,
            bits_per_sample: 8,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    //let mut wav_writer_i16 = wav_writer.get_i16_writer(encoded.len() as u32);
    encoded
        .iter()
        .for_each(|sample| wav_writer.write_sample(*sample).unwrap());
    //wav_writer_i16.flush().unwrap();
    wav_writer.finalize().unwrap();
}
