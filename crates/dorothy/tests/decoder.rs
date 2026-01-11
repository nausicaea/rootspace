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
fn decode_files_from_py_kcs(#[case] source: &str, #[case] expected: &str) {
    use dorothy::Spec;

    let r = WavReader::open(TEST_DIR.join(source)).unwrap();

    // Verify decoder assumptions
    let wav_spec = r.spec();
    assert_eq!(
        wav_spec.sample_format,
        hound::SampleFormat::Int,
        "Sample data type should be Int"
    );
    assert!(wav_spec.bits_per_sample <= 16, "Bits per sample should be at most 16");

    let mut spec = Spec::<i16>::with_kcs();
    spec.channels = wav_spec.channels;
    spec.sample_rate = wav_spec.sample_rate;

    let output = decode(&spec, r.into_samples::<i16>().map(|s| s.unwrap())).unwrap();

    let mut expected_data = Vec::new();
    BufReader::new(File::open(TEST_DIR.join(expected)).unwrap())
        .read_to_end(&mut expected_data)
        .unwrap();
    // py_kcs's kcs_encode.py adds several null bytes whenever it encounters a carriage return. I
    // don't want my parser to have special logic to decode that, so we need to adjust the expected
    // output by inserting those null bytes.
    expected_data.pop();
    expected_data.extend(&[13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10]);

    assert_eq!(output.len(), 1);
    let output = &output[0];

    assert_eq!(
        output[..expected_data.len() - 1],
        expected_data[..expected_data.len() - 1],
        "lenient equivalency failed"
    );
    assert_eq!(output, &expected_data, "strict equivalency failed");
}
