use self::bit_decoder::{BitDecoder, Error};
use crate::ring_buffer::RingBuffer;
use crate::util;
use itertools::Itertools;
use num_traits::Signed;
use std::borrow::Borrow;
use std::task::Poll;

mod bit_decoder;

pub fn decode<N, I>(channels: usize, sample_rate: usize, target_freq: usize, samples: I) -> Vec<Vec<u8>>
where
    N: Copy + Signed,
    I: IntoIterator<Item = N>,
{
    // Determine how many audio samples are used to encode a single bit
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation
    )]
    let samples_per_bit: usize = (sample_rate as f32 * 8.0 / target_freq as f32).abs().round() as usize;

    let framed_iter = samples.into_iter()
        // Retrieve the raw sample data
        .map(|sample| *sample.borrow())
        // Separate the individual interleaved channels
        .chunks(channels);

    // Create an iterator over all audio samples, grouped by channel, indexed by time and channel
    let per_channel_iter = framed_iter.into_iter()
        // Create an index for each channel
        .flat_map(std::iter::Iterator::enumerate)
        // Group by channels, thus de-interleaving audio samples for each channel
        .chunk_by(|(channel_idx, _)| *channel_idx);

    per_channel_iter
        .into_iter()
        .map(|(_, channel_samples)| decode_channel(channel_samples.map(|(_, sample)| sample), samples_per_bit))
        .collect()
}

fn decode_channel<S>(channel_data: impl Iterator<Item = S>, samples_per_bit: usize) -> Vec<u8>
where
    S: Copy + Signed,
{
    // Per-channel decoder state
    let mut look_behind: RingBuffer<u8> = RingBuffer::new(samples_per_bit);
    let mut output = Vec::default();

    let mut sign_change_iter = channel_data
        .map(|sample| util::to_sign_bit(sample))
        .scan(0_u8, |p, sample| Some(util::to_sign_change(sample, p)));

    let mut fsm = BitDecoder::new(sign_change_iter.by_ref(), &mut look_behind, samples_per_bit);
    while !fsm.is_complete() {
        match fsm.poll() {
            Poll::Pending => (),
            Poll::Ready(Ok(output_byte)) => output.push(output_byte),
            Poll::Ready(Err(Error::EndOfIterator)) => break,
            Poll::Ready(Err(e)) => panic!("{e}"),
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::LazyLock};

    use crate::decode::decode;
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
        );

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
}
