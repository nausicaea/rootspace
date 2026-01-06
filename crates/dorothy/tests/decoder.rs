use std::{path::PathBuf, sync::LazyLock};

use dorothy::{decode, encode, SquareWaveSpec};
use hound::WavReader;
use rstest::rstest;
use std::num::NonZeroUsize;
use std::{
    fs::File,
    io::{BufReader, Read},
};

const TEST_DIR: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests")));

#[test]
fn roundtrip() {
    let channels = unsafe { NonZeroUsize::new_unchecked(1) };
    let sample_rate = unsafe { NonZeroUsize::new_unchecked(9600) };
    let target_freq = unsafe { NonZeroUsize::new_unchecked(2400) };
    let ones_num_periods = unsafe { NonZeroUsize::new_unchecked(8) };

    let source = "Hello, World!".as_bytes();
    let encoded = encode(
        SquareWaveSpec {
            offset: 128,
            amplitude: 128,
            sample_rate: sample_rate.get(),
            target_freq: target_freq.get(),
            num_periods: ones_num_periods.get(),
        },
        source
    );
    let decoded = decode(channels, sample_rate, target_freq, encoded).unwrap();
    let decoded = &decoded[0];

    assert_eq!(source, decoded);
}

#[rstest]
#[case("hello-world.wav", "hello-world.txt")]
fn decoding_files_works_as_expected(#[case] source: &str, #[case] expected: &str) {
    let r = WavReader::open(TEST_DIR.join(source)).unwrap();

    // Verify decoder assumptions
    let spec = r.spec();
    assert_eq!(
        spec.sample_format,
        hound::SampleFormat::Int,
        "Sample data type should be Int"
    );
    assert!(spec.bits_per_sample <= 16, "Bits per sample should be at most 16");

    let output = decode(
        NonZeroUsize::new(spec.channels as usize).unwrap(),
        NonZeroUsize::new(spec.sample_rate as usize).unwrap(),
        unsafe { NonZeroUsize::new_unchecked(2400) },
        r.into_samples::<i16>().map(|s| s.unwrap()),
    )
    .unwrap();

    let mut expected_data = Vec::new();
    BufReader::new(File::open(TEST_DIR.join(expected)).unwrap())
        .read_to_end(&mut expected_data)
        .unwrap();

    assert_eq!(output.len(), 1);
    let output = &output[0];

    assert_eq!(
        output[..expected_data.len() - 1],
        expected_data[..expected_data.len() - 1],
        "lenient equivalency failed"
    );
    assert_eq!(output, &expected_data, "strict equivalency failed");
}
