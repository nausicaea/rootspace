use dorothy::{SquareWaveSpec, decode, encode};
use rstest::{fixture, rstest};

#[fixture]
fn kcs_spec() -> SquareWaveSpec {
    SquareWaveSpec {
        offset: 0,
        amplitude: i16::MAX,
        sample_rate: 9600,
        target_freq: 2400,
        num_periods: 8,
    }
}

#[rstest]
fn roundtrip(kcs_spec: SquareWaveSpec) {
    let channels = 1;

    let source = "Hello, World!".as_bytes();
    let encoded = encode(kcs_spec, source).collect::<Vec<_>>();

    let mut wav_writer = hound::WavWriter::create(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/roundtrip.wav"),
        hound::WavSpec {
            channels,
            sample_rate: kcs_spec.sample_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    let mut wav_writer_i16 = wav_writer.get_i16_writer(encoded.len() as u32);
    encoded.iter().for_each(|sample| wav_writer_i16.write_sample(*sample));
    wav_writer_i16.flush().unwrap();
    wav_writer.finalize().unwrap();

    let decoded = decode(channels as usize, kcs_spec.sample_rate, kcs_spec.target_freq, encoded).unwrap();

    assert_eq!(&decoded[0], source);
}
