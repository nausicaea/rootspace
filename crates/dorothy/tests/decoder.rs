use std::{path::PathBuf, sync::LazyLock};

use dorothy::decode;
use hound::WavReader;
use rstest::rstest;
use std::{
    fs::File,
    io::{BufReader, Read},
};

const TEST_DIR: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests")));

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
        spec.channels as usize,
        spec.sample_rate as usize,
        2400,
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
