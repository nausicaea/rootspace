use dorothy::{decode, encode, SquareWaveSpec};

#[test]
fn roundtrip() {
    let channels = 1;
    let sample_rate = 9600;
    let target_freq = 2400;
    let ones_num_periods = 8;

    let source = "Hello, World!".as_bytes();
    let encoded = encode(
        SquareWaveSpec {
            offset: 128,
            amplitude: 128,
            sample_rate: sample_rate as usize,
            target_freq: target_freq as usize,
            num_periods: ones_num_periods as usize,
        },
        source
    ).collect::<Vec<_>>();

    let mut wav_writer = hound::WavWriter::create(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/roundtrip.wav"), hound::WavSpec { channels: 1, sample_rate: sample_rate, bits_per_sample: 16, sample_format: hound::SampleFormat::Int }).unwrap();
    let mut wav_writer_i16 = wav_writer.get_i16_writer(encoded.len() as u32);
    encoded.iter().for_each(|sample| wav_writer_i16.write_sample(*sample));
    wav_writer_i16.flush().unwrap();
    wav_writer.finalize().unwrap();

    let decoded = decode(channels, sample_rate as usize, target_freq, encoded).unwrap();

    assert_eq!(&decoded[0], source);
}

