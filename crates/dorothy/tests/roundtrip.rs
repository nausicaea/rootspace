use dorothy::{SquareWaveSpec, decode, encode};
use rstest::{fixture, rstest};

#[fixture]
fn kcs_spec() -> SquareWaveSpec {
    SquareWaveSpec {
        offset: 0,
        amplitude: i8::MAX,
        sample_rate: 9600,
        target_freq: 2400,
        num_periods: 8,
    }
}

#[rstest]
fn roundtrip(kcs_spec: SquareWaveSpec) {
    let channels = 1;

    let source = "Hello, World!".as_bytes();
    let encoded = encode(kcs_spec, 5, source).collect::<Vec<_>>();
    let decoded = decode(channels as usize, kcs_spec.sample_rate, kcs_spec.target_freq, encoded).unwrap();

    // let spec_ex = hound::WavSpecEx {
    //     spec: hound::WavSpec {
    //         channels: todo!(),
    //         sample_rate: todo!(),
    //         bits_per_sample: todo!(),
    //         sample_format: todo!(),
    //     },
    //     bytes_per_sample: todo!(),
    // };

    assert_eq!(&decoded[0], source);
}
