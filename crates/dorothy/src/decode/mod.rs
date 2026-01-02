use self::bit_decoder::{BitDecoder, Error as BitDecoderError};
use crate::util;
use crate::util::{Sign, samples_per_bit};
use itertools::Itertools;
use num_traits::{ConstZero, Signed};
use std::num::NonZeroUsize;
use std::task::Poll;

mod bit_decoder;

/// Given an audio signal with a known sample rate, and number of channels, decode a
/// frequency-shift-keyed (FSK) signal in the vein of the Kansas City Standard (KCS).
///
/// # Assumptions
///
/// 1. The `sample_rate` is at least twice as large as `target_freq` (Nyquist)
/// 2. Binary one `0b01` is represented by the `target_freq` frequency
/// 3. Each audio sample is represented by a signed number (i.e. `i8`, `i16`, `f32`, etc.)
/// 4. Audio channels are interleaved
///
/// # Errors
///
/// 1. Errors with [`Error::NyquistViolation`] if the `sample_rate` is not at least twice as large as `target_freq`
pub fn decode<N, I>(
    channels: NonZeroUsize,
    sample_rate: NonZeroUsize,
    target_freq: NonZeroUsize,
    samples: I,
) -> Result<Vec<Vec<u8>>, Error>
where
    N: Copy + Signed + ConstZero + PartialOrd,
    I: IntoIterator<Item = N>,
{
    let channels = channels.get();
    let sample_rate = sample_rate.get();
    let target_freq = target_freq.get();

    if (target_freq << 1) > sample_rate {
        return Err(Error::NyquistViolation(sample_rate, target_freq));
    }

    // Create an iterator over all audio samples, grouped by channel, indexed by time and channel
    let per_channel_iter = samples.into_iter()
        // Index into each sample (remember: channels are interleaved)
        .enumerate()
        // Group by channels, thus de-interleaving audio samples for each channel
        .chunk_by(|(sample_idx, _)| sample_idx % channels);

    // This tells us how much we need to skip forward when decoding each byte
    let samples_per_bit: usize = samples_per_bit(sample_rate, target_freq);

    // Decode each channel separately
    let output = per_channel_iter
        .into_iter()
        .map(|(_, channel_samples)| decode_channel(channel_samples.map(|(_, sample)| sample), samples_per_bit))
        .collect();

    Ok(output)
}

fn decode_channel<S>(samples: impl Iterator<Item = S>, samples_per_bit: usize) -> Vec<u8>
where
    S: Copy + Signed + ConstZero + PartialOrd,
{
    // Each channel may produce independent output
    let mut output = Vec::default();

    // Perform edge detection on the audio signal, replacing each sample with `SignChange::Changed` if the sign
    // has changed wrt. to the previous sample, or `SignChange::Unchanged` if it stayed the same.
    let mut sign_change_iter = samples.scan(Sign::NonNegative, |p, sample| {
        Some(util::to_sign_change(util::to_sign_bit(sample), p))
    });

    let mut fsm = BitDecoder::new(sign_change_iter.by_ref(), samples_per_bit);
    while !fsm.is_complete() {
        match fsm.poll() {
            Poll::Pending => (),
            Poll::Ready(Ok(output_byte)) => output.push(output_byte),
            Poll::Ready(Err(BitDecoderError::EndOfIterator)) => break,
            Poll::Ready(Err(e)) => panic!("{e}"),
        }
    }

    output
}

/// KCS decoder errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The sample rate is not at least twice as large as the target frequency
    #[error("Sample rate {0} is not at least twice as large as target frequency {1}")]
    NyquistViolation(usize, usize),
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::LazyLock};

    use crate::decode::decode;
    use hound::WavReader;
    use rstest::rstest;
    use std::num::NonZeroUsize;
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
}
